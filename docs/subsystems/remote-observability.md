# Remote ingress and observability

Status: design-first subsystem plan. [Architecture](../ARCHITECTURE.md) is the sole shared-contract authority; [Plan](../PLAN.md) selects migration order. In particular, SnapshotEpoch is a **required future contract**, not a guarantee of the current HTTP/debug service.

## Current boundary

[remote_server.rs](../../src/remote_server.rs) owns the TCP listener and host-facing Arc/Mutex queues/caches: pending JSON controls, latest status/framebuffer, keyed debug texts, recent logs and audio cursors. POST input validates/enqueues a control and returns an ACK/pending count. The machine drains controls later. [ce/remote.rs](../../src/ce/remote.rs) maintains touch/key/serial/NMEA/IMU/audio state; Kernel/CPU/Desktop adapters route events to guest devices or GWES. Framebuffer publication copies RGB pixels; GET PNG and debug endpoints serve the latest cached data.

Status, framebuffer and separate debug keys are published independently. `claim_debug_publish_slot` uses host Instant to throttle publication. No shared epoch, publication-age guarantee or atomic cross-endpoint owner correlation exists. Equal active-before/active-after PID is a bracket observation, not proof against an intervening ABA switch. Current traces are bounded retained histories, not complete execution journals.

## Target responsibilities and interfaces

Host transport may validate request shape, enforce bounds/authentication, assign ingress identity and queue an owned event. It must not modify guest task/window/device state on server threads, advance guest time, force a readiness event or choose a runnable task. CeMachine, under the anchor's shared event/time rules, is the single application/consumption authority. Application-defined addresses remain guest addresses interpreted only by the address-space/API boundary, never transport pointers.

Separate **accepted**, **applied**, **consumed**, **guest-dispatched/returned** and **cancelled/failed** states. A POST 200/`ok` currently proves acceptance only. The target event correlation should allow observers to follow one ingress event through the real machine/device/message path without granting arbitrary control authority. Replay/delivery order and conversion from host arrival to guest event time must use the anchor's policy; do not silently equate the two clock domains.

At a machine safe boundary, build a bounded immutable snapshot containing provenance for task/frame/address-space, actual native attempt/outcome, window/queue/device views and published pixels. Export shared future epoch plus guest capture time, host publication time and completeness/truncation. Separately paced audio/video may reference a machine snapshot/frame but cannot imply a simultaneous whole-machine state. Old cached views remain explicitly old; no HTTP request receipt can refresh their guest provenance.

## Safety and lifetime

Own all buffers leaving the machine; no borrowed Kernel/framebuffer/native-hook pointers in Arc workers. Bound request bodies, pending input queues, logs, snapshot history and encoder work. Define overload refusal/backpressure before enqueue; do not silently discard an accepted event and then report consumption. Disconnect/cancellation removes host subscriptions without changing guest task state. Server shutdown closes only its owned listener/workers; observer cleanup cannot kill an unrelated emulator.

Diagnostic GETs must not alter scheduler state, consume guest messages, acquire wait objects or advance guest timers. Observation still has host CPU/memory/latency cost: disclose overhead and match measurement configurations for comparisons. Preserve token/bind policy and avoid secrets/raw giant payloads in telemetry. This subsystem does not authorize an external write merely because it exposes a control endpoint.

## Required telemetry

For every capture retain run/config/executable identity, actual launch/end UTC, per-request start/receive/failure UTC, endpoint/status/bytes/hash, epoch/age when implemented, requested and actual horizons, and truncation/error markers. File tags/sample-start filenames do not define response time or prelaunch coverage.

Native diagnostics must bind to the primary native-start owner before repairs/rotation. Keep nativeResult, actual expiry query, uncollected fields, no-fault fields and query errors distinct. A selected GetMessage snapshot cannot establish all earlier attempts were fault-free. Do not interpret configured timeout as expiry or HTTP queue emptiness as guest execution.

The 650 run (ledger E-2) makes these distinctions concrete: 16 location ACKs but only zero-byte guest reads in the retained trace; the last successful splash frame preceded the host crash by 0.217 s; no terminal PPM or full trace despite filenames in the result JSON; observer sessions exited -1 without a graceful finish record. None is a successful 330 s run or an atomic final epoch.

## Migration and acceptance

1. Document existing endpoint/ACK/cache semantics without changing them; add explicit freshness/unknown fields in a separately versioned, bounded contract.
2. Introduce machine-owned event correlation and consumption acknowledgements through existing real input/device/GWES consumers; remove direct guest mutation from host adapters.
3. Publish coherent immutable machine-boundary snapshots using the shared future epoch; retain per-component timing for independently paced streams.
4. Retire uncorrelated legacy debug caches only after consumers/tests migrate; label compatibility responses clearly during transition.

Acceptance includes malformed/oversized/auth failures with no enqueue, ordered duplicate/cancel/overload behavior, accepted-but-not-yet-consumed events, actual serial ReadFile bytes and WNDPROC returns, paused/blocked guest behavior, cache age/epoch skew and ABA controls, GET mutation-freedom, disconnect/shutdown, bounded audio/frame/log queues and stale frame/fault snapshots. Test server concurrency without inventing a task scheduler. Controlled performance comparisons require matched observers; coherent snapshots do not by themselves prove application map readiness.
