use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        audio::{MMSYSERR_NOERROR, WaveBuffer, WaveFormat, WaveOutState},
        com::{REGDB_E_CLASSNOTREG, S_FALSE, S_OK},
        devices::{CommDcb, CommTimeouts, PURGE_RXCLEAR, PURGE_TXCLEAR},
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        gwe::{
            GWL_USERDATA, MSGSRC_HARDWARE_KEYBOARD, MSGSRC_SOFTWARE_SEND, QS_POSTMESSAGE,
            QS_SENDMESSAGE, QS_TIMER, Rect, SMF_TIMEOUT, WA_ACTIVE, WM_ACTIVATE, WM_CHAR,
            WM_ERASEBKGND, WM_KILLFOCUS, WM_QUIT, WM_SETFOCUS, WM_TIMER, WM_USER, WS_CHILD,
            WS_POPUP, WS_VISIBLE,
        },
        kernel::{
            CE_CURRENT_PROCESS_PSEUDO_HANDLE, CE_CURRENT_THREAD_PSEUDO_HANDLE, CeKernel,
            MessagePumpResult,
        },
        object::{
            EventObject, KernelObject, MAX_SUSPEND_COUNT, MUTEX_MAX_LOCK_COUNT, ThreadResumeResult,
            ThreadSuspendResult,
        },
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE, REG_SZ},
        remote::{WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP},
        scheduler::SchedulerBlockedWaitKind,
        thread::{ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER},
        timer::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    emulator::{
        memory::{MemoryMap, MemoryPerms},
        unicorn::UnicornMips,
    },
};

mod support;
use support::unique_test_root;

#[test]
fn boots_and_smokes_basic_ce_subsystems() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let ident_key = kernel
        .registry
        .reg_open_key_exw(HKEY_LOCAL_MACHINE, Some("Ident"), 0, 0);
    assert_eq!(ident_key.status, ERROR_SUCCESS);
    let ident_key = ident_key.hkey.unwrap();

    let name = kernel
        .registry
        .reg_query_value_exw(ident_key, Some("Name"), Some(128));
    assert_eq!(name.status, ERROR_SUCCESS);
    assert_eq!(name.value_type, Some(REG_SZ));
    assert!(name.required_len > 2);
    assert_eq!(kernel.registry.reg_close_key(ident_key), ERROR_SUCCESS);

    let serial = kernel.devices.open("COM7")?;
    assert_eq!(serial.guest_name, "COM7:");

    let hwnd = kernel.gwe.create_window(42, "SmokeWindow", "smoke");
    assert!(kernel.gwe.get_message(42).is_none());
    let timer_id = kernel
        .timers
        .set_timer(42, Some(hwnd), Some(77), 0, WM_TIMER, None);
    assert_eq!(timer_id, 77);
    kernel.pump_timers_to_gwe(42);
    let timer_msg = kernel.gwe.get_message(42).unwrap();
    assert_eq!(timer_msg.hwnd, hwnd);
    assert_eq!(timer_msg.msg, WM_TIMER);
    assert_eq!(timer_msg.wparam, 77);

    let wave_id = kernel.audio.open_wave_out(WaveFormat::pcm_16bit(2, 44_100));
    assert!(kernel.audio.write(
        wave_id,
        WaveBuffer {
            guest_ptr: 0x2000,
            len: 512,
        },
    ));
    assert_eq!(
        kernel.audio.output(wave_id).unwrap().state,
        WaveOutState::Playing
    );
    assert_eq!(
        kernel
            .audio
            .complete_next_buffer(wave_id)
            .unwrap()
            .guest_ptr,
        0x2000
    );

    let event_handle = kernel.handles.insert(KernelObject::Event(EventObject {
        name: Some("smoke".to_owned()),
        manual_reset: true,
        signaled: false,
    }));
    assert!(matches!(
        kernel.handles.get(event_handle).unwrap(),
        KernelObject::Event(_)
    ));
    kernel.handles.close(event_handle)?;

    let mut cpu = UnicornMips::new()?;
    cpu.map_region(
        0x1000_0000,
        0x0001_0000,
        MemoryPerms::READ | MemoryPerms::WRITE,
        "smoke-ram",
    )?;
    assert_eq!(cpu.memory().regions().count(), 1);

    Ok(())
}

#[test]
fn com_subsystem_tracks_apartments_and_registered_class_creation() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    assert_eq!(kernel.com.co_initialize_ex(17, 0), S_OK);
    assert_eq!(kernel.com.co_initialize_ex(17, 0), S_FALSE);
    assert_eq!(
        kernel.com.co_create_instance(0x7000, 0x7010),
        Err(REGDB_E_CLASSNOTREG)
    );
    kernel.com.register_class(0x7000, 0x99);
    let object = kernel.com.co_create_instance(0x7000, 0x7010).unwrap();
    assert_eq!(kernel.com.object(object).unwrap().clsid_ptr, 0x7000);
    kernel.com.co_uninitialize(17);
    kernel.com.co_uninitialize(17);

    Ok(())
}

#[test]
fn memory_map_rejects_overlaps() -> Result<()> {
    let mut map = MemoryMap::default();
    map.map(0x2000_0000, 0x0001_0000, MemoryPerms::READ, "first")?;

    let overlap = map.map(0x2000_8000, 0x0001_0000, MemoryPerms::READ, "overlap");
    assert!(overlap.is_err());

    Ok(())
}

#[test]
fn memory_map_finds_containing_region() -> Result<()> {
    let mut map = MemoryMap::default();
    map.map(0x2000_0000, 0x0001_0000, MemoryPerms::READ, "first")?;

    let region = map.region_containing(0x2000_1234).unwrap();
    assert_eq!(region.name, "first");
    assert_eq!(region.base, 0x2000_0000);
    assert!(map.region_containing(0x2001_0000).is_none());

    Ok(())
}

#[test]
fn wait_for_multiple_validates_all_handles_before_acquiring() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let thread_id = 7;

    let auto_event = kernel.create_event_w(Some("auto-ready".to_owned()), false, true);
    let invalid_handle = 0xdead_beef;

    assert_eq!(
        kernel.wait_for_multiple_objects(&[auto_event, invalid_handle], false, 0, thread_id),
        WAIT_FAILED
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_HANDLE
    );
    assert_eq!(
        kernel.wait_for_single_object(auto_event, 0, thread_id),
        WAIT_OBJECT_0
    );

    let handles = vec![auto_event; 65];
    assert_eq!(
        kernel.wait_for_multiple_objects(&handles, false, 0, thread_id),
        WAIT_FAILED
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    Ok(())
}

#[test]
fn suspend_resume_thread_counts_follow_ce_cap() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let (thread, _thread_id) = kernel.create_guest_thread(0x1000, 0x2000, false);

    assert!(kernel.guest_thread_start(thread).is_some());
    for expected in 0..MAX_SUSPEND_COUNT {
        assert_eq!(
            kernel.suspend_thread(thread),
            ThreadSuspendResult::Previous(expected)
        );
        assert!(kernel.guest_thread_start(thread).is_none());
    }
    assert_eq!(
        kernel.suspend_thread(thread),
        ThreadSuspendResult::SignalRefused
    );
    assert_eq!(
        kernel.resume_thread(thread),
        ThreadResumeResult::Previous(MAX_SUSPEND_COUNT)
    );
    assert!(kernel.guest_thread_start(thread).is_none());
    for expected in (1..MAX_SUSPEND_COUNT).rev() {
        assert_eq!(
            kernel.resume_thread(thread),
            ThreadResumeResult::Previous(expected)
        );
    }
    assert!(kernel.guest_thread_start(thread).is_some());
    assert_eq!(
        kernel.resume_thread(thread),
        ThreadResumeResult::Previous(0)
    );

    Ok(())
}

