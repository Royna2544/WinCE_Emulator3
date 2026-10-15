# Remote ingress and observability

Status: design-first subsystem plan. [Architecture](../ARCHITECTURE.md) is the sole shared-contract authority; [Plan](../PLAN.md) selects migration order. In particular, SnapshotEpoch is a **required future contract**, not a guarantee of the current HTTP/debug service.

## Current boundary

[remote_server.rs](../../src/remote_server.rs) owns the TCP listener and host-facing Arc/Mutex queues/caches: pending JSON controls, latest status/framebuffer, keyed debug texts, recent logs and audio cursors. POST input validates/enqueues a control and returns an ACK/pending count. The machine drains controls later. [ce/remote.rs](../../src/ce/remote.rs) maintains touch/key/serial/NMEA/IMU/audio state; Kernel/CPU/Desktop adapters route events to guest devices or GWES. Framebuffer publication copies RGB pixels; GET PNG and debug endpoints serve the latest cached data.

Status, framebuffer and separate debug keys are published independently. `claim_debug_publish_slot` uses host Instant to throttle publication. No shared epoch, publication-age guarantee or atomic cross-endpoint owner correlation exists. Equal active-before/active-after PID is a bracket observation, not proof against an intervening ABA switch. Current traces are bounded retained histories, not complete execution journals.

### Source-tested summary freshness (not a shared snapshot)

The [telemetry source gate](../../evidence/stage3-live-evidence-20261015/telemetry-broad-a03-validation-receipt.md)
adds four fields to each accepted `/api/v1/debug/summary.txt` publication:
`publication_sequence`, `publication_host_monotonic_ns`,
`publication_utc_unix_ms`, and
`live_guest_stable_page_budget_used_bytes`. The publisher samples its host
monotonic clock after acquiring the per-server publication lock, then assigns
a checked, increasing per-server sequence and UTC milliseconds to that slot.
Throttled attempts and repeated HTTP GETs of a cached body do not assign a new
sequence. Compare the embedded sequence, not the observer's receive time, to
identify a repeated summary; UTC can tie or move backward. An advanced
sequence proves a new summary publication, not guest progress or a fresh
frame/window/status snapshot. The summary stamp has no common epoch with
those independently published endpoints, and its monotonic origin is not an
observer clock origin.

The budget scalar reads `kernel.memory.private_commit_used_bytes()`: charged
stable guest pages, including shared-view charges and retained pins, but not
the separately tracked boot reservation or every guest backing. It is not
total CE RAM use, native translation/cache memory, or host RSS/private bytes.
A public `VirtualAlloc` reserve-only/one-page-commit control showed the
expected unchanged/+4-KiB accounting on source; no live iNavi budget curve
has yet been measured.

The [bounded summary observer](../../evidence/stage3-live-evidence-20261015/observer-summary-validation.receipt.md)
is source-tested to attempt a summary GET on each nominal two-second slot,
retain at most 256 complete bodies of at most 64 KiB, and record request/
receive clocks, status, SHA-256, parsed fields and repeated/regressed
publication identity. Oversized, malformed, failed and incomplete responses
remain explicit failures rather than truncated valid samples. The runner's
[first-frame coverage checks](../../evidence/stage3-live-evidence-20261015/runner-offline-a03-first-frame-proof-receipt.md)
are launch-relative, but the observer has not validated a live run with this
new summary schema. Actual request spacing and any missed coverage must be
read from the resulting per-request clocks and flags.

### W-B1c mixed touch/key admission order (source-tested)

The common [A1/A5 anchor](../ARCHITECTURE.md) targets one machine-wide
accepted-event order. The bounded [W-B1c source gate](../../evidence/stage3-mixed-input-order-20261015/validation.receipt.md)
changes only the `CeRemote` → `CeKernel` adapter's local touch/key admission
order. The preserved pre-fix expected red showed the former separate queues:
Kernel drained a later B-point DOWN before an earlier A-focused key and then
posted that key to B. It was a host-admission ordering defect, not evidence of
CE6 WNDPROC execution order.

