# GWES windows and messaging

Status: design-first subsystem plan; no new runtime implementation is authorized by this document. [Architecture](../ARCHITECTURE.md) is the sole normative source for shared machine, identity, frame, address-space, clock, outcome and snapshot contracts. [Plan](../PLAN.md) owns sequencing. Proposed shared names below are not claims that corresponding Rust types already exist.

## Responsibility and current implementation

GWES owns window/class state, parent/owner relationships, z-order, focus/capture, invalidation, posted/sent message queues, quit state and message-specific lifecycle. It supplies work and wake information; it must not choose CPU execution, invent application readiness or paint a replacement UI.

Current state resides in `CeKernel.gwe` and [gwe.rs](../../src/ce/gwe.rs): HWND tables, per-numeric-TID posted queues, sent-message IDs/stacks, typed `MessagePointerPayload`, clipboard/IME/SIP/caret state. Guest API serialization/dispatch is spread across [coredll.rs](../../src/ce/coredll.rs), Kernel adapters and [unicorn.rs](../../src/emulator/unicorn.rs). Native WNDPROC/send/GetMessage continuations live in CPU containers rather than one canonical task-frame owner. Visibility/receiver-work checks also influence existing run-loop rotation. These are present compatibility paths, not the proposed clean boundary.

## Target ownership and interfaces

| Owner | Exclusive responsibility | Boundary |
| --- | --- | --- |
| Machine authority | Serialize lifecycle/events and activate authoritative task frames | GWES requests wake/delivery; cannot activate CPU directly |
| GWES | Window/message/queue/send state and IDs | Resolve recipients through the shared task contract; never infer owner from whichever PID is selected |
| Task/frame owner | Blocked caller, nested callback ancestry, CPU continuation and completion | One wait/frame identity per real continuation; retained outer waits are not a second runnable task |
| Address-space service | Validate/prepare guest payload/output and track temporary ownership | GWES keeps host-owned typed data until destination preparation succeeds |
| Rendering | Execute drawing/presentation from actual window/DC state | Invalidation is work, not a scheduler-priority override |

Design the facade around existing operations: post/filter/peek/commit one message; begin/reply/complete/expire one send; invalidate/begin/end paint; create/destroy one window subtree. Return effects to the machine rather than carrying mutable Kernel/CPU/raw framebuffer pointers inside persistent GWES callbacks. API adapters perform ABI encoding; they do not own queue policy. The machine supplies guest time for queue timestamps, timers and send expiry; host observation budgets stay outside these decisions.

A send transaction needs its actual sender and receiver frame plus send ID. A raw HWND/TID or latest trace cannot authorize an old reply. Cross-address-space typed payloads must be materialized in the recipient space, not expose a sender pointer. Unknown application-defined LPARAM values require an explicit ABI compatibility contract; do not reinterpret every integer as a pointer.

Occupied native SMT slot: when A owns its one parked frame, an eligible B
cross-thread send terminates the attempt with `Backend` error, without raw
dispatch, B send/wait/result/register changes or disturbing A. Zero timeout,
same-thread, invalid/destroyed/broadcast HWND, unsupported flags and hung
`SMTO_ABORTIFHUNG` retain raw handling. This is not concurrent A+B support;
per-task SMT frames remain required. Controls: `occupied_send_message_timeout_slot_fails_closed_for_second_sender`, `occupied_send_message_timeout_slot_preserves_raw_compatibility`.

### Bounded thread-directed birth gate (design before implementation)

The next Stage 3 slice concerns only `PostThreadMessageW`, `PostMessageW` with
`hwnd == 0`, and the caller's `PostQuitMessage`. Window, broadcast, timer,
keyboard, remote-input, sent, paint and modal producers retain their current
numeric compatibility paths; this slice does not certify those families. A
production thread-directed post captures a *live* target `ThreadBirthKey`
before publishing queue, quit, changed-status, ready-timestamp, trace or wake
effects. A stale captured birth fails before those effects, never resolving a
replacement birth through the same numeric TID. Self-quit must validate the
native caller's captured birth, not merely resolve its current numeric TID.
The current success/error behavior for an unknown target is not a CE parity
claim; local CE headers show an HTHREAD queue owner but do not settle the
precise API return and LastError contract. Explicit legacy-unknown posting is
fixture-only, never a production fallback.

