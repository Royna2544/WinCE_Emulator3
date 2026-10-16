# GDI resources and framebuffer presentation

Status: design-first subsystem plan. [Architecture](../ARCHITECTURE.md) alone defines the proposed shared contracts; [Plan](../PLAN.md) controls migration. This document applies those contracts and does not declare new Rust modules/types implemented.

## Responsibility and current implementation

Rendering translates genuine guest drawing into pixels. It does not decide task readiness, dismiss splash, fabricate map content or replace a missing guest callback with a cosmetically complete screen.

Current GDI objects/DC selection/save stacks/clips are part of `ResourceSystem` in [resource.rs](../../src/ce/resource.rs), not a standalone gdi.rs. Bitmap metadata includes guest bits pointer, owned/writable flags, format/masks and section linkage. API drawing/serialization spans [coredll.rs](../../src/ce/coredll.rs) and Kernel resource helpers. [framebuffer.rs](../../src/ce/framebuffer.rs) provides the Framebuffer interface, RGB565 default 800×480 virtual pixel store, stride/format checks, dirty rectangles, backing-store snapshots and PPM serialization. [desktop.rs](../../src/ce/desktop.rs) supplies input/presenter abstractions; platform desktops and remote server copy/encode pixels.

GWE separately owns invalidation/paint windows and caret state. The native runner currently borrows raw framebuffer pointers into per-slice hooks for ticks/drawing. The persistent native owner must remove those hooks before their borrow ends; rendering references cannot become process-lifetime native callbacks.

Section-backed DIB bits may still use a legacy copied mapping while that DIB
view is live. The first public file view is permitted only after the legacy
views are deleted; it then migrates the whole legacy byte store into VM-owned
shared pages and drops the old copy. A later DIB request on that slab-backed
mapping fails rather than creating a second writable authority. This is a
focused D008 source/control boundary, not GDI rendering or map-screen acceptance;
the [memory page](guest-memory.md) owns the backing and budget rules.

## Target ownership and interfaces

| Owner | Responsibility | Must not own |
| --- | --- | --- |
| Machine authority | Serialize guest drawing, resource lifetime and presentation events | Host encoder's private queues or independently invented guest time |
| GDI/resource service | DC/object state, selection references, clip/ROP/transforms and guest bitmap relationships | Task scheduler or duplicated process-memory ownership |
| Address-space service | Bounds/permissions/backing ownership for guest bits and structures | GDI interpretation or implicit pointer provenance guesses |
| Surface/presentation service | Pixel storage/stride/dirty regions and immutable frame publication | Guest callback/frame identity or map readiness |
| Host presenter/encoder | Display/encode an owned published image | Mutation of live guest/DC/task state |

Keep ABI dispatch as a thin adapter. A drawing request resolves a legitimate DC/object and active address-space access, validates dimensions/stride/clip/source/destination extents, then executes the operation. Surface mutation and dirty marking belong to the same accepted operation. Window/DC coordinates and explicit bitmap/memory-DC coordinates must not be conflated by a global selected-process assumption.

Guest caret/timer changes use machine-supplied guest time; host video pacing uses the host contract. Presenting more frequently must not advance guest timers or create extra guest execution. The future shared snapshot contract couples published frame/window provenance; a PNG encoding timestamp today is not that guarantee.

## Failure and resource lifetime

Track selected-object references, save-stack references, bitmap-bit ownership, mapped-section references and backing stores explicitly before extracting the service. DeleteDC/DeleteObject/window destruction/process termination/checkpoint disposal need one release path per owned object; selecting a stock or borrowed object is not allocation ownership. Do not copy current bitmap `bits_owned` into an assumed process-heap policy without resolving its real allocation/backing contract.

Checked arithmetic must precede any out-of-bounds source/destination access or public pointer/handle publication. Guest-created bitmap/DIB failure releases only an unpublished candidate; borrowed bits/sections are not freed as owned heap. Faults propagate through authentic API/backend outcome rather than return success with partial invented pixels. Existing multi-write operations must state any non-atomic boundary; no guessed rollback.

Surface snapshots are owned byte copies. Host presenters/remote workers must never retain mutable guest-memory/framebuffer borrows. Checkpoint clone must keep pixel/resource state independent and reconcile guest backing through the address-space service. Overlay rendering must avoid drawing a caret twice when it is already in the framebuffer; verify existing flags and presentation behavior before consolidation.

The bounded [D-011 stopped-exit gate](../../evidence/stage3-host-exit-framebuffer-preflight-20261015/validation.receipt.md)
now preflights each frozen-birth saved window image against the current
destination's format, dimensions, stride and byte extent before copying or
retiring any exit-owned state. A malformed or incompatible later image cannot
partially restore earlier images: it returns typed recovery refusal with the
stopped owner retained. Focused framebuffer and Kernel controls include
already-destroyed HWND backing, short destination layout and a compatible
retry. This source gate does not prove whole-exit atomicity after preflight,
general drawing parity or live iNavi rendering.

### D-011 modal backing presence gate (bounded source/test cut)