#[test]
fn current_thread_pseudo_handle_updates_priority_and_suspend_state() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let main_thread_id = 1;

    assert_eq!(
        kernel.thread_win32_priority_for_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE, main_thread_id),
        Some(3)
    );
    assert!(kernel.set_thread_win32_priority_for_handle(
        CE_CURRENT_THREAD_PSEUDO_HANDLE,
        1,
        main_thread_id
    ));
    assert_eq!(
        kernel.thread_win32_priority_for_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE, main_thread_id),
        Some(1)
    );
    assert_eq!(
        kernel.thread_priority_for_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE, main_thread_id),
        Some(249)
    );
    assert_eq!(
        kernel.suspend_thread_for_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE, main_thread_id),
        ThreadSuspendResult::Previous(0)
    );
    assert_eq!(
        kernel.resume_thread_for_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE, main_thread_id),
        ThreadResumeResult::Previous(1)
    );

    let (worker_handle, worker_thread_id) = kernel.create_guest_thread(0x1000, 0x2000, false);
    assert!(kernel.set_thread_ce_priority_for_handle(
        CE_CURRENT_THREAD_PSEUDO_HANDLE,
        42,
        worker_thread_id
    ));
    assert_eq!(kernel.thread_priority(worker_handle), Some(42));
    assert_eq!(
        kernel.suspend_thread_for_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE, worker_thread_id),
        ThreadSuspendResult::Previous(0)
    );
    assert!(kernel.guest_thread_start(worker_handle).is_none());
    assert_eq!(
        kernel.resume_thread_for_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE, worker_thread_id),
        ThreadResumeResult::Previous(1)
    );
    assert!(kernel.guest_thread_start(worker_handle).is_some());

    Ok(())
}

#[test]
fn current_process_pseudo_handle_is_waitable_after_terminate() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let thread_id = 7;

    assert_eq!(
        kernel.is_wait_ready(CE_CURRENT_PROCESS_PSEUDO_HANDLE, thread_id),
        Some(false)
    );
    assert_eq!(
        kernel.wait_for_single_object(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0, thread_id),
        WAIT_TIMEOUT
    );
    assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x1234));
    assert_eq!(
        kernel.is_wait_ready(CE_CURRENT_PROCESS_PSEUDO_HANDLE, thread_id),
        Some(true)
    );
    assert_eq!(
        kernel.wait_for_single_object(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0, thread_id),
        WAIT_OBJECT_0
    );
    assert_eq!(
        kernel.process_exit_code_for_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE),
        Some(0x1234)
    );

    Ok(())
}

#[test]
fn mutex_waits_track_recursive_owner_lock_count() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mutex = kernel.create_mutex_w(Some("recursive-mx".to_owned()), None);

    assert_eq!(kernel.wait_for_single_object(mutex, 0, 7), WAIT_OBJECT_0);
    assert_eq!(kernel.wait_for_single_object(mutex, 0, 7), WAIT_OBJECT_0);
    assert_eq!(kernel.wait_for_single_object(mutex, 0, 8), WAIT_TIMEOUT);

    let KernelObject::Mutex(state) = kernel.handles.get(mutex)? else {
        panic!("expected mutex object")
    };
    assert_eq!(state.owner_thread, Some(7));
    assert_eq!(state.lock_count, 2);

    assert!(!kernel.release_mutex(mutex, 8));
    assert!(kernel.release_mutex(mutex, 7));
    assert_eq!(kernel.wait_for_single_object(mutex, 0, 8), WAIT_TIMEOUT);

    let KernelObject::Mutex(state) = kernel.handles.get(mutex)? else {
        panic!("expected mutex object")
    };
    assert_eq!(state.owner_thread, Some(7));
    assert_eq!(state.lock_count, 1);

    assert!(kernel.release_mutex(mutex, 7));
    assert_eq!(kernel.wait_for_single_object(mutex, 0, 8), WAIT_OBJECT_0);

    Ok(())
}

#[test]
fn mutex_recursive_wait_fails_at_ce_max_lock_count() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mutex = kernel.create_mutex_w(None, Some(7));
    let KernelObject::Mutex(state) = kernel.handles.get_mut(mutex)? else {
        panic!("expected mutex object")
    };
    state.lock_count = MUTEX_MAX_LOCK_COUNT;

    assert_eq!(kernel.wait_for_single_object(mutex, 0, 7), WAIT_FAILED);
    assert_eq!(kernel.threads.get_last_error(7), ERROR_INVALID_HANDLE);

    Ok(())
}

#[test]
fn object_transitions_queue_scheduler_wait_candidates() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let event = kernel.create_event_w(Some("wake_event".to_owned()), false, false);
    let event_wait = kernel.register_blocked_waiter(
        8,
        0x108,
        vec![event],
        SchedulerBlockedWaitKind::Kernel,
        10,
        INFINITE,
    );
    assert!(kernel.set_event(event));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 10, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(event_wait)
    );
    let stats = kernel.scheduler_stats();
    assert_eq!(stats.object_signal_count, 1);
    assert_eq!(stats.object_wake_candidate_count, 1);
    assert_eq!(stats.max_pending_wakes, 1);
    kernel.remove_blocked_waiter(event_wait).unwrap();

    let semaphore = kernel
        .create_semaphore_w(Some("wake_sem".to_owned()), 0, 2)
        .unwrap();
    let sem_wait = kernel.register_blocked_waiter(
        9,
        0x109,
        vec![semaphore],
        SchedulerBlockedWaitKind::Kernel,
        20,
        INFINITE,
    );
    assert_eq!(kernel.release_semaphore(semaphore, 1), Some(0));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 20, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(sem_wait)
    );
    kernel.remove_blocked_waiter(sem_wait).unwrap();

    let mutex = kernel.create_mutex_w(Some("wake_mutex".to_owned()), Some(7));
    assert_eq!(kernel.wait_for_single_object(mutex, 0, 7), WAIT_OBJECT_0);
    let mutex_wait = kernel.register_blocked_waiter(
        10,
        0x10a,
        vec![mutex],
        SchedulerBlockedWaitKind::Kernel,
        30,
        INFINITE,
    );
    assert!(kernel.release_mutex(mutex, 7));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 30, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        None
    );
    assert!(kernel.release_mutex(mutex, 7));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 30, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(mutex_wait)
    );

    let stats = kernel.scheduler_stats();
    assert_eq!(stats.object_signal_count, 3);
    assert_eq!(stats.object_wake_candidate_count, 3);
    Ok(())
}

#[test]
fn pulse_event_releases_registered_waiter_after_reset() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let event = kernel.create_event_w(Some("pulse_event".to_owned()), false, false);
    let wait_id = kernel.register_blocked_waiter(
        8,
        0x108,
        vec![event],
        SchedulerBlockedWaitKind::Kernel,
        10,
        INFINITE,
    );

    assert!(kernel.pulse_event(event));
    assert_eq!(kernel.is_wait_ready(event, 8), Some(false));
    assert_eq!(kernel.pulsed_wait_handle(wait_id), Some(event));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 10, |blocked, kernel| {
            kernel.pulsed_wait_handle(blocked.id).is_some()
                || blocked
                    .wait_handles
                    .iter()
                    .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(wait_id)
    );

    kernel.remove_blocked_waiter(wait_id).unwrap();
    assert_eq!(kernel.pulsed_wait_handle(wait_id), None);

    Ok(())
}

