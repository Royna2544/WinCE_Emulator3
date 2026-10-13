# Stale snapshot host decisions: proposed red boundary (2026-10-14)

Status: design only. No fixture, production change, Cargo run, or live-app
result is claimed here.

## Two distinct consumers

`main.rs::host_idle_message_poll_slice` caps the host desktop's idle-message
slice at 100 ms (or a shorter explicit limit) when the last CPU snapshot
*looks* like only a GetMessage
wait and no CPU context is saved. It does not inspect the live Kernel wait ID.
The separate `should_idle_host_message_pump` already checks a snapshot wait
ID against `CeKernel`. A stale snapshot can therefore still select the
shorter pacing path even after its wait was removed. This is a host pacing
classification, not proof of guest delivery or a native stop cause.

`main.rs::blocked_remote_input_target` has a different risk. If there is no
active visible receiver work, it prefers `snapshot.blocked_get_message` when
its HWND is still visible. Visibility does not establish that the captured
thread is still waiting. The live Kernel-based
`saved_message_remote_input_target` fallback is skipped, so an obsolete
snapshot may route a remote tap to the old thread/window ahead of a live
GetMessage waiter. `UnicornBlockedGetMessage` also represents modal and popup
message loops; treating it as only a GetMessage wait would regress those
routes. HTTP acceptance remains distinct from guest WNDPROC consumption.

## First deterministic expected-red fixture: routing

Add one `src/main.rs` unit test next to the existing remote-target tests.
Use the real `CeKernel::boot`, two real visible nonzero-area windows, and
`register_blocked_waiter` rather than a fabricated guest outcome:

1. Register GetMessage wait `old_id` for thread 7 / `old_hwnd` and capture a
   `UnicornDebugSnapshot` containing `blocked_get_message` for that same
   thread/HWND and an `active_blocked_waits` record with `id = old_id`.
2. Remove `old_id` from the Kernel, leaving `old_hwnd` visible. Register a
   new GetMessage wait `live_id` for thread 8 / `live_hwnd`; assert the old ID
   is absent and the new ID is present.
3. With no active visible receiver work, assert the selected
   `BlockedRemoteInputTarget` is thread 8 / `live_hwnd`. The current
   snapshot-first predicate instead selects thread 7 / `old_hwnd`.

`UnicornMips::last_debug` is private to the library, so the small test seam
should extract the existing target-selection body into a private helper that
takes `(active_visible_receiver_work, Option<&UnicornDebugSnapshot>,
&CeKernel)`. The production wrapper must pass its actual CPU state. Keep the
old selection logic in the helper for the expected-red run; do not make the
test green by substituting the desired answer. Preserve source and binary
hashes plus failing assertion before changing the predicate.

## Bounded repair and verification

Validate a snapshot candidate against a currently registered Kernel wait for
the captured thread and relevant kind/target before preferring it; if absent,
fall through to the existing live waiter selection. Preserve live modal and
popup routes (the snapshot is not exclusively GetMessage), and do not post,
consume, replay, or synthesize a remote event in this guard. A same-thread,
same-HWND replacement frame could still alias under a match on TID/HWND;
exact frame/wait-ID provenance needs a separate contract if this narrow guard
is insufficient. Add a positive control for a live snapshot candidate and
one modal/popup control before claiming route preservation. Run focused
red/green, nearby remote-target tests, and current broader gates on the final
source. No live iNavi interpretation follows from these unit tests alone.

The host-pacing case should have its *own* red using the already present
`idle_poll_requires_a_live_get_message_waiter` setup: remove the captured
wait ID while leaving the snapshot unchanged, then require the ordinary
host slice instead of `HOST_IDLE_MESSAGE_POLL_SLICE_MS`. That wrapper also
needs a snapshot/Kernel test seam because its current signature receives only
`UnicornMips` and desktop mode. Do not conflate its timing assertion with the
remote routing assertion or change scheduling priority as part of either fix.
