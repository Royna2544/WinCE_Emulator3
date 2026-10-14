//! Stage 3 O-K3 execution- and wait-pin controls.

use wince_emulation_v3::{
    Result,
    ce::{
        kernel::CeKernel,
        object::TaskWaitPin,
        scheduler::SchedulerBlockedWaitKind,
        timer::{INFINITE, WAIT_OBJECT_0},
    },
    config::RuntimeConfig,
};

fn boot() -> Result<CeKernel> {
    Ok(CeKernel::boot(RuntimeConfig::load_default()?))
}

fn thread_pin(kernel: &CeKernel, handle: u32) -> TaskWaitPin {
    TaskWaitPin::Thread(
        kernel
            .handles
            .thread_birth_key_for_handle(handle)
            .expect("thread handle must name a live birth"),
    )
}

#[test]
fn live_task_survives_closing_its_final_public_handle() -> Result<()> {
    let mut kernel = boot()?;
    let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0x55, false);
    let pin = thread_pin(&kernel, handle);
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
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));

    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .any(|task| task.thread_id == thread_id && !task.signaled),
        "closing the last public alias lost a live task's execution-owned record"
    );
    assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0x1234));
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .all(|task| task.thread_id != thread_id),
        "a completed aliasless task with no wait pins must retire"
    );
    Ok(())
}

#[test]
fn closing_final_public_thread_alias_preserves_execution_and_wait_pins() -> Result<()> {
    let mut kernel = boot()?;
    let (handle, thread_id) = kernel.create_guest_thread(0x401000, 0x55, false);
    let pin = thread_pin(&kernel, handle);
    let wait_id = kernel.register_blocked_waiter(
        77,
        0,
        vec![handle],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert_eq!(kernel.guest_thread_id(handle), Some(thread_id));
    assert!(kernel.close_handle(handle)?);
    assert_eq!(kernel.guest_thread_id(handle), None);
    assert_eq!(
        kernel
            .blocked_waiter(wait_id)
            .map(|wait| wait.wait_handles.as_slice()),
        Some([handle].as_slice()),
        "the scheduler retains the ordered public handle for API reporting"
    );
    assert!(!kernel.registered_wait_has_ready_handle(wait_id));
    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .any(|thread| thread.thread_id == thread_id && !thread.signaled),
        "the execution-owned thread must survive final public close"
    );
    assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0x5678));
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert!(kernel.registered_wait_has_ready_handle(wait_id));
    assert_eq!(
        kernel.wait_for_registered_wait_without_scheduler_record(wait_id, 77),
        WAIT_OBJECT_0
    );
    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .any(|thread| thread.thread_id == thread_id && thread.signaled),
        "the registered wait must retain the completed target"
    );
    assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .all(|thread| thread.thread_id != thread_id),
        "the completed target must retire after its final wait pin releases"
    );

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
    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .any(|thread| thread.thread_id == sibling_thread_id && !thread.signaled)
    );
    assert!(kernel.mark_guest_thread_exited_by_id(sibling_thread_id, 0));
    Ok(())
}

#[test]
fn thread_exit_through_one_alias_indexes_waiter_on_another_alias() -> Result<()> {
    let mut kernel = boot()?;
    let (source, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
    let pin = thread_pin(&kernel, source);
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
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
    assert!(kernel.close_handle(source)?);
    assert!(kernel.close_handle(alias)?);
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    assert_eq!(kernel.open_thread_handle(thread_id), None);
    Ok(())
}

#[test]
fn thread_exit_by_id_wakes_surviving_alias_after_source_close() -> Result<()> {
    let mut kernel = boot()?;
    let (source, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
    let pin = thread_pin(&kernel, source);
    let alias = kernel.duplicate_handle(source, kernel.current_process_id(), false)?;
    let wait_id = kernel.register_blocked_waiter(
        1,
        0,
        vec![alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert!(kernel.close_handle(source)?);
    assert_eq!(kernel.guest_thread_id(source), None);
    assert_eq!(kernel.guest_thread_id(alias), Some(thread_id));
    let before = kernel.scheduler.stats().object_wake_candidate_count;

    assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0x5678));
    assert_eq!(
        kernel.scheduler.stats().object_wake_candidate_count,
        before + 1,
        "exit by TID must enqueue the waiter registered on the surviving alias"
    );
    assert_eq!(kernel.is_wait_ready(alias, 1), Some(true));
    assert_eq!(kernel.guest_thread_exit_code(alias), Some(0x5678));
    assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0x9999));
    assert_eq!(
        kernel.scheduler.stats().object_wake_candidate_count,
        before + 1,
        "a repeated exit must not enqueue the same waiter again"
    );
    assert_eq!(
        kernel.guest_thread_exit_code(alias),
        Some(0x5678),
        "a repeated exit must preserve the first exit code"
    );

    // Release the wait pin before final close.
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
    assert!(kernel.close_handle(alias)?);
    assert_eq!(kernel.open_thread_handle(thread_id), None);
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    Ok(())
}

