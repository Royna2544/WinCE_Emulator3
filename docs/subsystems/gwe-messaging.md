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

### Bounded thread-directed birth gate (source-tested)

`PostThreadMessageW`, `PostMessageW(hwnd=0)` and self `PostQuitMessage` capture a live `ThreadBirthKey`; stale births reject queue, quit, status, timestamp, trace and wake publication.
One per-TID FIFO keeps each post's exact-birth, fixture-only legacy-unknown or unscoped compatibility owner; sent → posted → quit → paint priority and HWND/message filtering remain.
Get/Peek, status/new-input, timestamp, wait wake/readiness and exact exit use birth-eligible state; A retirement preserves B and unscoped work, while window-originated status remains compatibility state.
Unknown-target return/LastError parity, window/broadcast/HWND-owned-timer/keyboard/remote/sent/paint/modal producer births outside the bounded gates below, and full GWES/live iNavi acceptance remain open. [Thread-post receipt](../../evidence/stage3-gwe-thread-birth-20261015/validation.receipt.md): release 1,952/0/9; no-feature 1,417/0/3. [HWND-less timer receipt](../../evidence/stage3-thread-timer-birth-20261015/validation.receipt.md): release 1,960/0/9; no-feature 1,424/0/3.

### W-B1 window-owner birth (W-B1a and W-B1b source-tested; teardown open)

One immutable Window owner is exact `(ThreadBirthKey, creation ProcessBirthKey)`, explicit legacy-unknown, or system desktop; numeric TID/PID are creation-only projections, never a second owner authority.
W-B1a preflights the captured native caller birth for `CreateWindowExW`, `CreateDialogIndirectParamW`, and `DialogBoxIndirectParamW` before any GWES/LastError effect; absent birth is a fixture-only compatibility path.
Verified explicit HWND posts enter the existing FIFO as Birth-owned work, committing status, ready time, trace, and exact-birth wake together; stale ordinary HWNDs publish nothing, while desktop/legacy compatibility stays explicit.
Typed-payload recipient and native CreateWindow callback ownership use the frozen process birth, never a current numeric PID lookup; CE6 return/LastError parity remains unverified where local GWES source is silent.
Exact A exit retires A's Birth-owned posts and makes later explicit posts ineligible without a numeric-TID sweep. CE-family [DestroyWindow](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/aa452942%28v%3Dmsdn.10%29) recursively destroys child and owned windows, while [SetParent](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/aa453645%28v%3Dmsdn.10%29) requires the child and new parent to belong to the same application. Cross-birth parent links are not established valid CE6 topology; recovery from foreign parent/owner links is unproved, not an accepted B-descendant-survival rule.
W-B1b remote touch/key ownership passed its bounded
[source gate](../../evidence/stage3-window-input-birth-20261015/validation.receipt.md).
For a Birth-owned window, an explicitly host-targeted HWND has a frozen recipient: if it is stale or destroyed, consume
and drop that input before focus, state, queue/status, timestamp, post trace or
wake publication; never reroute it to a later same-TID window. A guest
`GetMessage`/`PeekMessage` HWND is only a retrieval filter, never external-input
destination or drop authority. Coordinate-only touch instead selects a live
capture window or delivery-time desktop hit-test; a blocked-wait/modal HWND is
only a fallback hint after those checks. Keys select a live focus/active owner,
not mouse capture. `LegacyUnknown` fixture windows and `SystemDesktop` retain
numeric compatibility routing; this gate does not prove their reuse isolation.

