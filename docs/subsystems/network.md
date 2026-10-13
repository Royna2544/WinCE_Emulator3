# Guest networking / Winsock

Common anchor: [ARCHITECTURE.md](../ARCHITECTURE.md), A1–A5/A6.
Implementation order: [PLAN.md](../PLAN.md). Target interfaces are proposals.

## Current boundary

[winsock.rs](../../src/winsock.rs) dispatches guest imports and implements
socket/readiness/connect/timeout helpers. The CPU wait path records Winsock
readiness and resumes scalar continuations. This is distinct from the host
remote-control server: accepting a local control request is not guest network
progress. Existing networking mode/policy remains explicit; architecture work
does not authorize external transmission or invent successful connections.
Current socket state lives in process-global OnceLock<Mutex<WinsockState>>,
not a machine-owned/checkpoint-isolated service. That ownership is a migration
gap, not a guarantee supplied by cloning the Kernel.

## Target owner and interfaces

A machine-owned networking service owns socket identities and protocol state.
Host sockets are adapters behind that service, not a global mutable state
alternative to the machine. Nonblocking readiness and typed I/O completions
enter the common sequencer with request/object generation and HostInstant.
Guest API results, buffers and deadlines are committed using TaskIdentity,
FrameIdentity, AddressSpace and GuestInstant from the anchor.

Guest memory access is checked before publishing transferred bytes/results.
Readiness is a candidate, not data already consumed. Registered wait frames pin
socket identity until completion/cancellation; handle-number reuse cannot
complete an older request. Remote control and networking share event ingress
contracts, never execution ownership or a hidden priority override.

## Failure, cancellation and telemetry

Preserve would-block, pending connect, timeout, peer close, partial transfer,
invalid pointer/handle and host error as distinct outcomes. Do not suppress
an error because a positive timeout was configured. Closing a socket cancels
the correct generation and retires references without duplicating completion.
Bound receive/send queues and diagnostic histories; backpressure remains
observable. Guest and host deadlines must not add elapsed time twice.

## Plan and gates

Inventory current socket service/global ownership, API dispatch, readiness
waiters and configuration before moving authority. Specify a replayable host
adapter and live clock/event fencing; do not guess external packet timing.
Test would-block→ready→partial read, close during wait, connect error/timeout,
generation reuse, invalid full output buffers and process termination. Verify
actual returned payloads, not queued bytes or API acceptance. Report guest
transfers, queue capacity and completion latency; no real-network gain follows
from scheduler tests or a local HTTP ACK.