#[test]
fn thread_and_process_exit_queue_scheduler_wait_candidates() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let (thread_handle, _) = kernel.create_guest_thread(0x1000, 0, false);
    let thread_wait = kernel.register_blocked_waiter(
        11,
        0x10b,
        vec![thread_handle],
        SchedulerBlockedWaitKind::Kernel,
        40,
        INFINITE,
    );
    assert!(kernel.mark_guest_thread_exited(thread_handle, 0x55));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 40, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(thread_wait)
    );
    kernel.remove_blocked_waiter(thread_wait).unwrap();

    let launch = kernel.queue_process_launch(Some("fixture-child.exe".to_owned()), None);
    let process_wait = kernel.register_blocked_waiter(
        12,
        0x10c,
        vec![launch.process_handle],
        SchedulerBlockedWaitKind::Kernel,
        50,
        INFINITE,
    );
    let launch_thread_wait = kernel.register_blocked_waiter(
        13,
        0x10d,
        vec![launch.thread_handle],
        SchedulerBlockedWaitKind::Kernel,
        50,
        INFINITE,
    );
    kernel.mark_process_launch_exited(&launch, 0x66);
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 50, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(process_wait)
    );
    kernel.remove_blocked_waiter(process_wait).unwrap();
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 50, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(launch_thread_wait)
    );
    kernel.remove_blocked_waiter(launch_thread_wait).unwrap();

    let launch = kernel.queue_process_launch(Some("terminated-child.exe".to_owned()), None);
    let terminate_wait = kernel.register_blocked_waiter(
        14,
        0x10e,
        vec![launch.process_handle],
        SchedulerBlockedWaitKind::Kernel,
        60,
        INFINITE,
    );
    assert!(kernel.terminate_process(launch.process_handle, 0x77));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 60, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(terminate_wait)
    );
    kernel.remove_blocked_waiter(terminate_wait).unwrap();

    let current_process_wait = kernel.register_blocked_waiter(
        15,
        0x10f,
        vec![CE_CURRENT_PROCESS_PSEUDO_HANDLE],
        SchedulerBlockedWaitKind::Kernel,
        70,
        INFINITE,
    );
    assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x88));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 70, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(current_process_wait)
    );

    let stats = kernel.scheduler_stats();
    assert_eq!(stats.object_signal_count, 5);
    assert_eq!(stats.object_wake_candidate_count, 5);
    Ok(())
}

#[test]
fn message_and_timer_transitions_queue_scheduler_msg_wait_candidates() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let select_ready = |kernel: &CeKernel, active_thread_id: u32, now_ms: u32| {
        kernel.select_ready_blocked_waiter(
            active_thread_id,
            now_ms,
            |blocked, kernel| match blocked.kind {
                SchedulerBlockedWaitKind::Kernel => true,
                SchedulerBlockedWaitKind::Sleep => false,
                SchedulerBlockedWaitKind::SerialRead { handle } => kernel.serial_read_ready(handle),
                SchedulerBlockedWaitKind::SerialCommEvent { handle } => {
                    kernel.comm_event_mask_changed_wait(blocked.id)
                        || kernel.serial_comm_event_ready(handle)
                }
                SchedulerBlockedWaitKind::SendMessage { send_id } => {
                    kernel.sent_message_result_ready(send_id)
                }
                SchedulerBlockedWaitKind::GetMessage {
                    hwnd,
                    min_msg,
                    max_msg,
                } => kernel
                    .gwe
                    .has_message_filtered(blocked.thread_id, hwnd, min_msg, max_msg),
                SchedulerBlockedWaitKind::MsgWait {
                    wake_mask,
                    input_available,
                } => {
                    if input_available {
                        kernel.gwe.has_queue_input(blocked.thread_id, wake_mask)
                    } else {
                        kernel.gwe.has_new_queue_input(blocked.thread_id, wake_mask)
                    }
                }
            },
        )
    };
    let register_global_ready_wait = |kernel: &mut CeKernel, sequence: u32| {
        kernel.register_blocked_waiter(
            20 + sequence,
            0x200 + sequence,
            vec![0x400 + sequence],
            SchedulerBlockedWaitKind::Kernel,
            0,
            INFINITE,
        )
    };
    let register_msg_wait = |kernel: &mut CeKernel, thread_id: u32, wake_mask: u32| {
        kernel.register_blocked_waiter(
            thread_id,
            0x500 + thread_id,
            Vec::new(),
            SchedulerBlockedWaitKind::MsgWait {
                wake_mask,
                input_available: false,
            },
            0,
            INFINITE,
        )
    };

    let post_global = register_global_ready_wait(&mut kernel, 1);
    let post_wait = register_msg_wait(&mut kernel, 42, QS_POSTMESSAGE);
    assert!(kernel.post_thread_message_w(42, WM_USER + 10, 1, 2));
    assert_eq!(select_ready(&kernel, 1, 0), Some(post_wait));
    kernel.remove_blocked_waiter(post_wait).unwrap();
    kernel.remove_blocked_waiter(post_global).unwrap();
    assert_eq!(kernel.gwe.get_message(42).unwrap().msg, WM_USER + 10);

    let timer_global = register_global_ready_wait(&mut kernel, 2);
    let timer_wait = register_msg_wait(&mut kernel, 43, QS_TIMER);
    let hwnd = kernel.gwe.create_window(43, "MsgTimerWake", "timer");
    assert_eq!(kernel.set_timer(Some(hwnd), Some(77), 0), 77);
    kernel.pump_timers_to_gwe(43);
    assert_eq!(select_ready(&kernel, 1, 0), Some(timer_wait));
    kernel.remove_blocked_waiter(timer_wait).unwrap();
    kernel.remove_blocked_waiter(timer_global).unwrap();
    assert_eq!(kernel.get_message_w(43).unwrap().msg, WM_TIMER);

    let thread_timer_global = register_global_ready_wait(&mut kernel, 4);
    let thread_timer_wait = register_msg_wait(&mut kernel, 45, QS_TIMER);
    assert_eq!(kernel.set_timer_for_thread(45, None, Some(88), 0, None), 88);
    kernel.pump_timers_to_gwe(1);
    assert_eq!(select_ready(&kernel, 1, 0), Some(thread_timer_wait));
    kernel.remove_blocked_waiter(thread_timer_wait).unwrap();
    kernel.remove_blocked_waiter(thread_timer_global).unwrap();
    let thread_timer_msg = kernel.get_message_w(45).unwrap();
    assert_eq!(thread_timer_msg.hwnd, 0);
    assert_eq!(thread_timer_msg.msg, WM_TIMER);
    assert_eq!(thread_timer_msg.wparam, 88);

    let send_global = register_global_ready_wait(&mut kernel, 3);
    let send_wait = register_msg_wait(&mut kernel, 44, QS_SENDMESSAGE);
    let hwnd = kernel.gwe.create_window(44, "MsgSendWake", "send");
    assert!(
        kernel
            .begin_cross_thread_send_message_w(7, hwnd, WM_USER + 11, 3, 4, None)
            .is_some()
    );
    assert_eq!(select_ready(&kernel, 1, 0), Some(send_wait));
    kernel.remove_blocked_waiter(send_wait).unwrap();
    kernel.remove_blocked_waiter(send_global).unwrap();
    assert_eq!(kernel.gwe.get_message(44).unwrap().msg, WM_USER + 11);

    let quit_global = register_global_ready_wait(&mut kernel, 4);
    let quit_wait = register_msg_wait(&mut kernel, 45, QS_POSTMESSAGE);
    kernel.post_quit_message(45, 0x45);
    assert_eq!(select_ready(&kernel, 1, 0), Some(quit_wait));
    kernel.remove_blocked_waiter(quit_wait).unwrap();
    kernel.remove_blocked_waiter(quit_global).unwrap();
    assert_eq!(kernel.gwe.get_message(45).unwrap().msg, WM_QUIT);

    let stats = kernel.scheduler_stats();
    assert_eq!(stats.message_input_signal_count, 6);
    assert_eq!(stats.message_input_wake_candidate_count, 5);
    Ok(())
}