One typed FIFO in `CeRemote` is now the pending touch/key authority. Each
expanded event receives a checked, monotonically increasing
*per-remote-instance* admission sequence on successful enqueue. A tap's DOWN
and UP reserve an atomic, contiguous pair in that order. The existing enqueue
signatures, control/desktop/monitor/startup producers, per-kind status counts,
and host-observed `enqueued_at_ms` remain; sequence, not wall time, orders the
drain. Invalid input or sequence exhaustion rejects the entire local
`CeRemote::enqueue_*` call before queue/log mutation; it does not retroactively
reject an upstream HTTP ACK. Draining consumes each event once. The queue and
sequence copy with `CeRemote::Clone` as instance-local state; this is not a
machine-wide checkpoint lineage or globally unique ingress ID.

Kernel's existing active, blocked-wait/modal-hint and explicit-HWND drain
entrypoints consume this one FIFO in order. For active/hint drains, each key
resolves live focus then stored active at its own position, replacing the
former once-per-drain key-target snapshot. The explicit-HWND drain still
targets both keys and touches to its supplied HWND, subject to per-event
live-owner checks.
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
active-window paths are drain consumers. The old type-specific drain methods
were retired from source and tests, so they cannot form a second reordering
route. Per-kind HTTP status fields and logging remain observations,
not routing authority. HTTP and desktop have separate upstream ingress paths:
this gate proves touch/key order *after admission to CeRemote*, not physical
arrival order across those sources, cross-thread WNDPROC return order, or a
coherent snapshot. NMEA/serial controls can take effect during HTTP batch
dispatch while touch/key are posted after the batch, so cross-device order is
also unproved. Current unbounded `CeRemote` input capacity and HTTP
backpressure/oldest-control eviction are separate remote-boundary work; do not
add a silent drop to this gate.

The preserved expected red used a real Birth-owned A/B fixture: key(A), then
DOWN(B), drained once. Its corrected and extended key(A) → DOWN(B) → key(B)
control passes, proving per-key target resolution and admission-order post/trace
in this adapter. B focus on DOWN is the emulator's existing drain-time
`WM_LBUTTONDOWN` policy, not established CE6 focus/activation parity. Focused
tap-pair, invalid/sequence-exhaustion atomicity and per-kind count controls,
inherited W-B1b controls, full release and no-feature suites, and two alternate
feature checks passed on the recorded source. The release seal was captured
*after* that run began, with matching committed file hashes and no observed
intervening source write; the receipt does not call it a pre-run seal. A source
pass is not a live iNavi tap or latency result; a separately approved mounted
run must re-verify guest tap delivery under the changed routing. Owner-filtered
teardown, implicit gesture ownership and host `GetMessage(NULL)` wake without
an HWND remain separate gates.

## Target responsibilities and interfaces

Host transport may validate request shape, enforce bounds/authentication, assign ingress identity and queue an owned event. It must not modify guest task/window/device state on server threads, advance guest time, force a readiness event or choose a runnable task. CeMachine, under the anchor's shared event/time rules, is the single application/consumption authority. Application-defined addresses remain guest addresses interpreted only by the address-space/API boundary, never transport pointers.

Separate **accepted**, **applied**, **consumed**, **guest-dispatched/returned** and **cancelled/failed** states. A POST 200/`ok` currently proves acceptance only. The target event correlation should allow observers to follow one ingress event through the real machine/device/message path without granting arbitrary control authority. Replay/delivery order and conversion from host arrival to guest event time must use the anchor's policy; do not silently equate the two clock domains.

At a machine safe boundary, build a bounded immutable snapshot containing provenance for task/frame/address-space, actual native attempt/outcome, window/queue/device views and published pixels. Export shared future epoch plus guest capture time, host publication time and completeness/truncation. Separately paced audio/video may reference a machine snapshot/frame but cannot imply a simultaneous whole-machine state. Old cached views remain explicitly old; no HTTP request receipt can refresh their guest provenance.

