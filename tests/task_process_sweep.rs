//! Stage 3 process-exit boundary controls. A termination request records the
//! requested code without signaling a process wait; only verified execution
//! completion signals it and retires workers/fixed-stack backing.

use wince_emulation_v3::{
    Result,
    ce::{
        gwe::{Rect, WS_VISIBLE},
        kernel::{CeKernel, STILL_ACTIVE},
        memory::VirtualAllocationSource,
        scheduler::{SchedulerBlockedWaitKind, WaitObjectKey},
        timer::{INFINITE, WAIT_OBJECT_0},
    },
    config::RuntimeConfig,
};

fn boot() -> Result<CeKernel> {
    Ok(CeKernel::boot(RuntimeConfig::load_default()?))
}

#[test]
fn termination_request_waits_for_verified_execution_completion() -> Result<()> {
    let mut kernel = boot()?;
    let launch = kernel.queue_process_launch(None, None);
    let child_birth = kernel
        .handles
        .process_birth_key(launch.process_id)
        .expect("queued child birth");
    // The parent is selected while the child primary creates its worker.
    kernel.set_current_process_id(1);
    let (worker_handle, worker_tid) =
        kernel.try_create_guest_thread_for_creator(launch.thread_id, 0x401000, 0, false)?;
    assert_eq!(
        kernel.handles.thread_creation_owner_by_id(worker_tid),
        Some(child_birth)
    );
    let process_alias = kernel
        .open_process_handle(launch.process_id)
        .expect("child process alias");
    let process_wait = kernel.register_blocked_waiter(
        1,
        0,
        vec![process_alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    let wakes_before = kernel.scheduler_stats().object_wake_candidate_count;

    assert!(kernel.terminate_process(launch.process_handle, 0x55));
    assert_eq!(kernel.process_exit_code(process_alias), Some(STILL_ACTIVE));
    assert_eq!(kernel.is_wait_ready(process_alias, 1), Some(false));
    assert_eq!(
        kernel.scheduler_stats().object_wake_candidate_count,
        wakes_before
    );
    assert!(kernel.handles.thread_execution_live_by_id(worker_tid));
    assert_eq!(kernel.is_wait_ready(worker_handle, 1), Some(false));

    let signals_after_first_request = kernel.scheduler_stats().object_signal_count;
    assert!(kernel.terminate_process(process_alias, 0x77));
    assert_eq!(
        kernel.scheduler_stats().object_wake_candidate_count,
        wakes_before,
        "repeated termination request must not enqueue a completion wake"
    );
    assert_eq!(
        kernel.scheduler_stats().object_signal_count,
        signals_after_first_request
    );
    assert!(kernel.handles.thread_execution_live_by_id(worker_tid));

    assert!(kernel.mark_process_launch_exited(&launch, 0x88));
    assert_eq!(kernel.process_exit_code(process_alias), Some(0x55));
    assert_eq!(kernel.is_wait_ready(process_alias, 1), Some(true));
    assert_eq!(
        kernel.scheduler_stats().object_wake_candidate_count,
        wakes_before + 1
    );
    assert!(kernel.remove_blocked_waiter(process_wait).is_some());
    Ok(())
}

#[test]
fn execution_completion_sweeps_only_its_birth_and_wakes_waiters_once() -> Result<()> {
    let mut kernel = boot()?;
    let parent_birth = kernel.current_process_birth_key().expect("boot birth");
    let launch = kernel.queue_process_launch(None, None);
    let child_birth = kernel
        .handles
        .process_birth_key(launch.process_id)
        .expect("queued child birth");

    // Mutate the selected PID opposite the actual creator in both directions.
    kernel.set_current_process_id(launch.process_id);
    let (parent_worker_handle, parent_worker_tid) =
        kernel.try_create_guest_thread_for_creator(1, 0x402000, 0, false)?;
    kernel.set_current_process_id(1);
    let (first_handle, first_tid) =
        kernel.try_create_guest_thread_for_creator(launch.thread_id, 0x403000, 0, false)?;
    let (second_handle, second_tid) =
        kernel.try_create_guest_thread_for_creator(launch.thread_id, 0x404000, 0, false)?;
    assert_eq!(
        kernel
            .handles
            .thread_creation_owner_by_id(parent_worker_tid),
        Some(parent_birth)
    );
    for child_tid in [launch.thread_id, first_tid, second_tid] {
        assert_eq!(
            kernel.handles.thread_creation_owner_by_id(child_tid),
            Some(child_birth)
        );
    }

    let first_alias = kernel.duplicate_handle(first_handle, 1, false)?;
    assert!(kernel.close_handle(first_handle)?);
    let second_birth = kernel
        .handles
        .thread_birth_key_by_id(second_tid)
        .expect("second child worker birth");
    let aliasless_worker_wait = kernel.register_blocked_waiter(
        1,
        0,
        vec![second_handle],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert!(kernel.close_handle(second_handle)?);
    assert_eq!(kernel.guest_thread_handle_by_id(second_tid), None);
    assert!(kernel.handles.contains_thread_birth(second_birth));
    assert_eq!(
        kernel
            .blocked_waiter(aliasless_worker_wait)
            .map(|wait| wait.wait_keys.as_slice()),
        Some([WaitObjectKey::Thread(second_birth)].as_slice()),
        "the closed alias's blocked wait must retain the worker birth as its wake key"
    );

    let process_alias = kernel
        .open_process_handle(launch.process_id)
        .expect("child process alias");
    assert!(kernel.close_handle(launch.process_handle)?);
    let process_wait = kernel.register_blocked_waiter(
        1,
        0,
        vec![process_alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    let worker_wait = kernel.register_blocked_waiter(
        parent_worker_tid,
        parent_worker_handle,
        vec![first_alias],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert_eq!(kernel.is_wait_ready(first_alias, 1), Some(false));
    let queued = kernel.take_pending_process_launches();
    assert_eq!(queued.as_slice(), &[launch.clone()]);
    let wakes_before = kernel.scheduler_stats().object_wake_candidate_count;

    // TerminateProcess is a request boundary, not proof that parked or active
    // child CPU execution has ended. The worker wait must remain pending here.
    assert!(kernel.terminate_process(process_alias, 0x66));
    assert_eq!(kernel.is_wait_ready(process_alias, 1), Some(false));
    assert_eq!(kernel.is_wait_ready(first_alias, 1), Some(false));
    assert!(kernel.handles.contains_thread_birth(second_birth));
    assert!(kernel.blocked_waiter(aliasless_worker_wait).is_some());
    assert_eq!(
        kernel.scheduler_stats().object_wake_candidate_count,
        wakes_before
    );

    kernel.set_current_process_id(parent_birth.process_id());
    kernel.mark_process_launch_exited(&queued[0], 0x66);
    // CE6 NKPrepareThreadExit kills secondary workers with code 0; the
    // requested process code belongs to the primary and process object.
    assert_eq!(kernel.guest_thread_exit_code(first_alias), Some(0));
    assert_eq!(kernel.is_wait_ready(first_alias, 1), Some(true));
    assert_eq!(
        kernel.wait_for_single_object(first_alias, 0, 1),
        WAIT_OBJECT_0
    );
    assert!(!kernel.handles.contains_thread_birth(second_birth));
    assert_eq!(kernel.is_wait_ready(launch.thread_handle, 1), Some(true));
    assert!(
        kernel
            .handles
            .thread_execution_live_by_id(parent_worker_tid)
    );
    assert_eq!(kernel.is_wait_ready(parent_worker_handle, 1), Some(false));
    assert_eq!(
        kernel.scheduler_stats().object_wake_candidate_count,
        wakes_before + 3,
        "the process, retained-alias worker, and aliasless worker waits should each queue once"
    );
    // The aliasless wait's canonical key can still enqueue a candidate. E11
    // wait-target pins are not implemented, so this does not yet prove that
    // the closed handle can complete its blocked wait after object retirement.
    assert!(kernel.blocked_waiter(aliasless_worker_wait).is_some());

    let after_first_completion = kernel.scheduler_stats();
    kernel.mark_process_launch_exited(&queued[0], 0x99);
    let after_repeated_completion = kernel.scheduler_stats();
    assert_eq!(
        after_repeated_completion.object_wake_candidate_count,
        after_first_completion.object_wake_candidate_count
    );
    assert_eq!(
        after_repeated_completion.object_signal_count, after_first_completion.object_signal_count,
        "repeated completion must not signal an already completed birth"
    );
    assert_eq!(kernel.guest_thread_exit_code(first_alias), Some(0));
    assert_eq!(kernel.process_exit_code(process_alias), Some(0x66));

    assert!(kernel.remove_blocked_waiter(worker_wait).is_some());
    assert!(
        kernel
            .remove_blocked_waiter(aliasless_worker_wait)
            .is_some()
    );
    assert!(kernel.remove_blocked_waiter(process_wait).is_some());
    assert!(kernel.close_handle(first_alias)?);
    assert!(kernel.close_handle(launch.thread_handle)?);
    assert!(kernel.close_handle(process_alias)?);
    assert!(!kernel.handles.contains_process_birth(child_birth));
    assert_eq!(kernel.open_process_handle(launch.process_id), None);
    Ok(())
}

#[test]
fn fixed_stack_budget_retires_only_completed_birth_not_termination_request() -> Result<()> {
    const STACK_BASE: u32 = 0x7100_0000;
    const STACK_SIZE: u32 = 0x2000;
    const PAGE_BYTES: u64 = 0x1000;
    let mut kernel = boot()?;
    let parent_birth = kernel.current_process_birth_key().expect("boot birth");
    let launch = kernel.queue_process_launch(None, None);
    let child_birth = kernel
        .handles
        .process_birth_key(launch.process_id)
        .expect("queued child birth");
    let initial_budget = kernel.memory.private_commit_used_bytes();

    // The same VA belongs to two distinct process births, so a PID selector
    // or VA-only sweep would also remove the parent's committed page.
    for owner in [parent_birth, child_birth] {
        assert_eq!(
            kernel.memory.reserve_fixed_private_range(
                owner,
                VirtualAllocationSource::ThreadStack,
                STACK_BASE,
                STACK_SIZE,
                0x04,
            ),
            Some(STACK_BASE)
        );
        let prepared = kernel
            .memory
            .prepare_fixed_private_page(owner, STACK_BASE + 4)
            .expect("first stack touch");
        assert_eq!(
            kernel.memory.commit_prepared_private_vm(prepared),
            Some(STACK_BASE)
        );
    }
    assert_eq!(
        kernel.memory.private_commit_used_bytes(),
        initial_budget + 2 * PAGE_BYTES
    );

    kernel.set_current_process_id(parent_birth.process_id());
    assert!(kernel.terminate_process(launch.process_handle, 0x77));
    assert!(
        kernel
            .memory
            .private_page_pin(child_birth, STACK_BASE)
            .is_some()
    );
    assert!(
        kernel
            .memory
            .private_page_pin(parent_birth, STACK_BASE)
            .is_some()
    );
    assert_eq!(
        kernel.memory.private_commit_used_bytes(),
        initial_budget + 2 * PAGE_BYTES,
        "termination request alone must not release a still-running child stack"
    );

    kernel.mark_process_launch_exited(&launch, 0x77);
    assert!(
        kernel
            .memory
            .virtual_allocation(child_birth, STACK_BASE)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .private_page_pin(child_birth, STACK_BASE)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .virtual_allocation(parent_birth, STACK_BASE)
            .is_some()
    );
    assert!(
        kernel
            .memory
            .private_page_pin(parent_birth, STACK_BASE)
            .is_some()
    );
    assert_eq!(
        kernel.memory.private_commit_used_bytes(),
        initial_budget + PAGE_BYTES
    );
    Ok(())
}

#[test]
fn stale_completion_does_not_touch_reused_pid_and_tid_birth() -> Result<()> {
    const STACK_BASE: u32 = 0x7100_0000;
    let mut kernel = boot()?;
    let launch = kernel.queue_process_launch(None, None);
    let old_birth = kernel
        .handles
        .process_birth_key(launch.process_id)
        .expect("original child birth");
    kernel.take_pending_process_launches();
    kernel.mark_process_launch_exited(&launch, 0x11);
    assert!(kernel.close_handle(launch.thread_handle)?);
    assert!(kernel.close_handle(launch.process_handle)?);
    assert!(!kernel.handles.contains_process_birth(old_birth));
    assert_eq!(
        kernel.handles.thread_birth_key_by_id(launch.thread_id),
        None
    );

    // Deliberately reuse the exact public numbers in the canonical directory.
    // A stale launch completion must carry and check the old birth keys.
    let replacement_process_handle = kernel.handles.create_process(launch.process_id);
    let replacement_birth = kernel
        .handles
        .process_birth_key_for_handle(replacement_process_handle)
        .expect("replacement process birth");
    assert_ne!(replacement_birth, old_birth);
    let replacement_thread_handle = kernel.handles.try_create_thread_in_birth(
        replacement_birth,
        launch.thread_id,
        0x405000,
        0,
        false,
    )?;
    assert_eq!(
        kernel.memory.reserve_fixed_private_range(
            replacement_birth,
            VirtualAllocationSource::ThreadStack,
            STACK_BASE,
            0x2000,
            0x04,
        ),
        Some(STACK_BASE)
    );
    let prepared = kernel
        .memory
        .prepare_fixed_private_page(replacement_birth, STACK_BASE + 4)
        .expect("replacement stack touch");
    assert_eq!(
        kernel.memory.commit_prepared_private_vm(prepared),
        Some(STACK_BASE)
    );
    let replacement_budget = kernel.memory.private_commit_used_bytes();

    kernel.mark_process_launch_exited(&launch, 0x22);
    assert_eq!(
        kernel.handles.process_signaled_by_id(launch.process_id),
        Some(false),
        "old completion signaled the replacement process"
    );
    assert!(kernel.handles.thread_execution_live_by_id(launch.thread_id));
    assert_eq!(
        kernel.handles.thread_creation_owner_by_id(launch.thread_id),
        Some(replacement_birth)
    );
    assert_eq!(
        kernel.is_wait_ready(replacement_thread_handle, 1),
        Some(false)
    );
    assert!(
        kernel
            .memory
            .private_page_pin(replacement_birth, STACK_BASE)
            .is_some(),
        "old completion retired the replacement stack"
    );
    assert_eq!(
        kernel.memory.private_commit_used_bytes(),
        replacement_budget
    );
    Ok(())
}

#[test]
fn reconstructed_launch_without_birth_cannot_retire_live_process() -> Result<()> {
    let mut kernel = boot()?;
    let launch = kernel.queue_process_launch(None, None);
    let birth = launch.process_birth.expect("original launch birth");
    let (worker_handle, worker_tid) =
        kernel.try_create_guest_thread_for_creator(launch.thread_id, 0x406000, 0, false)?;
    let mut reconstructed = launch.clone();
    reconstructed.process_birth = None;

    assert!(!kernel.mark_process_launch_exited(&reconstructed, 0x33));
    assert!(kernel.handles.process_execution_live_by_birth(birth));
    assert!(kernel.handles.thread_execution_live_by_id(launch.thread_id));
    assert!(kernel.handles.thread_execution_live_by_id(worker_tid));
    assert_eq!(kernel.is_wait_ready(worker_handle, 1), Some(false));
    assert!(!kernel.recent_process_ops().iter().any(|record| {
        record.op == "CreateProcessExited" && record.process_id == Some(launch.process_id)
    }));

    assert!(kernel.mark_process_launch_exited(&launch, 0x33));
    assert!(!kernel.handles.process_execution_live_by_birth(birth));
    Ok(())
}

#[test]
fn process_completion_removes_owned_waits_but_wakes_external_waiters() -> Result<()> {
    let mut kernel = boot()?;
    let launch = kernel.queue_process_launch(None, None);
    let (worker_handle, worker_tid) =
        kernel.try_create_guest_thread_for_creator(launch.thread_id, 0x408000, 0, false)?;
    let owned_wait = kernel.register_blocked_waiter(
        worker_tid,
        worker_handle,
        vec![launch.process_handle],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    let external_wait = kernel.register_blocked_waiter(
        1,
        0,
        vec![worker_handle],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    let wakes_before = kernel.scheduler_stats().object_wake_candidate_count;

    assert!(kernel.mark_process_launch_exited(&launch, 0x44));
    assert!(kernel.blocked_waiter(owned_wait).is_none());
    assert!(kernel.blocked_waiter(external_wait).is_some());
    assert!(kernel.scheduler_stats().object_wake_candidate_count > wakes_before);
    assert_eq!(kernel.is_wait_ready(worker_handle, 1), Some(true));
    Ok(())
}

#[test]
fn process_completion_does_not_destroy_window_of_reused_primary_tid() -> Result<()> {
    let mut kernel = boot()?;
    let launch = kernel.queue_process_launch(None, None);
    let old_window = kernel.gwe.create_window_ex_with_process_and_rect(
        launch.thread_id,
        launch.process_id,
        "old_child",
        "old",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 100),
    );
    assert_ne!(old_window, 0);

    assert!(kernel.mark_guest_thread_exited_by_id(launch.thread_id, 0));
    assert!(kernel.close_handle(launch.thread_handle)?);
    assert!(
        kernel
            .handles
            .thread_birth_key_by_id(launch.thread_id)
            .is_none()
    );
    let unrelated_process_handle = kernel.handles.create_process(99);
    let unrelated_birth = kernel
        .handles
        .process_birth_key_for_handle(unrelated_process_handle)
        .expect("unrelated process birth");
    let unrelated_thread_handle = kernel.handles.try_create_thread_in_birth(
        unrelated_birth,
        launch.thread_id,
        0x409000,
        0,
        false,
    )?;
    let unrelated_window = kernel.gwe.create_window_ex_with_process_and_rect(
        launch.thread_id,
        unrelated_birth.process_id(),
        "new_child",
        "new",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 100),
    );
    assert_ne!(unrelated_window, 0);

    assert!(kernel.mark_process_launch_exited(&launch, 0x55));
    assert!(
        kernel
            .gwe
            .window(old_window)
            .is_none_or(|window| window.destroyed)
    );
    assert!(
        kernel
            .gwe
            .window(unrelated_window)
            .is_some_and(|window| !window.destroyed)
    );
    assert!(kernel.handles.thread_execution_live_by_id(launch.thread_id));
    assert_eq!(
        kernel.is_wait_ready(unrelated_thread_handle, 1),
        Some(false)
    );
    Ok(())
}