#[test]
fn window_timers_with_same_id_keep_independent_owners() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let hwnd_a = kernel.gwe.create_window(61, "TimerA", "a");
    let hwnd_b = kernel.gwe.create_window(62, "TimerB", "b");
    assert_eq!(kernel.set_timer(Some(hwnd_a), Some(5), 0), 5);
    assert_eq!(kernel.set_timer(Some(hwnd_b), Some(5), 0), 5);
    assert_eq!(kernel.timers.timer_count(), 2);

    assert!(kernel.kill_timer(Some(hwnd_a), 5));
    assert_eq!(kernel.timers.timer_count(), 1);

    kernel.pump_timers_to_gwe(62);
    assert!(kernel.gwe.get_message(61).is_none());
    let timer_msg = kernel.gwe.get_message(62).unwrap();
    assert_eq!(timer_msg.hwnd, hwnd_b);
    assert_eq!(timer_msg.msg, WM_TIMER);
    assert_eq!(timer_msg.wparam, 5);

    Ok(())
}

#[test]
fn kernel_timer_messages_coalesce_while_pending() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let hwnd = kernel.gwe.create_window(61, "TimerCoalesce", "timer");
    assert_eq!(kernel.set_timer(Some(hwnd), Some(77), 0), 77);

    kernel.pump_timers_to_gwe(61);
    kernel.pump_timers_to_gwe(61);

    let queued_timers = kernel
        .gwe
        .queue_snapshot()
        .into_iter()
        .find(|(thread_id, _)| *thread_id == 61)
        .map(|(_, messages)| {
            messages
                .into_iter()
                .filter(|message| message.msg == WM_TIMER && message.wparam == 77)
                .count()
        })
        .unwrap_or(0);
    assert_eq!(queued_timers, 1);

    let timer_msg = kernel.get_message_w(61).unwrap();
    assert_eq!(timer_msg.hwnd, hwnd);
    assert_eq!(timer_msg.msg, WM_TIMER);
    assert_eq!(timer_msg.wparam, 77);

    kernel.pump_timers_to_gwe(61);
    assert_eq!(kernel.get_message_w(61).unwrap().msg, WM_TIMER);

    Ok(())
}

#[test]
fn destroy_window_removes_hwnd_timers_but_keeps_thread_timers() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let hwnd = kernel.gwe.create_window(63, "TimerDestroy", "timer");
    assert_eq!(kernel.set_timer(Some(hwnd), Some(9), 0), 9);
    assert_eq!(kernel.set_timer_for_thread(63, None, Some(9), 0, None), 9);
    assert_eq!(kernel.timers.timer_count(), 2);

    assert!(kernel.destroy_window(hwnd));
    let remaining = kernel.timers.pending_timers();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].hwnd, None);
    assert_eq!(remaining[0].thread_id, 63);
    assert_eq!(remaining[0].id, 9);

    Ok(())
}

#[test]
fn destroy_owner_removes_owned_popup_timers() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let owner = kernel
        .gwe
        .create_window_ex_with_process_parent_owner_and_rect(
            1,
            64,
            "TimerOwner",
            "owner",
            None,
            None,
            WS_VISIBLE,
            0,
            0,
            Rect::from_origin_size(0, 0, 100, 100),
        );
    let owned = kernel
        .gwe
        .create_window_ex_with_process_parent_owner_and_rect(
            1,
            64,
            "TimerOwned",
            "owned",
            None,
            Some(owner),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            Rect::from_origin_size(0, 0, 100, 100),
        );
    assert_eq!(kernel.set_timer(Some(owned), Some(0x19fe), 0), 0x19fe);
    assert_eq!(kernel.timers.timer_count(), 1);

    assert!(kernel.destroy_window(owner));
    assert!(!kernel.gwe.is_window(owner));
    assert!(!kernel.gwe.is_window(owned));
    assert_eq!(kernel.timers.timer_count(), 0);
    kernel.pump_timers_to_gwe(64);
    assert!(kernel.gwe.get_message(64).is_none());

    Ok(())
}

#[test]
fn send_message_transitions_queue_scheduler_reply_wait_candidates() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let select_send_wait =
        |kernel: &CeKernel, active_thread_id: u32, now_ms: u32| {
            kernel.select_ready_blocked_waiter(active_thread_id, now_ms, |blocked, kernel| {
                match blocked.kind {
                    SchedulerBlockedWaitKind::SendMessage { send_id } => {
                        kernel.sent_message_result_ready(send_id)
                    }
                    _ => false,
                }
            })
        };
    let register_send_wait = |kernel: &mut CeKernel, sender_thread: u32, send_id: u64| {
        kernel.register_blocked_waiter(
            sender_thread,
            0x700 + sender_thread,
            Vec::new(),
            SchedulerBlockedWaitKind::SendMessage { send_id },
            0,
            INFINITE,
        )
    };

    let completion_sender = 60;
    let completion_receiver = 61;
    let completion_hwnd = kernel
        .gwe
        .create_window(completion_receiver, "SendCompleteWake", "send");
    let completion_send = kernel
        .begin_cross_thread_send_message_w(
            completion_sender,
            completion_hwnd,
            WM_ERASEBKGND,
            0,
            0,
            Some(500),
        )
        .expect("queued completing send");
    assert_ne!(
        kernel
            .gwe
            .sent_message(completion_send)
            .expect("completion send state")
            .flags
            & SMF_TIMEOUT,
        0
    );
    let completion_wait = register_send_wait(&mut kernel, completion_sender, completion_send);
    let message = kernel.gwe.get_message(completion_receiver).unwrap();
    assert_eq!(
        kernel.dispatch_message_w_for_thread(completion_receiver, message),
        1
    );
    assert_eq!(select_send_wait(&kernel, 1, 0), Some(completion_wait));
    kernel.remove_blocked_waiter(completion_wait).unwrap();
    assert_eq!(
        kernel.take_completed_send_message_result(completion_send),
        Some(1)
    );

    let timeout_sender = 62;
    let timeout_receiver = 63;
    let timeout_hwnd = kernel
        .gwe
        .create_window(timeout_receiver, "SendTimeoutWake", "send");
    let timeout_send = kernel
        .begin_cross_thread_send_message_w(
            timeout_sender,
            timeout_hwnd,
            WM_USER + 63,
            0,
            0,
            Some(0),
        )
        .expect("queued timeout send");
    assert_ne!(
        kernel
            .gwe
            .sent_message(timeout_send)
            .expect("timeout send state")
            .flags
            & SMF_TIMEOUT,
        0
    );
    let timeout_wait = register_send_wait(&mut kernel, timeout_sender, timeout_send);
    assert_eq!(kernel.expire_timed_out_send_messages(), vec![timeout_send]);
    assert_eq!(select_send_wait(&kernel, 1, 0), Some(timeout_wait));
    kernel.remove_blocked_waiter(timeout_wait).unwrap();
    assert_eq!(
        kernel.take_completed_send_message_result(timeout_send),
        Some(0)
    );

    let destroy_sender = 64;
    let destroy_receiver = 65;
    let destroy_hwnd = kernel
        .gwe
        .create_window(destroy_receiver, "SendDestroyWake", "send");
    let destroy_send = kernel
        .begin_cross_thread_send_message_w(
            destroy_sender,
            destroy_hwnd,
            WM_USER + 65,
            0,
            0,
            Some(500),
        )
        .expect("queued destroy send");
    let destroy_wait = register_send_wait(&mut kernel, destroy_sender, destroy_send);
    assert!(kernel.destroy_window(destroy_hwnd));
    assert_eq!(select_send_wait(&kernel, 1, 0), Some(destroy_wait));
    kernel.remove_blocked_waiter(destroy_wait).unwrap();
    assert_eq!(
        kernel.take_completed_send_message_result(destroy_send),
        Some(0)
    );

    let stats = kernel.scheduler_stats();
    assert_eq!(stats.send_reply_signal_count, 3);
    assert_eq!(stats.send_reply_wake_candidate_count, 3);
    Ok(())
}