### Input-to-frame correlation gate (proposed; not implemented)

This is an A1/A2/A5/A6 design gate for eventual **exactly identified** tap/GPS-to-visual measurements, not a claim about today's endpoints or app readiness. Keep two identities: a bounded transport `IngressReceipt` (run/server lineage plus checked sequence) returned only after a request is actually retained in the ingress queue, and a machine-lineage `ExternalEventId` assigned when the machine sequencer adopts that receipt into its accepted event order. Record a one-to-one binding only for adopted receipts; rejection, expiry or pre-adoption eviction/cancellation has no machine event ID and must be explicit. A server-local number or W-B1c's cloned `CeRemote` sequence is not a machine event ID. Sequence exhaustion rejects before mutation, and a restored/checkpoint-cloned machine starts a new lineage. No cross-producer physical-arrival order is inferred from HTTP, desktop or NMEA timestamps; [O-C5](../DECISIONS.md) still decides guest-time assignment.

The proposed immutable evidence chain is `IngressReceipt → ExternalEventId → delivery record(s) → guest execution record(s) → FrameCommit(SnapshotEpoch)`. Every established edge carries its source identity, lineage and outcome; guest owner identity remains explicitly unbound at ingress and is checked only when machine dispatch resolves it. Absent or truncated edges stay `unknown`, never filled from matching PID/HWND, a later timestamp, queue emptiness or a retained trace's last item. A tap has one parent receipt and separately numbered DOWN/UP children, preserving their actual order and each child's checked target `WindowDeliveryOwner`, `TaskIdentity` and `FrameIdentity` once resolved. Track queue post, guest Get/Peek retrieval where applicable, actual WNDPROC entry and return separately; posting does not prove execution. A dropped/stale target is terminal for that child and cannot be attached to a later numeric HWND/TID incarnation.

For raw NMEA, retain the accepted payload hash/length and each normalization step, then carry byte-span provenance through remote queue, `COM7:` RX, and successful guest `ReadFile`/overlapped completion. One guest read may combine spans from multiple inputs or split one sentence; record exact transferred spans, task/frame and bytes, not a substring match or a positive byte count alone. A guest parser accepting a position and any resulting navigation-state change are separate, currently unproved edges. Host ACK, device drain and positive guest read must not be reported as a parsed fix or visual response.

`FrameCommit` is an immutable copy of pixels at a machine-sequenced presentation boundary, with a fresh `SnapshotEpoch`, owner/run-attempt/frame provenance, guest capture instant, host monotonic commit instant, dimensions, pixel hash and completeness. A cached frame GET reuses its old epoch; a new guest commit may have a new epoch but an unchanged pixel hash, so it is not a visual response. A new HTTP GET or JPEG encoding cannot mint a guest commit. A frame published through a separately paced presenter must retain the referenced commit epoch and its distinct host publication time. A same-epoch GET of status/debug/frame is coherent only if all three serve that immutable epoch; independent current caches remain explicitly skewed. A frontier saying "event was applied before this frame" proves ordering only. A causal event→pixel edge requires observed guest work from that event (or an explicit, identity-checked asynchronous handoff) through the drawing change into that commit. Background animation, unrelated window paint and any lineage gap yield **post-input first changed frame**, not **input-caused visual lag**; neither implies a map.

Use one host-monotonic origin for receipt, delivery, guest-execution and frame-commit interval arithmetic; retain UTC for external correlation and `GuestInstant` separately for CE semantics. Report ACK→delivery, delivery→WNDPROC return or guest-read completion, and supported guest-effect→frame-commit intervals independently. For NMEA, label the chosen endpoint (first byte, complete sentence, parsed fix or later app state). If only two-second frame GET samples exist, report an observation bracket, not an exact commit instant. A bounded trace gap, lost ACKed event, clock-origin mismatch, failed read, missing guest frame, native error or PC zero invalidates an exact-lag claim.

