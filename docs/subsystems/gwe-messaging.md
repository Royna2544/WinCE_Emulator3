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
The earlier host-exit gate did not birth-filter `destroy_window`'s recursive
parent/owner traversal or numeric sent-message process cleanup. The later
sender gate narrows the latter, not foreign-link recovery, full teardown or
live-app/CE6 acceptance.

### Proposed host-exit exact-window-set gate (design; not implemented)

This Stage 3 cut applies the common [exit-effect ownership](../ARCHITECTURE.md)
contract to the window-target portion of process exit. The stopped
`mark_process_launch_exited_*` path verifies a canonical `ProcessBirthKey`;
the current-process pseudo `terminate_process` path also calls the shared
cleanup but still needs a captured guest caller lease/frame before its whole
exit can satisfy the anchor. GWES owns the one live-HWND
selection and exact-set mutation from each window's frozen creation process
birth; parent and owner links are not target authority. `LegacyUnknown` with a
captured process birth remains selectable by that birth, while an unbound
legacy/system window is not guessed from its numeric PID. Keep guest-requested
`DestroyWindow` and its current child/owned recursion separate and unchanged.

Current consumers are both Kernel exit paths (`mark_process_launch_exited_*`
and current-process pseudo `terminate_process`), GWE's window, queue, send,
focus/capture, clipboard, IME, region, z-order and invalidation state, and
Kernel's typed-payload, shell, HWND-timer, trace and backing-store cleanup.
The already-destroyed backing-store selector remains separate: a dead HWND
has no live GWES teardown target but its captured backing can still belong to
the exiting birth. `Gwe` is cloned with its windows and immutable birth keys;
this cut does not implement whole-machine checkpoint relineage or make a
cloned key valid in another machine.

At the stopped host-exit boundary, preflight the canonical birth before any
window effect. GWES captures the complete live target set for that birth and
retires **exactly that set** in one operation, returning the actually retired
HWNDs.
Retirement preserves the existing destroyed-window tombstones. The shared
exact-set destructor performs the existing per-target queue/send,
focus/capture and rendering cleanup once; the public recursive destructor may
reuse that mutation routine only after independently deriving its recursive
target set at its current GWE call point. Do not freeze that public set before
Kernel's existing `WM_RENDERALLFORMATS`/`WM_DESTROY` callbacks or omit its
send wakes and framebuffer behavior. Host exit remains a distinct path with
no synthesized destroy callbacks. Kernel uses the returned set, not a guessed
descendant list, for
payload, shell and timer cleanup and tracing. Kernel's separate frozen-birth
backing-store list drives restore/discard even for already-dead HWNDs. An
unknown/mismatched exit birth fails before mutation; no cross-birth parent or
owner edge expands the selected set. Preserve the existing sent-message
sender-exit gate, one-shot cancellation/reply behavior, and guest API return
behavior; do not add callback synthesis or a scheduling policy in this cut.

Expected-red controls first: strand an A-birth HWND/backing under a recycled
PID, then create B-birth windows and fixture-only cross-birth **parent** and
**owner** links from B to A in separate cases. B's real host exit must retire
B's complete set while A's HWND, frozen owner, backing, A-targeted queued
work, timers and typed payloads remain unclaimed. A-authored work targeting a
B window must still be canceled when that receiver is destroyed. A same-birth
child/owned-window exit
control must still retire the full B set, and a normal guest `DestroyWindow`
control must still recurse through its valid same-birth subtree. Check the
stopped host-exit path, and separately check the pseudo path's exact target
selection and no-birth refusal without claiming its caller authority; preserve
the pre-fix red log and then run focused, full release-feature, no-default and
alternate-feature checks on a frozen source. Tests prove identity isolation,
not CE6 permission for a cross-process parent/owner topology.

[O-G4](../DECISIONS.md) remains open for CE6 topology and the fate of a
foreign window's link to a retired parent/owner. An exact-set teardown must
not silently destroy or reparent that foreign HWND to make the fixture pass;
its visual visibility after a synthetic invalid link is not a CE6 claim.
Partial host-exit teardown is not a successful completion. Currently the
stopped path mutates windows/backing before its process-completion step can
still fail, and ignores framebuffer-restore success. This gate records those
risks and does not claim atomic whole-process exit or framebuffer rollback;
they need a separate preflight/recovery gate. This source gate neither accepts
full Stage 3
nor authorizes a writable-mount iNavi run.

### Bounded sender-owned send transaction (source-tested)

The [sender-ownership receipt](../../evidence/stage3-sender-birth-20261015/validation.receipt.md)
seals the tested Send/Timeout transaction from captured raw/native caller
birth through atomic GWES publication, exact process/thread-birth exit
selection, and native wait/result/re-entry checks. `SentMessage` has one
sender-owner field: immutable authenticated `Birth`, explicit numeric
fixture-compatible `LegacyUnknown`, or `SystemNoWait`. Numeric IDs for `Birth`
are derived, not a second owner; the legacy-unknown process ID remains mutable
for compatibility. An
invalid captured birth fails before a send ID, queue/status/wake, trace or
result-pointer effect. Active cleanup detaches the sender result pointer while
allowing the receiver callback to finish. The synthetic stranded-A queued and
active fixtures prove B's reused PID cannot cancel A; they do not claim normal
A exit leaves a stranded send. All three original expected-red controls are
enabled after the fix.

The same bounded source gate preflights identified guest-originated no-wait
producers before raw side effects: accelerator command, delayed clipboard
render, and time/display/settings/font broadcasts. Their queued messages
remain `SystemNoWait`, without sender wait/result ownership. Host/device
notifications keep their system origin. The final 18-group suites passed
release **2,067/0/9 ignored** and no-default **1,495/0/3 ignored**. Direct
WndProc dispatch, conditional system-origin device broadcasts from storage
exports, receiver-side cross-process result writes, recursive foreign
parent/owner traversal, general stale-native mutation safety, full Stage 3
and live-app behavior remain separate gates. The local CE6 process-exit
send-cancellation rule is not established from `C:\WINCE600` or device dumps;
these controls prove emulator birth-identity safety, not CE6 parity.

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