#[test]
fn reply_message_wakes_sender_waiter_before_receiver_dispatch_returns() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let sender_thread = 66;
    let receiver_thread = 67;
    let hwnd = kernel
        .gwe
        .create_window(receiver_thread, "ReplyWake", "send");
    let send_id = kernel
        .begin_cross_thread_send_message_w(
            sender_thread,
            hwnd,
            WM_ERASEBKGND,
            0x66,
            0x67,
            Some(500),
        )
        .expect("queued send");
    let send_wait = kernel.register_blocked_waiter(
        sender_thread,
        0x700 + sender_thread,
        Vec::new(),
        SchedulerBlockedWaitKind::SendMessage { send_id },
        0,
        INFINITE,
    );
    let message = kernel.gwe.get_message(receiver_thread).unwrap();
    assert!(kernel.gwe.in_send_message(receiver_thread));

    assert!(kernel.reply_message(receiver_thread, 0xfeed));
    assert!(!kernel.gwe.in_send_message(receiver_thread));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| match blocked.kind {
            SchedulerBlockedWaitKind::SendMessage { send_id } => {
                kernel.sent_message_result_ready(send_id)
            }
            _ => false,
        }),
        Some(send_wait)
    );
    kernel.remove_blocked_waiter(send_wait).unwrap();

    assert_eq!(
        kernel.dispatch_message_w_for_thread(receiver_thread, message),
        1
    );
    assert_eq!(
        kernel.take_completed_send_message_result(send_id),
        Some(0xfeed)
    );
    let stats = kernel.scheduler_stats();
    assert_eq!(stats.send_reply_signal_count, 1);
    assert_eq!(stats.send_reply_wake_candidate_count, 1);
    Ok(())
}

#[test]
fn get_message_waiter_uses_filtered_scheduler_message_readiness() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let thread_id = 46;
    let hwnd = kernel
        .gwe
        .create_window(thread_id, "GetMessageWake", "wake");
    let waiter = kernel.register_blocked_waiter(
        thread_id,
        0x546,
        Vec::new(),
        SchedulerBlockedWaitKind::GetMessage {
            hwnd: Some(hwnd),
            min_msg: WM_USER + 10,
            max_msg: WM_USER + 20,
        },
        0,
        INFINITE,
    );

    assert!(kernel.post_message_w(hwnd, WM_USER + 1, 1, 2));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| match blocked.kind {
            SchedulerBlockedWaitKind::GetMessage {
                hwnd,
                min_msg,
                max_msg,
            } => kernel
                .gwe
                .has_message_filtered(blocked.thread_id, hwnd, min_msg, max_msg),
            _ => false,
        }),
        None
    );

    assert!(kernel.post_message_w(hwnd, WM_USER + 12, 3, 4));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| match blocked.kind {
            SchedulerBlockedWaitKind::GetMessage {
                hwnd,
                min_msg,
                max_msg,
            } => kernel
                .gwe
                .has_message_filtered(blocked.thread_id, hwnd, min_msg, max_msg),
            _ => false,
        }),
        Some(waiter)
    );
    kernel.remove_blocked_waiter(waiter).unwrap();
    assert_eq!(
        kernel
            .gwe
            .get_message_filtered(thread_id, Some(hwnd), WM_USER + 10, WM_USER + 20)
            .unwrap()
            .msg,
        WM_USER + 12
    );

    let stats = kernel.scheduler_stats();
    assert_eq!(stats.message_input_signal_count, 2);
    assert_eq!(stats.message_input_wake_candidate_count, 1);
    Ok(())
}

#[test]
fn remote_serial_injection_queues_scheduler_serial_read_candidates() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let com = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
    assert!(kernel.is_serial_device_handle(com));
    assert!(!kernel.serial_read_ready(com));

    let global_wait = kernel.register_blocked_waiter(
        50,
        0x350,
        vec![0x800],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    let serial_wait = kernel.register_blocked_waiter(
        51,
        0x351,
        Vec::new(),
        SchedulerBlockedWaitKind::SerialRead { handle: com },
        0,
        INFINITE,
    );

    let response = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "nmea",
        "sentences": ["$GPRMC,serial*00"]
    }));
    assert_eq!(response["accepted"], 1);
    assert!(kernel.serial_read_ready(com));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| match blocked.kind {
            SchedulerBlockedWaitKind::Kernel => true,
            SchedulerBlockedWaitKind::Sleep => false,
            SchedulerBlockedWaitKind::SerialRead { handle } => kernel.serial_read_ready(handle),
            SchedulerBlockedWaitKind::SerialCommEvent { handle } => {
                kernel.serial_comm_event_ready(handle)
            }
            SchedulerBlockedWaitKind::GetMessage { .. } => false,
            SchedulerBlockedWaitKind::MsgWait { .. } => false,
            SchedulerBlockedWaitKind::SendMessage { send_id } => {
                kernel.sent_message_result_ready(send_id)
            }
        }),
        Some(serial_wait)
    );
    kernel.remove_blocked_waiter(serial_wait).unwrap();
    kernel.remove_blocked_waiter(global_wait).unwrap();

    let bytes = kernel.read_file(com, 64)?;
    assert_eq!(bytes, b"$GPRMC,serial*00\r\n");
    assert!(!kernel.serial_read_ready(com));
    let stats = kernel.scheduler_stats();
    assert_eq!(stats.serial_read_signal_count, 1);
    assert_eq!(stats.serial_read_wake_candidate_count, 1);
    Ok(())
}