Current seams for a connected migration are [remote_server.rs](../../src/remote_server.rs) `queue_control_message`/`publish_framebuffer` (the present bounded queue can evict its oldest accepted JSON), [CeKernel](../../src/ce/kernel.rs) remote-control dispatch, W-B1c input drain/post, `drain_remote_serial_to_handle` and `read_file`/`read_file_into`, [Unicorn](../../src/emulator/unicorn.rs) WNDPROC execution/return, and [main.rs](../../src/main.rs) `RemoteLiveFramebuffer`/`HostLiveFramebuffer` publication. The current summary-local sequence, retained message/file traces and cached RGB image do not connect these seams. First expected-red controls should expose ACK-then-eviction, DOWN/UP sibling identity, split/coalesced NMEA reads, stale HWND/TID or checkpoint lineage, cached-frame GET repetition, and unrelated post-input paint. The connected fix must turn each control green without making GET mutate guest state, adding unbounded tracing, fabricating a parser/map event or altering scheduler policy. Then separately validate on an approved live run; source tests alone cannot establish visual lag or full bring-up.

## Safety and lifetime

Own all buffers leaving the machine; no borrowed Kernel/framebuffer/native-hook pointers in Arc workers. Bound request bodies, pending input queues, logs, snapshot history and encoder work. Define overload refusal/backpressure before enqueue; do not silently discard an accepted event and then report consumption. Disconnect/cancellation removes host subscriptions without changing guest task state. Server shutdown closes only its owned listener/workers; observer cleanup cannot kill an unrelated emulator.

Diagnostic GETs must not alter scheduler state, consume guest messages, acquire wait objects or advance guest timers. Observation still has host CPU/memory/latency cost: disclose overhead and match measurement configurations for comparisons. Preserve token/bind policy and avoid secrets/raw giant payloads in telemetry. This subsystem does not authorize an external write merely because it exposes a control endpoint.

## Required telemetry

For every capture retain run/config/executable identity, actual launch/end UTC, per-request start/receive/failure UTC, endpoint/status/bytes/hash, epoch/age when implemented, requested and actual horizons, and truncation/error markers. File tags/sample-start filenames do not define response time or prelaunch coverage.

Native diagnostics must bind to the primary native-start owner before repairs/rotation. Keep nativeResult, actual expiry query, uncollected fields, no-fault fields and query errors distinct. A selected GetMessage snapshot cannot establish all earlier attempts were fault-free. Do not interpret configured timeout as expiry or HTTP queue emptiness as guest execution.

The 650 run (ledger E-2) makes these distinctions concrete: 16 location ACKs but only zero-byte guest reads in the retained trace; the last successful splash frame preceded the host crash by 0.217 s; no terminal PPM or full trace despite filenames in the result JSON; observer sessions exited -1 without a graceful finish record. None is a successful 330 s run or an atomic final epoch.

## Migration and acceptance

1. Preserve the source-tested summary-local freshness stamp while documenting existing endpoint/ACK/cache semantics; add shared provenance and explicit unknown fields in a separately versioned, bounded contract.
2. Introduce machine-owned event correlation and consumption acknowledgements through existing real input/device/GWES consumers; remove direct guest mutation from host adapters.
3. Publish coherent immutable machine-boundary snapshots using the shared future epoch; retain per-component timing for independently paced streams.
4. Retire uncorrelated legacy debug caches only after consumers/tests migrate; label compatibility responses clearly during transition.

Acceptance includes malformed/oversized/auth failures with no enqueue, ordered duplicate/cancel/overload behavior, accepted-but-not-yet-consumed events, actual serial ReadFile bytes and WNDPROC returns, paused/blocked guest behavior, cache age/epoch skew and ABA controls, GET mutation-freedom, disconnect/shutdown, bounded audio/frame/log queues and stale frame/fault snapshots. Test server concurrency without inventing a task scheduler. Controlled performance comparisons require matched observers; coherent snapshots do not by themselves prove application map readiness.
