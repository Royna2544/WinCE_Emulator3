//! D-007 historical Stage 3 E-8 task-ownership controls.
//! These call real Kernel lifecycle consumers, not a test-only state model.
//! Alias-sharing controls are active; creation/primary identity reds stay ignored.

use wince_emulation_v3::{
    Result,
    ce::{
        kernel::CeKernel,
        object::{ThreadResumeResult, ThreadSuspendResult},
        scheduler::SchedulerBlockedWaitKind,
        timer::{INFINITE, WAIT_OBJECT_0},
    },
    config::RuntimeConfig,
};

fn kernel() -> Result<CeKernel> {
    Ok(CeKernel::boot(RuntimeConfig::load_default()?))
}

#[test]
fn task_ownership_thread_aliases_share_ce_priority() -> Result<()> {
    let mut kernel = kernel()?;
    let (original, tid) = kernel.create_guest_thread(0x1000, 0x2000, false);
    let (neighbor, _) = kernel.create_guest_thread(0x3000, 0x4000, false);
    let initial_neighbor = kernel.thread_priority(neighbor);
    let alias = kernel.duplicate_handle(original, kernel.current_process_id(), false)?;
    assert_eq!(kernel.guest_thread_id(alias), Some(tid));
    assert!(kernel.set_thread_ce_priority(original, 42));
    assert_eq!(kernel.thread_priority(alias), Some(42));
    assert!(kernel.set_thread_ce_priority(alias, 73));
    assert_eq!(kernel.thread_priority(original), Some(73));
    assert_eq!(kernel.thread_priority_by_id(tid), 73);
    assert_eq!(kernel.thread_priority(neighbor), initial_neighbor);
    Ok(())
}

#[test]
fn task_ownership_thread_aliases_share_suspend_count() -> Result<()> {
    let mut kernel = kernel()?;
    let (original, _) = kernel.create_guest_thread(0x1000, 0x2000, false);
    let (neighbor, _) = kernel.create_guest_thread(0x3000, 0x4000, false);
    let alias = kernel.duplicate_handle(original, kernel.current_process_id(), false)?;
    assert_eq!(
        kernel.suspend_thread(alias),
        ThreadSuspendResult::Previous(0)
    );
    assert!(kernel.guest_thread_start(original).is_none());
    assert!(kernel.guest_thread_start(alias).is_none());
    assert!(kernel.guest_thread_start(neighbor).is_some());
    assert_eq!(
        kernel.suspend_thread(original),
        ThreadSuspendResult::Previous(1)
    );
    assert_eq!(kernel.resume_thread(alias), ThreadResumeResult::Previous(2));
    assert!(kernel.guest_thread_start(original).is_none());
    assert_eq!(
        kernel.resume_thread(original),
        ThreadResumeResult::Previous(1)
    );
    assert!(kernel.guest_thread_start(alias).is_some());
    Ok(())
}