#[test]
fn remote_serial_injection_queues_scheduler_comm_event_candidates() -> Result<()> {
    const EV_RXCHAR: u32 = 0x0001;

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let com = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
    kernel.set_comm_mask(com, EV_RXCHAR)?;
    assert!(!kernel.serial_comm_event_ready(com));

    let global_wait = kernel.register_blocked_waiter(
        50,
        0x350,
        vec![0x800],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    let comm_wait = kernel.register_blocked_waiter(
        51,
        0x351,
        Vec::new(),
        SchedulerBlockedWaitKind::SerialCommEvent { handle: com },
        0,
        INFINITE,
    );

    let response = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "nmea",
        "sentences": ["$GPRMC,event*00"]
    }));
    assert_eq!(response["accepted"], 1);
    assert!(kernel.serial_comm_event_ready(com));
    assert_eq!(kernel.serial_comm_event_value(com), EV_RXCHAR);
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| match blocked.kind {
            SchedulerBlockedWaitKind::Kernel => true,
            SchedulerBlockedWaitKind::Sleep => false,
            SchedulerBlockedWaitKind::SerialRead { handle } => kernel.serial_read_ready(handle),
            SchedulerBlockedWaitKind::SerialCommEvent { handle } => {
                kernel.comm_event_mask_changed_wait(blocked.id)
                    || kernel.serial_comm_event_ready(handle)
            }
            SchedulerBlockedWaitKind::GetMessage { .. } => false,
            SchedulerBlockedWaitKind::MsgWait { .. } => false,
            SchedulerBlockedWaitKind::SendMessage { send_id } => {
                kernel.sent_message_result_ready(send_id)
            }
        }),
        Some(comm_wait)
    );
    kernel.remove_blocked_waiter(comm_wait).unwrap();
    kernel.remove_blocked_waiter(global_wait).unwrap();

    let stats = kernel.scheduler_stats();
    assert_eq!(stats.serial_event_signal_count, 1);
    assert_eq!(stats.serial_event_wake_candidate_count, 1);
    Ok(())
}

#[test]
fn set_comm_mask_wakes_pending_comm_event_with_zero_event() -> Result<()> {
    const EV_RXCHAR: u32 = 0x0001;

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let com = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
    let comm_wait = kernel.register_blocked_waiter(
        51,
        0x351,
        Vec::new(),
        SchedulerBlockedWaitKind::SerialCommEvent { handle: com },
        0,
        INFINITE,
    );

    kernel.set_comm_mask(com, EV_RXCHAR)?;
    assert!(kernel.comm_event_mask_changed_wait(comm_wait));
    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| match blocked.kind {
            SchedulerBlockedWaitKind::SerialCommEvent { handle } => {
                kernel.comm_event_mask_changed_wait(blocked.id)
                    || kernel.serial_comm_event_ready(handle)
            }
            _ => false,
        }),
        Some(comm_wait)
    );
    assert!(kernel.take_comm_event_mask_changed_wait(comm_wait));
    assert!(!kernel.comm_event_mask_changed_wait(comm_wait));
    kernel.remove_blocked_waiter(comm_wait).unwrap();

    let stats = kernel.scheduler_stats();
    assert_eq!(stats.serial_event_signal_count, 1);
    assert_eq!(stats.serial_event_wake_candidate_count, 1);
    Ok(())
}

#[test]
fn serial_comm_timeouts_control_empty_read_parking() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let com = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;

    assert_eq!(kernel.serial_empty_read_timeout_ms(com, 64), None);

    kernel.set_comm_timeouts(
        com,
        CommTimeouts {
            read_interval_timeout: 50,
            read_total_timeout_multiplier: 2,
            read_total_timeout_constant: 10,
            write_total_timeout_multiplier: 0,
            write_total_timeout_constant: 0,
        },
    )?;
    assert_eq!(kernel.serial_empty_read_timeout_ms(com, 64), Some(138));

    kernel.set_comm_timeouts(
        com,
        CommTimeouts {
            read_interval_timeout: u32::MAX,
            ..CommTimeouts::default()
        },
    )?;
    assert_eq!(kernel.serial_empty_read_timeout_ms(com, 64), Some(0));

    Ok(())
}

#[test]
fn serial_comm_state_mask_and_purge_are_handle_state() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let com = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;

    let mut dcb_bytes = [0u8; CommDcb::SIZE];
    dcb_bytes[0..4].copy_from_slice(&(CommDcb::SIZE as u32).to_le_bytes());
    dcb_bytes[4..8].copy_from_slice(&38400u32.to_le_bytes());
    dcb_bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    dcb_bytes[18] = 7;
    dcb_bytes[19] = 2;
    dcb_bytes[20] = 1;
    let dcb = CommDcb::from_bytes(&dcb_bytes).unwrap();
    kernel.set_comm_dcb(com, dcb)?;
    assert_eq!(kernel.get_comm_dcb(com)?.bytes(), &dcb_bytes);

    kernel.set_comm_mask(com, 0x0001 | 0x0004)?;
    assert_eq!(kernel.get_comm_mask(com)?, 0x0005);

    assert_eq!(kernel.write_file(com, b"$PUBX")?.bytes_transferred, 5);
    kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "nmea",
        "sentences": ["$GPRMC,state*00"]
    }));
    assert!(kernel.serial_read_ready(com));
    let (rx_len, tx_len) = kernel.comm_queue_lengths(com)?;
    assert!(rx_len > 0);
    assert_eq!(tx_len, 5);

    kernel.read_file(com, 4)?;
    let (rx_len, tx_len) = kernel.comm_queue_lengths(com)?;
    assert!(rx_len > 0);
    assert_eq!(tx_len, 5);

    kernel.purge_comm(com, PURGE_RXCLEAR | PURGE_TXCLEAR)?;
    assert_eq!(kernel.comm_queue_lengths(com)?, (0, 0));
    assert!(!kernel.serial_read_ready(com));

    Ok(())
}

