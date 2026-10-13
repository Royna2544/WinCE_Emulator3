# Resources, shell and COM support

Common anchor: [ARCHITECTURE.md](../ARCHITECTURE.md), A1–A4/A6.
Implementation order: [PLAN.md](../PLAN.md). Target interfaces are proposals.

## Current boundary and coverage

[resource.rs](../../src/ce/resource.rs), [shell.rs](../../src/ce/shell.rs),
[com.rs](../../src/ce/com.rs) and [ole.rs](../../src/ce/ole.rs) implement
resource/icon/version/shell/COM-facing state and helpers. COREDLL/CRT shims
expose the ABI; PE loading supplies module/resource metadata; GWE/GDI render
actual guest-driven UI. Notification/COM/enum callbacks currently participate
in CPU per-API continuation containers rather than one completed Task model.
Pure math/text/ordinal helpers do not introduce another machine authority.

## Target ownership and interfaces

Module resource bytes belong to their AddressSpace/module lifetime. Resource,
notification and COM identities resolve through canonical object ownership;
guest pointers are checked views, not permanent unversioned host references.
Shell launch emits a typed process-creation request to the machine; it cannot
execute a child through an uncontrolled inline CPU path. Parent/child loading,
execution and return ordering must follow the audited CE6 lifecycle. Local
NK/KERNEL/process.c resumes the new thread and waits for loading before the
creation call returns; do not impose universal parent-return-before-child order.

Resource parsing produces checked metadata/bytes; GDI handles drawing and GWE
message delivery. COM/notification/enum callback requests identify TaskIdentity
and FrameIdentity and enter through the shared API/continuation boundary.
No subsystem paints app readiness, bypasses a missing object or invents a
callback merely to produce a desired screenshot.

## Failure and release contract

Reference counting, unregister/update/close, receiver termination, failed
creation and module detach have explicit per-object/frame release rules.
Datafile/no-resolve resources do not silently become executable module state.
Prepare guest allocations/output before metadata/pointer publication; retained
input/view lifetimes must outlast native use. Failed/stale receiver delivery
does not consume unrelated work. A published live partial byte output is not
promised rollback. Unknown interface/module/payload stays an explicit failure.

## Plan, evidence and gates

Inventory resource/module and COM/notification ownership, then migrate one
complete lifecycle including ABI, queue, callback, error and release consumers.
Use dumped CE6 binaries/headers for exact wire layout (NotifyIcon152bytes is
already source-backed), and existing raw/native shell/resource regressions.
Test malformed resources, module detach, stale handles, callback-selected
icons, adjacent allocation sentinels, nested callbacks and receiver teardown.
Current raw coverage is not complete shell/taskbar/COM parity or map readiness.