#[test]
fn task_ownership_thread_aliases_share_exit_and_wait_signal() -> Result<()> {
    let mut kernel = kernel()?;
    let (original, tid) = kernel.create_guest_thread(0x1000, 0x2000, false);
    let (neighbor, neighbor_tid) = kernel.create_guest_thread(0x3000, 0x4000, false);
    let alias = kernel.duplicate_handle(original, kernel.current_process_id(), false)?;
    let wait_id = kernel.register_blocked_waiter(
        1,
        0,
        vec![alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert_eq!(kernel.is_wait_ready(alias, 1), Some(false));
    let wake_candidates_before = kernel.scheduler_stats().object_wake_candidate_count;
    assert!(kernel.mark_guest_thread_exited(original, 0x1234));
    assert_eq!(kernel.guest_thread_id(alias), Some(tid));
    assert_eq!(kernel.guest_thread_exit_code(alias), Some(0x1234));
    assert_eq!(kernel.is_wait_ready(alias, 1), Some(true));
    assert_eq!(kernel.wait_for_single_object(alias, 0, 1), WAIT_OBJECT_0);
    assert_eq!(kernel.is_wait_ready(original, 1), Some(true));
    assert_eq!(kernel.is_wait_ready(neighbor, 1), Some(false));
    assert_eq!(
        kernel.scheduler_stats().object_wake_candidate_count,
        wake_candidates_before + 1,
        "exit must wake a wait registered through the thread alias",
    );
    assert_eq!(
        kernel.select_ready_blocked_waiter(neighbor_tid, 0, |wait, kernel| {
            kernel.is_wait_ready(wait.wait_handles[0], wait.thread_id) == Some(true)
        }),
        Some(wait_id),
    );
    assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    Ok(())
}

#[test]
fn task_ownership_duplicate_close_source_keeps_thread_alive() -> Result<()> {
    let mut kernel = kernel()?;
    let (original, tid) = kernel.create_guest_thread(0x1000, 0x2000, false);
    assert!(kernel.set_thread_ce_priority(original, 47));
    let alias = kernel.duplicate_handle(original, kernel.current_process_id(), true)?;
    assert_eq!(kernel.guest_thread_id(original), None);
    assert_eq!(kernel.guest_thread_id(alias), Some(tid));
    assert_eq!(kernel.thread_priority(alias), Some(47));
    assert_eq!(
        kernel.guest_thread_start(alias),
        Some((tid, 0x1000, 0x2000))
    );
    assert!(kernel.mark_guest_thread_exited(alias, 0x5678));
    assert_eq!(kernel.guest_thread_exit_code(alias), Some(0x5678));
    Ok(())
}

#[test]
#[ignore = "expected red: worker creation owner and Toolhelp still use selected PID"]
fn task_ownership_pid_is_creation_owner_not_selected_process() -> Result<()> {
    let mut kernel = kernel()?;
    assert_eq!(kernel.current_process_id(), 1);
    let (_, parent_worker) = kernel.create_guest_thread(0x1000, 0, false);
    let launch = kernel.queue_process_launch(None, None);
    assert_ne!(launch.process_id, 1);
    kernel.set_current_process_id(launch.process_id);
    let (_, child_worker) = kernel.create_guest_thread(0x2000, 0, false);
    // Production consumes the launch queue before child execution.
    let queued = kernel.take_pending_process_launches();
    assert_eq!(queued.len(), 1);
    let snapshots = kernel.toolhelp_thread_snapshots(launch.thread_id);
    let owner = |tid| {
        snapshots
            .iter()
            .find(|thread| thread.thread_id == tid)
            .map(|thread| thread.owner_process_id)
    };
    assert_eq!(owner(parent_worker), Some(1));
    assert_eq!(owner(launch.thread_id), Some(launch.process_id));
    assert_eq!(owner(child_worker), Some(launch.process_id));
    kernel.set_current_process_id(1);
    let snapshots = kernel.toolhelp_thread_snapshots(1);
    assert_eq!(
        snapshots
            .iter()
            .find(|thread| thread.thread_id == child_worker)
            .map(|thread| thread.owner_process_id),
        Some(launch.process_id),
    );
    Ok(())
}

#[test]
#[ignore = "expected red: boot main lacks execution-owned Task record without public alias"]
fn task_ownership_boot_main_is_retained_without_a_public_thread_handle() -> Result<()> {
    let mut kernel = kernel()?;
    assert_eq!(kernel.guest_thread_handle_by_id(1), None);
    assert!(kernel.is_primary_thread_for_current_process(1));
    let launch = kernel.queue_process_launch(None, None);
    kernel.take_pending_process_launches();
    kernel.set_current_process_id(launch.process_id);
    let snapshots = kernel.toolhelp_thread_snapshots(launch.thread_id);
    assert_eq!(
        snapshots
            .iter()
            .find(|thread| thread.thread_id == 1)
            .map(|thread| thread.owner_process_id),
        Some(1),
    );
    Ok(())
}

#[test]
#[ignore = "expected red: child primary identity still depends on launch queue or trace ring"]
fn task_ownership_primary_identity_outlives_launch_queue_and_trace_ring() -> Result<()> {
    let mut kernel = kernel()?;
    let first = kernel.queue_process_launch(None, None);
    kernel.take_pending_process_launches();
    kernel.set_current_process_id(first.process_id);
    assert!(kernel.is_primary_thread_for_current_process(first.thread_id));
    // Queue real generic lifecycle requests only; no app is launched or guest
    // message invented. This exceeds the existing 128-record diagnostic ring.
    for _ in 0..129 {
        kernel.queue_process_launch(None, None);
    }
    kernel.take_pending_process_launches();
    assert!(kernel.is_primary_thread_for_current_process(first.thread_id));
    assert!(!kernel.is_primary_thread_for_current_process(1));
    Ok(())
}

#[test]
fn task_ownership_cloned_kernel_does_not_share_mutable_thread_state() -> Result<()> {
    let mut original = kernel()?;
    let (handle, tid) = original.create_guest_thread(0x1000, 0, false);
    let initial_priority = original.thread_priority(handle);
    let alias = original.duplicate_handle(handle, original.current_process_id(), false)?;
    let mut cloned = original.clone();
    assert!(cloned.set_thread_ce_priority(alias, 35));
    assert_eq!(cloned.thread_priority(handle), Some(35));
    assert_eq!(original.thread_priority(handle), initial_priority);
    assert_eq!(original.thread_priority(alias), initial_priority);
    assert_eq!(original.guest_thread_id(alias), Some(tid));
    assert!(cloned.mark_guest_thread_exited(alias, 0x9999));
    assert_eq!(cloned.is_wait_ready(handle, 1), Some(true));
    assert_eq!(original.is_wait_ready(handle, 1), Some(false));
    Ok(())
}
