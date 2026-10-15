# Remote ingress and observability

Status: design-first subsystem plan. [Architecture](../ARCHITECTURE.md) is the sole shared-contract authority; [Plan](../PLAN.md) selects migration order. In particular, SnapshotEpoch is a **required future contract**, not a guarantee of the current HTTP/debug service.

## Current boundary

[remote_server.rs](../../src/remote_server.rs) owns the TCP listener and host-facing Arc/Mutex queues/caches: pending JSON controls, latest status/framebuffer, keyed debug texts, recent logs and audio cursors. POST input validates/enqueues a control and returns an ACK/pending count. The machine drains controls later. [ce/remote.rs](../../src/ce/remote.rs) maintains touch/key/serial/NMEA/IMU/audio state; Kernel/CPU/Desktop adapters route events to guest devices or GWES. Framebuffer publication copies RGB pixels; GET PNG and debug endpoints serve the latest cached data.

Status, framebuffer and separate debug keys are published independently. `claim_debug_publish_slot` uses host Instant to throttle publication. No shared epoch, publication-age guarantee or atomic cross-endpoint owner correlation exists. Equal active-before/active-after PID is a bracket observation, not proof against an intervening ABA switch. Current traces are bounded retained histories, not complete execution journals.

### W-B1c mixed touch/key admission order (design gate; not implemented)

The common [A1/A5 anchor](../ARCHITECTURE.md) targets one machine-wide
accepted-event order. This interim gate changes only the current `CeRemote` →
`CeKernel` adapter's local touch/key admission order. `CeRemote` currently owns separate touch/key
FIFOs (`src/ce/remote.rs`); Kernel drains every touch before keys and resolves
key focus afterward (`src/ce/kernel.rs`). A key accepted while A is focused can
therefore be posted to B after a later DOWN focuses B. This is a host-admission
ordering defect, not evidence of CE6 WNDPROC execution order.

Keep one typed FIFO in `CeRemote` as the sole pending touch/key authority. Assign
one checked, monotonically increasing *per-remote-instance* admission sequence
to each expanded event when `enqueue_touch` or `enqueue_key` succeeds. A tap's
DOWN and UP are an atomic, contiguous pair in this order. Preserve the current
enqueue signatures, control/desktop/monitor/startup producers, per-kind status
counts, and host-observed `enqueued_at_ms`; sequence, not wall time, orders the
drain. Invalid input or sequence exhaustion rejects the entire local
`CeRemote::enqueue_*` call before queue/log mutation; it does not retroactively
reject an upstream HTTP ACK. Draining consumes each event once. The queue and sequence
copy with `CeRemote::Clone` as instance-local state; this is not a machine-wide
checkpoint lineage or globally unique ingress ID.

Kernel's existing active, blocked-wait/modal-hint and explicit-HWND drain
entrypoints consume this one FIFO in order. For active/hint drains, each key
resolves live focus then stored active at its own position, replacing today's
once-per-drain key-target snapshot. The explicit-HWND drain still targets both
keys and touches to its supplied HWND, subject to per-event live-owner checks.
Each touch retains W-B1b capture/hit-test, exact-Birth validation,
stale-orphan/drop and owner-qualified post/wake rules. An explicit stale target
still consumes/drops, without rerouting a later event.
The per-event trace follows admission order; the existing target-thread list
retains first-occurrence order with duplicate thread IDs omitted, and the
remote control drain retains its existing `handled` count. Preserve the
existing `Message.time_ms` projection for now; host wall time can tie or move
backward, and a tap's synthetic UP is stamped 80 ms after DOWN even though both
are admitted together. [O-C5](../DECISIONS.md) remains open: this gate neither
sorts by timestamp nor chooses the future guest-time policy.

Current producers are HTTP pending-control dispatch through Kernel, host
desktop polling, and monitor/startup enqueues; Get/Peek, host blocked-wait and
active-window paths are drain consumers. Existing type-specific drain methods
are used only by Kernel and tests and must be retired, not left as a second
reordering route. Per-kind HTTP status fields and logging remain observations,
not routing authority. HTTP and desktop have separate upstream ingress paths:
this gate proves touch/key order *after admission to CeRemote*, not physical
arrival order across those sources, cross-thread WNDPROC return order, or a
coherent snapshot. NMEA/serial controls can take effect during HTTP batch
dispatch while touch/key are posted after the batch, so cross-device order is
also unproved. Current unbounded `CeRemote` input capacity and HTTP
backpressure/oldest-control eviction are separate remote-boundary work; do not
add a silent drop to this gate.

Test first with a real Birth-owned A/B fixture: focus A, enqueue key(A), then
DOWN at B, drain once, and require key post/trace to A before DOWN post/trace
to B, with B focused afterward under the current emulator's drain-time
`WM_LBUTTONDOWN` focus mutation. That mutation is existing local policy, not
CE6 focus/activation parity. Preserve the pre-fix red as evidence. Add a
key(A) → DOWN(B) → key(B) control to prove per-key target resolution, plus
reverse/interleaved, tap-pair, invalid/overflow atomicity, per-kind count,
explicit stale HWND with both touch and key consumed/dropped without effects,
and no-live-target controls. Run the W-B1b focused set,
full release and no-feature suites, and Unicorn-only/desktop-without-trace
checks on sealed source. A source pass is not a live iNavi tap or latency
result; the next separately approved mounted run must re-verify guest tap
delivery under the changed routing. Owner-filtered teardown, implicit gesture
ownership and host `GetMessage(NULL)` wake without an HWND remain separate
gates.

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