#[test]
fn closed_thread_alias_wait_keeps_an_exited_target_pinned() -> Result<()> {
    let mut kernel = boot()?;
    let (source, thread_id) = kernel.create_guest_thread(0x401000, 0, false);
    let pin = thread_pin(&kernel, source);
    let alias = kernel.duplicate_handle(source, kernel.current_process_id(), false)?;
    let wait_id = kernel.register_blocked_waiter(
        1,
        0,
        vec![alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert!(kernel.close_handle(alias)?);
    assert!(kernel.mark_guest_thread_exited(source, 0x5678));
    assert!(kernel.close_handle(source)?);
    assert_eq!(kernel.guest_thread_id(alias), None);
    assert_eq!(kernel.guest_thread_id(source), None);
    assert!(kernel.blocked_waiter(wait_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert_eq!(kernel.handles.wait_target_signaled(pin), Some(true));
    assert!(kernel.registered_wait_has_ready_handle(wait_id));
    assert_eq!(
        kernel.wait_for_registered_wait_without_scheduler_record(wait_id, 1),
        WAIT_OBJECT_0
    );

    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .any(|task| task.thread_id == thread_id && task.signaled && task.exit_code == 0x5678),
        "a registered wait did not retain its exited target after both public aliases closed"
    );
    assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    assert!(
        kernel
            .handles
            .thread_snapshots()
            .iter()
            .all(|task| task.thread_id != thread_id),
        "a completed aliasless target must retire after the registered wait ends"
    );
    Ok(())
}

#[test]
fn duplicate_wait_slots_hold_two_pins_and_release_each_once() -> Result<()> {
    let mut kernel = boot()?;
    let (source, thread_id) = kernel.create_guest_thread(0x403000, 0, false);
    let alias = kernel.duplicate_handle(source, kernel.current_process_id(), false)?;
    let pin = thread_pin(&kernel, source);
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));

    let wait_id = kernel
        .try_register_blocked_waiter(
            77,
            0,
            vec![source, alias],
            SchedulerBlockedWaitKind::Kernel,
            0,
            INFINITE,
        )
        .expect("both aliases name the same live target");
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(2));
    assert_eq!(
        kernel.blocked_waiter(wait_id).unwrap().wait_handles,
        vec![source, alias]
    );

    assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
    assert!(kernel.remove_blocked_waiter(wait_id).is_none());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
    assert!(kernel.close_handle(source)?);
    assert!(kernel.close_handle(alias)?);
    assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0));
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    Ok(())
}

#[test]
fn invalid_second_wait_target_rolls_back_first_pin_without_publishing_wait() -> Result<()> {
    let mut kernel = boot()?;
    let (source, thread_id) = kernel.create_guest_thread(0x404000, 0, false);
    let pin = thread_pin(&kernel, source);
    let before = kernel.scheduler.waiter_count();
    assert_eq!(
        kernel.try_register_blocked_waiter(
            77,
            0,
            vec![source, 0xdead_beef],
            SchedulerBlockedWaitKind::Kernel,
            0,
            INFINITE,
        ),
        None
    );
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
    assert_eq!(kernel.scheduler.waiter_count(), before);
    assert!(kernel.close_handle(source)?);
    assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0));
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    Ok(())
}

#[test]
fn timeout_and_cancellation_release_registered_wait_pins() -> Result<()> {
    let mut kernel = boot()?;
    let (source, thread_id) = kernel.create_guest_thread(0x405000, 0, false);
    let pin = thread_pin(&kernel, source);
    let timeout_id = kernel
        .try_register_blocked_waiter(
            77,
            0,
            vec![source],
            SchedulerBlockedWaitKind::Kernel,
            100,
            25,
        )
        .expect("valid thread wait");
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert_eq!(
        kernel.select_ready_blocked_waiter(0, 124, |_, _| false),
        None
    );
    assert_eq!(
        kernel.select_ready_blocked_waiter(0, 125, |_, _| false),
        Some(timeout_id)
    );
    assert!(kernel.remove_blocked_waiter(timeout_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));

    let _cancel_id = kernel
        .try_register_blocked_waiter(
            77,
            0,
            vec![source],
            SchedulerBlockedWaitKind::Kernel,
            200,
            INFINITE,
        )
        .expect("valid thread wait");
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert_eq!(kernel.remove_blocked_waiters_for_thread(77), 1);
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
    assert_eq!(kernel.remove_blocked_waiters_for_thread(77), 0);
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(0));
    assert!(kernel.close_handle(source)?);
    assert!(kernel.mark_guest_thread_exited_by_id(thread_id, 0));
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    Ok(())
}

#[test]
fn closed_process_alias_wait_stays_bound_to_completed_birth() -> Result<()> {
    let mut kernel = boot()?;
    let launch = kernel.queue_process_launch(None, None);
    let pin = TaskWaitPin::Process(
        kernel
            .handles
            .process_birth_key_for_handle(launch.process_handle)
            .expect("queued child process birth"),
    );
    let wait_id = kernel
        .try_register_blocked_waiter(
            1,
            0,
            vec![launch.process_handle],
            SchedulerBlockedWaitKind::Kernel,
            0,
            INFINITE,
        )
        .expect("valid child process wait");
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert!(kernel.close_handle(launch.process_handle)?);
    assert!(!kernel.registered_wait_has_ready_handle(wait_id));

    assert!(kernel.mark_process_launch_exited(&launch, 0x7a));
    assert_eq!(kernel.handles.wait_target_signaled(pin), Some(true));
    assert_eq!(kernel.handles.wait_target_pin_count(pin), Some(1));
    assert!(kernel.registered_wait_has_ready_handle(wait_id));
    assert_eq!(
        kernel.wait_for_registered_wait_without_scheduler_record(wait_id, 1),
        WAIT_OBJECT_0
    );
    assert!(kernel.remove_blocked_waiter(wait_id).is_some());
    assert_eq!(kernel.handles.wait_target_pin_count(pin), None);
    assert!(
        kernel
            .handles
            .process_birth_key(launch.process_id)
            .is_none()
    );
    assert!(kernel.close_handle(launch.thread_handle)?);
    Ok(())
}