The frozen GWE window-backing set is not the only display state.
`MessageBoxModalState` owns a separately captured rectangle, whose current
restore can silently return on an incompatible row stride; rendering can also
succeed without a captured backing. A synthetic blocked MessageBox can reach
stopped child completion. `TrackPopupMenuEx` can render and park in production,
but `PopupMenuModalState` has no captured backing. Neither should be silently
discarded because a GWE-only preflight passed.

The bounded [modal-presence source/test gate](../../evidence/stage3-modal-exit-failclosed-20261016/validation.receipt.md)
is **fail-closed**, not a restoration implementation. After exact-birth
authority selects an exit target, source preflight inspects the stopped CPU
and affected parked subtrees, including modal-bearing same-birth duplicates
and nested foreign-birth descendants. Their presence yields typed
`HostExitRecoveryRequired` before Kernel effects. Duplicate pruning retains
modal-bearing parked CPUs, and the terminal-child handoff checks completion
before selecting a runnable successor. The three preserved expected-red
controls for duplicate popup, nested foreign modal and parked boot popup are
green; 19-block release and no-default suites pass. That proves those tested
shapes, not every path in the source survey. The refusal is conservative even
when a MessageBox rectangle might be restorable. A descendant's modal
presence blocks its disposal; its numeric PID/TID does not authorize the
parent's exit. No claim covers an earlier sibling already completed in a
prune batch or makes the private successor helper globally queue-atomic.

The later checked-restoration cut must collect exact modal state and Kernel's
frozen-birth GWE stores and preflight **all** against one held destination
before any exit effect. Future restoration needs captured modal wait/thread
birth to match the terminating process birth. `BlockedModalMessageBox`
currently stores numeric thread ID/handle/wait ID, not a birth; the existing
exact-wait verifier is a starting point. An unowned or stale modal must never
authorize teardown. Persist the source pixel format with a MessageBox
capture: equal bytes per pixel or row stride alone cannot prove format
identity. Validate format, dimensions, stride, unclipped rectangle and pixel
extents; `rendered && backing_store.is_none()` is un-restorable. Popup
rendering needs owned backing/restore or a defined redraw contract, not a
silent dismissal. A compatible prepared restore may apply only after
whole-display preflight under the same destination lease. An unexpected
commit failure remains terminal, not a completed exit; checkpoint restore is
the recovery path.

Further expected-red controls should cover rendered MessageBox and popup
state in active and parked completions, a later GWE-plus-modal target in one
completion, natural and initial-child `Exited` routes, pseudo self-exit, and
terminal-child handoff. Keep existing GWE-only D-011 controls green in release
and no-default builds. Pending launches with no CPU/modal still use GWE-only
preflight. Successful completion with nonmodal descendant CPU queues needs a
separate ownership/hoisting gate. Already-exited parked CPU cleanup needs a
typed retention or quarantine rule for residual modal state; this later
cleanup cannot make an earlier external Kernel exit atomic. Compatible modal
retry, whole-display restoration, CE6 visual parity, post-preflight
whole-exit atomicity and iNavi map readiness remain open.

## Telemetry and visual evidence

Record genuine drawing API/operation, DC/object/window identity, resolved address-space owner, validated extent, result, dirty region and published frame sequence. Separate guest draw/paint completion from host encode/present time. Report pixels/allocated capacities/backing-store/native-engine memory independently; a small logical surface does not explain whole-host RSS.

Use hashes and bounded image comparison to detect changes, then inspect content. A changed splash background, WM_PAINT, nonblack framebuffer or fast presenter is not map readiness. In the 650 run (ledger E-2) the first decorative splash appeared at +4.800 s and the last frame at +128.892 s was still splash; no terminal PPM was produced after the host crash. BED2 (E-1) also stayed splash. These observations cannot be converted into a rendering success claim for navigation.

## Migration and acceptance

1. Inventory actual ResourceSystem/GWE/Coredll/Kernel drawing and ownership consumers; record selected/borrowed/owned lifetimes before introducing an object facade.
2. Put checked guest-memory views and DC/object resolution behind the address-space/resource boundary without changing pixel behavior.
3. Separate surface mutation from host presentation; remove persistent borrowed callbacks and publish immutable frames at machine boundaries.
4. Move one complete object/operation family including selection, save/restore, delete, debug and checkpoint behavior; preserve explicit compatibility adapters until all consumers migrate.

Acceptance includes all supported pixel formats/stride/padding, top-down/bottom-up DIBs, masks/palettes, clipping/transforms/ROP, overlapping blits, selected-object lifetime, SaveDC/RestoreDC, invalid handles/read-only bits, overflow/cross-boundary extents, section-backed DIB lifetime, paint invalidation/end-paint, caret overlay and independent checkpoint pixels. Add genuine native guest stores→draw→frame evidence and mutation-free concurrent encoding tests. Failure must not publish a successful new handle or unowned pointer.

Rendering qualification requires executed guest draw/paint paths and matching pixels; application map qualification additionally requires genuine splash dismissal and navigation behavior under real input. Resolve deletion/reference parity and exact host/guest presentation timing through the common plan before implementation.
