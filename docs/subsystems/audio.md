# Audio and waveOut

Common anchor: [ARCHITECTURE.md](../ARCHITECTURE.md), A1–A5/A6.
Implementation order: [PLAN.md](../PLAN.md). Target interfaces are proposals.

## Current boundary

[audio.rs](../../src/ce/audio.rs) owns wave format/buffers, waveOut device
state and sink plumbing. COREDLL dispatch serializes guest structures and
the CPU layer enters guest audio callbacks. Host/null/WebSocket/logging
sinks are presentation adapters, not evidence that guest playback completed.
After successful waveOutWrite, current raw dispatch immediately sets
WHDR_INQUEUE|WHDR_DONE and signals an event callback; the native adapter can
enter a WOM_DONE function callback immediately. This compatibility completion
is not evidence of host playback finishing. Callback ownership participates in
the per-API continuation migration; audio has no separate proven guest clock.

## Target owner and dependencies

Audio owns canonical stream/buffer state keyed by kernel object identity.
Guest buffers are accessed through AddressSpace-scoped GuestMemory; submitted
bytes are copied/pinned by an explicit lifetime contract before guest release.
Backend sinks accept bounded chunks and return typed completion/failure events
to CeMachine. They never execute a guest callback or mutate scheduler state.

Actual wave completion schedules a typed callback/message frame for the
canonical TaskIdentity through the API/GWE boundary. Native callback entry and
return preserve that FrameIdentity and outer continuation. Guest time defines
emulated stream timing; HostInstant measures host queue/output latency.

## Failure and cleanup

Separate format rejection, invalid buffer/handle, reset/close, sink failure,
buffer completion and callback delivery. A null sink is explicitly selected
capability, not fake audible output. Backpressure and queue capacity are
bounded/configured; dropping output must be observable and must not manufacture
successful guest consumption. Cancellation/reset retires buffer references and
completion identity once, preserving CE return semantics and callback order.

## Design and verification gates

Before changing timing: define the clock mode, buffer pin/copy policy, reset
race ordering and callback generation contract against CE evidence. Cover
prepare/write/unprepare/reset/close, nested callback return, duplicate/stale
handles, sink shutdown, bounded WebSocket clients and guest-memory failures.
Existing waveOut/raw tests are a baseline, not proof of realtime playback or
full task migration. Measure queued/live bytes, completion/callback lag and
sink capacity separately from host RSS and presentation success.