#[test]
fn virtual_win32_api_smoke_covers_file_device_sync_gwe_and_audio() -> Result<()> {
    let root = unique_test_root("virtual_win32_api_smoke");
    fs::create_dir_all(&root).unwrap();

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.set_file_root(&root);

    let file = kernel.create_file_w(
        "\\ResidentFlash\\test.bin",
        GENERIC_READ | GENERIC_WRITE,
        CREATE_ALWAYS,
    )?;
    let written = kernel.write_file(file, b"abc")?;
    assert!(written.success);
    assert_eq!(written.bytes_transferred, 3);
    assert_eq!(kernel.wait_for_single_object(file, 0, 7), WAIT_FAILED);
    assert!(kernel.close_handle(file)?);
    assert_eq!(
        fs::read(root.join("ResidentFlash/test.bin")).unwrap(),
        b"abc"
    );

    let com = kernel.create_file_w("COM3:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
    let serial_write = kernel.write_file(com, b"$PMTK")?;
    assert_eq!(serial_write.bytes_transferred, 5);
    let serial_ioctl = kernel.device_io_control(com, 0x1234, &[], 16)?;
    assert!(!serial_ioctl.success);
    assert!(kernel.close_handle(com)?);

    let uid = kernel.create_file_w("UID1:", GENERIC_READ, CREATE_ALWAYS)?;
    let uid_ioctl = kernel.device_io_control(uid, 0x2222, &[], 16)?;
    assert!(uid_ioctl.success);
    assert!(uid_ioctl.bytes_returned > 0);
    assert!(kernel.close_handle(uid)?);

    let auto_event = kernel.create_event_w(Some("auto".to_owned()), false, true);
    assert_eq!(
        kernel.wait_for_single_object(auto_event, 0, 7),
        WAIT_OBJECT_0
    );
    assert_eq!(
        kernel.wait_for_single_object(auto_event, 0, 7),
        WAIT_TIMEOUT
    );
    assert!(kernel.set_event(auto_event));
    assert_eq!(
        kernel.wait_for_single_object(auto_event, 0, 7),
        WAIT_OBJECT_0
    );

    let mutex = kernel.create_mutex_w(Some("mx".to_owned()), None);
    assert_eq!(kernel.wait_for_single_object(mutex, 0, 7), WAIT_OBJECT_0);
    assert!(kernel.release_mutex(mutex, 7));
    let ready_event = kernel.create_event_w(Some("ready".to_owned()), true, true);
    let pending_event = kernel.create_event_w(Some("pending".to_owned()), true, false);
    assert_eq!(
        kernel.wait_for_multiple_objects(&[pending_event, ready_event], false, 123, 7),
        WAIT_OBJECT_0 + 1
    );
    let scheduler_stats = kernel.scheduler_stats();
    assert_eq!(scheduler_stats.wait_single_count, 5);
    assert_eq!(scheduler_stats.wait_multiple_count, 1);
    assert_eq!(scheduler_stats.wait_acquired_count, 4);
    assert_eq!(scheduler_stats.wait_timeout_count, 1);
    assert_eq!(scheduler_stats.wait_failed_count, 1);
    assert_eq!(scheduler_stats.max_wait_handles, 2);
    assert_eq!(scheduler_stats.max_timeout_ms, 123);

    let hwnd = kernel.create_window_ex_w(7, "STATIC", "old", None, 100, 0x4000_0000, 0);
    assert_eq!(kernel.message_pump_step(7), MessagePumpResult::Idle);
    assert!(kernel.gwe.set_window_text(hwnd, "new title"));
    assert_eq!(
        kernel.gwe.get_window_text(hwnd, 32).as_deref(),
        Some("new title")
    );
    assert_eq!(
        kernel.gwe.set_window_long(hwnd, GWL_USERDATA, 0xbeef),
        Some(0)
    );
    assert_eq!(kernel.gwe.get_window_long(hwnd, GWL_USERDATA), Some(0xbeef));
    assert!(kernel.post_message_w(hwnd, WM_USER + 0x40, 11, 22));
    assert_eq!(
        kernel.message_pump_step(7),
        MessagePumpResult::Dispatched(0)
    );
    kernel.gwe.post_quit_message(7, 99, 0);
    assert_eq!(kernel.message_pump_step(7), MessagePumpResult::Quit(99));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                7,
                None,
                WM_QUIT,
                WM_QUIT,
                wince_emulation_v3::ce::gwe::PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let wave = kernel
        .wave_out_open(WaveFormat::pcm_16bit(2, 44_100))
        .unwrap();
    assert_eq!(
        kernel.wave_out_write(
            wave,
            WaveBuffer {
                guest_ptr: 0x3000,
                len: 1024,
            },
        ),
        MMSYSERR_NOERROR
    );
    assert_eq!(kernel.audio.pause(wave), MMSYSERR_NOERROR);
    assert_eq!(
        kernel.audio.output(wave).unwrap().state,
        WaveOutState::Paused
    );
    assert_eq!(kernel.audio.restart(wave), MMSYSERR_NOERROR);

    Ok(())
}

#[test]
fn remote_server_api_state_queues_input_serial_audio_and_status() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.remote.set_framebuffer_size(800, 480);

    let hwnd = kernel.create_window_ex_w(99, "REMOTE", "remote", None, 1, 0, 0);
    assert_eq!(kernel.message_pump_step(99), MessagePumpResult::Idle);

    let touch = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "touch",
        "phase": "tap",
        "x": 12,
        "y": 34
    }));
    assert_eq!(touch["ok"], true);

    let key = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "key",
        "phase": "down",
        "vk": 0x26
    }));
    assert_eq!(key["ok"], true);

    assert_eq!(kernel.drain_remote_input_to_gwe(99, hwnd), 3);
    let activate = kernel.gwe.get_message(99).unwrap();
    assert_eq!(activate.hwnd, hwnd);
    assert_eq!(activate.msg, WM_ACTIVATE);
    assert_eq!(activate.wparam, WA_ACTIVE);
    let focus = kernel.gwe.get_message(99).unwrap();
    assert_eq!(focus.hwnd, hwnd);
    assert_eq!(focus.msg, WM_SETFOCUS);
    assert_eq!(kernel.gwe.get_focus(), Some(hwnd));

    let down = kernel.gwe.get_message(99).unwrap();
    assert_eq!(down.msg, WM_LBUTTONDOWN);
    assert_eq!(down.lparam & 0xffff, 12);
    assert_eq!((down.lparam >> 16) & 0xffff, 34);
    assert_eq!(kernel.gwe.get_message(99).unwrap().msg, WM_LBUTTONUP);
    let key_down = kernel.gwe.get_message(99).unwrap();
    assert_eq!(key_down.msg, WM_KEYDOWN);
    assert_eq!(key_down.wparam, 0x26);

    let nmea = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "nmea",
        "sentences": ["$GPRMC,remote*00"]
    }));
    assert_eq!(nmea["accepted"], 1);
    assert_eq!(kernel.read_remote_serial_bytes(64), b"$GPRMC,remote*00\r\n");

    let location = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "location",
        "lat": 37.5,
        "lon": 127.25,
        "timestampMs": 0
    }));
    assert_eq!(location["sentencesGenerated"], 3);
    assert!(kernel.remote.serial_byte_count() > 0);

    assert_eq!(
        kernel.dispatch_remote_control_message(&serde_json::json!({"type": "pause"}))["paused"],
        true
    );
    let status = kernel.remote_status_json();
    assert_eq!(status["paused"], true);
    assert_eq!(status["guest_width"], 800);
    assert!(
        status["gps_target"]
            .as_str()
            .is_some_and(|target| !target.is_empty())
    );

    let first_audio_client = kernel.remote.register_audio_client(1000);
    assert_eq!(kernel.remote.audio_client_count(), 1);
    assert_eq!(
        kernel.remote.publish_audio_chunk(vec![1, 2, 3, 4], 20),
        Some(1)
    );
    assert!(kernel.remote.audio_client_needs_flush(first_audio_client));
    let first_chunks = kernel
        .remote
        .take_audio_chunks_for_client(first_audio_client, 1);
    assert_eq!(first_chunks[0].pts_ms, 1000);
    assert!(first_chunks[0].flush);

    assert_eq!(
        kernel
            .remote
            .publish_audio_chunk(vec![10, 11, 12, 13, 14, 15, 16, 17], 40),
        Some(2)
    );
    let late_audio_client = kernel.remote.register_audio_client(1030);
    let late_chunks = kernel
        .remote
        .take_audio_chunks_for_client(late_audio_client, 1);
    assert_eq!(late_chunks[0].sequence, 2);
    assert_eq!(late_chunks[0].pts_ms, 1030);
    assert_eq!(late_chunks[0].duration_ms, 30);
    assert_eq!(late_chunks[0].payload, vec![12, 13, 14, 15, 16, 17]);
    assert!(late_chunks[0].flush);

    Ok(())
}

