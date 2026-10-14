//! D-007 historical Stage 3 E-11 execution-pin controls.
//! Alias-indexed wake is active; execution and wait-target pins remain red.

use wince_emulation_v3::{
    Result,
    ce::{kernel::CeKernel, scheduler::SchedulerBlockedWaitKind, timer::INFINITE},
    config::RuntimeConfig,
};

fn boot() -> Result<CeKernel> {
    Ok(CeKernel::boot(RuntimeConfig::load_default()?))
}

#[test]
#[ignore = "expected red: live Task needs execution ownership after final public close"]
fn live_task_survives_closing_its_final_public_handle() -> Result<()> {
    let mut kernel = boot()?;
    let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0x55, false);
    assert_eq!(
        kernel.guest_thread_start(handle),
        Some((thread_id, 0x401000, 0x55))
    );
    assert!(kernel.close_handle(handle)?);
    assert_eq!(
        kernel.guest_thread_id(handle),
        None,
        "the public alias must be closed"
    );

    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .any(|task| task.thread_id == thread_id && !task.signaled),
        "closing the last public alias lost a live task's execution-owned record"
    );
    Ok(())
}

#[test]
fn closing_final_public_thread_alias_preserves_only_the_raw_wait_record() -> Result<()> {
    let mut kernel = boot()?;
    let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0x55, false);
    let wait_id = kernel.register_blocked_waiter(
        77,
        0,
        vec![handle],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert_eq!(kernel.guest_thread_id(handle), Some(thread_id));
    assert!(kernel.close_handle(handle)?);
    assert_eq!(kernel.guest_thread_id(handle), None);
    assert_eq!(
        kernel
            .blocked_waiter(wait_id)
            .map(|wait| wait.wait_handles.as_slice()),
        Some([handle].as_slice()),
        "the scheduler's existing raw wait record is not a Task/object pin"
    );
    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .all(|thread| thread.thread_id != thread_id),
        "the public thread snapshot has no remaining alias"
    );
    assert!(kernel.remove_blocked_waiter(wait_id).is_some());

    // Closing one of two public aliases must not close its sibling.
    let (source, sibling_thread_id) = kernel.create_guest_thread(0x402000, 0x66, false);
    let alias = kernel.duplicate_handle(source, kernel.current_process_id(), false)?;
    assert_ne!(alias, source);
    assert!(kernel.close_handle(source)?);
    assert_eq!(kernel.guest_thread_id(source), None);
    assert_eq!(kernel.guest_thread_id(alias), Some(sibling_thread_id));
    assert_eq!(
        kernel.guest_thread_start(alias),
        Some((sibling_thread_id, 0x402000, 0x66))
    );
    assert!(kernel.close_handle(alias)?);
    assert_eq!(kernel.guest_thread_id(alias), None);
    Ok(())
}

#[test]
fn thread_exit_through_one_alias_indexes_waiter_on_another_alias() -> Result<()> {
    let mut kernel = boot()?;
    let (source, _thread_id) = kernel.create_guest_thread(0x401000, 0, false);
    let alias = kernel.duplicate_handle(source, kernel.current_process_id(), false)?;
    let wait_id = kernel.register_blocked_waiter(
        1,
        0,
        vec![alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert_eq!(kernel.scheduler.waiter_ids_for_handle(alias), vec![wait_id]);
    let before = kernel.scheduler.stats().object_wake_candidate_count;

    assert!(kernel.mark_guest_thread_exited(source, 0x1234));
    assert_eq!(
        kernel.scheduler.stats().object_wake_candidate_count,
        before + 1,
        "exit through the source alias did not enqueue the wait indexed by its sibling alias"
    );
    assert_eq!(kernel.is_wait_ready(alias, 1), Some(true));
    Ok(())
}

#[test]
#[ignore = "expected red: registered wait needs canonical target pin after alias close"]
fn closed_thread_alias_wait_keeps_an_exited_target_pinned() -> Result<()> {
    let mut kernel = boot()?;
    let (source, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
    let alias = kernel.duplicate_handle(source, kernel.current_process_id(), false)?;
    let wait_id = kernel.register_blocked_waiter(
        1,
        0,
        vec![alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert!(kernel.close_handle(alias)?);
    assert!(kernel.mark_guest_thread_exited(source, 0x5678));
    assert!(kernel.close_handle(source)?);
    assert_eq!(kernel.guest_thread_id(alias), None);
    assert_eq!(kernel.guest_thread_id(source), None);
    assert!(kernel.blocked_waiter(wait_id).is_some());

    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .any(|task| task.thread_id == thread_id && task.signaled && task.exit_code == 0x5678),
        "a registered wait did not retain its exited target after both public aliases closed"
    );
    Ok(())
}