For this bounded input gate, stale capture must not authorize a recycled
recipient: a fresh DOWN can use a live coordinate hit-test, while MOVE/UP tied
to the stale gesture are dropped rather than reassigned. Full capture
retirement on exit, implicit gesture ownership and owner-filtered host teardown
remain separate gates. These are bounded source-tested policies, not established CE6
implementation parity. The CE-family [GetMessage retrieval-filter description](https://learn.microsoft.com/en-us/previous-versions/ms960724%28v%3Dmsdn.10%29)
and [SetCapture mouse/stylus description](https://learn.microsoft.com/en-us/previous-versions/windows/embedded/ms924858%28v%3Dmsdn.10%29)
inform the boundary but do not verify this emulator's CE6 behavior.
W-B1b passed 27/27 focused controls, a38 release 1,988/0/9 and a39
no-feature 1,448/0/3 across 18 blocks each, plus a40/a41 feature checks.
The a33/a34 broad failures on an obsolete hidden-window fixture preceded its
a35 test repair, a36/a37 focused reruns and the fresh a38 source seal. The
separate W-B1c mixed touch/key [source gate](../../evidence/stage3-mixed-input-order-20261015/validation.receipt.md)
passed the expected-red-to-green key(A) → DOWN(B) → key(B) control, release
1,992/0/9 and no-feature 1,452/0/3 across 18 blocks each, and two alternate
feature checks. Its release source seal was captured after that run began, with
matching committed file hashes; it is not pre-run evidence. The change orders
local `CeRemote` admission into Kernel and resolves key focus at each event's
position; it does not establish CE6 focus parity, machine-wide arrival order,
cross-thread WNDPROC execution order or live tap delivery. See
[remote-observability](remote-observability.md). Broadcast, HWND timers,
hotkeys, sent/paint/modal and other input producers, full GWES, and live iNavi
acceptance remain open.
Pre-fix expected-red controls covered explicit HWND post, native ordinals 246/688/690, and an in-slice MIPS self-termination followed by stale creation. The [W-B1a receipt](../../evidence/stage3-window-owner-birth-20261015/validation.receipt.md) seals the corrected source and records the initial failed broad attempt, focused fixes, release 1,967/0/9, no-feature 1,427/0/3, and two passing alternate-feature checks. This is a bounded source gate, not full Stage 3 or live-app acceptance.

### Bounded host-exit selection (independent windows; source-tested)

The bounded PID-reuse gate selects independent live windows and captured
backing stores, including already-destroyed windows, by the verified exiting
process birth and each window's frozen creation process birth rather than its
numeric PID/TID. Its control requires a stale prior-birth A window and backing
store to survive B's later exit after PID reuse. The normal reused-PID and
no-birth controls passed; the 18-group broad suites passed release
**2,055/0/9 ignored** and no-default **1,485/0/3 ignored**
([validation receipt](../../evidence/stage3-host-teardown-20261015/validation.receipt.md)).
This does not birth-filter `destroy_window`'s recursive parent/owner traversal
or numeric sent-message process cleanup. Foreign-link recovery, full host
teardown, exit capture retirement and live-app/CE6 acceptance remain open.

### Next proposed gate: sender-owned process-exit send cleanup (not implemented)

The [A2 exit-effect contract](../ARCHITECTURE.md) requires one canonical
exiting birth. Today `SentMessage` freezes the receiver birth but stores only
numeric sender thread/process IDs; `CeKernel::begin_cross_thread_send_message_w`
queues first and then stamps the mutable selected PID. Host exit verifies its
process birth for window selection but calls numeric
`terminate_sent_messages_from_process` and
`terminate_sent_messages_from_sender`. The earlier independent-window pass
does not cover those sends.

The owning GWES send record should retain an immutable sender process birth
obtained at admission from an authenticated caller thread/frame and its
creation process, not inferred from the latest selected PID or caller-supplied
TID alone. The native caller has a captured `NativeCurrentThread` birth and
raw COREDLL dispatch has `CoredllRawContext.caller_birth`; their Send/Timeout
paths must carry and validate those identities rather than discarding them.
Kernel/native/API adapters request the checked transition; GWES owns the send
ID, queue, active-receiver phase and reply/cancellation effects. The production
admission path must prepare the owner and destination before publishing the
send, so a missing or mismatched
caller birth cannot leave an unowned queued send, wake, status bit or result
pointer. Direct numeric-only fixture/compatibility records remain explicitly
unknown; [O-G5](../DECISIONS.md) is open rather than guessing their birth.

The connected consumer inventory includes native sent-message authorization
that still compares only `sender_process_id` with a numeric PID. Birth-stamped
admission without migrating those result/re-entry checks would leave a stale
acceptance route, so they must either require exact provenance or fail closed
in this gate. Raw Send/Timeout and the Kernel helper must share the same
prepared identity rule; a test-only numeric adapter cannot become production
authority.

The first fixture in this bounded gate isolates the process selector: queue a
fixture-created send from A's canonically registered primary thread to a
separate live receiver. Retire A's canonical thread/process through test-only
HandleTable operations that deliberately bypass normal GWES cleanup, then
reuse only A's PID for B with a distinct TID.
Queue B's send to the same receiver and complete B through normal host exit.
The expected red is that numeric process cleanup removes both sends; the
required green preserves A's stranded send and retires B's, without touching
the receiver HWND. This is a defensive stale-artifact fixture, not a claim
that ordinary A exit strands sends. Name the raw log before its first run and
never overwrite or rename it. Add normal same-birth queued cleanup and
active-receiver detachment controls, including a replied timeout send whose
result pointer must be detached on owner exit. For the process-selector
fixture, assert any cancellation, reply or wake by exact send ID: none affects
stranded A, and each selected B send has at most one such effect. Current GWE
active-send cleanup marks the sender terminated while allowing the callback
to finish; queued sends are removed.
Separately, inject stale A caller birth into raw Send/Timeout after PID/TID
reuse and require rejection before any B-owned send ID, queue/status/wake,
trace or result effect. A normal same-birth caller remains a positive control.
The user expanded this gate to exact sender-birth admission, publication,
process/thread exit cleanup and native result/re-entry authorization together.
The active result pointer must be cleared when its sender is detached;
receiver-side cross-process address-space writes still need a separate gate.
The local CE6 process-exit send-cancellation rule has not yet been established
from `C:\WINCE600` or the dumped device binaries; these controls establish
emulator birth-identity safety, not CE6 cancellation parity.

Explicit guest `DestroyWindow` still has recursive child/owner semantics.
Host-exit recursion through a foreign link is a separate target-selection
question under [O-G4](../DECISIONS.md); a cross-birth parent/owner survival
fixture is synthetic safety evidence, not CE6 parity. Do not globally remove
normal `DestroyWindow` recursion to make such a fixture pass.

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