#[test]
fn blocked_get_message_wait_wakes_when_remote_input_is_drained() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.remote.set_framebuffer_size(800, 480);

    let thread_id = 99;
    let hwnd = kernel.create_window_ex_w(thread_id, "REMOTE", "remote", None, 1, 0, 0);
    let wait_id = kernel.register_blocked_waiter(
        thread_id,
        CE_CURRENT_THREAD_PSEUDO_HANDLE,
        Vec::new(),
        SchedulerBlockedWaitKind::GetMessage {
            hwnd: Some(hwnd),
            min_msg: 0,
            max_msg: 0,
        },
        kernel.timers.tick_count(),
        INFINITE,
    );

    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| {
            matches!(blocked.kind, SchedulerBlockedWaitKind::GetMessage { .. })
                && kernel.gwe.has_queue_input(blocked.thread_id, u32::MAX)
        }),
        None
    );

    kernel.remote.enqueue_touch("down", 12, 34).unwrap();
    assert_eq!(
        kernel.drain_remote_input_to_thread_window(thread_id, Some(hwnd)),
        1
    );
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "remote_touch_target"
            && record.thread_id == thread_id
            && record.hwnd == Some(hwnd)
            && record.msg == Some(WM_LBUTTONDOWN)
            && record.lparam == Some((34 << 16) | 12)
    }));

    assert_eq!(
        kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| {
            matches!(blocked.kind, SchedulerBlockedWaitKind::GetMessage { .. })
                && kernel.gwe.has_queue_input(blocked.thread_id, u32::MAX)
        }),
        Some(wait_id)
    );
    let down = (0..4)
        .filter_map(|_| kernel.get_message_w(thread_id))
        .find(|message| message.msg == WM_LBUTTONDOWN)
        .unwrap();
    assert_eq!(down.hwnd, hwnd);
    assert_eq!(down.lparam & 0xffff, 12);
    assert_eq!((down.lparam >> 16) & 0xffff, 34);
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "get_message"
            && record.thread_id == thread_id
            && record.hwnd == Some(hwnd)
            && record.msg == Some(WM_LBUTTONDOWN)
            && record.lparam == Some((34 << 16) | 12)
    }));

    Ok(())
}

#[test]
fn public_message_entrypoints_record_durable_gwe_trace() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let window_thread = 51;
    let sender_thread = 52;
    let hwnd = kernel.create_window_ex_w(window_thread, "TRACEPOST", "trace", None, 1, 0, 0);

    assert!(kernel.post_message_w_for_thread(sender_thread, hwnd, WM_USER + 0x51, 0x51, 0x52,));
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "post_message"
            && record.thread_id == window_thread
            && record.hwnd == Some(hwnd)
            && record.msg == Some(WM_USER + 0x51)
            && record.detail.as_deref() == Some("window")
    }));

    assert!(kernel.post_thread_message_w(sender_thread, WM_USER + 0x52, 0x53, 0x54));
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "post_message"
            && record.thread_id == sender_thread
            && record.hwnd == Some(0)
            && record.msg == Some(WM_USER + 0x52)
            && record.detail.as_deref() == Some("thread")
    }));

    assert!(kernel.post_keybd_message_for_thread(
        window_thread,
        Some(hwnd),
        b'A' as u32,
        true,
        1,
        &[b'A' as u32],
    ));
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "post_message"
            && record.thread_id == window_thread
            && record.hwnd == Some(hwnd)
            && record.msg == Some(WM_CHAR)
            && record.wparam == Some(b'A' as u32)
            && record.source == Some(MSGSRC_HARDWARE_KEYBOARD)
    }));

    assert!(kernel.send_notify_message_w(sender_thread, hwnd, WM_USER + 0x53, 0x55, 0x56,));
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "send_notify_message"
            && record.thread_id == window_thread
            && record.hwnd == Some(hwnd)
            && record.msg == Some(WM_USER + 0x53)
            && record.source == Some(MSGSRC_SOFTWARE_SEND)
    }));

    assert!(
        kernel
            .begin_cross_thread_send_message_w(
                sender_thread,
                hwnd,
                WM_USER + 0x54,
                0x57,
                0x58,
                None,
            )
            .is_some()
    );
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "queue_send_message"
            && record.thread_id == window_thread
            && record.hwnd == Some(hwnd)
            && record.msg == Some(WM_USER + 0x54)
            && record.source == Some(MSGSRC_SOFTWARE_SEND)
    }));

    Ok(())
}

#[test]
fn get_message_drains_remote_touch_to_active_window() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.remote.set_framebuffer_size(800, 480);

    let hwnd = kernel.create_window_ex_w(42, "REMOTE", "remote", None, 1, 0, 0);
    kernel.gwe.set_focus(Some(hwnd));
    kernel.remote.enqueue_touch("tap", 21, 43).unwrap();

    let down = kernel.get_message_w(42).unwrap();
    assert_eq!(down.hwnd, hwnd);
    assert_eq!(down.msg, WM_LBUTTONDOWN);
    assert_eq!(down.lparam & 0xffff, 21);
    assert_eq!((down.lparam >> 16) & 0xffff, 43);

    let up = kernel.get_message_w(42).unwrap();
    assert_eq!(up.hwnd, hwnd);
    assert_eq!(up.msg, WM_LBUTTONUP);

    Ok(())
}

#[test]
fn get_message_hit_tests_remote_touch_to_visible_child() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.remote.set_framebuffer_size(800, 480);

    let parent = kernel.gwe.create_window_ex_with_rect(
        42,
        "PARENT",
        "parent",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 800, 480),
    );
    let child = kernel.gwe.create_window_ex_with_rect(
        42,
        "CHILD",
        "child",
        Some(parent),
        2,
        WS_VISIBLE | WS_CHILD,
        0,
        Rect::from_origin_size(0, 0, 800, 480),
    );
    kernel.gwe.set_focus(Some(parent));
    kernel.remote.enqueue_touch("tap", 400, 240).unwrap();

    let kill_focus = kernel.get_message_w(42).unwrap();
    assert_eq!(kill_focus.hwnd, parent);
    assert_eq!(kill_focus.msg, WM_KILLFOCUS);
    assert_eq!(kill_focus.wparam, child);

    let activate = kernel.get_message_w(42).unwrap();
    assert_eq!(activate.hwnd, child);
    assert_eq!(activate.msg, WM_ACTIVATE);
    assert_eq!(activate.wparam, WA_ACTIVE);

    let set_focus = kernel.get_message_w(42).unwrap();
    assert_eq!(set_focus.hwnd, child);
    assert_eq!(set_focus.msg, WM_SETFOCUS);
    assert_eq!(set_focus.wparam, parent);
    assert_eq!(kernel.gwe.get_focus(), Some(child));

    let down = kernel.get_message_w(42).unwrap();
    assert_eq!(down.hwnd, child);
    assert_eq!(down.msg, WM_LBUTTONDOWN);
    assert_eq!(down.lparam & 0xffff, 400);
    assert_eq!((down.lparam >> 16) & 0xffff, 240);
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "remote_touch_target"
            && record.thread_id == 42
            && record.hwnd == Some(child)
            && record.msg == Some(WM_LBUTTONDOWN)
            && record.lparam == Some((240 << 16) | 400)
    }));

    let up = kernel.get_message_w(42).unwrap();
    assert_eq!(up.hwnd, child);
    assert_eq!(up.msg, WM_LBUTTONUP);

    Ok(())
}