Keep the existing per-TID FIFO and sent → posted → quit → paint priority. Each
posted entry has one delivery owner: exact thread birth for these three
producers, explicit legacy-unknown fixture owner, or unscoped numeric/window
compatibility for unmigrated producers. Query, Get/Peek, queue-status, new-input
and ready-timestamp readers must filter the exact entries and quit/status
effects for their captured live birth while preserving the first eligible
entry under the existing HWND/message filters. An A entry must not become B's
input when its TID is reused; an unscoped window entry must not be silently
discarded by the new filter. A's exact exit retires only A-owned thread posts,
quit and associated status/timestamp, leaving B, legacy-unknown and unmigrated
window entries intact. Scheduler wake *and* direct readiness selection must
reject stale A waits even if a B or unscoped entry exists for the same TID.

The connected source gate needs executed-red A→exit→same-TID-B controls for
Peek remove/no-remove, GetMessage, `WM_QUIT`, `QS_POSTMESSAGE`/MsgWait, exact
wake, mixed FIFO/filtering, stale captured publication and native self-quit.
Preserve a direct stale-native-resume guard control after selector behavior
changes. Existing numeric-only test fixtures must either create canonical
thread births or use an explicitly named legacy-unknown API. Source checks
do not establish live iNavi acceptance or full GWES/TaskDirectory migration.

## Failure and lifetime rules

Preserve the existing prepare-before-consume boundary: preflight mapped writable MSG output, map/init persistent guest payload, then consume the exact message/send ID and expose its pointer. A missing destination-memory context must retain typed work for a real resume path, not deliver a host placeholder. Queue expiry/replacement between peek and commit must reject a mismatched ID without consuming its successor. Preparation failure preserves live queue/wait/caller state and authentic API failure; it is not an unhandled import fallthrough.

Creation requires a prepared CREATESTRUCT and real callback phases. On preparation/refusal failure, use established failed-create teardown and release unpublished scratch exactly once; no successful HWND publication or duplicate fallback. Retain callback scratch between phases, release at terminal success/refusal. Existing geometry mutation followed by SetWindowPos preparation failure is not an atomic rollback guarantee; document that boundary rather than invent state reversal.

Window/thread/process destruction cancels their actual queues/sends/frames, clears focus/capture/clipboard ownership and retires temporary payloads once. Do not delete every same-TID outer wait on a count/deadline service point. Handle/identity generations and checkpoint cloning must prevent old sends or callback archives from reviving a new incarnation. Genuine archived-return restoration uses validated caller metadata; historical PC-only recovery is not a target contract.

## Observability

Publish message/send/window IDs, resolved owner/frame, queue reason, preparation/commit/cancel/return phase, guest timestamp and bounded error. Observe ingress, post, actual guest dispatch and return separately. Use the shared future snapshot contract for coherent window/queue/frame views; it is not implemented today. Readiness/visibility statistics are diagnostic, not permission to create a scheduling exception.

The actual splash is HWND `0x20008`, class `afx:10000:3:0:b5000:0`; successful taskbar/SIP wrapper calls do not establish its dismissal. BED2 and the failed 650 run both retained the decorative splash, and the 650 run recovered the authentic `0x5237` caller but showed no later splash teardown before its crash (ledger E-1, E-2). These app-specific facts belong to verification, never a generic ready flag or forced event in GWES.

## Migration and acceptance

1. Inventory all queue/callback/wait enter, commit, return, expiry and teardown consumers; publish the current ownership map before moving code.
2. Resolve canonical task/frame ownership without changing message order or CE API behavior; eliminate duplicate writable identities as each consumer migrates.
3. Move one complete family (ordinary GetMessage/send or window creation) behind the facade, including cancellation/debug/checkpoint paths; leave an explicit compatibility adapter, not two authoritative stores.
4. Retire visibility-based scheduler overrides only when machine scheduling and real receiver/frame readiness cover their verified consumers.

Acceptance requires posted FIFO/filter/quit behavior; real cross-thread send/reply/timeout and nested callback→outer wait return; destroy during send; typed None→Some delivery; readonly/cross-page MSG and payload failures with unchanged bytes/queue/wait; exact-ID replacement; CREATESTRUCT scratch lifetime/refusals; independent checkpoints and stale-frame rejection. Run genuine native callbacks and inspect actual return contexts. Unit/API success alone does not prove splash-to-map readiness.

Open decisions (window-handle generation/lifetime, custom pointer-message marshalling, first complete continuation family): [DECISIONS.md](../DECISIONS.md) O-G1 to O-G3. Resolve through the common plan before implementation.
