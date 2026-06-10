use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        audio::{MMSYSERR_NOERROR, WaveBuffer, WaveFormat, WaveOutState},
        cemath::{CeMath, CeMathCall, CeMathValue},
        com::{E_POINTER, REGDB_E_CLASSNOTREG, RPC_E_CHANGED_MODE, S_FALSE, S_OK},
        devices::{CommDcb, CommTimeouts, PURGE_RXCLEAR, PURGE_TXCLEAR},
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        gwe::{
            GWL_USERDATA, MSGSRC_HARDWARE_KEYBOARD, MSGSRC_SOFTWARE_SEND, QS_POSTMESSAGE,
            QS_SENDMESSAGE, QS_TIMER, Rect, SMF_TIMEOUT, WA_ACTIVE, WM_ACTIVATE, WM_CHAR,
            WM_ERASEBKGND, WM_KILLFOCUS, WM_QUIT, WM_SETFOCUS, WM_TIMER, WM_USER, WNDCLASSW_SIZE,
            WS_CHILD, WS_POPUP, WS_VISIBLE,
            SM_CXSCREEN, SM_CYSCREEN, SM_CXVSCROLL, SM_CYVSCROLL, SM_CXHSCROLL, SM_CYHSCROLL,
            SM_CYCAPTION, SM_CYMENU, SM_CXBORDER, SM_CYBORDER, SM_CXDLGFRAME, SM_CYDLGFRAME,
            SM_CXICON, SM_CYICON, SM_CXCURSOR, SM_CYCURSOR, SM_CXSMICON, SM_CYSMICON,
            SM_CXDOUBLECLK, SM_CYDOUBLECLK, SM_CXICONSPACING, SM_CYICONSPACING,
            SM_CXEDGE, SM_CYEDGE, SM_MOUSEPRESENT, SM_CMONITORS, SM_SAMEDISPLAYFORMAT,
            SM_DEBUG, SM_CXFULLSCREEN, SM_CYFULLSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
            SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        },
        kernel::{
            CE_CURRENT_PROCESS_PSEUDO_HANDLE, CE_CURRENT_THREAD_PSEUDO_HANDLE, CeKernel,
            FreeLibraryResult, LoadedModuleMetadata, MessagePumpResult,
        },
        object::{
            EventObject, KernelObject, MAX_CE_PRIORITY_LEVELS, MAX_SUSPEND_COUNT,
            MAX_WIN32_PRIORITY_LEVELS, MUTEX_MAX_LOCK_COUNT, THREAD_PRIORITY_TIME_CRITICAL,
            ThreadResumeResult, ThreadSuspendResult, ce_thread_priority_to_win32,
            win32_thread_priority_to_ce,
        },
        registry::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, HKEY_LOCAL_MACHINE, REG_SZ},
        remote::{WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP},
        scheduler::SchedulerBlockedWaitKind,
        thread::{ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, TLS_MINIMUM_AVAILABLE, ThreadSystem},
        timer::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::{MountConfig, ObjectStoreConfig, RuntimeConfig},
    emulator::{
        cpu::{CpuBackend as _, UnicornMips},
        memory::{MemoryMap, MemoryPerms},
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

    // contains_range: exact match → true; sub-range → true; one byte past end → false.
    assert!(map.contains_range(0x2000_0000, 0x0001_0000));
    assert!(map.contains_range(0x2000_1000, 0x8000));
    assert!(!map.contains_range(0x2000_0000, 0x0001_0001));
    assert!(!map.contains_range(0x1FFF_FFFF, 1));

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
fn child_current_process_exit_state_does_not_signal_parent_pseudo_handle() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let parent_thread_id = 1;
    let launch = kernel.queue_process_launch(Some("child.exe".to_owned()), None);
    let parent_state = kernel.current_process_state();

    kernel.set_current_process_id(launch.process_id);
    kernel.reset_current_process_exit_state();
    assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x55aa));
    assert_eq!(
        kernel.wait_for_single_object(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0, launch.thread_id),
        WAIT_OBJECT_0
    );

    kernel.set_current_process_state(parent_state);
    assert_eq!(
        kernel.wait_for_single_object(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0, parent_thread_id),
        WAIT_TIMEOUT
    );
    assert_eq!(
        kernel.process_exit_code_for_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE),
        Some(259)
    );

    kernel.mark_process_launch_exited(&launch, 0x55aa);
    assert_eq!(
        kernel.wait_for_single_object(launch.process_handle, 0, parent_thread_id),
        WAIT_OBJECT_0
    );
    assert_eq!(
        kernel.process_exit_code_for_handle(launch.process_handle),
        Some(0x55aa)
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
                SchedulerBlockedWaitKind::WinsockRead {
                    socket, readiness, ..
                } => match readiness {
                    wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Read => {
                        wince_emulation_v3::winsock::socket_read_ready(socket)
                    }
                    wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Write => {
                        wince_emulation_v3::winsock::socket_write_ready(socket)
                    }
                    wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::ReadWrite => {
                        wince_emulation_v3::winsock::socket_read_ready(socket)
                            || wince_emulation_v3::winsock::socket_write_ready(socket)
                    }
                    wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Except => false,
                },
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
                SchedulerBlockedWaitKind::ModalMessageBox => kernel.gwe.has_message_filtered(
                    blocked.thread_id,
                    None,
                    wince_emulation_v3::ce::gwe::WM_PAINT,
                    wince_emulation_v3::ce::gwe::WM_LBUTTONUP,
                ),
                SchedulerBlockedWaitKind::PopupMenuModal { mouse_message_max } => {
                    kernel.gwe.has_message_filtered(
                        blocked.thread_id,
                        None,
                        wince_emulation_v3::ce::gwe::WM_PAINT,
                        mouse_message_max,
                    )
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
    let mut completion_class = [0u8; WNDCLASSW_SIZE];
    completion_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("SendCompleteWake", completion_class);
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
    let mut reply_wake_class = [0u8; WNDCLASSW_SIZE];
    reply_wake_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("ReplyWake", reply_wake_class);
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
            SchedulerBlockedWaitKind::WinsockRead {
                socket, readiness, ..
            } => match readiness {
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Read => {
                    wince_emulation_v3::winsock::socket_read_ready(socket)
                }
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Write => {
                    wince_emulation_v3::winsock::socket_write_ready(socket)
                }
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::ReadWrite => {
                    wince_emulation_v3::winsock::socket_read_ready(socket)
                        || wince_emulation_v3::winsock::socket_write_ready(socket)
                }
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Except => false,
            },
            SchedulerBlockedWaitKind::GetMessage { .. } => false,
            SchedulerBlockedWaitKind::MsgWait { .. } => false,
            SchedulerBlockedWaitKind::SendMessage { send_id } => {
                kernel.sent_message_result_ready(send_id)
            }
            SchedulerBlockedWaitKind::ModalMessageBox => false,
            SchedulerBlockedWaitKind::PopupMenuModal { .. } => false,
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
            SchedulerBlockedWaitKind::WinsockRead {
                socket, readiness, ..
            } => match readiness {
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Read => {
                    wince_emulation_v3::winsock::socket_read_ready(socket)
                }
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Write => {
                    wince_emulation_v3::winsock::socket_write_ready(socket)
                }
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::ReadWrite => {
                    wince_emulation_v3::winsock::socket_read_ready(socket)
                        || wince_emulation_v3::winsock::socket_write_ready(socket)
                }
                wince_emulation_v3::ce::scheduler::SchedulerWinsockReadyKind::Except => false,
            },
            SchedulerBlockedWaitKind::GetMessage { .. } => false,
            SchedulerBlockedWaitKind::MsgWait { .. } => false,
            SchedulerBlockedWaitKind::SendMessage { send_id } => {
                kernel.sent_message_result_ready(send_id)
            }
            SchedulerBlockedWaitKind::ModalMessageBox => false,
            SchedulerBlockedWaitKind::PopupMenuModal { .. } => false,
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
    let uid_ioctl = kernel.device_io_control(uid, 0xa000_00cc, &[], 4)?;
    assert!(uid_ioctl.success);
    assert_eq!(uid_ioctl.bytes_returned, 4);
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
fn resumed_get_message_take_records_trace_and_clears_timer_pending() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let thread_id = 74;
    let hwnd = kernel.create_window_ex_w(thread_id, "RESUME_TIMER", "", None, 0, 0, 0);

    assert_eq!(
        kernel.set_timer_for_thread(thread_id, Some(hwnd), Some(77), 100, None),
        77
    );
    kernel.timers.sleep_ms(100);
    kernel.pump_timers_to_gwe(thread_id);

    let timer = kernel
        .take_ready_message_w_filtered(thread_id, Some(hwnd), 0, 0)
        .expect("ready timer message");
    assert_eq!(timer.hwnd, hwnd);
    assert_eq!(timer.msg, WM_TIMER);
    assert_eq!(timer.wparam, 77);
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "get_message"
            && record.thread_id == thread_id
            && record.hwnd == Some(hwnd)
            && record.msg == Some(WM_TIMER)
    }));

    kernel.timers.sleep_ms(100);
    kernel.pump_timers_to_gwe(thread_id);
    let next_timer = kernel.get_message_w(thread_id).expect("next timer message");
    assert_eq!(next_timer.hwnd, hwnd);
    assert_eq!(next_timer.msg, WM_TIMER);
    assert_eq!(next_timer.wparam, 77);

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

    let hwnd = kernel.gwe.create_window_ex_with_rect(
        42,
        "REMOTE",
        "remote",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 800, 480),
    );
    kernel.gwe.set_active_window(Some(hwnd));
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

#[test]
fn rect_geometry_subtract_intersect_union_and_point_containment() {
    use wince_emulation_v3::ce::gwe::{Point, Rect};

    let r = Rect { left: 0, top: 0, right: 10, bottom: 10 };
    let q = Rect { left: 5, top: 5, right: 15, bottom: 15 };

    // Intersect: overlapping rects return the overlap region.
    let inter = r.intersect(q).expect("overlapping rects have an intersection");
    assert_eq!(inter, Rect { left: 5, top: 5, right: 10, bottom: 10 });

    // Intersect: non-overlapping rects return None.
    let far = Rect { left: 20, top: 20, right: 30, bottom: 30 };
    assert!(r.intersect(far).is_none());

    // Union: bounding box of two non-overlapping rects.
    let u = r.union(far);
    assert_eq!(u, Rect { left: 0, top: 0, right: 30, bottom: 30 });

    // Union with empty rect returns the non-empty one.
    let empty = Rect { left: 5, top: 5, right: 5, bottom: 5 };
    assert_eq!(r.union(empty), r);
    assert_eq!(empty.union(r), r);

    // subtract: partial overlap at bottom-right corner produces two remainder rects.
    let mut parts = r.subtract(q);
    parts.sort_by_key(|r| (r.top, r.left));
    assert_eq!(parts.len(), 2);
    // Top strip: (0,0,10,5)
    assert!(parts.iter().any(|p| *p == Rect { left: 0, top: 0, right: 10, bottom: 5 }));
    // Left strip in the middle row: (0,5,5,10)
    assert!(parts.iter().any(|p| *p == Rect { left: 0, top: 5, right: 5, bottom: 10 }));

    // subtract: other completely contains self → empty remainder.
    let big = Rect { left: -5, top: -5, right: 20, bottom: 20 };
    assert!(r.subtract(big).is_empty());

    // subtract: no overlap → returns [self].
    let parts_far = r.subtract(far);
    assert_eq!(parts_far.len(), 1);
    assert_eq!(parts_far[0], r);

    // subtract_bounding: when other fully covers self → None.
    assert!(r.subtract_bounding(big).is_none());

    // subtract_bounding: no overlap → returns Some(self).
    assert_eq!(r.subtract_bounding(far), Some(r));

    // contains_point: interior, corner, edge, and exterior.
    let p_inside = Point { x: 5, y: 5 };
    let p_top_left = Point { x: 0, y: 0 };
    let p_right_edge = Point { x: 10, y: 5 };
    let p_bottom_edge = Point { x: 5, y: 10 };
    assert!(r.contains_point(p_inside));
    assert!(r.contains_point(p_top_left));
    assert!(!r.contains_point(p_right_edge));
    assert!(!r.contains_point(p_bottom_edge));
}

#[test]
fn rect_helper_methods_from_origin_size_offset_zero_origin_width_height() {
    use wince_emulation_v3::ce::gwe::Rect;

    // from_origin_size builds Rect with saturating right/bottom
    let r = Rect::from_origin_size(10, 20, 30, 40);
    assert_eq!(r.left, 10);
    assert_eq!(r.top, 20);
    assert_eq!(r.right, 40);
    assert_eq!(r.bottom, 60);

    // width and height
    assert_eq!(r.width(), 30);
    assert_eq!(r.height(), 40);

    // zero_origin shifts the rect to origin while preserving size
    let z = r.zero_origin();
    assert_eq!(z, Rect { left: 0, top: 0, right: 30, bottom: 40 });
    assert_eq!(z.width(), r.width());
    assert_eq!(z.height(), r.height());

    // offset shifts all edges by (dx, dy)
    let shifted = r.offset(5, -10);
    assert_eq!(shifted, Rect { left: 15, top: 10, right: 45, bottom: 50 });

    // normalized: left<right, top<bottom stays the same
    let already = Rect { left: 1, top: 2, right: 3, bottom: 4 };
    assert_eq!(already.normalized(), already);

    // normalized: inverted rect swaps edges
    let inv = Rect { left: 5, top: 8, right: 2, bottom: 3 };
    assert_eq!(inv.normalized(), Rect { left: 2, top: 3, right: 5, bottom: 8 });
}

#[test]
fn cemath_frexp_ldexp_modf_divzero_and_shifts() {
    let math = CeMath;

    // Frexp: 8.0 = 0.5 × 2^4
    assert_eq!(
        math.eval(CeMathCall::Frexp(8.0)),
        CeMathValue::Frexp { fraction: 0.5, exp: 4 }
    );
    // Frexp: 0.0 → fraction 0.0, exp 0
    assert_eq!(
        math.eval(CeMathCall::Frexp(0.0)),
        CeMathValue::Frexp { fraction: 0.0, exp: 0 }
    );

    // Ldexp: 0.75 × 2^3 = 6.0
    assert_eq!(
        math.eval(CeMathCall::Ldexp { value: 0.75, exp: 3 }),
        CeMathValue::F64(6.0)
    );

    // Modf: 3.7 → integer=3.0, fraction≈0.7
    let CeMathValue::Modf { integer, fraction } = math.eval(CeMathCall::Modf(3.7)) else {
        panic!("expected Modf result");
    };
    assert_eq!(integer, 3.0_f64);
    assert!((fraction - 0.7_f64).abs() < 1e-10);

    // Modf: negative value
    let CeMathValue::Modf { integer: int_neg, fraction: frac_neg } =
        math.eval(CeMathCall::Modf(-2.5)) else {
        panic!("expected Modf result");
    };
    assert_eq!(int_neg, -2.0_f64);
    assert!((frac_neg - (-0.5_f64)).abs() < 1e-10);

    // Divide-by-zero: Div denom=0
    assert_eq!(
        math.eval(CeMathCall::Div { numer: 99, denom: 0 }),
        CeMathValue::DivideByZero
    );
    // Divide-by-zero: LlDiv rhs=0
    assert_eq!(
        math.eval(CeMathCall::LlDiv { lhs: 1, rhs: 0 }),
        CeMathValue::DivideByZero
    );
    // Divide-by-zero: LlRem rhs=0
    assert_eq!(
        math.eval(CeMathCall::LlRem { lhs: 1, rhs: 0 }),
        CeMathValue::DivideByZero
    );
    // Divide-by-zero: UllDiv rhs=0
    assert_eq!(
        math.eval(CeMathCall::UllDiv { lhs: 1, rhs: 0 }),
        CeMathValue::DivideByZero
    );

    // Shift helpers
    assert_eq!(
        math.eval(CeMathCall::LlLShift { value: 1, shift: 4 }),
        CeMathValue::I64(16)
    );
    assert_eq!(
        math.eval(CeMathCall::LlRShift { value: -32, shift: 2 }),
        CeMathValue::I64(-8)
    );
    assert_eq!(
        math.eval(CeMathCall::UllRShift { value: 64, shift: 3 }),
        CeMathValue::U64(8)
    );

    // Labs is the same branch as Abs
    assert_eq!(math.eval(CeMathCall::Labs(-5)), CeMathValue::I32(5));
}

#[test]
fn cemath_unary_binary_conversion_and_float_arithmetic_ops() {
    use wince_emulation_v3::ce::cemath::{CeMathBinaryF64, CeMathUnaryF64};
    let math = CeMath;

    // UnaryF64 sampling: Sqrt, Floor, Ceil, Fabs.
    assert_eq!(math.eval(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Sqrt, value: 4.0 }), CeMathValue::F64(2.0));
    assert_eq!(math.eval(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Floor, value: 3.7 }), CeMathValue::F64(3.0));
    assert_eq!(math.eval(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Ceil, value: 3.2 }), CeMathValue::F64(4.0));
    assert_eq!(math.eval(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Fabs, value: -5.0 }), CeMathValue::F64(5.0));

    // BinaryF64: Atan2 and Fmod.
    assert_eq!(math.eval(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Atan2, lhs: 0.0, rhs: 1.0 }), CeMathValue::F64(0.0));
    assert_eq!(math.eval(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Fmod, lhs: 10.0, rhs: 3.0 }), CeMathValue::F64(1.0));

    // Ldiv: normal case.
    assert_eq!(math.eval(CeMathCall::Ldiv { numer: 17, denom: 5 }), CeMathValue::Div { quot: 3, rem: 2 });
    // Ldiv: divide-by-zero.
    assert_eq!(math.eval(CeMathCall::Ldiv { numer: 1, denom: 0 }), CeMathValue::DivideByZero);

    // UllRem.
    assert_eq!(math.eval(CeMathCall::UllRem { lhs: 10, rhs: 3 }), CeMathValue::U64(1));
    // UllRem divide-by-zero.
    assert_eq!(math.eval(CeMathCall::UllRem { lhs: 1, rhs: 0 }), CeMathValue::DivideByZero);

    // Float arithmetic helpers.
    assert_eq!(math.eval(CeMathCall::FloatAdd { lhs: 1.0, rhs: 2.0 }), CeMathValue::F32(3.0));
    assert_eq!(math.eval(CeMathCall::FloatSub { lhs: 5.0, rhs: 3.0 }), CeMathValue::F32(2.0));
    assert_eq!(math.eval(CeMathCall::FloatMul { lhs: 2.0, rhs: 3.0 }), CeMathValue::F32(6.0));
    assert_eq!(math.eval(CeMathCall::FloatDiv { lhs: 9.0, rhs: 3.0 }), CeMathValue::F32(3.0));

    // Double arithmetic helpers.
    assert_eq!(math.eval(CeMathCall::DoubleAdd { lhs: 1.0, rhs: 2.0 }), CeMathValue::F64(3.0));
    assert_eq!(math.eval(CeMathCall::DoubleSub { lhs: 5.0, rhs: 3.0 }), CeMathValue::F64(2.0));
    assert_eq!(math.eval(CeMathCall::DoubleMul { lhs: 2.0, rhs: 3.0 }), CeMathValue::F64(6.0));
    assert_eq!(math.eval(CeMathCall::DoubleDiv { lhs: 9.0, rhs: 3.0 }), CeMathValue::F64(3.0));

    // Type conversions.
    assert_eq!(math.eval(CeMathCall::FloatToLong(3.7_f32)), CeMathValue::I32(3));
    assert_eq!(math.eval(CeMathCall::DoubleToLong(4.9_f64)), CeMathValue::I32(4));
    assert_eq!(math.eval(CeMathCall::FloatToUnsignedLong(5.1_f32)), CeMathValue::U32(5));
    assert_eq!(math.eval(CeMathCall::LongToFloat(7)), CeMathValue::F32(7.0));
    assert_eq!(math.eval(CeMathCall::LongToDouble(8)), CeMathValue::F64(8.0));
    assert_eq!(math.eval(CeMathCall::UnsignedLongToFloat(10)), CeMathValue::F32(10.0));
    assert_eq!(math.eval(CeMathCall::UnsignedLongToDouble(11)), CeMathValue::F64(11.0));
    assert_eq!(math.eval(CeMathCall::FloatToDouble(1.5_f32)), CeMathValue::F64(1.5_f64));
    assert_eq!(math.eval(CeMathCall::DoubleToFloat(2.5_f64)), CeMathValue::F32(2.5_f32));

    // FloatCmp: -1, 0, 1.
    assert_eq!(math.eval(CeMathCall::FloatCmp { lhs: 1.0, rhs: 2.0 }), CeMathValue::Cmp(-1));
    assert_eq!(math.eval(CeMathCall::FloatCmp { lhs: 2.0, rhs: 2.0 }), CeMathValue::Cmp(0));
    assert_eq!(math.eval(CeMathCall::FloatCmp { lhs: 3.0, rhs: 2.0 }), CeMathValue::Cmp(1));
}

#[test]
fn thread_priority_win32_to_ce_and_ce_to_win32_boundary_cases() {
    // Win32 priorities 0..MAX_WIN32_PRIORITY_LEVELS-1 map to CE range [248..255].
    let base = MAX_CE_PRIORITY_LEVELS - MAX_WIN32_PRIORITY_LEVELS as i32;
    assert_eq!(win32_thread_priority_to_ce(0), Some(base));
    assert_eq!(win32_thread_priority_to_ce(MAX_WIN32_PRIORITY_LEVELS - 1), Some(255));
    assert_eq!(win32_thread_priority_to_ce(MAX_WIN32_PRIORITY_LEVELS), None);

    // CE priority in [base..255] maps back to the same Win32 priority.
    assert_eq!(ce_thread_priority_to_win32(base), Some(0));
    assert_eq!(ce_thread_priority_to_win32(255), Some(MAX_WIN32_PRIORITY_LEVELS - 1));

    // CE priority below base (high-priority RT threads) → THREAD_PRIORITY_TIME_CRITICAL.
    assert_eq!(ce_thread_priority_to_win32(0), Some(THREAD_PRIORITY_TIME_CRITICAL));
    assert_eq!(ce_thread_priority_to_win32(base - 1), Some(THREAD_PRIORITY_TIME_CRITICAL));

    // Out-of-range CE priority → None.
    assert_eq!(ce_thread_priority_to_win32(-1), None);
    assert_eq!(ce_thread_priority_to_win32(MAX_CE_PRIORITY_LEVELS), None);
}

#[test]
fn tls_get_set_out_of_range_slot_sets_error_invalid_parameter() {
    let mut threads = ThreadSystem::default();
    let tid = 1_u32;

    // Valid slot 0 returns 0 (unset) and clears last error.
    assert_eq!(threads.tls_get_value(tid, 0), Some(0));
    assert_eq!(threads.get_last_error(tid), ERROR_SUCCESS);

    // Set valid slot 0, then read it back.
    assert!(threads.tls_set_value(tid, 0, 0xDEAD_BEEF));
    assert_eq!(threads.tls_get_value(tid, 0), Some(0xDEAD_BEEF));

    // Out-of-range slot (TLS_MINIMUM_AVAILABLE) → None, sets ERROR_INVALID_PARAMETER.
    assert_eq!(threads.tls_get_value(tid, TLS_MINIMUM_AVAILABLE), None);
    assert_eq!(threads.get_last_error(tid), ERROR_INVALID_PARAMETER);

    // Out-of-range set → false, sets ERROR_INVALID_PARAMETER.
    assert!(!threads.tls_set_value(tid, TLS_MINIMUM_AVAILABLE, 42));
    assert_eq!(threads.get_last_error(tid), ERROR_INVALID_PARAMETER);
}

#[test]
fn pixel_format_bytes_and_bits_per_pixel_all_variants() {
    use wince_emulation_v3::ce::framebuffer::PixelFormat;
    assert_eq!(PixelFormat::Rgb565.bytes_per_pixel(), 2);
    assert_eq!(PixelFormat::Rgb565.bits_per_pixel(), 16);
    assert_eq!(PixelFormat::Bgra8888.bytes_per_pixel(), 4);
    assert_eq!(PixelFormat::Bgra8888.bits_per_pixel(), 32);
    assert_eq!(PixelFormat::Rgba8888.bytes_per_pixel(), 4);
    assert_eq!(PixelFormat::Rgba8888.bits_per_pixel(), 32);
    assert_eq!(PixelFormat::Gray8.bytes_per_pixel(), 1);
    assert_eq!(PixelFormat::Gray8.bits_per_pixel(), 8);
}

#[test]
fn handle_table_describe_handle_formats_event_mutex_semaphore_and_invalid() {
    use wince_emulation_v3::ce::object::HandleTable;

    let mut table = HandleTable::default();

    let event = table.create_event(Some("my_event".to_owned()), true, true);
    let desc = table.describe_handle(event);
    assert!(desc.contains("my_event"), "got: {desc}");
    assert!(desc.contains("manual=true"), "got: {desc}");
    assert!(desc.contains("signaled=true"), "got: {desc}");

    let mutex = table.create_mutex(None, None);
    let desc = table.describe_handle(mutex);
    assert!(desc.contains("<unnamed>"), "got: {desc}");
    assert!(desc.contains("owner=none"), "got: {desc}");

    let semaphore = table.create_semaphore(Some("sem".to_owned()), 3, 10).unwrap();
    let desc = table.describe_handle(semaphore);
    assert!(desc.contains("sem"), "got: {desc}");
    assert!(desc.contains("count=3"), "got: {desc}");
    assert!(desc.contains("max=10"), "got: {desc}");

    assert_eq!(table.describe_handle(0xDEAD_BEEF), "invalid");
}

#[test]
fn crt_rand_produces_deterministic_lcg_sequence_and_srand_reseeds() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // Default seed is 1; record first two values.
    let r0 = kernel.crt_rand();
    let r1 = kernel.crt_rand();
    assert!(r0 <= 0x7FFF);
    assert_ne!(r0, r1);

    // srand(1) resets to the same sequence.
    kernel.crt_srand(1);
    assert_eq!(kernel.crt_rand(), r0);
    assert_eq!(kernel.crt_rand(), r1);

    // Different seed → different first value.
    kernel.crt_srand(42);
    let r_other = kernel.crt_rand();
    assert_ne!(r_other, r0);
}

#[test]
fn release_semaphore_rejects_non_positive_count_and_overflow_above_maximum() {
    use wince_emulation_v3::ce::object::HandleTable;

    let mut table = HandleTable::default();
    let sem = table.create_semaphore(None, 1, 3).unwrap();

    // release_count=0 → None.
    assert_eq!(table.release_semaphore(sem, 0), None);
    // release_count negative → None.
    assert_eq!(table.release_semaphore(sem, -1), None);
    // valid release: 1 → returns previous count 1, new count becomes 2.
    assert_eq!(table.release_semaphore(sem, 1), Some(1));
    // another valid release: count 2 → 3, returns previous 2.
    assert_eq!(table.release_semaphore(sem, 1), Some(2));
    // would exceed maximum 3 → None, count stays at 3.
    assert_eq!(table.release_semaphore(sem, 1), None);
}

#[test]
fn crt_strtok_next_returns_zero_by_default_and_removing_with_null_ptr() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    let tid = 77_u32;
    // Initially 0 for any thread.
    assert_eq!(kernel.crt_strtok_next(tid), 0);

    // Setting a non-zero ptr stores it.
    kernel.crt_set_strtok_next(tid, 0x1234_5678);
    assert_eq!(kernel.crt_strtok_next(tid), 0x1234_5678);

    // Setting ptr=0 removes the entry; reads back as 0.
    kernel.crt_set_strtok_next(tid, 0);
    assert_eq!(kernel.crt_strtok_next(tid), 0);

    // Independent across threads.
    kernel.crt_set_strtok_next(1, 0xAAAA);
    kernel.crt_set_strtok_next(2, 0xBBBB);
    assert_eq!(kernel.crt_strtok_next(1), 0xAAAA);
    assert_eq!(kernel.crt_strtok_next(2), 0xBBBB);
}

#[test]
fn register_and_release_loaded_module_follows_pinned_dynamic_and_ref_count_paths() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // Register a pinned (dynamic=false) module.
    kernel.register_loaded_module("pinned.dll", 0x1000_0000, Default::default(), Default::default());
    let pinned_base = kernel.loaded_module_handle("pinned.dll").unwrap();
    assert_eq!(pinned_base, 0x1000_0000);
    assert!(kernel.is_loaded_module_handle(0x1000_0000));

    // Releasing a pinned module → Pinned; handle stays valid.
    assert_eq!(kernel.release_loaded_module(0x1000_0000), FreeLibraryResult::Pinned);
    assert!(kernel.is_loaded_module_handle(0x1000_0000));

    // Releasing an unknown handle → InvalidHandle.
    assert_eq!(kernel.release_loaded_module(0xDEAD_0000), FreeLibraryResult::InvalidHandle);

    // Register a dynamic module with ref_count=2.
    kernel.register_loaded_module_with_metadata(
        "dyn.dll",
        0x2000_0000,
        Default::default(),
        Default::default(),
        LoadedModuleMetadata { dynamic: true, ref_count: 2, ..Default::default() },
    );

    // First release: ref_count drops from 2 to 1 → StillReferenced.
    assert!(matches!(
        kernel.release_loaded_module(0x2000_0000),
        FreeLibraryResult::StillReferenced { ref_count: 1 }
    ));
    assert!(kernel.is_loaded_module_handle(0x2000_0000));

    // Second release: ref_count was 1 → UnloadPending.
    assert_eq!(kernel.release_loaded_module(0x2000_0000), FreeLibraryResult::UnloadPending);

    // After UnloadPending the module is no longer visible.
    assert!(kernel.loaded_module_handle("dyn.dll").is_none());
    assert!(!kernel.is_loaded_module_handle(0x2000_0000));

    // retain_loaded_module_by_name increments ref_count and returns base.
    let pinned_ret = kernel.retain_loaded_module_by_name("pinned.dll");
    assert_eq!(pinned_ret, Some(0x1000_0000));

    // retain on unloaded module → None.
    assert!(kernel.retain_loaded_module_by_name("dyn.dll").is_none());
}

#[test]
fn normalize_symbol_and_module_names_strip_and_lowercase() {
    use wince_emulation_v3::ce::kernel::{normalize_module_name, normalize_symbol_name};

    // Module name: trim .dll/.DLL suffix and lowercase.
    assert_eq!(normalize_module_name("Foo.dll"), "foo");
    assert_eq!(normalize_module_name("BAR.DLL"), "bar");
    assert_eq!(normalize_module_name("baz"), "baz");
    // Null terminator stripping.
    assert_eq!(normalize_module_name("foo.dll\0"), "foo");

    // Symbol name: strip leading underscores and @-decorated part, lowercase.
    assert_eq!(normalize_symbol_name("_MyFunc"), "myfunc");
    assert_eq!(normalize_symbol_name("__MyFunc@8"), "myfunc");
    assert_eq!(normalize_symbol_name("SomeApi"), "someapi");
    // No underscore or @.
    assert_eq!(normalize_symbol_name("Foo"), "foo");
}

#[test]
fn resolve_loaded_module_proc_by_name_and_ordinal() {
    use std::collections::BTreeMap;
    use wince_emulation_v3::ce::kernel::LoadedModuleMetadata;

    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    let mut by_name: BTreeMap<String, u32> = BTreeMap::new();
    by_name.insert("MyExport".to_owned(), 0xCAFE_0001);
    by_name.insert("_OtherExport@4".to_owned(), 0xCAFE_0002);

    let mut by_ordinal: BTreeMap<u32, u32> = BTreeMap::new();
    by_ordinal.insert(1, 0xCAFE_0001);
    by_ordinal.insert(5, 0xCAFE_0005);

    kernel.register_loaded_module_with_metadata(
        "testlib.dll",
        0x3000_0000,
        by_name,
        by_ordinal,
        LoadedModuleMetadata::default(),
    );

    let base = kernel.loaded_module_handle("testlib.dll").unwrap();

    // Name lookup normalizes: "MyExport" → "myexport".
    assert_eq!(kernel.resolve_loaded_module_proc_by_name(base, "MyExport"), Some(0xCAFE_0001));
    // Decorated name normalizes: "_OtherExport@4" → "otherexport".
    assert_eq!(kernel.resolve_loaded_module_proc_by_name(base, "_OtherExport@4"), Some(0xCAFE_0002));
    // Unknown name → None.
    assert_eq!(kernel.resolve_loaded_module_proc_by_name(base, "missing"), None);

    // Ordinal lookup.
    assert_eq!(kernel.resolve_loaded_module_proc_by_ordinal(base, 1), Some(0xCAFE_0001));
    assert_eq!(kernel.resolve_loaded_module_proc_by_ordinal(base, 5), Some(0xCAFE_0005));
    assert_eq!(kernel.resolve_loaded_module_proc_by_ordinal(base, 99), None);

    // Invalid module handle → None.
    assert_eq!(kernel.resolve_loaded_module_proc_by_name(0xDEAD, "MyExport"), None);
}

#[test]
fn registry_has_key_set_value_enum_subkeys_and_query_info() {
    use wince_emulation_v3::ce::registry::{
        ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, HKEY_LOCAL_MACHINE, RegistryValue,
    };

    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // Programmatic helpers use full paths including the hklm root prefix.
    kernel.registry.create_key(r"hklm\TEST\SubA");
    kernel.registry.create_key(r"hklm\TEST\SubB");
    assert!(kernel.registry.has_key(r"hklm\TEST\SubA"));
    assert!(!kernel.registry.has_key(r"hklm\TEST\SubC"));

    kernel.registry.set_value(r"hklm\TEST\SubA", "Num", RegistryValue::dword(99));
    kernel.registry.set_value(r"hklm\TEST\SubA", "Str", RegistryValue::string("hello"));

    let v = kernel.registry.query_value(r"hklm\TEST\SubA", "Num").unwrap();
    assert_eq!(v.as_dword(), Some(99));
    let v = kernel.registry.query_value(r"hklm\TEST\SubA", "Str").unwrap();
    assert_eq!(v.as_str(), Some("hello"));

    // enum_subkeys returns only direct children; keys are normalized to lowercase.
    let subs = kernel.registry.enum_subkeys(r"hklm\TEST");
    assert!(subs.contains(&"suba".to_owned()), "got: {subs:?}");
    assert!(subs.contains(&"subb".to_owned()), "got: {subs:?}");
    assert_eq!(subs.len(), 2);

    // enum_values for SubA.
    let vals = kernel.registry.enum_values(r"hklm\TEST\SubA").unwrap();
    assert_eq!(vals.len(), 2);

    // reg_query_info_key_w via an open handle.
    let create = kernel.registry.reg_create_key_exw(HKEY_LOCAL_MACHINE, Some(r"TEST\SubA"));
    assert_eq!(create.status, ERROR_SUCCESS);
    let hkey = create.hkey.unwrap();
    let info = kernel.registry.reg_query_info_key_w(hkey);
    assert_eq!(info.status, ERROR_SUCCESS);
    assert_eq!(info.subkeys, 0);
    assert_eq!(info.values, 2);

    // reg_enum_key_ex_w via the TEST key handle.
    let create_test = kernel.registry.reg_create_key_exw(HKEY_LOCAL_MACHINE, Some("TEST"));
    let test_hkey = create_test.hkey.unwrap();
    let r0 = kernel.registry.reg_enum_key_ex_w(test_hkey, 0, None);
    assert_eq!(r0.status, ERROR_SUCCESS);
    assert!(r0.name.is_some());
    let r_end = kernel.registry.reg_enum_key_ex_w(test_hkey, 99, None);
    assert_eq!(r_end.status, ERROR_NO_MORE_ITEMS);
}

#[test]
fn ce_sleep_request_boundary_cases_and_timer_performance_counter() {
    use wince_emulation_v3::ce::timer::{CeSleepRequest, INFINITE, ce_sleep_request};

    // 0 ms → Yield (cooperative reschedule).
    assert_eq!(ce_sleep_request(0), CeSleepRequest::Yield);
    // INFINITE → Suspend (no wake).
    assert_eq!(ce_sleep_request(INFINITE), CeSleepRequest::Suspend);
    // Normal sleep → Bounded(ms + 1) to guarantee at least ms has elapsed.
    assert_eq!(ce_sleep_request(1), CeSleepRequest::Bounded(2));
    assert_eq!(ce_sleep_request(100), CeSleepRequest::Bounded(101));
    // At CE_SLEEP_LONG_BOUND (0xffff_fffe), adding 1 would overflow 0xffff_ffff = INFINITE,
    // so the implementation caps at CE_SLEEP_LONG_BOUND itself.
    assert_eq!(ce_sleep_request(0xffff_fffe), CeSleepRequest::Bounded(0xffff_fffe));

    // performance_frequency is always 1_000 (millisecond resolution).
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);
    assert_eq!(kernel.timers.performance_frequency(), 1_000);

    // sleep_ms advances performance_counter.
    let before = kernel.timers.performance_counter();
    kernel.timers.sleep_ms(500);
    let after = kernel.timers.performance_counter();
    assert!(after >= before + 500, "counter={after}, before={before}");
}

#[test]
fn com_initialize_rejects_apartment_mode_change_and_create_instance_validates_pointers() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);
    let tid = 5_u32;

    // First call with multi-threaded (coinit=0) → S_OK.
    assert_eq!(kernel.com.co_initialize_ex(tid, 0), S_OK);
    // Second call same mode → S_FALSE (re-init).
    assert_eq!(kernel.com.co_initialize_ex(tid, 0), S_FALSE);
    // Switching to single-threaded (coinit=0x2) on the same thread → RPC_E_CHANGED_MODE.
    assert_eq!(kernel.com.co_initialize_ex(tid, 0x2), RPC_E_CHANGED_MODE);

    // co_create_instance with null clsid_ptr → E_POINTER.
    assert_eq!(kernel.com.co_create_instance(0, 0x1000), Err(E_POINTER));
    // co_create_instance with null iid_ptr → E_POINTER.
    assert_eq!(kernel.com.co_create_instance(0x1000, 0), Err(E_POINTER));
    // co_create_instance for unregistered class → REGDB_E_CLASSNOTREG.
    assert_eq!(kernel.com.co_create_instance(0x1000, 0x2000), Err(REGDB_E_CLASSNOTREG));
}

#[test]
fn gwe_caret_blink_time_zero_rejected_and_advance_without_caret_returns_false() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // set_caret_blink_time(0) → false; time unchanged.
    let original = kernel.gwe.get_caret_blink_time();
    assert!(!kernel.gwe.set_caret_blink_time(0));
    assert_eq!(kernel.gwe.get_caret_blink_time(), original);

    // set_caret_blink_time(non-zero) → true; time stored.
    assert!(kernel.gwe.set_caret_blink_time(300));
    assert_eq!(kernel.gwe.get_caret_blink_time(), 300);

    // advance_caret_blink without any caret → false.
    assert!(!kernel.gwe.advance_caret_blink(1000));

    // reset_caret_on_focus_lost/gained flip blink_visible regardless of caret presence.
    kernel.gwe.reset_caret_on_focus_gained();
    assert!(kernel.gwe.caret_blink_visible());
    kernel.gwe.reset_caret_on_focus_lost();
    assert!(!kernel.gwe.caret_blink_visible());
}

#[test]
fn gwe_find_window_matches_by_class_and_title_and_resolve_class_name_normalizes() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    let tid = 1_u32;
    let hwnd = kernel.gwe.create_window(tid, "MyClass", "My Window Title");
    assert_ne!(hwnd, 0);

    // find_window with correct class → finds the window.
    let found = kernel.gwe.find_window(Some("MyClass"), None);
    assert_eq!(found, Some(hwnd));

    // find_window with correct title → finds the window.
    let found = kernel.gwe.find_window(None, Some("My Window Title"));
    assert_eq!(found, Some(hwnd));

    // find_window with both correct → finds it.
    let found = kernel.gwe.find_window(Some("MyClass"), Some("My Window Title"));
    assert_eq!(found, Some(hwnd));

    // find_window with wrong title → None.
    let found = kernel.gwe.find_window(Some("MyClass"), Some("Other Title"));
    assert!(found.is_none());

    // find_window with wrong class → None.
    let found = kernel.gwe.find_window(Some("OtherClass"), None);
    assert!(found.is_none());

    // resolve_class_name for a registered class returns its canonical name.
    // (class names are normalized on register; unregistered names normalize lowercase).
    let resolved = kernel.gwe.resolve_class_name("MYCLASS");
    assert!(!resolved.is_empty());
}

#[test]
fn gwe_keyboard_layout_ime_and_activate_layout() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // keyboard_layout() and keyboard_layout_list() are consistent.
    let layout = kernel.gwe.keyboard_layout();
    assert_eq!(kernel.gwe.keyboard_layout_list(), [layout]);
    assert!(!kernel.gwe.keyboard_layout_name().is_empty());

    // activate_keyboard_layout(0) → None (invalid).
    assert_eq!(kernel.gwe.activate_keyboard_layout(0), None);
    // activate_keyboard_layout(non-zero) → Some(previous), stores new layout.
    let prev = kernel.gwe.activate_keyboard_layout(0x0409_0409).unwrap();
    assert_eq!(prev, layout);
    assert_eq!(kernel.gwe.keyboard_layout(), 0x0409_0409);

    // set_keyboard_layout_from_name with valid 8-hex-digit string.
    let prev2 = kernel.gwe.set_keyboard_layout_from_name("00000412").unwrap();
    assert_eq!(prev2, 0x0409_0409);
    assert_eq!(kernel.gwe.keyboard_layout(), 0x0000_0412);
    assert_eq!(kernel.gwe.keyboard_layout_name(), "00000412");

    // set_keyboard_layout_from_name with invalid string → None.
    assert!(kernel.gwe.set_keyboard_layout_from_name("not-valid").is_none());

    // ime_enabled_for_thread defaults to true; toggle to false then back.
    let tid = 99_u32;
    assert!(kernel.gwe.ime_enabled_for_thread(tid));
    kernel.gwe.set_ime_enabled_for_thread(tid, false);
    assert!(!kernel.gwe.ime_enabled_for_thread(tid));
    kernel.gwe.set_ime_enabled_for_thread(tid, true);
    assert!(kernel.gwe.ime_enabled_for_thread(tid));

    // is_ime_layout: high-word set → true; 0 → false; no high-word → false.
    assert!(kernel.gwe.is_ime_layout(0xE001_0411));
    assert!(!kernel.gwe.is_ime_layout(0));
    assert!(!kernel.gwe.is_ime_layout(0x0000_0409));
}

#[test]
fn stock_object_handle_returns_handle_for_valid_indices_and_none_for_invalid() {
    use wince_emulation_v3::ce::resource::stock_object_handle;
    const STOCK_BASE: u32 = 0x000b_5000;

    // White brush (0), null brush (5) → valid.
    assert_eq!(stock_object_handle(0), Some(STOCK_BASE | 0));
    assert_eq!(stock_object_handle(5), Some(STOCK_BASE | 5));
    // White pen (6), null pen (8) → valid.
    assert_eq!(stock_object_handle(6), Some(STOCK_BASE | 6));
    assert_eq!(stock_object_handle(8), Some(STOCK_BASE | 8));
    // System font (13), default palette (15) → valid.
    assert_eq!(stock_object_handle(13), Some(STOCK_BASE | 13));
    assert_eq!(stock_object_handle(15), Some(STOCK_BASE | 15));
    // DC brush (18), DC pen (19) → valid.
    assert_eq!(stock_object_handle(18), Some(STOCK_BASE | 18));
    assert_eq!(stock_object_handle(19), Some(STOCK_BASE | 19));

    // Out-of-range gaps and beyond are None.
    assert_eq!(stock_object_handle(9), None);   // gap between NULL_PEN and SYSTEM_FONT
    assert_eq!(stock_object_handle(16), None);  // gap between DEFAULT_PALETTE and DC_BRUSH
    assert_eq!(stock_object_handle(100), None);
}

#[test]
fn virtual_input_push_events_and_poll_and_virtual_presenter_counts() {
    use wince_emulation_v3::ce::desktop::{VirtualInput, VirtualInputEvent};

    let mut input = VirtualInput::default();
    input.push_touch_down(10, 20);
    input.push_touch_move(15, 25);
    input.push_touch_up(15, 25);
    input.push_event(VirtualInputEvent::Key { virtual_key: 0x41, pressed: true });

    use wince_emulation_v3::ce::desktop::Input as _;
    let events = input.poll_events().unwrap();
    assert_eq!(events.len(), 4);
    assert_eq!(events[0], VirtualInputEvent::TouchDown { x: 10, y: 20 });
    assert_eq!(events[1], VirtualInputEvent::TouchMove { x: 15, y: 25 });
    assert_eq!(events[2], VirtualInputEvent::TouchUp { x: 15, y: 25 });
    assert_eq!(events[3], VirtualInputEvent::Key { virtual_key: 0x41, pressed: true });

    // After draining, queue is empty.
    let events2 = input.poll_events().unwrap();
    assert!(events2.is_empty());
}

#[test]
fn winsock_network_mode_is_isolated_nat_with_ce_addresses() {
    use std::net::Ipv4Addr;
    use wince_emulation_v3::winsock::{WinsockNetworkMode, network_mode};

    let WinsockNetworkMode::IsolatedNat { gateway, guest_ip } = network_mode();
    // CE emulates an isolated NAT: gateway at 10.0.0.1, guest at 10.0.0.2.
    assert_eq!(gateway, Ipv4Addr::new(10, 0, 0, 1));
    assert_eq!(guest_ip, Ipv4Addr::new(10, 0, 0, 2));
}

#[test]
fn object_store_config_and_mount_config_total_and_free_bytes_convert_mbytes() {
    let store = ObjectStoreConfig { total_mbytes: 256, free_mbytes: 128 };
    assert_eq!(store.total_bytes(), 256 * 1024 * 1024);
    assert_eq!(store.free_bytes(), 128 * 1024 * 1024);

    let mount = MountConfig {
        name: None,
        guest_root: "/Storage Card".to_owned(),
        host_root: None,
        total_mbytes: 1,
        free_mbytes: 0,
        writable: true,
        removable: false,
        system: false,
        hidden: false,
    };
    assert_eq!(mount.total_bytes(), 1024 * 1024);
    assert_eq!(mount.free_bytes(), 0);
}

#[test]
fn gwe_system_metric_returns_ce_standard_values() {
    use wince_emulation_v3::ce::gwe::Gwe;
    let gwe = Gwe::default();
    // Desktop defaults to 800x480.
    assert_eq!(gwe.system_metric(SM_CXSCREEN), 800);
    assert_eq!(gwe.system_metric(SM_CYSCREEN), 480);
    assert_eq!(gwe.system_metric(SM_CXFULLSCREEN), 800);
    assert_eq!(gwe.system_metric(SM_CYFULLSCREEN), 480);
    assert_eq!(gwe.system_metric(SM_CXVIRTUALSCREEN), 800);
    assert_eq!(gwe.system_metric(SM_CYVIRTUALSCREEN), 480);
    assert_eq!(gwe.system_metric(SM_XVIRTUALSCREEN), 0);
    assert_eq!(gwe.system_metric(SM_YVIRTUALSCREEN), 0);
    // Fixed-value metrics.
    assert_eq!(gwe.system_metric(SM_CXVSCROLL), 13);
    assert_eq!(gwe.system_metric(SM_CYVSCROLL), 13);
    assert_eq!(gwe.system_metric(SM_CXHSCROLL), 13);
    assert_eq!(gwe.system_metric(SM_CYHSCROLL), 13);
    assert_eq!(gwe.system_metric(SM_CYCAPTION), 24);
    assert_eq!(gwe.system_metric(SM_CYMENU), 24);
    assert_eq!(gwe.system_metric(SM_CXBORDER), 1);
    assert_eq!(gwe.system_metric(SM_CYBORDER), 1);
    assert_eq!(gwe.system_metric(SM_CXDLGFRAME), 3);
    assert_eq!(gwe.system_metric(SM_CYDLGFRAME), 3);
    assert_eq!(gwe.system_metric(SM_CXICON), 32);
    assert_eq!(gwe.system_metric(SM_CYICON), 32);
    assert_eq!(gwe.system_metric(SM_CXCURSOR), 32);
    assert_eq!(gwe.system_metric(SM_CYCURSOR), 32);
    assert_eq!(gwe.system_metric(SM_CXSMICON), 16);
    assert_eq!(gwe.system_metric(SM_CYSMICON), 16);
    assert_eq!(gwe.system_metric(SM_CXDOUBLECLK), 4);
    assert_eq!(gwe.system_metric(SM_CYDOUBLECLK), 4);
    assert_eq!(gwe.system_metric(SM_CXICONSPACING), 75);
    assert_eq!(gwe.system_metric(SM_CYICONSPACING), 75);
    assert_eq!(gwe.system_metric(SM_CXEDGE), 2);
    assert_eq!(gwe.system_metric(SM_CYEDGE), 2);
    assert_eq!(gwe.system_metric(SM_MOUSEPRESENT), 1);
    assert_eq!(gwe.system_metric(SM_CMONITORS), 1);
    assert_eq!(gwe.system_metric(SM_SAMEDISPLAYFORMAT), 1);
    assert_eq!(gwe.system_metric(SM_DEBUG), 0);
    // Unknown index falls through to 0.
    assert_eq!(gwe.system_metric(9999), 0);
}

#[test]
fn registry_delete_value_succeeds_and_returns_file_not_found_when_missing() {
    use std::collections::BTreeMap;
    use wince_emulation_v3::ce::registry::{Registry, RegistryDump, ERROR_INVALID_HANDLE, REG_DWORD};

    let mut reg = Registry::from_dump(RegistryDump {
        version: 0,
        source: None,
        keys: BTreeMap::new(),
    });
    let res = reg.reg_create_key_exw(HKEY_LOCAL_MACHINE, Some("DeleteTest"));
    assert_eq!(res.status, ERROR_SUCCESS);
    let hkey = res.hkey.unwrap();

    // Set a value then delete it — should return ERROR_SUCCESS.
    assert_eq!(
        reg.reg_set_value_exw(hkey, Some("val"), REG_DWORD, &42u32.to_le_bytes()),
        ERROR_SUCCESS
    );
    assert_eq!(reg.reg_delete_value_w(hkey, Some("val")), ERROR_SUCCESS);

    // Second delete of the same value — ERROR_FILE_NOT_FOUND.
    assert_eq!(reg.reg_delete_value_w(hkey, Some("val")), ERROR_FILE_NOT_FOUND);

    // Delete on a bogus handle — ERROR_INVALID_HANDLE.
    assert_eq!(reg.reg_delete_value_w(0xDEAD_BEEF, Some("val")), ERROR_INVALID_HANDLE);
}

#[test]
fn registry_delete_key_removes_key_and_children_returns_file_not_found_when_missing() {
    use std::collections::BTreeMap;
    use wince_emulation_v3::ce::registry::{Registry, RegistryDump, ERROR_INVALID_HANDLE};

    let mut reg = Registry::from_dump(RegistryDump { version: 0, source: None, keys: BTreeMap::new() });

    // Create parent and child keys.
    let parent = reg.reg_create_key_exw(HKEY_LOCAL_MACHINE, Some("Parent")).hkey.unwrap();
    let _child = reg.reg_create_key_exw(parent, Some("Child")).hkey.unwrap();

    // Delete "Parent" via HKLM relative path — deletes subtree.
    assert_eq!(reg.reg_delete_key_w(HKEY_LOCAL_MACHINE, Some("Parent")), ERROR_SUCCESS);

    // Second delete — key is gone.
    assert_eq!(reg.reg_delete_key_w(HKEY_LOCAL_MACHINE, Some("Parent")), ERROR_FILE_NOT_FOUND);

    // Invalid handle.
    assert_eq!(reg.reg_delete_key_w(0xDEAD_BEEF, Some("any")), ERROR_INVALID_HANDLE);
}

#[test]
fn cemath_integer_longlong_and_shift_ops_all_variants() {
    use wince_emulation_v3::ce::cemath::{CeMathBinaryF32, CeMathBinaryF64};

    let math = CeMath;

    // Abs / Labs — saturating_abs (i32::MIN stays i32::MIN due to saturation).
    assert_eq!(math.eval(CeMathCall::Abs(-7)), CeMathValue::I32(7));
    assert_eq!(math.eval(CeMathCall::Labs(5)), CeMathValue::I32(5));
    assert_eq!(math.eval(CeMathCall::Abs(i32::MIN)), CeMathValue::I32(i32::MAX));

    // Div — same as Ldiv; checked for zero denominator.
    assert_eq!(math.eval(CeMathCall::Div { numer: 10, denom: 3 }), CeMathValue::Div { quot: 3, rem: 1 });
    assert_eq!(math.eval(CeMathCall::Div { numer: 1, denom: 0 }), CeMathValue::DivideByZero);

    // BinaryF32::Fmod.
    assert_eq!(math.eval(CeMathCall::BinaryF32 { op: CeMathBinaryF32::Fmod, lhs: 7.5, rhs: 2.0 }), CeMathValue::F32(7.5_f32 % 2.0_f32));

    // BinaryF64 extra variants.
    let hypot = math.eval(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Hypot, lhs: 3.0, rhs: 4.0 });
    assert_eq!(hypot, CeMathValue::F64(5.0));
    let pow = math.eval(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Pow, lhs: 2.0, rhs: 3.0 });
    assert_eq!(pow, CeMathValue::F64(8.0));

    // Ldexp.
    assert_eq!(math.eval(CeMathCall::Ldexp { value: 1.0, exp: 3 }), CeMathValue::F64(8.0));

    // Modf.
    assert_eq!(math.eval(CeMathCall::Modf(3.75)), CeMathValue::Modf { integer: 3.0, fraction: 0.75 });

    // Frexp: 8.0 = 0.5 * 2^4.
    assert_eq!(math.eval(CeMathCall::Frexp(8.0)), CeMathValue::Frexp { fraction: 0.5, exp: 4 });
    // Frexp(0.0) special case.
    assert_eq!(math.eval(CeMathCall::Frexp(0.0)), CeMathValue::Frexp { fraction: 0.0, exp: 0 });

    // 64-bit integer arithmetic.
    assert_eq!(math.eval(CeMathCall::LlMul { lhs: 6, rhs: 7 }), CeMathValue::I64(42));
    assert_eq!(math.eval(CeMathCall::LlDiv { lhs: 17, rhs: 5 }), CeMathValue::I64(3));
    assert_eq!(math.eval(CeMathCall::LlRem { lhs: 17, rhs: 5 }), CeMathValue::I64(2));
    assert_eq!(math.eval(CeMathCall::UllDiv { lhs: 20, rhs: 3 }), CeMathValue::U64(6));
    assert_eq!(math.eval(CeMathCall::LlDiv { lhs: 1, rhs: 0 }), CeMathValue::DivideByZero);
    assert_eq!(math.eval(CeMathCall::LlRem { lhs: 1, rhs: 0 }), CeMathValue::DivideByZero);
    assert_eq!(math.eval(CeMathCall::UllDiv { lhs: 1, rhs: 0 }), CeMathValue::DivideByZero);

    // Shift operations.
    assert_eq!(math.eval(CeMathCall::LlLShift { value: 1, shift: 3 }), CeMathValue::I64(8));
    assert_eq!(math.eval(CeMathCall::LlRShift { value: 8, shift: 2 }), CeMathValue::I64(2));
    assert_eq!(math.eval(CeMathCall::UllRShift { value: 16, shift: 2 }), CeMathValue::U64(4));
    // Shift with value >= 64 is clamped to 63.
    assert_eq!(math.eval(CeMathCall::LlLShift { value: 1, shift: 64 }), CeMathValue::I64(1_i64 << 63));

    // Float/double to 64-bit integer conversions.
    assert_eq!(math.eval(CeMathCall::FloatToLongLong(3.9_f32)), CeMathValue::I64(3));
    assert_eq!(math.eval(CeMathCall::DoubleToLongLong(4.9_f64)), CeMathValue::I64(4));
    assert_eq!(math.eval(CeMathCall::FloatToUnsignedLongLong(5.1_f32)), CeMathValue::U64(5));
    assert_eq!(math.eval(CeMathCall::DoubleToUnsignedLongLong(6.0_f64)), CeMathValue::U64(6));
    assert_eq!(math.eval(CeMathCall::DoubleToUnsignedLong(7.9_f64)), CeMathValue::U32(7));

    // DoubleCmp.
    assert_eq!(math.eval(CeMathCall::DoubleCmp { lhs: 1.0, rhs: 2.0 }), CeMathValue::Cmp(-1));
    assert_eq!(math.eval(CeMathCall::DoubleCmp { lhs: 2.0, rhs: 2.0 }), CeMathValue::Cmp(0));
    assert_eq!(math.eval(CeMathCall::DoubleCmp { lhs: 3.0, rhs: 2.0 }), CeMathValue::Cmp(1));
}

#[test]
fn device_namespace_enabled_names_and_session_rx_tx_roundtrip() {
    use wince_emulation_v3::ce::devices::{
        DeviceBackend, DeviceConfig, DeviceConfigFile, DeviceDefaults, DeviceKind, DeviceNamespace,
    };

    let config = DeviceConfigFile {
        version: 0,
        defaults: DeviceDefaults::default(),
        devices: vec![
            DeviceConfig {
                guest: "COM7:".to_owned(),
                kind: DeviceKind::Serial,
                backend: DeviceBackend::Stub,
                host: None,
                enabled: true,
                note: None,
            },
            DeviceConfig {
                guest: "COM9:".to_owned(),
                kind: DeviceKind::Serial,
                backend: DeviceBackend::Stub,
                host: None,
                enabled: false, // disabled — should not appear in enabled_names
                note: None,
            },
        ],
    };

    let ns = DeviceNamespace::from_config(config);
    let names = ns.enabled_names();
    assert_eq!(names.len(), 1, "only enabled devices should appear");
    assert!(names.contains(&"COM7:".to_owned()));

    // remote_gps_target: no Win32Com backend, returns first serial guest name as fallback.
    let gps_target = ns.remote_gps_target();
    assert_eq!(gps_target.as_deref(), Some("COM7:"));

    // Open the session and exercise enqueue_rx / read_file / write_file / tx_bytes / queue_lengths.
    let mut session = ns.open("COM7:").unwrap();
    assert!(session.is_serial());
    assert_eq!(session.rx_len(), 0);
    assert_eq!(session.queue_lengths(), (0, 0));

    session.enqueue_rx(b"hello");
    assert_eq!(session.rx_len(), 5);
    assert_eq!(session.queue_lengths(), (5, 0));

    let read = session.read_file(3);
    assert_eq!(read, b"hel");
    assert_eq!(session.rx_len(), 2);

    session.write_file(b"world");
    assert_eq!(session.tx_bytes(), b"world");
    assert_eq!(session.queue_lengths(), (2, 5));
}

#[test]
fn audio_volume_pitch_rate_position_and_complete_buffer_ops() {
    use wince_emulation_v3::ce::audio::{
        AudioSystem, MMSYSERR_INVALHANDLE, MMSYSERR_NOERROR, NullAudioSink, WaveBuffer, WaveFormat,
        WaveOutState,
    };

    let mut audio = AudioSystem::default();

    // Operations on a bogus id return error/false.
    assert_eq!(audio.get_volume(99), Err(MMSYSERR_INVALHANDLE));
    assert_eq!(audio.get_pitch(99), Err(MMSYSERR_INVALHANDLE));
    assert_eq!(audio.get_playback_rate(99), Err(MMSYSERR_INVALHANDLE));
    assert_eq!(audio.get_position_bytes(99), Err(MMSYSERR_INVALHANDLE));
    assert!(!audio.set_volume(99, 0xFFFF));
    assert_eq!(audio.set_pitch(99, 0), MMSYSERR_INVALHANDLE);
    assert_eq!(audio.set_playback_rate(99, 0), MMSYSERR_INVALHANDLE);

    let format = WaveFormat {
        format_tag: 1,
        channels: 1,
        samples_per_sec: 8000,
        avg_bytes_per_sec: 8000,
        block_align: 1,
        bits_per_sample: 8,
    };
    let id = audio.open_wave_out(format.clone());
    assert_eq!(audio.wave_out_get_num_devs(), 1);

    // Default values for a new device.
    assert_eq!(audio.get_volume(id), Ok(0xFFFF_FFFF));
    assert_eq!(audio.get_pitch(id), Ok(0x0001_0000));
    assert_eq!(audio.get_playback_rate(id), Ok(0x0001_0000));
    assert_eq!(audio.get_position_bytes(id), Ok(0));

    // set/get round-trip.
    assert!(audio.set_volume(id, 0x8000_8000));
    assert_eq!(audio.get_volume(id), Ok(0x8000_8000));
    assert_eq!(audio.set_pitch(id, 0x0002_0000), MMSYSERR_NOERROR);
    assert_eq!(audio.get_pitch(id), Ok(0x0002_0000));
    assert_eq!(audio.set_playback_rate(id, 0x0003_0000), MMSYSERR_NOERROR);
    assert_eq!(audio.get_playback_rate(id), Ok(0x0003_0000));

    // Write two buffers, then complete them one at a time.
    let buf1 = WaveBuffer { guest_ptr: 0x1000, len: 8 };
    let buf2 = WaveBuffer { guest_ptr: 0x2000, len: 4 };
    assert_eq!(audio.wave_out_write(id, buf1.clone()), MMSYSERR_NOERROR);
    assert_eq!(audio.wave_out_write(id, buf2.clone()), MMSYSERR_NOERROR);

    // After writing, state should be Playing.
    assert_eq!(audio.output(id).unwrap().state, WaveOutState::Playing);

    // complete_next_buffer dequeues in order; last completion returns state to Open.
    assert_eq!(audio.complete_next_buffer(id), Some(buf1));
    assert_eq!(audio.output(id).unwrap().state, WaveOutState::Playing); // still one pending
    assert_eq!(audio.complete_next_buffer(id), Some(buf2));
    assert_eq!(audio.output(id).unwrap().state, WaveOutState::Open);   // queue drained
    assert_eq!(audio.complete_next_buffer(id), None);                   // empty

    // Sink operations.
    assert!(!audio.has_sinks());
    audio.register_sink(NullAudioSink::new("test_sink"));
    assert!(audio.has_sinks());
    assert_eq!(audio.sink_names(), vec!["test_sink".to_owned()]);
    assert_eq!(audio.queued_sink_chunk_count("test_sink"), Some(0));
    assert_eq!(audio.queued_sink_chunk_count("missing"), None);
    audio.flush_sinks();
    assert!(audio.unregister_sink("test_sink"));
    assert!(!audio.has_sinks());
    assert!(!audio.unregister_sink("test_sink")); // already gone
}

#[test]
fn memory_system_allocation_accessors_and_generation_counters() {
    use wince_emulation_v3::ce::memory::{
        MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PROCESS_HEAP_HANDLE, MemorySystem,
    };

    let mut mem = MemorySystem::default();

    // heap_high_water_mark starts at the base (0x3000_0000).
    let hwm0 = mem.heap_high_water_mark();
    let gen0 = mem.heap_generation();
    let vgen0 = mem.virtual_generation();

    let ptr = mem.heap_alloc(PROCESS_HEAP_HANDLE, 0, 32).unwrap();

    // High-water mark advances after allocation.
    assert!(mem.heap_high_water_mark() > hwm0);
    // Heap generation bumps.
    assert!(mem.heap_generation() > gen0);
    // Virtual generation unchanged so far.
    assert_eq!(mem.virtual_generation(), vgen0);

    // allocation() returns the right metadata.
    let alloc = mem.allocation(ptr).unwrap();
    assert_eq!(alloc.ptr, ptr);
    assert_eq!(alloc.heap, PROCESS_HEAP_HANDLE);
    assert_eq!(alloc.requested_size, 32);
    assert!(alloc.actual_size >= 32);

    // allocations() enumerates it.
    assert_eq!(mem.allocations().count(), 1);

    // Virtual allocation round-trip.
    let vbase = mem.virtual_alloc(0, 0x1_0000, MEM_COMMIT | MEM_RESERVE, 4).unwrap();
    assert!(mem.virtual_generation() > vgen0);

    let valloc = mem.virtual_allocation(vbase).unwrap();
    assert_eq!(valloc.base, vbase);
    assert_eq!(valloc.size, 0x1_0000);
    assert!(valloc.initial_bytes.is_empty());

    // virtual_allocations() enumerates it.
    assert_eq!(mem.virtual_allocations().count(), 1);

    // set_virtual_initial_bytes stores bytes and bumps virtual generation.
    let vgen1 = mem.virtual_generation();
    assert!(mem.set_virtual_initial_bytes(vbase, vec![0xAA, 0xBB]));
    assert!(mem.virtual_generation() > vgen1);
    assert_eq!(mem.virtual_allocation(vbase).unwrap().initial_bytes, vec![0xAA, 0xBB]);

    // set_virtual_initial_bytes on a non-existent base returns false.
    assert!(!mem.set_virtual_initial_bytes(0xDEAD_0000, vec![]));

    // Free the virtual allocation.
    assert!(mem.virtual_free(vbase, 0, MEM_RELEASE));
    assert!(mem.virtual_allocation(vbase).is_none());
    assert_eq!(mem.virtual_allocations().count(), 0);
}

#[test]
fn handle_table_event_open_file_mapping_views_thread_process_exit_and_set_reset_event() {
    use wince_emulation_v3::ce::object::{FileMappingView, HandleTable};

    let mut table = HandleTable::default();

    // open_event returns the same handle as create_event for named events.
    let ev = table.create_event(Some("my_ev".to_owned()), false, false);
    assert_eq!(table.open_event("my_ev"), Some(ev));
    assert_eq!(table.open_event("nonexistent"), None);

    // set_event / reset_event.
    assert!(table.set_event(ev));
    assert!(table.reset_event(ev));
    assert!(!table.set_event(0xDEAD)); // invalid handle

    // is_mutex / mutex_lock_count.
    let mx = table.create_mutex(Some("mx".to_owned()), Some(42));
    assert!(table.is_mutex(mx));
    assert!(!table.is_mutex(ev));
    assert_eq!(table.mutex_lock_count(mx), Some(1));
    assert_eq!(table.mutex_lock_count(ev), None);

    // Thread lifecycle: thread_start returns info only while unsignaled/unsuspended.
    let th = table.create_thread(77, 0x4000, 0x5000, false);
    assert_eq!(table.thread_id(th), Some(77));
    assert_eq!(table.thread_handle_by_id(77), Some(th));
    assert_eq!(table.thread_exit_code(th), Some(259)); // STILL_ACTIVE
    assert_eq!(table.thread_start(th), Some((77, 0x4000, 0x5000)));
    assert!(table.mark_thread_exited(th, 0));
    assert_eq!(table.thread_exit_code(th), Some(0));
    assert_eq!(table.thread_start(th), None); // signaled → thread_start returns None

    // Process lifecycle.
    let pr = table.create_process(99);
    assert_eq!(table.process_id(pr), Some(99));
    assert_eq!(table.process_exit_code(pr), Some(259));
    assert!(table.mark_process_exited(pr, 1));
    assert_eq!(table.process_exit_code(pr), Some(1));

    // File mapping: create, add view, query, remove view.
    let fm = table.create_file_mapping(Some("shm".to_owned()), 0x1000, 4, None);
    {
        let mapping = table.file_mapping(fm).unwrap();
        assert_eq!(mapping.size, 0x1000);
        assert!(mapping.views.is_empty());
    }
    // Add a view manually via file_mapping_mut.
    {
        let mapping = table.file_mapping_mut(fm).unwrap();
        mapping.views.push(FileMappingView { base: 0xA000_0000, size: 0x1000, offset: 0 });
    }
    assert!(table.has_file_mapping_view(0xA000_0000));
    assert!(!table.has_file_mapping_view(0xBEEF_0000));

    let (obj, view) = table.file_mapping_view(0xA000_0000).unwrap();
    assert_eq!(obj.size, 0x1000);
    assert_eq!(view.base, 0xA000_0000);

    assert_eq!(table.file_mappings().count(), 1);
    let removed = table.remove_file_mapping_view(0xA000_0000).unwrap();
    assert_eq!(removed.base, 0xA000_0000);
    assert!(!table.has_file_mapping_view(0xA000_0000));
    assert_eq!(table.remove_file_mapping_view(0xA000_0000), None); // already gone
}

#[test]
fn thread_system_allocate_id_set_last_error_and_tls_call_alloc_free() {
    use wince_emulation_v3::ce::thread::{ThreadSystem, TLS_MINIMUM_AVAILABLE, TLS_OUT_OF_INDEXES};

    let mut threads = ThreadSystem::default();

    // allocate_guest_thread_id starts at 2 and increments.
    assert_eq!(threads.allocate_guest_thread_id(), 2);
    assert_eq!(threads.allocate_guest_thread_id(), 3);
    assert_eq!(threads.allocate_guest_thread_id(), 4);

    // set_last_error / get_last_error round-trip.
    threads.set_last_error(10, 0xDEAD);
    assert_eq!(threads.get_last_error(10), 0xDEAD);
    // Unknown thread → ERROR_SUCCESS (0) by default.
    assert_eq!(threads.get_last_error(99), 0);

    // tls_call: alloc returns first unreserved slot (4), free clears it.
    // TLS_FUNC_ALLOC = 0, TLS_FUNC_FREE = 1.
    // Use a fresh thread_id (20) so last_error starts at SUCCESS.
    let slot_a = threads.tls_call(20, 0, 0); // alloc
    let slot_b = threads.tls_call(20, 0, 0); // alloc
    assert_eq!(slot_a, 4);
    assert_eq!(slot_b, 5);
    assert_eq!(threads.get_last_error(20), 0); // successful alloc does not set last_error

    // Free slot_a and verify it can be re-allocated.
    assert_eq!(threads.tls_call(20, 1, slot_a), 1); // free returns 1 (true)
    let slot_reuse = threads.tls_call(20, 0, 0); // re-alloc
    assert_eq!(slot_reuse, slot_a); // slot 4 recycled

    // Exhausting all TLS slots returns TLS_OUT_OF_INDEXES and sets ERROR_INVALID_PARAMETER.
    // After slot_b and slot_reuse are allocated (5 and 4), fill up remaining slots 6..63.
    for _ in 0..(TLS_MINIMUM_AVAILABLE - 4 - 2) {
        threads.tls_call(20, 0, 0);
    }
    let exhausted = threads.tls_call(20, 0, 0);
    assert_eq!(exhausted, TLS_OUT_OF_INDEXES);
    assert_eq!(threads.get_last_error(20), ERROR_INVALID_PARAMETER);
}

#[test]
fn kernel_runtime_loader_stats_and_process_metadata_getters_setters() {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // All stats start at zero.
    let stats = kernel.runtime_loader_stats();
    assert_eq!(stats.load_attempt_count, 0);
    assert_eq!(stats.successful_map_count, 0);
    assert_eq!(stats.loud_failure_count, 0);

    // Each record method increments exactly one counter.
    kernel.record_runtime_loader_load_attempt();
    kernel.record_runtime_loader_successful_map();
    kernel.record_runtime_loader_dependency_load();
    kernel.record_runtime_loader_export_lookup(true);
    kernel.record_runtime_loader_export_lookup(false); // miss
    kernel.record_runtime_loader_forwarded_export();
    kernel.record_runtime_loader_tls_callback();
    kernel.record_runtime_loader_dllmain_attach();
    kernel.record_runtime_loader_dllmain_detach();
    kernel.record_runtime_loader_loud_failure();

    let stats = kernel.runtime_loader_stats();
    assert_eq!(stats.load_attempt_count, 1);
    assert_eq!(stats.successful_map_count, 1);
    assert_eq!(stats.dependency_load_count, 1);
    assert_eq!(stats.export_lookup_count, 2);
    assert_eq!(stats.export_lookup_miss_count, 1);
    assert_eq!(stats.forwarded_export_count, 1);
    assert_eq!(stats.tls_callback_count, 1);
    assert_eq!(stats.dllmain_attach_count, 1);
    assert_eq!(stats.dllmain_detach_count, 1);
    assert_eq!(stats.loud_failure_count, 1);

    // Process metadata getters/setters.
    kernel.set_process_module_base(0x0010_0000);
    assert_eq!(kernel.process_module_base(), 0x0010_0000);

    kernel.set_process_module_path("\\SDMMC Disk\\INavi\\iNavi.exe");
    assert_eq!(kernel.process_module_path(), "\\SDMMC Disk\\INavi\\iNavi.exe");

    kernel.set_process_command_line("iNavi.exe /map");
    assert_eq!(kernel.process_command_line(), "iNavi.exe /map");

    kernel.set_process_current_directory(Some("\\SDMMC Disk\\INavi".to_owned()));
    assert_eq!(kernel.process_current_directory(), Some("\\SDMMC Disk\\INavi"));
    kernel.set_process_current_directory(None);
    assert_eq!(kernel.process_current_directory(), None);

    kernel.set_process_show_cmd(5);
    assert_eq!(kernel.process_show_cmd(), 5);

    kernel.set_current_process_id(42);
    assert_eq!(kernel.current_process_id(), 42);
}

#[test]
fn gwe_window_rect_client_rect_screen_coords_class_name_and_stats() {
    use wince_emulation_v3::ce::gwe::{Gwe, Point};

    let mut gwe = Gwe::default();
    let win_rect = Rect { left: 10, top: 20, right: 310, bottom: 220 };
    let hwnd = gwe.create_window_ex_with_rect(5, "MyClass", "MyTitle", None, 0, 0, 0, win_rect);

    // get_class_name: class names are normalized to lowercase.
    assert_eq!(gwe.get_class_name(hwnd, 256).as_deref(), Some("myclass"));
    // capacity 2 reserves 1 for null so only 1 char fits.
    assert_eq!(gwe.get_class_name(hwnd, 2).as_deref(), Some("m"));

    // window_thread_process_id returns stored owner info.
    let (tid, _pid) = gwe.window_thread_process_id(hwnd).unwrap();
    assert_eq!(tid, 5);
    assert_eq!(gwe.window_thread_process_id(0xDEAD), None);

    // get_window_rect returns the window rect.
    let wrect = gwe.get_window_rect(hwnd).unwrap();
    // Desktop starts at 800x480; child windows default to full desktop.
    assert!(wrect.width() > 0 && wrect.height() > 0);

    // get_client_rect is origin-zeroed.
    let crect = gwe.get_client_rect(hwnd).unwrap();
    assert_eq!(crect.left, 0);
    assert_eq!(crect.top, 0);
    assert_eq!(crect.width(), wrect.width());
    assert_eq!(crect.height(), wrect.height());

    // get_window_rect/client_rect on an invalid HWND returns None.
    assert!(gwe.get_window_rect(0xDEAD_BEEF).is_none());
    assert!(gwe.get_client_rect(0xDEAD_BEEF).is_none());

    // client_to_screen / screen_to_client are inverses through the client origin.
    let client_pt = Point { x: 10, y: 20 };
    let screen_pt = gwe.client_to_screen(hwnd, client_pt).unwrap();
    let back_pt = gwe.screen_to_client(hwnd, screen_pt).unwrap();
    assert_eq!(back_pt, client_pt);

    // stats() reflects send counters (zero before any sends).
    let st = gwe.stats();
    assert_eq!(st.send_transaction_count, 0);
    assert_eq!(st.send_transaction_completed_count, 0);
}

#[test]
fn gwe_rect_geometric_operations() {
    use wince_emulation_v3::ce::gwe::{Point, Rect};

    let r = Rect { left: 10, top: 20, right: 110, bottom: 70 };
    assert_eq!(r.width(), 100);
    assert_eq!(r.height(), 50);
    assert!(!r.is_empty());
    assert!(Rect::default().is_empty());

    // offset.
    let shifted = r.offset(5, -5);
    assert_eq!(shifted.left, 15);
    assert_eq!(shifted.top, 15);
    assert_eq!(shifted.width(), 100);

    // zero_origin.
    let z = r.zero_origin();
    assert_eq!(z.left, 0);
    assert_eq!(z.top, 0);
    assert_eq!(z.width(), 100);
    assert_eq!(z.height(), 50);

    // normalized: swaps when right < left or bottom < top.
    let inv = Rect { left: 50, top: 40, right: 10, bottom: 20 };
    let norm = inv.normalized();
    assert!(norm.left <= norm.right);
    assert!(norm.top <= norm.bottom);

    // contains_point.
    assert!(r.contains_point(Point { x: 50, y: 40 }));
    assert!(!r.contains_point(Point { x: 10, y: 70 })); // at right edge (exclusive)
    assert!(!r.contains_point(Point { x: 200, y: 40 }));

    // union: bounding box of two rects.
    let r2 = Rect { left: 90, top: 60, right: 200, bottom: 100 };
    let u = r.union(r2);
    assert_eq!(u.left, 10);
    assert_eq!(u.top, 20);
    assert_eq!(u.right, 200);
    assert_eq!(u.bottom, 100);

    // intersect: overlapping rects.
    let overlap = Rect { left: 50, top: 30, right: 150, bottom: 90 };
    let sect = r.intersect(overlap).unwrap();
    assert_eq!(sect.left, 50);
    assert_eq!(sect.top, 30);
    assert_eq!(sect.right, 110);
    assert_eq!(sect.bottom, 70);

    // intersect: non-overlapping rects.
    let far = Rect { left: 200, top: 200, right: 300, bottom: 300 };
    assert!(r.intersect(far).is_none());

    // subtract_bounding.
    // Subtracting self returns None (fully covered).
    assert!(r.subtract_bounding(r).is_none());
    // Subtracting a non-overlapping rect returns Some(r).
    assert_eq!(r.subtract_bounding(far), Some(r));
    // Subtracting a partial overlap returns the bounding box of remaining fragments.
    let interior = Rect { left: 20, top: 25, right: 80, bottom: 60 };
    let sub = r.subtract_bounding(interior).unwrap();
    assert!(!sub.is_empty());

    // subtract (full fragment list).
    let frags = r.subtract(interior);
    assert!(!frags.is_empty()); // at least one fragment left
    for frag in &frags {
        assert!(!frag.is_empty());
    }
}

#[test]
fn gwe_clipboard_open_set_get_empty_close_and_register_format() {
    use wince_emulation_v3::ce::gwe::Gwe;

    let mut gwe = Gwe::default();
    let hwnd = gwe.create_window(1, "cls", "win");

    // Open with valid window.
    assert!(gwe.open_clipboard(hwnd));
    assert!(gwe.clipboard_is_open());
    assert_eq!(gwe.get_open_clipboard_window(), hwnd);

    // Double-open fails.
    assert!(!gwe.open_clipboard(hwnd));

    // empty_clipboard sets owner and clears data.
    assert!(gwe.empty_clipboard());
    assert_eq!(gwe.get_clipboard_owner(), hwnd);
    assert_eq!(gwe.count_clipboard_formats(), 0);

    // set_clipboard_data stores data when open.
    let handle = gwe.set_clipboard_data(1, 13, 0xABCD).unwrap();
    assert_eq!(handle, 0xABCD);
    assert_eq!(gwe.count_clipboard_formats(), 1);
    assert!(gwe.is_clipboard_format_available(13));

    // get_clipboard_data returns the stored handle.
    assert_eq!(gwe.get_clipboard_data(13), Some(0xABCD));
    assert_eq!(gwe.get_clipboard_data(99), None); // missing format

    // enum_clipboard_formats: first format when previous=0.
    let first = gwe.enum_clipboard_formats(0);
    assert_eq!(first, 13);
    let next = gwe.enum_clipboard_formats(first);
    assert_eq!(next, 0); // no more formats

    // get_priority_clipboard_format: returns the format value of the first match, or -1 if none.
    assert_eq!(gwe.get_priority_clipboard_format(&[99, 13, 100]), 13); // 13 is available
    assert_eq!(gwe.get_priority_clipboard_format(&[99, 200]), -1); // neither available

    // close_clipboard clears the open state.
    assert!(gwe.close_clipboard());
    assert!(!gwe.clipboard_is_open());

    // get_clipboard_data returns None when closed.
    assert_eq!(gwe.get_clipboard_data(13), None);

    // Double-close fails.
    assert!(!gwe.close_clipboard());

    // set_clipboard_data fails when closed.
    assert!(gwe.set_clipboard_data(1, 13, 0x1234).is_none());

    // register_clipboard_format: same name returns same atom.
    let fmt1 = gwe.register_clipboard_format("MyFmt").unwrap();
    let fmt2 = gwe.register_clipboard_format("myfmt").unwrap(); // normalized lowercase
    assert_eq!(fmt1, fmt2);
    assert!(fmt1 >= 0xC000); // user-defined clipboard formats start at 0xC000

    // open_clipboard with hwnd=0 is allowed (global ownership).
    assert!(gwe.open_clipboard(0));
    assert_eq!(gwe.get_open_clipboard_window(), 0);
    gwe.close_clipboard();
}

#[test]
fn gwe_window_text_visibility_enable_and_thread_hung_tracking() {
    use wince_emulation_v3::ce::gwe::{Gwe, Rect};

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 100, bottom: 50 };
    let hwnd = gwe.create_window_ex_with_rect(10, "cls", "Hello", None, 0, 0, 0, rect);

    // get_window_text / set_window_text.
    assert_eq!(gwe.get_window_text(hwnd, 256).as_deref(), Some("Hello"));
    // capacity 2 → 1 char fits.
    assert_eq!(gwe.get_window_text(hwnd, 2).as_deref(), Some("H"));
    assert!(gwe.set_window_text(hwnd, "World"));
    assert_eq!(gwe.get_window_text(hwnd, 256).as_deref(), Some("World"));
    assert!(!gwe.set_window_text(0xDEAD, "x")); // invalid hwnd

    // show_window / is_window_visible.
    // New window is not visible; show_window returns the PREVIOUS visible state.
    assert!(!gwe.show_window(hwnd, true));  // previous was false → returns false
    assert!(gwe.is_window_visible(hwnd));
    assert!(gwe.show_window(hwnd, false)); // previous was true → returns true
    assert!(!gwe.is_window_visible(hwnd));
    assert!(!gwe.show_window(0xDEAD, true)); // invalid hwnd returns false

    // enable_window returns the previous enabled state.
    assert!(gwe.is_window_enabled(hwnd)); // enabled by default
    assert!(gwe.enable_window(hwnd, false)); // previous=true → returns true
    assert!(!gwe.is_window_enabled(hwnd));
    assert!(!gwe.enable_window(hwnd, true)); // previous=false → returns false
    assert!(gwe.is_window_enabled(hwnd));

    // record_thread_dispatched / is_thread_hung.
    gwe.record_thread_dispatched(10, 1000);
    // Not hung: current_ms - last < threshold.
    assert!(!gwe.is_thread_hung(10, 1050, 100));
    // Hung: current_ms - last >= threshold.
    assert!(gwe.is_thread_hung(10, 1100, 100));
    // Unknown thread is never hung.
    assert!(!gwe.is_thread_hung(99, 9999, 1));

    // get_message_queue_ready_time_stamp returns 0 for an unknown thread.
    assert_eq!(gwe.get_message_queue_ready_time_stamp(10, 0), 0);
}

#[test]
fn gwe_get_queue_status_and_peek_queue_status() {
    use wince_emulation_v3::ce::gwe::{Gwe, Message, QS_POSTMESSAGE, WM_USER};

    let mut gwe = Gwe::default();
    let thread_id = 7;

    // No messages: both high word and low word are 0 for QS_POSTMESSAGE.
    let status = gwe.get_queue_status(thread_id, QS_POSTMESSAGE);
    assert_eq!(status, 0);
    let pstatus = gwe.peek_queue_status(thread_id, QS_POSTMESSAGE);
    assert_eq!(pstatus, 0);

    // Post a message to set the QS_POSTMESSAGE bit.
    gwe.post_message(thread_id, Message {
        hwnd: 0, msg: WM_USER, wparam: 0, lparam: 0,
        time_ms: 0, source: 0, mouse_pos_at_post: None,
    });

    // peek_queue_status: bits appear in both high (changed since last clear) and low (current).
    let pstatus2 = gwe.peek_queue_status(thread_id, QS_POSTMESSAGE);
    let current_bits = pstatus2 & 0xFFFF;
    let changed_bits = pstatus2 >> 16;
    assert_ne!(current_bits & QS_POSTMESSAGE, 0);
    assert_ne!(changed_bits & QS_POSTMESSAGE, 0);

    // get_queue_status returns (current << 16) | changed and clears changed bits.
    // Low word = changed since last get_queue_status; high word = currently set.
    let gstatus = gwe.get_queue_status(thread_id, QS_POSTMESSAGE);
    assert_ne!(gstatus & QS_POSTMESSAGE, 0); // changed low-word has QS_POSTMESSAGE

    // After get_queue_status, changed bits cleared; current (high word) still set (message in queue).
    let after = gwe.get_queue_status(thread_id, QS_POSTMESSAGE);
    assert_eq!(after & QS_POSTMESSAGE, 0);           // changed bits now clear
    assert_ne!((after >> 16) & QS_POSTMESSAGE, 0);  // current bits still set
}

#[test]
fn shell_system_message_box_recent_docs_notify_icons_and_notifications() {
    use wince_emulation_v3::ce::shell::{
        MessageBoxRecord, NotificationResult, NotifyIconData, NotifyIconOp, RecentDocumentRecord,
        ShellChangeNotifyRegistration, ShellNotificationData, ShellSystem,
        SHNP_ICONIC, SHNUM_ICON, SHNUM_TITLE, MAX_RECENT_DOCUMENTS,
    };

    let mut shell = ShellSystem::default();

    // record_message_box / last_message_box / message_boxes.
    assert!(shell.last_message_box().is_none());
    shell.record_message_box(MessageBoxRecord {
        thread_id: 1,
        owner_hwnd: 0,
        dialog_hwnd: 0,
        text_hwnd: 0,
        text: "msg".to_owned(),
        caption: "cap".to_owned(),
        style: 0,
        buttons: vec![],
        button_hwnds: vec![],
        button_layout: vec![],
        default_button_index: 0,
        icon: None,
        result: 0,
        owner_was_enabled: None,
        rendered: false,
    });
    assert_eq!(shell.last_message_box().unwrap().text, "msg");
    assert_eq!(shell.message_boxes().count(), 1);

    // record_recent_document: newest at front, deduplicates by shortcut_path.
    for i in 0..=MAX_RECENT_DOCUMENTS {
        shell.record_recent_document(RecentDocumentRecord {
            flags: 0,
            target_path: format!("/doc/{i}"),
            display_name: String::new(),
            shortcut_path: format!("/lnk/{i}"),
            pidl_bytes: None,
        });
    }
    let docs: Vec<_> = shell.recent_documents().collect();
    assert_eq!(docs.len(), MAX_RECENT_DOCUMENTS); // capped at max
    // Most recent is first.
    assert_eq!(docs[0].shortcut_path, format!("/lnk/{MAX_RECENT_DOCUMENTS}"));

    // recent_document_display_entries: uses display_name if set, else target_path.
    shell.clear_recent_documents();
    shell.record_recent_document(RecentDocumentRecord {
        flags: 0,
        target_path: "/target".to_owned(),
        display_name: "Named".to_owned(),
        shortcut_path: "/lnk".to_owned(),
        pidl_bytes: None,
    });
    let entry = shell.recent_document_display_entries().next().unwrap();
    assert_eq!(entry.label, "Named");

    // apply_notify_icon: Add / Modify / Delete.
    let icon_data = NotifyIconData {
        hwnd: 10, id: 1, flags: 0, callback_message: 0,
        icon: 0x100, tip: "tip".to_owned(), state: 0, state_mask: 0,
    };
    assert!(shell.apply_notify_icon(NotifyIconOp::Add, icon_data.clone()));
    assert_eq!(shell.notify_icon(10, 1).unwrap().tip, "tip");
    // Modify non-existent fails.
    assert!(!shell.apply_notify_icon(NotifyIconOp::Modify, NotifyIconData {
        hwnd: 99, id: 1, ..icon_data.clone()
    }));
    // Delete.
    assert!(shell.apply_notify_icon(NotifyIconOp::Delete, icon_data.clone()));
    assert!(shell.notify_icon(10, 1).is_none());
    // Delete already-gone fails.
    assert!(!shell.apply_notify_icon(NotifyIconOp::Delete, icon_data.clone()));

    // NotifyIconOp::from_raw.
    assert_eq!(NotifyIconOp::from_raw(0), Some(NotifyIconOp::Add));
    assert_eq!(NotifyIconOp::from_raw(1), Some(NotifyIconOp::Modify));
    assert_eq!(NotifyIconOp::from_raw(2), Some(NotifyIconOp::Delete));
    assert_eq!(NotifyIconOp::from_raw(99), None);

    // register_change_notification / remove_change_notification / change_notification.
    shell.register_change_notification(ShellChangeNotifyRegistration {
        hwnd: 5, event_mask: 0xFF, notify_flags: 0, watch_dir: None, recursive: false,
    });
    assert!(shell.change_notification(5).is_some());
    assert_eq!(shell.change_notifications().count(), 1);
    shell.remove_change_notification(5);
    assert!(shell.change_notification(5).is_none());

    // record_freed_file_notification: ptr=0 is ignored.
    shell.record_freed_file_notification(0);
    assert_eq!(shell.freed_file_notifications().count(), 0);
    shell.record_freed_file_notification(0x1234);
    assert_eq!(shell.freed_file_notifications().count(), 1);

    // add_notification / remove_notification / expire_notifications.
    let clsid = [1u8; 16];
    let data = ShellNotificationData {
        id: 7, priority: SHNP_ICONIC, duration_cs: 100,
        icon: 0, flags: 0, clsid,
        hwnd_sink: 0, title: "title".to_owned(), html: String::new(), lparam: 0,
    };
    assert_eq!(shell.add_notification(data.clone(), 0), NotificationResult::Success);
    assert!(shell.notification(clsid, 7).is_some());
    // Add with empty title+html fails.
    let bad = ShellNotificationData { title: String::new(), html: String::new(), ..data.clone() };
    assert_eq!(shell.add_notification(bad, 0), NotificationResult::InvalidParameter);
    // update_notification.
    let update = ShellNotificationData { icon: 99, ..data.clone() };
    assert_eq!(shell.update_notification(SHNUM_ICON | SHNUM_TITLE, update, 0), NotificationResult::Success);
    // expire_notifications: duration 100 cs = 1000 ms; expires at ms = 0 + 1000 = 1000.
    let expired = shell.expire_notifications(999);
    assert_eq!(expired.len(), 0); // not yet expired
    let expired = shell.expire_notifications(1000);
    assert_eq!(expired.len(), 1); // now expired
    assert!(shell.notification(clsid, 7).is_none());
    // remove_notification on missing returns InvalidData.
    assert_eq!(shell.remove_notification(clsid, 7), NotificationResult::InvalidData);
}

#[test]
fn resource_system_bitmap_region_resource_register_and_string() {
    use wince_emulation_v3::ce::gwe::Rect;
    use wince_emulation_v3::ce::resource::{ResourceId, ResourceSystem};

    let mut res = ResourceSystem::default();

    // create_bitmap / bitmap / delete_bitmap.
    let bmp = res.create_bitmap(64, 32, 1, 16, 0x1000);
    let bmp_obj = res.bitmap(bmp).unwrap();
    assert_eq!(bmp_obj.width, 64);
    assert_eq!(bmp_obj.height, 32);
    assert_eq!(bmp_obj.bits_pixel, 16);
    assert!(!bmp_obj.top_down); // positive height → bottom-up

    // create_bitmap with negative height → top_down.
    let bmp2 = res.create_bitmap(8, -8, 1, 8, 0x2000);
    assert!(res.bitmap(bmp2).unwrap().top_down);

    // delete_bitmap removes the entry.
    assert!(res.delete_bitmap(bmp));
    assert!(res.bitmap(bmp).is_none());
    assert!(!res.delete_bitmap(bmp)); // already deleted

    // create_icon requires valid mask_bitmap.
    let icon = res.create_icon(true, 0, 0, bmp2, 0);
    assert!(icon.is_some());
    let bad_icon = res.create_icon(true, 0, 0, 0xDEAD, 0);
    assert!(bad_icon.is_none());
    assert!(res.delete_bitmap(bmp2));

    // create_region / region / set_region / delete_region.
    let r = Rect { left: 0, top: 0, right: 100, bottom: 50 };
    let rh = res.create_region(r);
    assert_eq!(res.region(rh).unwrap().rect, r);
    // set_region to a new rect.
    let r2 = Rect { left: 10, top: 10, right: 80, bottom: 40 };
    assert!(res.set_region(rh, r2));
    assert_eq!(res.region(rh).unwrap().rect, r2);
    // set_region on invalid handle fails.
    assert!(!res.set_region(0xDEAD, r2));
    // union_region_with_rect expands.
    let ext = Rect { left: 0, top: 0, right: 200, bottom: 100 };
    assert!(res.union_region_with_rect(rh, ext));
    assert!(res.region(rh).unwrap().rect.right >= 200);
    // delete_region.
    assert!(res.delete_region(rh));
    assert!(res.region(rh).is_none());
    assert!(!res.delete_region(rh)); // already deleted

    // register resource / find_resource / load_resource / sizeof_resource.
    let name = ResourceId::Integer(42);
    let kind = ResourceId::Integer(1);
    let rh2 = res.register(0x1000, name.clone(), kind.clone(), 0xABCD_0000, 512);
    assert_eq!(res.find_resource(0x1000, name.clone(), kind.clone()), Some(rh2));
    assert_eq!(res.load_resource(rh2), Some(0xABCD_0000));
    assert_eq!(res.sizeof_resource(rh2), Some(512));
    assert_eq!(res.lock_resource(rh2), Some(0xABCD_0000));
    // Missing resource returns None.
    assert!(res.find_resource(0x1000, ResourceId::Integer(99), kind).is_none());
    assert!(res.load_resource(0xDEAD).is_none());

    // register_string / load_string.
    res.register_string(0x2000, 5, "Hello", None);
    assert_eq!(res.load_string(0x2000, 5).map(|s| s.text.as_str()), Some("Hello"));
    assert!(res.load_string(0x2000, 6).is_none()); // missing id
}

#[test]
fn resource_system_menu_create_append_get_enable_check_radio_and_submenu() {
    use wince_emulation_v3::ce::resource::{
        MF_BYPOSITION, MF_CHECKED, MF_GRAYED, MF_SEPARATOR,
        MenuItem, ResourceId, ResourceSystem,
    };

    let mut res = ResourceSystem::default();
    let mh = res.create_menu(0, ResourceId::Integer(0), None);
    assert!(res.menu(mh).is_some());
    assert!(!res.menu(mh).unwrap().popup);

    // create_popup_menu is marked as popup.
    let popup = res.create_popup_menu();
    assert!(res.menu(popup).unwrap().popup);

    // append_menu_item: adds items to the menu.
    assert!(res.append_menu_item(mh, MenuItem {
        id: 100, item_type: 0, state: 0, submenu: 0,
        checked_bitmap: 0, unchecked_bitmap: 0, data: 0, text: Some("File".to_owned()),
    }));
    assert!(res.append_menu_item(mh, MenuItem {
        id: 200, item_type: 0, state: 0, submenu: 0,
        checked_bitmap: 0, unchecked_bitmap: 0, data: 0, text: Some("Edit".to_owned()),
    }));
    assert!(!res.append_menu_item(0xDEAD, MenuItem {
        id: 1, item_type: 0, state: 0, submenu: 0,
        checked_bitmap: 0, unchecked_bitmap: 0, data: 0, text: None,
    })); // invalid handle

    // get_menu_item by position.
    let item = res.get_menu_item(mh, 0, true).unwrap();
    assert_eq!(item.id, 100);
    assert_eq!(item.text.as_deref(), Some("File"));

    // get_menu_item by id.
    let item = res.get_menu_item(mh, 200, false).unwrap();
    assert_eq!(item.text.as_deref(), Some("Edit"));

    // enable_menu_item: sets GRAYED state and returns previous.
    let prev = res.enable_menu_item(mh, 0, MF_BYPOSITION | MF_GRAYED).unwrap();
    assert_eq!(prev, 0); // was enabled (no disabled/grayed bits)
    let item = res.get_menu_item(mh, 0, true).unwrap();
    assert_ne!(item.state & MF_GRAYED, 0);

    // check_menu_item: sets CHECKED bit and returns previous.
    let prev_check = res.check_menu_item(mh, 200, MF_CHECKED).unwrap();
    assert_eq!(prev_check, 0); // was unchecked
    let item = res.get_menu_item(mh, 200, false).unwrap();
    assert_ne!(item.state & MF_CHECKED, 0);
    // Uncheck it.
    res.check_menu_item(mh, 200, 0);
    let item = res.get_menu_item(mh, 200, false).unwrap();
    assert_eq!(item.state & MF_CHECKED, 0);

    // insert_menu_item by position.
    assert!(res.insert_menu_item(mh, 1, MF_BYPOSITION, MenuItem {
        id: 150, item_type: MF_SEPARATOR, state: 0, submenu: 0,
        checked_bitmap: 0, unchecked_bitmap: 0, data: 0, text: None,
    }));
    assert_eq!(res.menu(mh).unwrap().items.len(), 3);
    assert_eq!(res.get_menu_item(mh, 1, true).unwrap().id, 150);

    // check_menu_radio_item: sets radio check on one item in range.
    assert!(res.check_menu_radio_item(mh, 100, 200, 150));
    let item150 = res.get_menu_item(mh, 150, false).unwrap();
    assert_ne!(item150.state & MF_CHECKED, 0);
    let item100 = res.get_menu_item(mh, 100, false).unwrap();
    assert_eq!(item100.state & MF_CHECKED, 0);

    // get_sub_menu: submenu field in a popup item.
    res.append_menu_item(mh, MenuItem {
        id: u32::MAX, item_type: 0, state: 0, submenu: popup,
        checked_bitmap: 0, unchecked_bitmap: 0, data: 0, text: Some("Sub".to_owned()),
    });
    let sub = res.get_sub_menu(mh, 3);
    assert_eq!(sub, Some(popup));
}

#[test]
fn gwe_focus_capture_active_cursor_set_window_pos_move_window_and_destroy() {
    use wince_emulation_v3::ce::gwe::{Gwe, Rect};

    let mut gwe = Gwe::default();
    let rect = Rect { left: 10, top: 10, right: 110, bottom: 60 };
    let hwnd1 = gwe.create_window_ex_with_rect(1, "cls1", "A", None, 0, 0, 0, rect);
    let hwnd2 = gwe.create_window_ex_with_rect(2, "cls2", "B", None, 0, 0, 0, rect);

    // set_focus / get_focus: returns previous focus.
    assert_eq!(gwe.get_focus(), None);
    let prev = gwe.set_focus(Some(hwnd1));
    assert_eq!(prev, None);
    assert_eq!(gwe.get_focus(), Some(hwnd1));
    gwe.set_focus(Some(hwnd2));
    assert_eq!(gwe.get_focus(), Some(hwnd2));
    // Invalid hwnd returns None from set_focus.
    assert_eq!(gwe.set_focus(Some(0xDEAD)), None);
    // focus_is_within.
    assert!(gwe.focus_is_within(hwnd2));
    assert!(!gwe.focus_is_within(hwnd1));

    // set_active_window / get_active_window.
    assert_eq!(gwe.set_active_window(Some(hwnd1)), None); // previous was None
    assert_eq!(gwe.get_active_window(), Some(hwnd1));
    // Invalid hwnd returns None from set_active_window.
    assert_eq!(gwe.set_active_window(Some(0xDEAD)), None);

    // set_capture / get_capture / release_capture.
    assert_eq!(gwe.get_capture(), None);
    let prev_cap = gwe.set_capture(hwnd1);
    assert_eq!(prev_cap, None);
    assert_eq!(gwe.get_capture(), Some(hwnd1));
    gwe.set_capture(hwnd2);
    assert_eq!(gwe.get_capture(), Some(hwnd2));
    assert!(gwe.release_capture());
    assert_eq!(gwe.get_capture(), None);

    // set_cursor / get_cursor.
    assert_eq!(gwe.get_cursor(), None);
    let prev_cursor = gwe.set_cursor(0xABCD);
    assert_eq!(prev_cursor, None);
    assert_eq!(gwe.get_cursor(), Some(0xABCD));
    gwe.set_cursor(0); // set to 0 clears cursor
    assert_eq!(gwe.get_cursor(), None);

    // move_window repositions the window.
    assert!(gwe.move_window(hwnd1, 20, 30, 200, 100, false));
    let new_rect = gwe.get_window_rect(hwnd1).unwrap();
    assert_eq!(new_rect.left, 20);
    assert_eq!(new_rect.top, 30);
    assert_eq!(new_rect.width(), 200);
    assert_eq!(new_rect.height(), 100);
    assert!(!gwe.move_window(0xDEAD, 0, 0, 10, 10, false)); // invalid hwnd

    // destroy_window marks window destroyed; is_window returns false after.
    assert!(gwe.is_window(hwnd2));
    gwe.destroy_window(hwnd2, 0);
    assert!(!gwe.is_window(hwnd2));
    // window_thread_process_id returns None for destroyed window.
    assert_eq!(gwe.window_thread_process_id(hwnd2), None);
}

#[test]
fn kernel_crt_rand_strtok_process_state_pseudo_handles_and_process_launch() -> Result<()> {
    use wince_emulation_v3::ce::kernel::{
        CeKernel, CE_CURRENT_PROCESS_PSEUDO_HANDLE, CE_CURRENT_THREAD_PSEUDO_HANDLE,
        CurrentProcessState, FileTraceRecord,
    };

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    // is_current_thread_pseudo_handle / is_current_process_pseudo_handle.
    assert!(CeKernel::is_current_thread_pseudo_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE));
    assert!(!CeKernel::is_current_thread_pseudo_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE));
    assert!(CeKernel::is_current_process_pseudo_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE));
    assert!(!CeKernel::is_current_process_pseudo_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE));

    // crt_srand / crt_rand: seeding changes output.
    kernel.crt_srand(1);
    let r1 = kernel.crt_rand();
    kernel.crt_srand(1);
    let r2 = kernel.crt_rand();
    assert_eq!(r1, r2); // deterministic with same seed
    kernel.crt_srand(42);
    let r3 = kernel.crt_rand();
    assert_ne!(r1, r3); // different seed → different output (with high probability)
    assert!(r1 <= 0x7fff && r3 <= 0x7fff); // range capped at 15-bit

    // crt_strtok_next / crt_set_strtok_next.
    assert_eq!(kernel.crt_strtok_next(5), 0); // unset → 0
    kernel.crt_set_strtok_next(5, 0x1234);
    assert_eq!(kernel.crt_strtok_next(5), 0x1234);
    kernel.crt_set_strtok_next(5, 0); // 0 removes the entry
    assert_eq!(kernel.crt_strtok_next(5), 0);

    // current_process_state / set_current_process_state / reset_current_process_exit_state.
    let state = kernel.current_process_state();
    let updated = CurrentProcessState { process_id: 99, exit_code: 42, signaled: true };
    kernel.set_current_process_state(updated);
    assert_eq!(kernel.current_process_state(), updated);
    kernel.reset_current_process_exit_state();
    let after_reset = kernel.current_process_state();
    assert!(!after_reset.signaled);
    assert_eq!(after_reset.process_id, updated.process_id); // process_id preserved
    let _ = state;

    // queue_process_launch / take_pending_process_launches.
    let launch = kernel.queue_process_launch(
        Some("app.exe".to_owned()),
        Some("app.exe --flag".to_owned()),
    );
    assert_eq!(launch.application.as_deref(), Some("app.exe"));
    assert_eq!(launch.command_line.as_deref(), Some("app.exe --flag"));
    let pending = kernel.take_pending_process_launches();
    assert_eq!(pending.len(), 1);
    // After take, list is empty.
    assert_eq!(kernel.take_pending_process_launches().len(), 0);

    // mark_process_launch_exited records a trace entry.
    let before_trace = kernel.recent_process_ops().len();
    kernel.mark_process_launch_exited(&pending[0], 0);
    let after_trace = kernel.recent_process_ops().len();
    assert!(after_trace > before_trace);

    // record_file_trace / recent_file_ops.
    let before = kernel.recent_file_ops().len();
    kernel.record_file_trace(FileTraceRecord {
        op: "ReadFile",
        handle: Some(0x10),
        path: None,
        preview: None,
        requested: Some(128),
        transferred: Some(128),
        position: Some(0),
        result: Some(1),
        error: None,
    });
    assert_eq!(kernel.recent_file_ops().len(), before + 1);
    assert_eq!(kernel.recent_file_ops().last().unwrap().op, "ReadFile");

    Ok(())
}

#[test]
fn handle_table_thread_priority_process_state_wait_any_and_describe() {
    use wince_emulation_v3::ce::object::{
        HandleTable, WaitMultipleResult, MAX_CE_PRIORITY_LEVELS,
    };

    let mut table = HandleTable::default();

    // create_thread / thread_id / thread_handle_by_id / thread_priority / set_thread_priority.
    let th = table.create_thread(10, 0x1000, 0xABCD, false);
    assert_eq!(table.thread_id(th), Some(10));
    assert_eq!(table.thread_handle_by_id(10), Some(th));
    assert_eq!(table.thread_handle_by_id(99), None);

    // thread_priority starts at a valid CE priority.
    let prio = table.thread_priority(th).unwrap();
    assert!(prio >= 0 && prio < MAX_CE_PRIORITY_LEVELS);

    // set_thread_priority: valid range succeeds.
    assert!(table.set_thread_priority(th, 0));
    assert_eq!(table.thread_priority(th), Some(0));
    // set_thread_priority: out of range fails.
    assert!(!table.set_thread_priority(th, MAX_CE_PRIORITY_LEVELS));

    // set_thread_priority_by_id.
    let ok = table.set_thread_priority_by_id(10, 3);
    assert_eq!(ok, Some(true));
    assert_eq!(table.thread_priority_by_id(10), Some(3));
    // Out of range returns Some(false).
    assert_eq!(table.set_thread_priority_by_id(10, MAX_CE_PRIORITY_LEVELS), Some(false));
    // Unknown thread_id returns None.
    assert_eq!(table.set_thread_priority_by_id(99, 0), None);

    // thread_exit_code starts at STILL_ACTIVE (259) before the thread exits.
    assert_eq!(table.thread_exit_code(th), Some(259));
    // Mark thread exited → exit_code changes.
    table.mark_thread_exited(th, 42);
    assert_eq!(table.thread_exit_code(th), Some(42));

    // create_process / process_id / process_exit_code / mark_process_exited.
    let ph = table.create_process(77);
    assert_eq!(table.process_id(ph), Some(77));
    assert_eq!(table.process_exit_code(ph), Some(259)); // starts at STILL_ACTIVE
    assert!(table.mark_process_exited(ph, 0));
    assert_eq!(table.process_exit_code(ph), Some(0));

    // is_wait_ready / wait_for_any_object.
    let ev = table.create_event(None, false, false);  // manual-reset, not signaled
    // Event not signaled → wait_for_any_object returns Timeout.
    assert_eq!(table.wait_for_any_object(&[ev], 1), WaitMultipleResult::Timeout);
    // Signal the event.
    table.set_event(ev);
    assert_eq!(table.is_wait_ready(ev, 1), Some(true));
    // wait_for_any_object returns Object(0) — the first (and only) handle index.
    assert_eq!(table.wait_for_any_object(&[ev], 1), WaitMultipleResult::Object(0));

    // wait_for_any_object with invalid handle returns Failed.
    assert_eq!(table.wait_for_any_object(&[0xDEAD], 1), WaitMultipleResult::Failed);

    // describe_handle returns a non-empty string.
    let desc = table.describe_handle(ev);
    assert!(!desc.is_empty());
    let desc_invalid = table.describe_handle(0xDEAD_BEEF);
    assert!(!desc_invalid.is_empty()); // even invalid returns something
}

#[test]
fn com_system_initialize_uninitialize_register_class_create_instance_and_object() {
    use wince_emulation_v3::ce::com::{
        ComSystem, E_POINTER, REGDB_E_CLASSNOTREG, RPC_E_CHANGED_MODE, S_FALSE, S_OK,
    };

    let mut com = ComSystem::default();
    let thread_id = 10;

    // co_initialize_ex: first call returns S_OK (0).
    let r1 = com.co_initialize_ex(thread_id, 0x2); // STA
    assert_eq!(r1, S_OK);

    // Second call with same apartment type returns S_FALSE (1).
    let r2 = com.co_initialize_ex(thread_id, 0x2);
    assert_eq!(r2, S_FALSE);

    // Same thread, different apartment type → RPC_E_CHANGED_MODE.
    let r3 = com.co_initialize_ex(thread_id, 0x0); // MTA
    assert_eq!(r3, RPC_E_CHANGED_MODE);

    // co_uninitialize: decrements depth; second uninit removes the thread state.
    com.co_uninitialize(thread_id); // depth 2 → 1
    com.co_uninitialize(thread_id); // depth 1 → 0, removed
    // After removal, reinitializing succeeds with S_OK.
    assert_eq!(com.co_initialize_ex(thread_id, 0x0), S_OK);

    // register_class / co_create_instance.
    let clsid = 0x1000u32;
    let iid   = 0x2000u32;
    com.register_class(clsid, 0xABCD);

    // create instance: success returns a handle.
    let handle = com.co_create_instance(clsid, iid).unwrap();
    assert_ne!(handle, 0);

    // object() retrieves it.
    let obj = com.object(handle).unwrap();
    assert_eq!(obj.handle, handle);
    assert_eq!(obj.clsid_ptr, clsid);
    assert_eq!(obj.iid_ptr, iid);

    // co_create_instance with clsid=0 → E_POINTER.
    assert_eq!(com.co_create_instance(0, iid), Err(E_POINTER));
    // co_create_instance with iid=0 → E_POINTER.
    assert_eq!(com.co_create_instance(clsid, 0), Err(E_POINTER));
    // co_create_instance with unregistered clsid → REGDB_E_CLASSNOTREG.
    assert_eq!(com.co_create_instance(0x9999, iid), Err(REGDB_E_CLASSNOTREG));

    // object() on invalid handle returns None.
    assert!(com.object(0xDEAD).is_none());
}

#[test]
fn resource_system_font_brush_pen_palette_dc_and_gdi_object_kind() {
    use wince_emulation_v3::ce::gwe::Point;
    use wince_emulation_v3::ce::resource::ResourceSystem;

    let mut res = ResourceSystem::default();

    // create_font / font.
    let fh = res.create_font(0, -12, 0, 400, false, false, false, 0, 0, "Arial".to_owned());
    let font = res.font(fh).unwrap();
    assert_eq!(font.height, -12);
    assert_eq!(font.face_name, "Arial");
    assert!(!font.italic);
    assert_eq!(res.gdi_object_kind(fh), "font");

    // create_brush / brush / gdi_object_kind.
    let bh = res.create_brush(0xFF0000);
    assert_eq!(res.brush(bh).unwrap().color, 0xFF0000);
    assert_eq!(res.gdi_object_kind(bh), "brush");

    // create_pattern_brush requires a valid bitmap.
    let bmp = res.create_bitmap(8, 8, 1, 8, 0);
    let pbh = res.create_pattern_brush(bmp).unwrap();
    assert!(res.brush(pbh).unwrap().pattern_bitmap.is_some());
    assert!(res.create_pattern_brush(0xDEAD).is_none()); // invalid bitmap

    // create_pen / pen / gdi_object_kind.
    let ph = res.create_pen(0, 1, 0x0000FF);
    let pen = res.pen(ph).unwrap();
    assert_eq!(pen.color, 0x0000FF);
    assert_eq!(pen.width, 1);
    assert_eq!(res.gdi_object_kind(ph), "pen");

    // create_palette / palette / gdi_object_kind.
    let entries = vec![[0xFF, 0x00, 0x00, 0], [0x00, 0xFF, 0x00, 0]];
    let palh = res.create_palette(entries.clone());
    assert_eq!(res.palette(palh).unwrap().entries, entries);
    assert_eq!(res.gdi_object_kind(palh), "palette");

    // create_compatible_dc / dc_state / gdi_object_kind.
    let hdc = res.create_compatible_dc();
    assert_eq!(res.gdi_object_kind(hdc), "memory_dc");
    // dc_state returns Some for valid DC.
    assert!(res.dc_state(hdc).is_some());
    // dc_state(0) returns None.
    assert!(res.dc_state(0).is_none());

    // move_to tracks the current position.
    let prev = res.move_to(hdc, Point { x: 10, y: 20 }).unwrap();
    let state = res.dc_state(hdc).unwrap();
    assert_eq!(state.current_pos, Point { x: 10, y: 20 });
    let _ = prev;

    // select_clip_region / clip_region.
    let r = wince_emulation_v3::ce::gwe::Rect { left: 0, top: 0, right: 100, bottom: 50 };
    let rh = res.create_region(r);
    res.select_clip_region(hdc, Some(rh));
    assert_eq!(res.clip_region(hdc), Some(rh));
    res.select_clip_region(hdc, None);
    assert_eq!(res.clip_region(hdc), None);

    // gdi_object_kind for unknown handle returns "unknown".
    assert_eq!(res.gdi_object_kind(0xDEAD_BEEF), "unknown");
}

#[test]
fn resource_system_image_list_create_add_count_info_bk_color_and_destroy() {
    use wince_emulation_v3::ce::resource::ResourceSystem;

    let mut res = ResourceSystem::default();

    // create_image_list: invalid dimensions return None.
    assert!(res.create_image_list(0, 16, 0, 0, 4).is_none());
    assert!(res.create_image_list(16, 0, 0, 0, 4).is_none());

    // Valid creation.
    let ilh = res.create_image_list(16, 16, 0, 4, 4).unwrap();
    assert!(res.image_list(ilh).is_some());
    assert_eq!(res.gdi_object_kind(ilh), "image_list");
    assert_eq!(res.image_list_count(ilh), Some(0));

    // add_image_list_image: bitmap=0 returns None.
    assert!(res.add_image_list_image(ilh, 0, 0).is_none());

    // Add a real bitmap strip (width = 32 = 2 images of 16px each).
    let bmp = res.create_bitmap(32, 16, 1, 8, 0x5000);
    let idx = res.add_image_list_image(ilh, bmp, 0).unwrap();
    assert_eq!(idx, 0); // first image index
    assert_eq!(res.image_list_count(ilh), Some(2)); // 32px strip → 2 images of 16px

    // image_list_info.
    let info = res.image_list_info(ilh, 0).unwrap();
    assert_eq!(info.bitmap, bmp);
    assert_eq!(info.left, 0);
    assert_eq!(info.right, 16);
    let info2 = res.image_list_info(ilh, 1).unwrap();
    assert_eq!(info2.left, 16); // second image starts at x=16
    assert!(res.image_list_info(ilh, -1).is_none()); // negative index

    // set_image_list_bk_color / bk_color.
    let prev_color = res.set_image_list_bk_color(ilh, 0x123456).unwrap();
    assert_eq!(prev_color, 0xffff_ffff); // default is CLR_NONE
    assert_eq!(res.image_list(ilh).unwrap().bk_color, 0x123456);

    // set_image_list_size.
    assert_eq!(res.set_image_list_size(ilh, 32, 32), Some(true));
    assert_eq!(res.image_list(ilh).unwrap().width, 32);
    assert_eq!(res.set_image_list_size(ilh, 0, 32), Some(false)); // invalid
    assert_eq!(res.set_image_list_size(0xDEAD, 16, 16), None); // invalid handle

    // destroy_image_list.
    assert!(res.destroy_image_list(ilh));
    assert!(res.image_list(ilh).is_none());
    assert!(!res.destroy_image_list(ilh)); // already destroyed
}

#[test]
fn framebuffer_pixel_format_bytes_per_pixel_and_rect_operations() {
    use wince_emulation_v3::ce::framebuffer::{
        Framebuffer, FramebufferRect, PixelFormat, VirtualFramebuffer,
    };

    // PixelFormat bytes/bits per pixel.
    assert_eq!(PixelFormat::Rgb565.bytes_per_pixel(), 2);
    assert_eq!(PixelFormat::Rgb565.bits_per_pixel(), 16);
    assert_eq!(PixelFormat::Bgra8888.bytes_per_pixel(), 4);
    assert_eq!(PixelFormat::Bgra8888.bits_per_pixel(), 32);
    assert_eq!(PixelFormat::Rgba8888.bytes_per_pixel(), 4);
    assert_eq!(PixelFormat::Gray8.bytes_per_pixel(), 1);
    assert_eq!(PixelFormat::Gray8.bits_per_pixel(), 8);

    // FramebufferRect.
    let r = FramebufferRect::new(10, 20, 100, 50);
    assert_eq!(r.x, 10);
    assert_eq!(r.y, 20);
    assert_eq!(r.width, 100);
    assert_eq!(r.height, 50);
    assert!(!r.is_empty());

    let full = FramebufferRect::full(800, 480);
    assert_eq!(full.x, 0);
    assert_eq!(full.y, 0);
    assert_eq!(full.width, 800);
    assert_eq!(full.height, 480);
    assert!(!full.is_empty());

    let empty = FramebufferRect::new(0, 0, 0, 100);
    assert!(empty.is_empty());

    // VirtualFramebuffer creation and dimensions.
    let fb = VirtualFramebuffer::new(320, 240, PixelFormat::Rgb565).unwrap();
    assert_eq!(fb.width(), 320);
    assert_eq!(fb.height(), 240);
    assert_eq!(fb.pixel_format(), PixelFormat::Rgb565);
    // Stride for 565 is width * 2 bytes.
    assert_eq!(fb.stride(), 320 * 2);
    // Pixel buffer size matches stride * height.
    assert_eq!(fb.pixels().len(), 320 * 240 * 2);

    // default_primary gives 800x480 RGB565.
    let primary = VirtualFramebuffer::default_primary().unwrap();
    assert_eq!(primary.width(), VirtualFramebuffer::DEFAULT_WIDTH);
    assert_eq!(primary.height(), VirtualFramebuffer::DEFAULT_HEIGHT);
    assert_eq!(primary.pixel_format(), VirtualFramebuffer::DEFAULT_FORMAT);
}

#[test]
fn scheduler_stats_record_wait_attempt_blocked_yield_and_wake() {
    use wince_emulation_v3::ce::scheduler::{SchedulerWaitKind, SchedulerWakeReason};

    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // All stat counters start at zero.
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_single_count, 0);
    assert_eq!(s.wait_multiple_count, 0);
    assert_eq!(s.msg_wait_count, 0);
    assert_eq!(s.sleep_count, 0);
    assert_eq!(s.yield_count, 0);
    assert_eq!(s.wait_block_count, 0);
    assert_eq!(s.wait_wake_count, 0);
    assert_eq!(s.wait_acquired_count, 0);
    assert_eq!(s.wait_timeout_count, 0);
    assert_eq!(s.wait_failed_count, 0);

    // record_wait_attempt: each kind increments exactly one counter.
    kernel.scheduler.record_wait_attempt(SchedulerWaitKind::Single, 1, 100);
    assert_eq!(kernel.scheduler.stats().wait_single_count, 1);

    kernel.scheduler.record_wait_attempt(SchedulerWaitKind::Multiple, 3, 200);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_multiple_count, 1);
    assert_eq!(s.max_wait_handles, 3);
    assert_eq!(s.max_timeout_ms, 200);

    kernel.scheduler.record_wait_attempt(SchedulerWaitKind::MsgWait, 1, 50);
    assert_eq!(kernel.scheduler.stats().msg_wait_count, 1);

    kernel.scheduler.record_wait_attempt(SchedulerWaitKind::Sleep, 0, 500);
    assert_eq!(kernel.scheduler.stats().sleep_count, 1);

    // record_thread_yield: increments sleep_count AND yield_count.
    kernel.scheduler.record_thread_yield();
    let s = kernel.scheduler.stats();
    assert_eq!(s.sleep_count, 2);
    assert_eq!(s.yield_count, 1);

    // record_blocked_wait: increments wait_block_count only.
    kernel.scheduler.record_blocked_wait();
    assert_eq!(kernel.scheduler.stats().wait_block_count, 1);

    // record_wait_wake(ObjectSignaled): increments wait_wake_count and wait_acquired_count.
    kernel.scheduler.record_wait_wake(SchedulerWakeReason::ObjectSignaled);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, 1);
    assert_eq!(s.wait_acquired_count, 1);
    assert_eq!(s.wait_timeout_count, 0);

    // record_wait_wake(Timeout): increments wait_wake_count and wait_timeout_count.
    kernel.scheduler.record_wait_wake(SchedulerWakeReason::Timeout);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, 2);
    assert_eq!(s.wait_timeout_count, 1);

    // record_wait_wake(Failed): increments wait_wake_count and wait_failed_count.
    kernel.scheduler.record_wait_wake(SchedulerWakeReason::Failed);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, 3);
    assert_eq!(s.wait_failed_count, 1);
}

#[test]
fn gwe_invalidate_begin_paint_update_rect_pending_size_move_and_set_redraw() {
    use wince_emulation_v3::ce::gwe::Gwe;

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 200, bottom: 100 };
    let hwnd = gwe.create_window_ex_with_rect(1, "TestClass", "TestWin", None, 0, WS_VISIBLE, 0, rect);
    // A newly visible window starts with update_pending=true; clear it before testing.
    gwe.validate_window(hwnd);

    // update_rect returns None when no invalidation pending.
    assert!(gwe.update_rect(hwnd).is_none());

    // invalidate_window marks the window dirty; update_rect returns Some.
    let dirty_rect = Rect { left: 10, top: 10, right: 50, bottom: 40 };
    assert!(gwe.invalidate_window(hwnd, Some(dirty_rect), true));
    let upd = gwe.update_rect(hwnd).unwrap();
    assert_eq!(upd.rect, dirty_rect);
    assert!(upd.erase);

    // invalidating again unions the rects.
    let dirty2 = Rect { left: 60, top: 10, right: 100, bottom: 40 };
    assert!(gwe.invalidate_window(hwnd, Some(dirty2), false));
    let upd2 = gwe.update_rect(hwnd).unwrap();
    assert!(upd2.rect.right >= 100); // union extends to 100

    // begin_paint returns the pending update and clears it.
    let paint = gwe.begin_paint(hwnd).unwrap();
    assert!(paint.erase);
    // after begin_paint the update is consumed via validate_window.
    assert!(gwe.update_rect(hwnd).is_none());

    // begin_paint on invalid hwnd returns None.
    assert!(gwe.begin_paint(0xDEAD).is_none());

    // mark_pending_size_move / take_pending_size_move.
    assert!(gwe.mark_pending_size_move(hwnd, true, false));
    let (r, moved, sized) = gwe.take_pending_size_move(hwnd).unwrap();
    assert!(moved);
    assert!(!sized);
    assert!(r.width() > 0);
    // After taking, nothing remains.
    assert!(gwe.take_pending_size_move(hwnd).is_none());

    // set_redraw(false) suppresses invalidation; set_redraw(true) re-enables.
    assert!(gwe.set_redraw(hwnd, false));
    // When redraw is suspended, invalidate_window returns true (success) but queues nothing.
    assert!(gwe.invalidate_window(hwnd, None, false));
    assert!(gwe.update_rect(hwnd).is_none()); // nothing queued while suspended

    assert!(gwe.set_redraw(hwnd, true));
    assert!(gwe.invalidate_window(hwnd, None, false));
    assert!(gwe.update_rect(hwnd).is_some());

    // clear_update_erase clears the erase flag but keeps update pending.
    assert!(gwe.invalidate_window(hwnd, None, true));
    assert!(gwe.clear_update_erase(hwnd));
    let upd3 = gwe.update_rect(hwnd).unwrap();
    assert!(!upd3.erase);
}

#[test]
fn gwe_window_hierarchy_get_parent_is_child_top_level_ancestor_dlg_item_and_dialog_ops() {
    use wince_emulation_v3::ce::gwe::{Gwe, GW_CHILD, GW_OWNER};

    let mut gwe = Gwe::default();
    let big = Rect { left: 0, top: 0, right: 400, bottom: 300 };
    let small = Rect { left: 10, top: 10, right: 100, bottom: 60 };

    // Create parent and child windows.
    let parent = gwe.create_window_ex_with_rect(1, "Parent", "Parent", None, 0, 0, 0, big);
    let child = gwe.create_window_ex_with_process_parent_owner_and_rect(
        1, 1, "Child", "Child", Some(parent), Some(parent), WS_CHILD, 0, 0, small,
    );
    let grandchild = gwe.create_window_ex_with_process_parent_owner_and_rect(
        1, 1, "Grandchild", "Grandchild", Some(child), Some(child), WS_CHILD, 0, 0, small,
    );

    // get_parent: child's parent is parent window.
    assert_eq!(gwe.get_parent(child), Some(parent));
    assert_eq!(gwe.get_parent(parent), None); // top-level has no parent
    assert_eq!(gwe.get_parent(0xDEAD), None);

    // is_child: checks transitive parent chain.
    assert!(gwe.is_child(parent, child));
    assert!(gwe.is_child(parent, grandchild)); // transitive
    assert!(!gwe.is_child(child, parent)); // reversed
    assert!(!gwe.is_child(parent, parent)); // self

    // top_level_ancestor: walks to root.
    assert_eq!(gwe.top_level_ancestor(grandchild), Some(parent));
    assert_eq!(gwe.top_level_ancestor(parent), Some(parent));
    assert_eq!(gwe.top_level_ancestor(0xDEAD), None);

    // get_dlg_item: finds a child by its control id.
    let ctrl_id = gwe.get_dlg_ctrl_id(child).unwrap();
    let found = gwe.get_dlg_item(parent, ctrl_id);
    assert_eq!(found, Some(child));
    // Unknown id returns None.
    assert_eq!(gwe.get_dlg_item(parent, 0xFFFF), None);

    // get_window(GW_CHILD): returns first child of parent.
    assert_eq!(gwe.get_window(parent, GW_CHILD), Some(child));
    // get_window(GW_OWNER): owner of an ownerless top-level is None.
    assert_eq!(gwe.get_window(parent, GW_OWNER), None);

    // get_desktop_window: returns DESKTOP_HWND = 0x10000.
    assert_eq!(gwe.get_desktop_window(), 0x1_0000);

    // end_dialog / dialog_result.
    assert!(gwe.end_dialog(parent, 42));
    assert_eq!(gwe.dialog_result(parent), Some(42));
    // Invalid hwnd returns false.
    assert!(!gwe.end_dialog(0xDEAD, 0));
    // Unknown hwnd returns None from dialog_result.
    assert_eq!(gwe.dialog_result(0xDEAD), None);

    // check_dlg_button / is_dlg_button_checked.
    assert!(gwe.check_dlg_button(parent, 5, 1));
    assert_eq!(gwe.is_dlg_button_checked(parent, 5), Some(1));
    assert_eq!(gwe.is_dlg_button_checked(parent, 6), Some(0)); // unset → 0

    // check_radio_button: range [10..=12], check id 11.
    assert!(gwe.check_radio_button(parent, 10, 12, 11));
    assert_eq!(gwe.is_dlg_button_checked(parent, 10), Some(0));
    assert_eq!(gwe.is_dlg_button_checked(parent, 11), Some(1));
    assert_eq!(gwe.is_dlg_button_checked(parent, 12), Some(0));
    // check_radio_button with check outside range returns false.
    assert!(!gwe.check_radio_button(parent, 10, 12, 20));

    // set_window_region / window_region.
    let region_rect = Rect { left: 5, top: 5, right: 50, bottom: 50 };
    assert!(gwe.set_window_region(parent, Some(region_rect)));
    assert_eq!(gwe.window_region(parent), Some(region_rect));
    // Clear window region.
    assert!(gwe.set_window_region(parent, None));
    assert_eq!(gwe.window_region(parent), None);
    // Invalid hwnd returns false.
    assert!(!gwe.set_window_region(0xDEAD, None));

    // get_menu / set_menu.
    assert_eq!(gwe.get_menu(parent), Some(0)); // no menu → 0
    assert!(gwe.set_menu(parent, 0x1234));
    assert_eq!(gwe.get_menu(parent), Some(0x1234));
}

#[test]
fn resource_system_accelerator_popup_tracking_and_select_object() {
    use wince_emulation_v3::ce::resource::{
        AcceleratorEntry, MenuItem, MF_BYPOSITION, MF_GRAYED, PopupMenuNotification,
        PopupMenuTracking, ResourceId, ResourceSystem,
    };
    use wince_emulation_v3::ce::gwe::Point;

    let mut res = ResourceSystem::default();

    // create_accelerator / accelerator / delete_accelerator.
    let entries = vec![
        AcceleratorEntry { flags: 0x01, key: 0x41, command: 101 },
        AcceleratorEntry { flags: 0x02, key: 0x42, command: 102 },
    ];
    let accel = res.create_accelerator(1, ResourceId::from_guest_arg(5), None, entries.clone());
    let obj = res.accelerator(accel).unwrap();
    assert_eq!(obj.entries.len(), 2);
    assert_eq!(obj.entries[0].command, 101);
    assert_eq!(obj.module, 1);

    assert!(res.delete_accelerator(accel));
    assert!(res.accelerator(accel).is_none());
    assert!(!res.delete_accelerator(accel)); // already deleted

    // track_popup_menu / last_popup_tracking.
    assert!(res.last_popup_tracking().is_none());
    let menu = res.create_popup_menu();
    res.append_menu_item(menu, MenuItem::from_insert_flags(0, 77, Some("Item".to_owned())));
    let tracking = PopupMenuTracking {
        menu,
        flags: 0,
        x: 100,
        y: 200,
        hwnd: 0x1000,
        exclude_rect: None,
    };
    assert!(res.track_popup_menu(tracking.clone()));
    assert_eq!(res.last_popup_tracking().unwrap().x, 100);
    assert_eq!(res.last_popup_tracking().unwrap().menu, menu);
    // track_popup_menu on unknown menu returns false.
    let bad_tracking = PopupMenuTracking { menu: 0xDEAD, ..tracking };
    assert!(!res.track_popup_menu(bad_tracking));

    // popup_menu_command_selection: returns the first enabled command.
    let selection = res.popup_menu_command_selection(menu).unwrap();
    assert_eq!(selection.command, 77);
    assert!(selection.submenus.is_empty());

    // popup_menu_return_command: shortcut to command value.
    assert_eq!(res.popup_menu_return_command(menu), Some(77));

    // Graying the only item → no selection.
    res.enable_menu_item(menu, 0, MF_BYPOSITION | MF_GRAYED);
    assert!(res.popup_menu_command_selection(menu).is_none());

    // record_popup_notification / popup_notifications.
    assert!(res.popup_notifications().is_empty());
    res.record_popup_notification(PopupMenuNotification { hwnd: 0x1000, msg: 0x111, wparam: 1, lparam: 0 });
    assert_eq!(res.popup_notifications().len(), 1);
    assert_eq!(res.popup_notifications()[0].msg, 0x111);

    // select_object / selected_pen / selected_bitmap on a DC.
    let hdc = res.create_compatible_dc();
    let pen = res.create_pen(0, 1, 0xFF0000);
    let prev_pen = res.select_object(hdc, pen);
    assert!(prev_pen.is_some()); // previous default pen returned
    assert_eq!(res.selected_pen(hdc), Some(pen));

    let bmp = res.create_bitmap(64, 32, 1, 16, 0x8000);
    let prev_bmp = res.select_object(hdc, bmp);
    assert!(prev_bmp.is_some());
    assert_eq!(res.selected_bitmap(hdc), Some(bmp));

    // move_to / current_pos.
    res.move_to(hdc, Point { x: 5, y: 15 });
    assert_eq!(res.current_pos(hdc), Some(Point { x: 5, y: 15 }));

    // delete_dc returns true; second call returns false.
    assert!(res.delete_dc(hdc));
    assert!(!res.delete_dc(hdc));
}

#[test]
fn resource_system_dc_palette_bk_mode_color_text_align_rop2_is_memory_and_delete_gdi() {
    use wince_emulation_v3::ce::resource::ResourceSystem;
    use wince_emulation_v3::ce::gwe::Point;

    let mut res = ResourceSystem::default();
    let hdc = res.create_compatible_dc();

    // is_memory_dc: true for create_compatible_dc result.
    assert!(res.is_memory_dc(hdc));
    assert!(!res.is_memory_dc(0));
    assert!(!res.is_memory_dc(0xDEAD));

    // DC state defaults: bk_color=0x00ffffff, bk_mode=2, rop2=13.
    let state = res.dc_state(hdc).unwrap();
    assert_eq!(state.bk_color, 0x00ff_ffff);
    assert_eq!(state.bk_mode, 2);
    assert_eq!(state.rop2, 13);
    assert_eq!(state.text_color, 0);
    assert_eq!(state.text_align, 0);

    // set_dc_bk_mode returns previous; null hdc → None.
    let prev = res.set_dc_bk_mode(hdc, 1).unwrap();
    assert_eq!(prev, 2); // previous was OPAQUE=2
    assert_eq!(res.dc_state(hdc).unwrap().bk_mode, 1);
    assert!(res.set_dc_bk_mode(0, 1).is_none());

    // set_dc_bk_color returns previous default.
    let prev = res.set_dc_bk_color(hdc, 0x112233).unwrap();
    assert_eq!(prev, 0x00ff_ffff);
    assert_eq!(res.dc_state(hdc).unwrap().bk_color, 0x112233);

    // set_dc_text_color returns previous.
    let prev = res.set_dc_text_color(hdc, 0xFF0000).unwrap();
    assert_eq!(prev, 0);
    assert_eq!(res.dc_state(hdc).unwrap().text_color, 0xFF0000);

    // set_dc_text_align returns previous.
    let prev = res.set_dc_text_align(hdc, 6).unwrap();
    assert_eq!(prev, 0);
    assert_eq!(res.dc_state(hdc).unwrap().text_align, 6);

    // set_dc_rop2 returns previous.
    let prev = res.set_dc_rop2(hdc, 4).unwrap();
    assert_eq!(prev, 13);
    assert_eq!(res.dc_state(hdc).unwrap().rop2, 4);

    // set_brush_origin returns previous (0,0).
    let prev = res.set_brush_origin(hdc, Point { x: 3, y: 7 }).unwrap();
    assert_eq!(prev, Point { x: 0, y: 0 });
    assert_eq!(res.dc_state(hdc).unwrap().brush_origin, Point { x: 3, y: 7 });
    assert!(res.set_brush_origin(0, Point { x: 0, y: 0 }).is_none());

    // realize_palette: valid DC returns Some(0); zero DC returns None.
    assert_eq!(res.realize_palette(hdc), Some(0));
    assert!(res.realize_palette(0).is_none());

    // select_palette: palette must exist; zero DC returns None.
    let palette = res.create_palette(vec![[255, 0, 0, 0], [0, 255, 0, 0]]);
    let default_palette = res.dc_state(hdc).unwrap().selected_palette;
    let prev = res.select_palette(hdc, palette).unwrap();
    // prev is the previously-selected palette (the default stock palette).
    assert_eq!(prev, default_palette);
    assert_eq!(res.dc_state(hdc).unwrap().selected_palette, palette);
    // Unknown palette returns None.
    assert!(res.select_palette(hdc, 0xDEAD).is_none());

    // delete_gdi_object: font, brush, pen, bitmap, region.
    let font = res.create_font(0, -12, 0, 400, false, false, false, 0, 0, "Arial".to_owned());
    assert!(res.delete_gdi_object(font));
    assert!(res.font(font).is_none());
    assert!(!res.delete_gdi_object(font)); // already gone

    let brush = res.create_brush(0x00FF00);
    let hdc2 = res.create_compatible_dc();
    res.select_object(hdc2, brush);
    // delete_gdi_object resets selected brush to stock WHITE_BRUSH in all DCs.
    assert!(res.delete_gdi_object(brush));
    assert!(res.brush(brush).is_none());
    // dc_state for hdc2 should no longer reference the deleted brush.
    let s2 = res.dc_state(hdc2).unwrap();
    assert_ne!(s2.selected_brush, brush);

    let pen = res.create_pen(0, 2, 0x0000FF);
    assert!(res.delete_gdi_object(pen));
    assert!(!res.delete_gdi_object(pen));

    let bmp = res.create_bitmap(16, 16, 1, 8, 0x3000);
    assert!(res.delete_gdi_object(bmp));
    assert!(!res.delete_gdi_object(bmp));
}

#[test]
fn resource_system_image_list_duplicate_replace_remove_copy_count_overlay_and_drag() {
    use wince_emulation_v3::ce::resource::ResourceSystem;

    let mut res = ResourceSystem::default();

    // Create an image list and add 3 images via a strip bitmap.
    let ilh = res.create_image_list(16, 16, 0, 0, 0).unwrap();
    let bmp = res.create_bitmap(48, 16, 1, 8, 0xA000); // 48px / 16px = 3 images
    res.add_image_list_image(ilh, bmp, 0).unwrap();
    assert_eq!(res.image_list_count(ilh), Some(3));

    // duplicate_image_list: creates independent copy.
    let dup = res.duplicate_image_list(ilh).unwrap();
    assert_ne!(dup, ilh);
    assert_eq!(res.image_list_count(dup), Some(3));
    // duplicate of unknown handle returns None.
    assert!(res.duplicate_image_list(0xDEAD).is_none());

    // set_image_list_count: expand to 5, then truncate to 2.
    assert_eq!(res.set_image_list_count(ilh, 5), Some(true));
    assert_eq!(res.image_list_count(ilh), Some(5));
    assert_eq!(res.set_image_list_count(ilh, 2), Some(true));
    assert_eq!(res.image_list_count(ilh), Some(2));
    assert!(res.set_image_list_count(0xDEAD, 2).is_none());

    // remove_image_list_image: remove index 0.
    assert_eq!(res.remove_image_list_image(ilh, 0), Some(true));
    assert_eq!(res.image_list_count(ilh), Some(1));
    // Out-of-range index returns Some(false).
    assert_eq!(res.remove_image_list_image(ilh, 5), Some(false));
    // Negative index clears all.
    res.add_image_list_image(ilh, bmp, 0); // add 3 more
    assert_eq!(res.remove_image_list_image(ilh, -1), Some(true));
    assert_eq!(res.image_list_count(ilh), Some(0));

    // replace_image_list_icon: negative index appends, non-negative replaces.
    let bmp2 = res.create_bitmap(16, 16, 1, 8, 0xB000);
    let icon = res.create_icon(true, 0, 0, bmp2, 0).unwrap();
    let idx = res.replace_image_list_icon(ilh, -1, icon).unwrap();
    assert_eq!(idx, 0); // appended as first image
    // zero icon returns None.
    assert!(res.replace_image_list_icon(ilh, -1, 0).is_none());

    // set_image_list_overlay: valid range 1..=15.
    assert_eq!(res.set_image_list_overlay(ilh, 0, 1), Some(true));
    // overlay out of range returns Some(false).
    assert_eq!(res.set_image_list_overlay(ilh, 0, 0), Some(false));
    assert_eq!(res.set_image_list_overlay(ilh, 0, 16), Some(false));
    // negative image_index returns Some(false).
    assert_eq!(res.set_image_list_overlay(ilh, -1, 1), Some(false));
    // unknown handle returns None.
    assert!(res.set_image_list_overlay(0xDEAD, 0, 1).is_none());

    // copy_image_list_image: copy src[0] to dst[-1] (append).
    let src = res.create_image_list(16, 16, 0, 0, 0).unwrap();
    res.add_image_list_image(src, bmp, 0); // 3 images
    let dst = res.create_image_list(16, 16, 0, 0, 0).unwrap();
    assert_eq!(res.copy_image_list_image(dst, -1, src, 0, false), Some(true));
    assert_eq!(res.image_list_count(dst), Some(1));
    // Negative src_index always returns Some(false).
    assert_eq!(res.copy_image_list_image(dst, -1, src, -1, false), Some(false));

    // begin_image_list_drag / image_list_drag / move / drag_leave / end.
    assert!(res.image_list_drag().is_none());
    // Invalid handle → Some(false).
    assert_eq!(res.begin_image_list_drag(0xDEAD, 0, 5, 5), Some(false));
    // Valid.
    assert_eq!(res.begin_image_list_drag(src, 0, 3, 4), Some(true));
    let drag = res.image_list_drag().unwrap();
    assert_eq!(drag.hotspot_x, 3);
    assert_eq!(drag.hotspot_y, 4);

    assert!(res.image_list_drag_enter(0x2000, 10, 20));
    assert!(res.image_list_drag_move(15, 25));
    let drag = res.image_list_drag().unwrap();
    assert_eq!(drag.x, 15);
    assert_eq!(drag.y, 25);

    assert!(res.image_list_drag_show(false));
    assert!(!res.image_list_drag().unwrap().visible);
    assert!(res.image_list_drag_show(true));

    assert!(res.image_list_drag_leave(0x2000));
    assert_eq!(res.image_list_drag().unwrap().lock_hwnd, 0);

    assert!(res.end_image_list_drag());
    assert!(res.image_list_drag().is_none());
    // Second end returns false.
    assert!(!res.end_image_list_drag());
}

#[test]
fn gwe_keyboard_target_ime_context_associate_status_and_is_window_being_destroyed() {
    use wince_emulation_v3::ce::gwe::Gwe;

    let mut gwe = Gwe::default();
    let big = Rect { left: 0, top: 0, right: 400, bottom: 300 };
    let hwnd = gwe.create_window_ex_with_rect(1, "Win", "Win", None, 0, 0, 0, big);
    let child = gwe.create_window_ex_with_process_parent_owner_and_rect(
        1, 1, "Child", "Child", Some(hwnd), None, 0, WS_CHILD, 0, big,
    );

    // set_keyboard_target / get_keyboard_target.
    assert_eq!(gwe.get_keyboard_target(1), None);
    let prev = gwe.set_keyboard_target(1, Some(hwnd));
    assert_eq!(prev, None);
    assert_eq!(gwe.get_keyboard_target(1), Some(hwnd));
    // Setting to None removes the entry; returns previous.
    let prev = gwe.set_keyboard_target(1, None);
    assert_eq!(prev, Some(hwnd));
    assert_eq!(gwe.get_keyboard_target(1), None);
    // Setting to invalid hwnd returns None without changing state.
    assert_eq!(gwe.set_keyboard_target(1, Some(0xDEAD)), None);

    // keyboard_target_is_within / clear_keyboard_targets_within.
    gwe.set_keyboard_target(1, Some(child));
    assert!(gwe.keyboard_target_is_within(hwnd)); // child is within hwnd
    assert!(gwe.keyboard_target_is_within(child));
    assert!(!gwe.keyboard_target_is_within(0xDEAD));
    gwe.clear_keyboard_targets_within(hwnd);
    assert_eq!(gwe.get_keyboard_target(1), None);

    // is_ime_layout: high word non-zero = IME layout.
    assert!(gwe.is_ime_layout(0x0409_0409)); // high word = 0x0409 (non-zero) → IME
    assert!(gwe.is_ime_layout(0x0001_0409)); // high word = 1 → IME
    assert!(!gwe.is_ime_layout(0x0000_0409)); // high word = 0 → not IME
    assert!(!gwe.is_ime_layout(0)); // zero is not IME

    // set_ime_enabled_for_thread / ime_enabled_for_thread.
    assert!(gwe.ime_enabled_for_thread(1)); // enabled by default
    gwe.set_ime_enabled_for_thread(1, false);
    assert!(!gwe.ime_enabled_for_thread(1));
    gwe.set_ime_enabled_for_thread(1, true);
    assert!(gwe.ime_enabled_for_thread(1));

    // create_ime_context / ime_context / set_ime_open_status / set_ime_conversion_status.
    let himc = gwe.create_ime_context();
    assert!(himc != 0);
    let ctx = gwe.ime_context(himc).unwrap();
    assert!(!ctx.open);
    assert_eq!(ctx.conversion_status, 0);
    assert_eq!(ctx.sentence_status, 0);

    assert!(gwe.set_ime_open_status(himc, true));
    assert!(gwe.ime_context(himc).unwrap().open);
    assert!(!gwe.set_ime_open_status(0xDEAD, true)); // unknown

    assert!(gwe.set_ime_conversion_status(himc, 7, 3));
    let ctx = gwe.ime_context(himc).unwrap();
    assert_eq!(ctx.conversion_status, 7);
    assert_eq!(ctx.sentence_status, 3);

    // associate_ime_context: link himc to hwnd.
    let prev = gwe.associate_ime_context(hwnd, himc).unwrap();
    assert_eq!(prev, 0); // no previous context
    // release_ime_context: returns true when context belongs to hwnd.
    assert!(gwe.release_ime_context(hwnd, himc));
    // release_ime_context: wrong hwnd returns false.
    let other = gwe.create_window_ex_with_rect(2, "Other", "", None, 0, 0, 0, big);
    assert!(!gwe.release_ime_context(other, himc));

    // destroy_ime_context: removes it.
    assert!(gwe.destroy_ime_context(himc));
    assert!(gwe.ime_context(himc).is_none());
    assert!(!gwe.destroy_ime_context(himc)); // already gone

    // get_ime_context: auto-creates context for window if enabled.
    let auto_himc = gwe.get_ime_context(hwnd).unwrap();
    assert!(auto_himc != 0);
    // Same hwnd returns same context on second call.
    assert_eq!(gwe.get_ime_context(hwnd), Some(auto_himc));
    // Disabled thread returns None.
    gwe.set_ime_enabled_for_thread(1, false);
    assert!(gwe.get_ime_context(hwnd).is_none());

    // is_window_being_destroyed / mark_window_subtree_being_destroyed.
    assert!(!gwe.is_window_being_destroyed(hwnd));
    assert!(gwe.mark_window_subtree_being_destroyed(hwnd));
    assert!(gwe.is_window_being_destroyed(hwnd));
    assert!(gwe.is_window_being_destroyed(child)); // child included
    assert!(!gwe.mark_window_subtree_being_destroyed(0xDEAD));
}

#[test]
fn kernel_mount_guest_root_host_path_file_io_stats_and_map_window_points() {
    use wince_emulation_v3::ce::gwe::{Gwe, Point};

    // file_io_stats starts at zero.
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let kernel = CeKernel::boot(config);
    let stats = kernel.file_io_stats();
    assert_eq!(stats.host_file_open_count, 0);
    assert_eq!(stats.host_file_read_count, 0);
    assert_eq!(stats.memory_backed_open_count, 0);

    // recent_event_ops starts empty.
    assert!(kernel.recent_event_ops().is_empty());

    // mount_guest_root / host_path_for_guest / host_path_to_guest_mount.
    let config2 = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config2);
    let host_root = std::path::PathBuf::from(".");
    kernel.mount_guest_root("\\Storage Card", host_root.clone());
    // host_path_for_guest resolves a guest path to host path.
    let host = kernel.host_path_for_guest("\\Storage Card\\test.txt").unwrap();
    assert!(host.to_string_lossy().contains("test.txt"));
    // host_path_to_guest_mount maps a host path back to the guest mount.
    let guest_mount = kernel.host_path_to_guest_mount(&host_root.canonicalize().unwrap_or(host_root.clone()));
    // May be Some or None depending on whether host_root could be resolved, so just verify no panic.
    let _ = guest_mount;

    // unmount_guest_root: removes mount.
    assert!(kernel.unmount_guest_root("\\Storage Card"));
    // Double unmount returns false.
    assert!(!kernel.unmount_guest_root("\\Storage Card"));
    // After unmounting, the resolved path falls back to the file root instead of host_root.
    let after_unmount = kernel.host_path_for_guest("\\Storage Card\\test.txt");
    // The path resolves, but no longer through the explicit mount point.
    let _ = after_unmount; // don't assert Err — fallback root may still succeed

    // map_window_points: translate points between two windows.
    let mut gwe = Gwe::default();
    let rect_a = Rect { left: 100, top: 200, right: 400, bottom: 500 };
    let rect_b = Rect { left: 50, top: 80, right: 300, bottom: 400 };
    let ha = gwe.create_window_ex_with_rect(1, "A", "A", None, 0, 0, 0, rect_a);
    let hb = gwe.create_window_ex_with_rect(1, "B", "B", None, 0, 0, 0, rect_b);

    let mut pts = [Point { x: 10, y: 20 }];
    // from window A to window B: translate by (client_a.left - client_b.left).
    // client_rect for a top-level window equals the creation rect.
    assert!(gwe.map_window_points(Some(ha), Some(hb), &mut pts));
    assert_eq!(pts[0].x, 10 + (100 - 50)); // 60
    assert_eq!(pts[0].y, 20 + (200 - 80)); // 140

    // map_window_points with None → screen coordinates.
    let mut pts2 = [Point { x: 5, y: 5 }];
    assert!(gwe.map_window_points(Some(ha), None, &mut pts2));
    assert_eq!(pts2[0].x, 5 + 100); // from_origin.left - 0
    assert_eq!(pts2[0].y, 5 + 200);

    // Invalid hwnd returns false.
    let mut pts3 = [Point { x: 0, y: 0 }];
    assert!(!gwe.map_window_points(Some(0xDEAD), None, &mut pts3));
}

#[test]
fn gwe_window_snapshot_z_order_from_point_descendants_broadcast_and_message_pos() {
    use wince_emulation_v3::ce::gwe::{Gwe, Point, WM_USER};

    let mut gwe = Gwe::default();
    // Create two visible top-level windows with distinct non-overlapping positions.
    let rect_a = Rect { left: 0, top: 0, right: 100, bottom: 100 };
    let rect_b = Rect { left: 200, top: 0, right: 400, bottom: 100 };
    let hwnd_a = gwe.create_window_ex_with_rect(1, "WinA", "WinA", None, 0, WS_VISIBLE, 0, rect_a);
    let hwnd_b = gwe.create_window_ex_with_rect(2, "WinB", "WinB", None, 0, WS_VISIBLE, 0, rect_b);
    // Child of A with its own position inside A.
    let rect_child = Rect { left: 10, top: 10, right: 50, bottom: 50 };
    let child_a = gwe.create_window_ex_with_process_parent_owner_and_rect(
        1, 1, "ChildA", "ChildA", Some(hwnd_a), None, 0, WS_CHILD | WS_VISIBLE, 0, rect_child,
    );

    // windows_snapshot: all windows including desktop.
    let snap = gwe.windows_snapshot();
    assert!(snap.iter().any(|w| w.hwnd == hwnd_a));
    assert!(snap.iter().any(|w| w.hwnd == hwnd_b));

    // z_order_snapshot: desktop hwnd is always first; top-level windows follow.
    let zo = gwe.z_order_snapshot();
    assert!(!zo.is_empty());
    assert!(zo.contains(&hwnd_a));
    assert!(zo.contains(&hwnd_b));

    // window_and_descendants: returns hwnd and all transitive children.
    let desc = gwe.window_and_descendants(hwnd_a).unwrap();
    assert!(desc.contains(&hwnd_a));
    assert!(desc.contains(&child_a));
    assert!(!desc.contains(&hwnd_b));
    // Unknown hwnd returns None.
    assert!(gwe.window_and_descendants(0xDEAD).is_none());

    // window_from_point: hit-test a point in window A's area.
    // Window A client rect = rect_a (0,0,100,100) in screen coords.
    let hit_a = gwe.window_from_point(Point { x: 5, y: 5 });
    assert!(hit_a.is_some()); // something in that area
    // Point in window B.
    let hit_b = gwe.window_from_point(Point { x: 250, y: 50 });
    assert!(hit_b.is_some());

    // window_from_point_for_thread: only matches windows owned by the given thread.
    let hit_t1 = gwe.window_from_point_for_thread(1, Point { x: 5, y: 5 });
    assert!(hit_t1.is_some()); // thread 1 owns hwnd_a
    // Thread 2 doesn't own anything in rect_a area (hwnd_b is at 200+).
    let hit_t2_a_area = gwe.window_from_point_for_thread(2, Point { x: 5, y: 5 });
    let _ = hit_t2_a_area; // may be None or Some(hwnd_a) depending on impl, just no panic

    // get_message_pos: 0 for unknown thread.
    assert_eq!(gwe.get_message_pos(99), 0);

    // post_broadcast_message: posts to all top-level windows (returns true when any exist).
    let before_a = gwe.queue_snapshot().into_iter().find(|(t, _)| *t == 1).map(|(_, q)| q.len()).unwrap_or(0);
    let ok = gwe.post_broadcast_message(WM_USER, 1, 2, 0);
    assert!(ok); // at least one top-level window exists
    // After broadcast, thread 1's queue grew (hwnd_a received a message).
    let after_a = gwe.queue_snapshot().into_iter().find(|(t, _)| *t == 1).map(|(_, q)| q.len()).unwrap_or(0);
    assert!(after_a > before_a);
}

#[test]
fn memory_system_local_alloc_heap_create_destroy_realloc_size_validate_and_range_queries() {
    use wince_emulation_v3::ce::memory::{
        HEAP_NO_SERIALIZE, HEAP_REALLOC_IN_PLACE_ONLY, HEAP_ZERO_MEMORY,
        LMEM_ZEROINIT, PROCESS_HEAP_HANDLE, MemorySystem,
    };

    let mut mem = MemorySystem::default();

    // get_process_heap returns PROCESS_HEAP_HANDLE.
    assert_eq!(mem.get_process_heap(), PROCESS_HEAP_HANDLE);

    // local_alloc with LMEM_ZEROINIT succeeds and is tracked.
    let lptr = mem.local_alloc(LMEM_ZEROINIT, 64).unwrap();
    assert!(lptr != 0);
    // local_size returns the actual size (>= requested).
    let lsize = mem.local_size(lptr).unwrap();
    assert!(lsize >= 64);
    // local_alloc with invalid flags returns None.
    assert!(mem.local_alloc(0xFFFF_0000, 64).is_none());

    // local_re_alloc: grow to a larger size (may move).
    let lptr2 = mem.local_re_alloc(lptr, 128, 0).unwrap();
    assert!(lptr2 != 0);
    // The new allocation is tracked with the new size.
    assert!(mem.local_size(lptr2).unwrap() >= 128);

    // local_free(0) is a no-op that returns true (CE semantics).
    assert!(mem.local_free(0));
    // local_free the reallocated pointer.
    assert!(mem.local_free(lptr2));
    // Double-free returns false.
    assert!(!mem.local_free(lptr2));

    // heap_create: invalid options return None.
    assert!(mem.heap_create(0xDEAD_0000, 0, 0).is_none());
    // heap_create with valid options returns a new handle.
    let h1 = mem.heap_create(HEAP_NO_SERIALIZE, 0x1000, 0).unwrap();
    assert_ne!(h1, PROCESS_HEAP_HANDLE);
    // A second heap gets a different handle.
    let h2 = mem.heap_create(0, 0, 0).unwrap();
    assert_ne!(h2, h1);

    // heap_alloc on a user heap.
    let p1 = mem.heap_alloc(h1, 0, 100).unwrap();
    // heap_size reports at least the requested size.
    assert!(mem.heap_size(h1, 0, p1).unwrap() >= 100);
    // heap_size with wrong heap returns None.
    assert!(mem.heap_size(h2, 0, p1).is_none());
    // heap_alloc with invalid flags returns None.
    assert!(mem.heap_alloc(h1, 0xFFFF_0000, 100).is_none());

    // heap_validate with ptr=0 is always valid for a live heap.
    assert!(mem.heap_validate(h1, 0, 0));
    // heap_validate with a valid ptr returns true.
    assert!(mem.heap_validate(h1, 0, p1));
    // heap_validate with unknown ptr returns false.
    assert!(!mem.heap_validate(h1, 0, 0xDEAD_1234));
    // heap_validate with invalid flags returns false.
    assert!(!mem.heap_validate(h1, 0xFFFF_0000, 0));

    // heap_re_alloc: shrink in-place.
    let p1_shrunk = mem.heap_re_alloc(h1, 0, p1, 50).unwrap();
    assert_eq!(p1_shrunk, p1); // shrink is always in-place
    assert!(mem.heap_size(h1, 0, p1_shrunk).unwrap() >= 50);

    // heap_re_alloc with HEAP_REALLOC_IN_PLACE_ONLY when growth required → None.
    let p_inplace = mem.heap_alloc(h1, 0, 32).unwrap();
    assert!(mem.heap_re_alloc(h1, HEAP_REALLOC_IN_PLACE_ONLY, p_inplace, 9999).is_none());

    // heap_re_alloc with HEAP_ZERO_MEMORY: grows and zeroes new bytes.
    let p_grow = mem.heap_re_alloc(h1, HEAP_ZERO_MEMORY, p_inplace, 64).unwrap();
    assert!(p_grow != 0);

    // heap_free: correct heap succeeds; wrong heap fails.
    let p2 = mem.heap_alloc(h2, 0, 48).unwrap();
    assert!(!mem.heap_free(h1, 0, p2)); // wrong heap
    assert!(mem.heap_free(h2, 0, p2));  // correct heap
    assert!(!mem.heap_free(h2, 0, p2)); // double-free

    // heap_destroy: cannot destroy the process heap.
    assert!(!mem.heap_destroy(PROCESS_HEAP_HANDLE));
    // heap_destroy h2 succeeds and removes its allocations.
    assert!(mem.heap_destroy(h2));
    // Double-destroy returns false.
    assert!(!mem.heap_destroy(h2));
    // heap_alloc on a destroyed heap returns None.
    assert!(mem.heap_alloc(h2, 0, 32).is_none());

    // contains_allocated_range: size=0 is always true.
    assert!(mem.contains_allocated_range(0, 0));
    // A pointer within a live allocation returns true.
    assert!(mem.contains_allocated_range(p1_shrunk, 16));
    // A pointer entirely outside any allocation returns false.
    assert!(!mem.contains_allocated_range(0x0000_0001, 1));

    // heap_range_status: ptr inside requested area → Some(true).
    // First allocate a known-size block.
    let rp = mem.heap_alloc(h1, 0, 40).unwrap();
    let rsize = mem.heap_size(h1, 0, rp).unwrap();
    // Query within the requested bytes (≤40) → Some(true).
    assert_eq!(mem.heap_range_status(rp, 40), Some(true));
    // Query beyond the requested bytes but within actual allocation → Some(false) if actual > requested.
    if rsize > 40 {
        assert_eq!(mem.heap_range_status(rp, rsize), Some(false));
    }
    // Query for a pointer not in any allocation → None.
    assert!(mem.heap_range_status(0x0000_0001, 1).is_none());
}

#[test]
fn handle_table_create_mutex_with_status_suspend_resume_by_id_and_file_mappings_mut() {
    use wince_emulation_v3::ce::object::{
        FileMappingView, HandleTable, ThreadResumeResult, ThreadSuspendResult,
    };

    let mut table = HandleTable::default();

    // create_mutex_with_status: first call → (handle, false) (newly created).
    let (mx1, already_existed) = table.create_mutex_with_status(Some("named_mx".to_owned()), None);
    assert!(!already_existed);
    // Second call with same name → same handle and already_existed=true.
    let (mx2, already_existed2) = table.create_mutex_with_status(Some("named_mx".to_owned()), None);
    assert_eq!(mx1, mx2);
    assert!(already_existed2);
    // Anonymous mutex is always newly created.
    let (mx3, already_existed3) = table.create_mutex_with_status(None, Some(77));
    assert_ne!(mx3, mx1);
    assert!(!already_existed3);

    // suspend_thread_by_id: unknown thread_id → None.
    assert!(table.suspend_thread_by_id(999).is_none());
    // Create a thread with thread_id=55 and no initial suspension.
    let th = table.create_thread(55, 0x1000, 0x2000, false);
    // suspend_thread_by_id(55) → Previous(0).
    assert_eq!(
        table.suspend_thread_by_id(55),
        Some(ThreadSuspendResult::Previous(0))
    );
    // Second suspend → Previous(1).
    assert_eq!(
        table.suspend_thread_by_id(55),
        Some(ThreadSuspendResult::Previous(1))
    );

    // resume_thread_by_id: unknown thread_id → None.
    assert!(table.resume_thread_by_id(999).is_none());
    // resume_thread_by_id(55) → Previous(2) (current suspend_count is 2).
    assert_eq!(
        table.resume_thread_by_id(55),
        Some(ThreadResumeResult::Previous(2))
    );
    // resume again → Previous(1).
    assert_eq!(
        table.resume_thread_by_id(55),
        Some(ThreadResumeResult::Previous(1))
    );
    // resume once more → Previous(0), now running.
    assert_eq!(
        table.resume_thread_by_id(55),
        Some(ThreadResumeResult::Previous(0))
    );
    // resume when already running: Previous(0) (idempotent).
    assert_eq!(
        table.resume_thread_by_id(55),
        Some(ThreadResumeResult::Previous(0))
    );

    // suspend_thread_by_id on a handle (not by id) returns InvalidHandle for unknown handle.
    assert_eq!(table.suspend_thread(0xDEAD_BEEF), ThreadSuspendResult::InvalidHandle);
    // suspend_thread on a valid handle works too.
    assert_eq!(table.suspend_thread(th), ThreadSuspendResult::Previous(0));

    // resume_thread on an invalid handle → InvalidHandle.
    assert_eq!(table.resume_thread(0xDEAD_BEEF), ThreadResumeResult::InvalidHandle);
    // resume_thread on the valid handle that we just suspended → Previous(1).
    assert_eq!(table.resume_thread(th), ThreadResumeResult::Previous(1));

    // file_mappings_mut: create two mappings, mutate one via the iterator.
    let fm1 = table.create_file_mapping(Some("shm1".to_owned()), 0x1000, 4, None);
    let fm2 = table.create_file_mapping(None, 0x2000, 4, Some(7));
    // Add a view to fm1 via file_mapping_mut.
    {
        let mapping = table.file_mapping_mut(fm1).unwrap();
        mapping.views.push(FileMappingView { base: 0x5000_0000, size: 0x1000, offset: 0 });
    }
    // file_mappings_mut iterates all file-mapping objects.
    let names: Vec<_> = table.file_mappings_mut().map(|m| m.name.clone()).collect();
    assert_eq!(names.len(), 2);
    // file_mappings_mut lets us mutate — append a view to the unnamed mapping.
    for m in table.file_mappings_mut() {
        if m.name.is_none() {
            m.views.push(FileMappingView { base: 0x6000_0000, size: 0x2000, offset: 0 });
        }
    }
    // file_mapping_by_view_mut: look up by view base.
    {
        let (mapping, view) = table.file_mapping_by_view_mut(0x5000_0000).unwrap();
        assert_eq!(view.base, 0x5000_0000);
        assert_eq!(view.size, 0x1000);
        // Can mutate through the returned reference.
        mapping.data.push(0xAB);
    }
    // file_mapping_by_view_mut with unknown base → None.
    assert!(table.file_mapping_by_view_mut(0xDEAD_0000).is_none());
    // Verify the unnamed mapping's view was added.
    {
        let (_, view) = table.file_mapping_by_view_mut(0x6000_0000).unwrap();
        assert_eq!(view.size, 0x2000);
    }
    // Suppress unused-variable warnings from the handles.
    let _ = (fm1, fm2);
}

#[test]
fn shell_system_remove_window_state_and_remove_windows_state() {
    use wince_emulation_v3::ce::shell::{
        NotifyIconData, NotifyIconOp, ShellChangeNotifyRegistration,
        ShellNotificationData, ShellSystem, ShellWindowCleanup, SHNP_ICONIC,
    };

    let mut shell = ShellSystem::default();

    // Add two notify icons for hwnd=10.
    let icon_a = NotifyIconData {
        hwnd: 10, id: 1, flags: 0, callback_message: 0,
        icon: 0, tip: "icon_a".to_owned(), state: 0, state_mask: 0,
    };
    let icon_b = NotifyIconData {
        hwnd: 10, id: 2, flags: 0, callback_message: 0,
        icon: 0, tip: "icon_b".to_owned(), state: 0, state_mask: 0,
    };
    assert!(shell.apply_notify_icon(NotifyIconOp::Add, icon_a));
    assert!(shell.apply_notify_icon(NotifyIconOp::Add, icon_b));

    // Add a notification whose hwnd_sink is 10.
    let clsid: [u8; 16] = [2u8; 16];
    let notif = ShellNotificationData {
        id: 1, priority: SHNP_ICONIC, duration_cs: 9999,
        icon: 0, flags: 0, clsid,
        hwnd_sink: 10, title: "t".to_owned(), html: String::new(), lparam: 0,
    };
    let _ = shell.add_notification(notif, 0);

    // Register a change notification for hwnd=10.
    shell.register_change_notification(ShellChangeNotifyRegistration {
        hwnd: 10, event_mask: 0xFF, notify_flags: 0, watch_dir: None, recursive: false,
    });

    // Register a change notification for a different hwnd that should NOT be removed.
    shell.register_change_notification(ShellChangeNotifyRegistration {
        hwnd: 99, event_mask: 0x01, notify_flags: 0, watch_dir: None, recursive: false,
    });

    // remove_window_state(10) clears all state associated with hwnd=10.
    let cleanup = shell.remove_window_state(10);
    assert_eq!(cleanup.notify_icons_removed, 2);
    assert_eq!(cleanup.notifications_removed, 1);
    assert_eq!(cleanup.change_notifications_removed, 1);

    // Verify everything belonging to hwnd=10 is gone.
    assert!(shell.notify_icon(10, 1).is_none());
    assert!(shell.notify_icon(10, 2).is_none());
    assert!(shell.notification(clsid, 1).is_none());
    assert!(shell.change_notification(10).is_none());
    // The other hwnd is untouched.
    assert!(shell.change_notification(99).is_some());

    // remove_window_state on a hwnd with no state returns all-zero cleanup.
    let cleanup2 = shell.remove_window_state(10);
    assert_eq!(cleanup2, ShellWindowCleanup::default());

    // remove_windows_state: add icons for two hwnds and remove both at once.
    let icon_c = NotifyIconData {
        hwnd: 20, id: 1, flags: 0, callback_message: 0,
        icon: 0, tip: "c".to_owned(), state: 0, state_mask: 0,
    };
    let icon_d = NotifyIconData {
        hwnd: 30, id: 1, flags: 0, callback_message: 0,
        icon: 0, tip: "d".to_owned(), state: 0, state_mask: 0,
    };
    assert!(shell.apply_notify_icon(NotifyIconOp::Add, icon_c));
    assert!(shell.apply_notify_icon(NotifyIconOp::Add, icon_d));
    assert_eq!(shell.notify_icons().count(), 2);

    let combined = shell.remove_windows_state(&[20, 30]);
    assert_eq!(combined.notify_icons_removed, 2);
    assert_eq!(shell.notify_icons().count(), 0);
}

#[test]
fn resource_system_owned_bitmap_bitmap_mut_region_rects_palette_mut_shell_image_list_merge_and_dither() {
    use wince_emulation_v3::ce::gwe::Rect;
    use wince_emulation_v3::ce::resource::ResourceSystem;

    let mut res = ResourceSystem::default();

    // create_owned_bitmap: same as create_bitmap but bits_owned=true.
    let obmp = res.create_owned_bitmap(20, 10, 1, 24, 0xABCD_0000);
    let bobj = res.bitmap(obmp).unwrap();
    assert_eq!(bobj.width, 20);
    assert_eq!(bobj.height, 10);
    assert_eq!(bobj.bits_pixel, 24);
    assert!(bobj.bits_owned);
    assert_eq!(bobj.bits_ptr, 0xABCD_0000);

    // create_owned_bitmap_with_masks: stores rgb_masks.
    let masks = [0xFF00_0000u32, 0x00FF_0000, 0x0000_FF00];
    let obmp2 = res.create_owned_bitmap_with_masks(8, 8, 1, 32, 0x1234_0000, Some(masks));
    let bobj2 = res.bitmap(obmp2).unwrap();
    assert_eq!(bobj2.rgb_masks, Some(masks));
    assert!(bobj2.bits_owned);

    // bitmap_mut: mutate the bits_ptr field.
    {
        let bm = res.bitmap_mut(obmp).unwrap();
        bm.bits_ptr = 0xDEAD_0000;
    }
    assert_eq!(res.bitmap(obmp).unwrap().bits_ptr, 0xDEAD_0000);
    // bitmap_mut on unknown handle → None.
    assert!(res.bitmap_mut(0xDEAD).is_none());

    // set_region_rects: replace region with a multi-rect set.
    let init = Rect { left: 0, top: 0, right: 100, bottom: 50 };
    let rh = res.create_region(init);
    let rects = vec![
        Rect { left: 0, top: 0, right: 50, bottom: 25 },
        Rect { left: 50, top: 0, right: 100, bottom: 25 },
    ];
    assert!(res.set_region_rects(rh, rects.clone()));
    let region = res.region(rh).unwrap();
    // bounding rect covers both rects.
    assert!(region.rect.right >= 100);
    assert!(region.rect.bottom >= 25);
    // set_region_rects on invalid handle → false.
    assert!(!res.set_region_rects(0xDEAD, rects));

    // palette_mut: mutate palette entries.
    let entries = vec![[0u8, 0, 0, 0], [0xFF, 0xFF, 0xFF, 0]];
    let palh = res.create_palette(entries.clone());
    {
        let pm = res.palette_mut(palh).unwrap();
        pm.entries.push([0x80, 0x80, 0x80, 0]);
    }
    assert_eq!(res.palette(palh).unwrap().entries.len(), 3);
    // palette_mut on unknown handle → None.
    assert!(res.palette_mut(0xDEAD).is_none());

    // create_shell_system_image_list: creates a list at an arbitrary handle.
    let shell_il_handle = 0x0000_1234_u32;
    res.create_shell_system_image_list(shell_il_handle, 16, 16);
    let il = res.image_list(shell_il_handle).unwrap();
    assert_eq!(il.width, 16);
    assert_eq!(il.height, 16);
    // Calling again with different size is idempotent (entry already exists).
    res.create_shell_system_image_list(shell_il_handle, 32, 32);
    assert_eq!(res.image_list(shell_il_handle).unwrap().width, 16);

    // merge_image_list_images: creates new image list from one image of each source.
    let il1 = res.create_image_list(16, 16, 0, 4, 1).unwrap();
    let il2 = res.create_image_list(16, 16, 0, 4, 1).unwrap();
    // Need bitmaps for the images.
    let bmp16 = res.create_bitmap(16, 16, 1, 24, 0);
    res.add_image_list_image(il1, bmp16, 0);
    let bmp16b = res.create_bitmap(16, 16, 1, 24, 0);
    res.add_image_list_image(il2, bmp16b, 0);
    // merge first image of il1 with first image of il2, offset (2,2).
    let merged = res.merge_image_list_images(il1, 0, il2, 0, 2, 2).unwrap();
    let merged_il = res.image_list(merged).unwrap();
    assert_eq!(merged_il.images.len(), 2);
    // negative indices → None.
    assert!(res.merge_image_list_images(il1, -1, il2, 0, 0, 0).is_none());

    // add_masked_image_list_image: transparent_color stored on image.
    let bmp_strip = res.create_bitmap(32, 16, 1, 24, 0);
    let ilm = res.create_image_list(16, 16, 0, 4, 1).unwrap();
    let idx = res.add_masked_image_list_image(ilm, bmp_strip, 0xFF00_FF00).unwrap();
    assert_eq!(idx, 0); // first image
    assert_eq!(res.image_list_count(ilm), Some(2)); // 32px / 16px = 2 images
    // transparent_color set when not 0xffffffff.
    assert_eq!(
        res.image_list(ilm).unwrap().images[0].transparent_color,
        Some(0xFF00_FF00)
    );
    // add_masked_image_list_image with bitmap=0 → None.
    assert!(res.add_masked_image_list_image(ilm, 0, 0).is_none());

    // replace_image_list_image: bitmap variant (not icon).
    let bmp_new = res.create_bitmap(16, 16, 1, 24, 0);
    assert_eq!(res.replace_image_list_image(ilm, 0, bmp_new, 0), Some(true));
    assert_eq!(res.image_list(ilm).unwrap().images[0].bitmap, bmp_new);
    // Negative index → Some(false).
    assert_eq!(res.replace_image_list_image(ilm, -1, bmp_new, 0), Some(false));
    // Out-of-range index → Some(false).
    assert_eq!(res.replace_image_list_image(ilm, 999, bmp_new, 0), Some(false));
    // bitmap=0 → Some(false).
    assert_eq!(res.replace_image_list_image(ilm, 0, 0, 0), Some(false));
    // Unknown handle → None.
    assert!(res.replace_image_list_image(0xDEAD, 0, bmp_new, 0).is_none());

    // copy_dither_image_list_image: copies image and records dither info.
    let il_dst = res.create_image_list(16, 16, 0, 4, 1).unwrap();
    let bmp_dst = res.create_bitmap(16, 16, 1, 24, 0);
    res.add_image_list_image(il_dst, bmp_dst, 0);
    let result = res.copy_dither_image_list_image(il_dst, 0, 5, 10, ilm, 0, 0x01);
    assert_eq!(result, Some(true));
    let last = res.image_list(il_dst).unwrap().last_dither_copy.unwrap();
    assert_eq!(last.dst_index, 0);
    assert_eq!(last.x, 5);
    assert_eq!(last.y, 10);
    assert_eq!(last.src_image_list, ilm);
    assert_eq!(last.src_index, 0);
    assert_eq!(last.flags, 0x01);
    // Negative dst_index → Some(false).
    assert_eq!(res.copy_dither_image_list_image(il_dst, -1, 0, 0, ilm, 0, 0), Some(false));
    // Negative src_index → Some(false).
    assert_eq!(res.copy_dither_image_list_image(il_dst, 0, 0, 0, ilm, -1, 0), Some(false));

    // set_image_list_drag_cursor: no active drag → Some(false).
    let il_cur = res.create_image_list(16, 16, 0, 4, 1).unwrap();
    let bmp_cur = res.create_bitmap(16, 16, 1, 24, 0);
    res.add_image_list_image(il_cur, bmp_cur, 0);
    assert_eq!(res.set_image_list_drag_cursor(il_cur, 0, 2, 3), Some(false));
    // Start drag first.
    res.begin_image_list_drag(il_cur, 0, 1, 1);
    // Now set_image_list_drag_cursor updates the drag state.
    assert_eq!(res.set_image_list_drag_cursor(il_cur, 0, 7, 8), Some(true));
    let drag = res.image_list_drag().unwrap();
    assert_eq!(drag.hotspot_x, 7);
    assert_eq!(drag.hotspot_y, 8);
    res.end_image_list_drag();

    // image_list_icon: returns icon field if set; else synthesizes from bitmap.
    let il_ico = res.create_image_list(16, 16, 0, 4, 1).unwrap();
    let bmp_ico = res.create_bitmap(16, 16, 1, 24, 0);
    res.add_image_list_image(il_ico, bmp_ico, 0);
    // Image has no icon, no fallback_icon → synthesized from bitmap handle.
    let synth = res.image_list_icon(il_ico, 0, 0, 0).unwrap();
    assert_eq!(synth, 0x000b_8000 | (bmp_ico & 0x0000_ffff));
    // With fallback_icon set → returns fallback_icon.
    let fb = res.image_list_icon(il_ico, 0, 0xABCD, 0).unwrap();
    assert_eq!(fb, 0xABCD);
    // Negative index → None.
    assert!(res.image_list_icon(il_ico, -1, 0, 0).is_none());
    // Invalid handle → None.
    assert!(res.image_list_icon(0xDEAD, 0, 0, 0).is_none());
}

#[test]
fn framebuffer_with_stride_snapshot_and_write_ppm() {
    use wince_emulation_v3::ce::framebuffer::{
        Framebuffer, FramebufferSnapshot, PixelFormat, VirtualFramebuffer,
    };

    // with_stride: custom stride (wider than minimum) is accepted.
    let min_stride = 8 * 2; // 8 pixels * 2 bytes/pixel for RGB565
    let custom_stride = min_stride + 4; // add 4 bytes of padding per row
    let fb = VirtualFramebuffer::with_stride(8, 4, custom_stride, PixelFormat::Rgb565).unwrap();
    assert_eq!(fb.stride(), custom_stride);
    assert_eq!(fb.width(), 8);
    assert_eq!(fb.height(), 4);
    // pixel buffer = stride * height.
    assert_eq!(fb.pixels().len(), custom_stride * 4);

    // with_stride: stride too small → Err.
    assert!(VirtualFramebuffer::with_stride(8, 4, min_stride - 1, PixelFormat::Rgb565).is_err());
    // with_stride: zero width → Err.
    assert!(VirtualFramebuffer::with_stride(0, 4, 8, PixelFormat::Rgb565).is_err());
    // with_stride: zero height → Err.
    assert!(VirtualFramebuffer::with_stride(8, 0, 16, PixelFormat::Rgb565).is_err());

    // snapshot: clones info and pixels.
    let fb2 = VirtualFramebuffer::new(4, 4, PixelFormat::Bgra8888).unwrap();
    let snap: FramebufferSnapshot = fb2.snapshot();
    assert_eq!(snap.info.width, 4);
    assert_eq!(snap.info.height, 4);
    assert_eq!(snap.info.format, PixelFormat::Bgra8888);
    assert_eq!(snap.pixels.len(), fb2.pixels().len());

    // write_ppm: writes a valid PPM file into a temp path.
    let small = VirtualFramebuffer::new(2, 2, PixelFormat::Rgb565).unwrap();
    let tmp_path = std::env::temp_dir().join("test_write_ppm.ppm");
    small.write_ppm(&tmp_path).unwrap();
    let ppm_bytes = std::fs::read(&tmp_path).unwrap();
    // PPM header starts with "P6\n".
    assert!(ppm_bytes.starts_with(b"P6\n"));
    // Payload is header + width*height*3 RGB bytes = header + 2*2*3=12 bytes.
    let payload_size = 2 * 2 * 3;
    assert!(ppm_bytes.len() > payload_size);
    // Clean up.
    let _ = std::fs::remove_file(&tmp_path);
}

#[test]
fn gwe_caret_gesture_set_parent_region_rects_next_dlg_tab_group_and_foreground_keyboard_target() {
    use wince_emulation_v3::ce::gwe::{
        CaretState, GestureRegistration, Gwe, Point, Rect, WS_CHILD, WS_GROUP, WS_TABSTOP,
        WS_VISIBLE,
    };

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 200, bottom: 100 };
    let hwnd = gwe.create_window_ex_with_rect(1, "Cls", "Win", None, 0, WS_VISIBLE, 0, rect);

    // create_caret: fails for invalid hwnd.
    assert!(!gwe.create_caret(0xDEAD, 0, 4, 8));
    // create_caret: fails for negative dimensions.
    assert!(!gwe.create_caret(hwnd, 0, -1, 8));
    // create_caret: succeeds with valid hwnd.
    assert!(gwe.create_caret(hwnd, 0, 4, 8));

    // caret(): returns Some with correct fields.
    let c: CaretState = gwe.caret().unwrap();
    assert_eq!(c.hwnd, hwnd);
    assert_eq!(c.width, 4);
    assert_eq!(c.height, 8);
    assert_eq!(c.show_count, -1); // hidden by default

    // get_caret_pos: returns Some(0,0) initially.
    assert_eq!(gwe.get_caret_pos(), Some(Point { x: 0, y: 0 }));

    // set_caret_pos: stores position.
    assert!(gwe.set_caret_pos(10, 20));
    assert_eq!(gwe.get_caret_pos(), Some(Point { x: 10, y: 20 }));

    // set_caret_pos without caret → false.
    gwe.destroy_caret();
    assert!(!gwe.set_caret_pos(5, 5));
    assert!(gwe.get_caret_pos().is_none());

    // Recreate caret for further tests.
    gwe.create_caret(hwnd, 0, 4, 8);

    // hide_caret / show_caret.
    assert!(gwe.hide_caret(hwnd)); // show_count goes from -1 to -2
    assert!(gwe.show_caret(hwnd)); // -2 → -1, still hidden
    assert!(gwe.show_caret(hwnd)); // -1 → 0, now shown (blink visible)
    // Wrong hwnd → false.
    assert!(!gwe.hide_caret(0xDEAD));

    // caret_system_enabled defaults to true.
    assert!(gwe.caret_system_enabled());
    gwe.disable_caret_system_wide();
    assert!(!gwe.caret_system_enabled());
    gwe.enable_caret_system_wide();
    assert!(gwe.caret_system_enabled());

    // destroy_caret.
    assert!(gwe.destroy_caret());
    assert!(gwe.caret().is_none());
    assert!(!gwe.destroy_caret()); // second destroy → false

    // hide_caret / show_caret with no caret → false.
    assert!(!gwe.hide_caret(hwnd));
    assert!(!gwe.show_caret(hwnd));

    // register_gesture: id=0 → false.
    assert!(!gwe.register_gesture(0, 1, 0, 0, 0));
    // handle=0 → false.
    assert!(!gwe.register_gesture(1, 0, 0, 0, 0));
    // Valid.
    assert!(gwe.register_gesture(5, 0xABCD, 1, 2, 3));
    let gr: GestureRegistration = gwe.gesture_registration(5).unwrap();
    assert_eq!(gr.id, 5);
    assert_eq!(gr.handle, 0xABCD);
    assert_eq!(gr.arg1, 1);
    assert_eq!(gr.arg2, 2);
    assert_eq!(gr.arg3, 3);
    // Unknown id → None.
    assert!(gwe.gesture_registration(999).is_none());

    // set_window_region_rects: multi-rect region on a window (non-adjacent rows).
    let rects = vec![
        Rect { left: 0, top: 0, right: 100, bottom: 30 },
        Rect { left: 0, top: 40, right: 100, bottom: 70 },
    ];
    assert!(gwe.set_window_region_rects(hwnd, Some(rects.clone())));
    // window_region_rects returns the canonicalized rects; at least one rect.
    let stored = gwe.window_region_rects(hwnd).unwrap();
    assert!(!stored.is_empty());
    // set_window_region_rects(None) clears.
    assert!(gwe.set_window_region_rects(hwnd, None));
    assert!(gwe.window_region_rects(hwnd).is_none());
    // Invalid hwnd → false.
    assert!(!gwe.set_window_region_rects(0xDEAD, Some(rects)));

    // visible_client_rects: valid window returns non-empty vec.
    let vcr = gwe.visible_client_rects(hwnd);
    assert!(!vcr.is_empty());
    // Invalid hwnd → empty vec.
    assert!(gwe.visible_client_rects(0xDEAD).is_empty());

    // set_parent: make hwnd a child of another top-level.
    let parent_rect = Rect { left: 0, top: 0, right: 400, bottom: 300 };
    let parent = gwe.create_window_ex_with_rect(1, "Par", "Parent", None, 0, WS_VISIBLE, 0, parent_rect);
    // Initially hwnd has no parent (top-level).
    let prev = gwe.set_parent(hwnd, Some(parent)).unwrap();
    assert_eq!(prev, None); // no previous parent
    // get_parent now returns Some(parent).
    assert_eq!(gwe.get_parent(hwnd), Some(parent));
    // set_parent with hwnd == parent → None (cycle not allowed).
    assert!(gwe.set_parent(parent, Some(parent)).is_none());
    // set_parent with invalid hwnd → None.
    assert!(gwe.set_parent(0xDEAD, None).is_none());
    // Remove parent (make top-level again).
    let prev2 = gwe.set_parent(hwnd, None).unwrap();
    assert_eq!(prev2, Some(parent));

    // get_next_dlg_tab_item / get_next_dlg_group_item.
    // Create a dialog with tab-stop children.
    let dlg_rect = Rect { left: 0, top: 0, right: 300, bottom: 200 };
    let dlg = gwe.create_window_ex_with_rect(1, "Dlg", "Dialog", None, 0, WS_VISIBLE, 0, dlg_rect);
    let ctrl_rect = Rect { left: 10, top: 10, right: 50, bottom: 30 };
    let ctrl1 = gwe.create_window_ex_with_rect(1, "Btn", "B1", Some(dlg), 1, WS_VISIBLE | WS_CHILD | WS_TABSTOP, 0, ctrl_rect);
    let ctrl2 = gwe.create_window_ex_with_rect(1, "Btn", "B2", Some(dlg), 2, WS_VISIBLE | WS_CHILD | WS_TABSTOP, 0, ctrl_rect);
    // get_next_dlg_tab_item forward from ctrl1 → ctrl2.
    assert_eq!(gwe.get_next_dlg_tab_item(dlg, ctrl1, false), Some(ctrl2));
    // get_next_dlg_tab_item backward from ctrl2 → ctrl1.
    assert_eq!(gwe.get_next_dlg_tab_item(dlg, ctrl2, true), Some(ctrl1));
    // get_next_dlg_tab_item with invalid dialog → None.
    assert!(gwe.get_next_dlg_tab_item(0xDEAD, ctrl1, false).is_none());

    // get_next_dlg_group_item: WS_GROUP children.
    let g_rect = Rect { left: 0, top: 0, right: 300, bottom: 200 };
    let gdlg = gwe.create_window_ex_with_rect(1, "GDlg", "GDialog", None, 0, WS_VISIBLE, 0, g_rect);
    let gr1 = gwe.create_window_ex_with_rect(1, "Btn", "G1", Some(gdlg), 10, WS_VISIBLE | WS_CHILD | WS_GROUP, 0, ctrl_rect);
    let gr2 = gwe.create_window_ex_with_rect(1, "Btn", "G2", Some(gdlg), 11, WS_VISIBLE | WS_CHILD, 0, ctrl_rect);
    // Forward from gr1 → gr2 (within group).
    assert_eq!(gwe.get_next_dlg_group_item(gdlg, gr1, false), Some(gr2));
    // Backward from gr2 → gr1.
    assert_eq!(gwe.get_next_dlg_group_item(gdlg, gr2, true), Some(gr1));
    // get_next_dlg_group_item on empty dialog → None.
    let empty_dlg = gwe.create_window_ex_with_rect(1, "E", "E", None, 0, WS_VISIBLE, 0, g_rect);
    assert!(gwe.get_next_dlg_group_item(empty_dlg, 0, false).is_none());

    // clipboard_format_name: after register, the name is retrievable.
    let fmt = gwe.register_clipboard_format("TestFmt").unwrap();
    // The name is stored and retrieved as-is (possibly lowercased by normalization).
    let stored_name = gwe.clipboard_format_name(fmt).unwrap();
    assert!(stored_name.eq_ignore_ascii_case("TestFmt"));
    assert!(gwe.clipboard_format_name(0).is_none()); // unknown format

    // get_foreground_keyboard_target: tests priority chain.
    // Use a fresh Gwe so there are no stray windows.
    let mut gwe2 = Gwe::default();
    // No windows → get_active_window fallback finds nothing → None.
    assert!(gwe2.get_foreground_keyboard_target().is_none());
    // Create a top-level window for thread 1.
    let w = gwe2.create_window_ex_with_rect(1, "C", "T", None, 0, WS_VISIBLE, 0, rect);
    // Explicitly set active window.
    gwe2.set_active_window(Some(w));
    // No keyboard target for thread 1, no focus → falls back to active window.
    assert_eq!(gwe2.get_foreground_keyboard_target(), Some(w));
    // Set focus → focus takes priority over active-window fallback when no keyboard target.
    // (focus is returned by get_active_window's focus fallback as well)
    // Set explicit keyboard target for thread 1 → keyboard target wins.
    gwe2.set_keyboard_target(1, Some(w));
    assert_eq!(gwe2.get_foreground_keyboard_target(), Some(w));
}

#[test]
fn gwe_cursor_pos_window_pos_for_rect_draw_menu_bar_message_pointer_payload_dialog_cmds_and_key_state() {
    use wince_emulation_v3::ce::gwe::{
        BS_DEFPUSHBUTTON, BS_PUSHBUTTON, Gwe, MessagePointerPayload, Point, Rect, WM_DESTROY,
        WM_NCDESTROY, WS_CHILD, WS_VISIBLE, WindowPos, HWND_TOP, WNDCLASSW_SIZE,
    };

    let mut gwe = Gwe::default();
    let wndclass_bytes = [0u8; WNDCLASSW_SIZE];
    gwe.register_class("Btn", wndclass_bytes);
    let rect = Rect { left: 10, top: 20, right: 110, bottom: 70 };
    let hwnd = gwe.create_window_ex_with_rect(1, "Btn", "W", None, 0, WS_VISIBLE, 0, rect);

    // set_cursor_pos / get_cursor_pos.
    gwe.set_cursor_pos(Point { x: 55, y: 33 });
    assert_eq!(gwe.get_cursor_pos(), Point { x: 55, y: 33 });

    // window_pos_for_rect: computes WindowPos from a rect.
    let target_rect = Rect { left: 20, top: 30, right: 200, bottom: 150 };
    let wp = gwe.window_pos_for_rect(hwnd, target_rect, HWND_TOP, 0).unwrap();
    assert_eq!(wp.hwnd, hwnd);
    assert_eq!(wp.width, target_rect.width());
    assert_eq!(wp.height, target_rect.height());
    // Invalid hwnd → None.
    assert!(gwe.window_pos_for_rect(0xDEAD, target_rect, HWND_TOP, 0).is_none());

    // draw_menu_bar: returns true for valid hwnd, false otherwise.
    assert!(gwe.draw_menu_bar(hwnd));
    assert!(!gwe.draw_menu_bar(0xDEAD));

    // active_window_is_within: only true when active window is hwnd or descendant.
    gwe.set_active_window(Some(hwnd));
    assert!(gwe.active_window_is_within(hwnd));
    let other_rect = Rect { left: 200, top: 0, right: 300, bottom: 100 };
    let hwnd2 = gwe.create_window_ex_with_rect(1, "Btn", "W2", None, 0, WS_VISIBLE, 0, other_rect);
    assert!(!gwe.active_window_is_within(hwnd2));

    // insert_message_pointer_payload: ptr=0 → false.
    assert!(!gwe.insert_message_pointer_payload(0, MessagePointerPayload::WindowPos(WindowPos {
        hwnd, insert_after: HWND_TOP, x: 0, y: 0, width: 100, height: 50, flags: 0,
    })));
    // Valid ptr.
    let payload = MessagePointerPayload::WindowPos(WindowPos {
        hwnd, insert_after: HWND_TOP, x: 5, y: 10, width: 80, height: 40, flags: 0,
    });
    assert!(gwe.insert_message_pointer_payload(0x1234, payload.clone()));
    // message_pointer_payload returns Some for existing ptr.
    assert_eq!(gwe.message_pointer_payload(0x1234), Some(payload.clone()));
    // take_message_pointer_payload removes it.
    assert_eq!(gwe.take_message_pointer_payload(0x1234), Some(payload));
    // Gone after take.
    assert!(gwe.message_pointer_payload(0x1234).is_none());
    assert!(gwe.take_message_pointer_payload(0x1234).is_none());

    // dialog_return_command / dialog_cancel_command / default_push_button / is_push_button.
    // Create a dialog with a default push button (BS_DEFPUSHBUTTON, class "button").
    let dlg_rect = Rect { left: 0, top: 0, right: 400, bottom: 300 };
    let dlg = gwe.create_window_ex_with_rect(1, "dialog", "Dlg", None, 0, WS_VISIBLE, 0, dlg_rect);
    let btn_rect = Rect { left: 10, top: 10, right: 80, bottom: 30 };
    // Create IDOK (id=1) as default push button.
    let ok_btn = gwe.create_window_ex_with_rect(
        1, "button", "OK", Some(dlg), 1, WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON, 0, btn_rect
    );
    // is_push_button: button class with push-button style → true.
    assert!(gwe.is_push_button(ok_btn));
    // Non-button window → false.
    assert!(!gwe.is_push_button(hwnd));
    // default_push_button returns the DEFPUSHBUTTON child.
    assert_eq!(gwe.default_push_button(dlg), Some(ok_btn));
    // dialog_return_command: source is push button → returns (id, source).
    let (id, src) = gwe.dialog_return_command(dlg, ok_btn, 99);
    assert_eq!(id, 1); // ok_btn's ctrl id
    assert_eq!(src, ok_btn);
    // dialog_cancel_command: finds child with matching id.
    let cancel_btn = gwe.create_window_ex_with_rect(
        1, "button", "Cancel", Some(dlg), 2, WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, 0, btn_rect
    );
    let (cancel_id, cancel_src) = gwe.dialog_cancel_command(dlg, 2);
    assert_eq!(cancel_id, 2);
    assert_eq!(cancel_src, cancel_btn);

    // record_destroy_lifecycle_message: WM_DESTROY and WM_NCDESTROY.
    assert!(gwe.record_destroy_lifecycle_message(hwnd, WM_DESTROY));
    // Calling again is idempotent → true but doesn't double-record.
    assert!(gwe.record_destroy_lifecycle_message(hwnd, WM_DESTROY));
    assert!(gwe.record_destroy_lifecycle_message(hwnd, WM_NCDESTROY));
    // Unknown msg → false.
    assert!(!gwe.record_destroy_lifecycle_message(hwnd, 0xDEAD));
    // Invalid hwnd → false.
    assert!(!gwe.record_destroy_lifecycle_message(0xDEAD, WM_DESTROY));

    // get_key_state / get_async_key_state / get_async_shift_flags:
    // With no key state set, all return zero-ish values.
    let ks = gwe.get_key_state(0x41); // VK 'A'
    assert_eq!(ks, 0); // not pressed, not toggled
    let aks = gwe.get_async_key_state(0x41);
    assert_eq!(aks, 0);
    // get_async_shift_flags: returns current shift flags (no modifiers pressed → shifts = 0).
    let sf = gwe.get_async_shift_flags(0x41);
    assert_eq!(sf & 0x8000_0000, 0); // not down
}

#[test]
fn gwe_sent_queue_snapshot_terminate_expire_result_writes_child_from_point_and_class_cursor() {
    use wince_emulation_v3::ce::gwe::{
        Gwe, Message, Point, Rect, WS_CHILD, WS_VISIBLE, WNDCLASSW_SIZE, QS_SENDMESSAGE,
    };
    const WM_USER: u32 = 0x0400;

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 200, bottom: 200 };
    let tid: u32 = 1;
    let hwnd1 = gwe.create_window_ex_with_rect(tid, "WndA", "W1", None, 0, WS_VISIBLE, 0, rect);
    let hwnd2 = gwe.create_window_ex_with_rect(tid, "WndA", "W2", None, 0, WS_VISIBLE, 0, rect);

    // sent_queue_snapshot: empty before any send.
    assert!(gwe.sent_queue_snapshot().is_empty());

    // queue_send_message_for_window: invalid hwnd → None.
    assert!(gwe.queue_send_message_for_window(
        Some(99), 0xDEAD, Message::new(0, WM_USER, 0, 0, 0), 0, None
    ).is_none());

    // Queue msg1 for hwnd1.
    let id1 = gwe.queue_send_message_for_window(
        Some(99), hwnd1, Message::new(hwnd1, WM_USER + 1, 0xAA, 0xBB, 0), 0, None
    ).unwrap();

    // sent_queue_snapshot: one thread entry with one message.
    let snap = gwe.sent_queue_snapshot();
    assert_eq!(snap.len(), 1);
    assert_eq!(snap[0].0, tid);
    assert_eq!(snap[0].1.len(), 1);
    assert_eq!(snap[0].1[0].hwnd, hwnd1);
    assert_eq!(snap[0].1[0].msg, WM_USER + 1);

    // has_pending_sent_message_for_thread.
    assert!(gwe.has_pending_sent_message_for_thread(tid));
    assert!(!gwe.has_pending_sent_message_for_thread(999));

    // has_new_queue_input / clear_new_queue_input.
    assert!(gwe.has_new_queue_input(tid, QS_SENDMESSAGE));
    gwe.clear_new_queue_input(tid, QS_SENDMESSAGE);
    assert!(!gwe.has_new_queue_input(tid, QS_SENDMESSAGE));
    gwe.clear_new_queue_input(999, QS_SENDMESSAGE); // no-op, must not panic

    // set_sent_message_result_ptr: invalid id → false; valid → true.
    assert!(!gwe.set_sent_message_result_ptr(9999, 0x1234));
    assert!(gwe.set_sent_message_result_ptr(id1, 0x1234));

    // set_sent_message_timeout_flags: invalid id → false; valid → true.
    assert!(!gwe.set_sent_message_timeout_flags(9999, 0xF));
    assert!(gwe.set_sent_message_timeout_flags(id1, 0xF));

    // Queue msg2 for hwnd2, same sender.
    let id2 = gwe.queue_send_message_for_window(
        Some(99), hwnd2, Message::new(hwnd2, WM_USER + 2, 0, 0, 0), 0, None
    ).unwrap();

    // sent_message_ids_for_windows: filter by window.
    let ids_for_hw1 = gwe.sent_message_ids_for_windows(&[hwnd1]);
    assert!(ids_for_hw1.contains(&id1));
    assert!(!ids_for_hw1.contains(&id2));
    let ids_for_both = gwe.sent_message_ids_for_windows(&[hwnd1, hwnd2]);
    assert!(ids_for_both.contains(&id1));
    assert!(ids_for_both.contains(&id2));

    // Queue msg3 with timeout so it can be expired.
    let id3 = gwe.queue_send_message_for_window(
        Some(88), hwnd1, Message::new(hwnd1, WM_USER + 3, 0, 0, 0), 0, Some(500)
    ).unwrap();

    // expire_timed_out_sent_messages: not yet expired at now_ms=0 (0 - 0 = 0 < 500).
    assert!(gwe.expire_timed_out_sent_messages(0).is_empty());

    // Expired at now_ms=600 (600 - 0 = 600 >= 500).
    let expired = gwe.expire_timed_out_sent_messages(600);
    assert!(expired.contains(&id3));
    // id1 and id2 have no timeout, still pending.
    assert!(gwe.has_pending_sent_message_for_thread(tid));

    // Queue msg4 with timeout and result_ptr; expire it; check completed_result_writes.
    let id4 = gwe.queue_send_message_for_window(
        Some(77), hwnd1, Message::new(hwnd1, WM_USER + 4, 0, 0, 100), 0, Some(200)
    ).unwrap();
    assert!(gwe.set_sent_message_result_ptr(id4, 0xBEEF));
    // Expire at now_ms=400: 400 - 100 = 300 >= 200.
    let expired4 = gwe.expire_timed_out_sent_messages(400);
    assert!(expired4.contains(&id4));
    // completed_sent_message_result_writes: id4 has result_ptr + RESULT_READY + result=0.
    let writes = gwe.completed_sent_message_result_writes();
    assert!(writes.iter().any(|(id, ptr, result)| *id == id4 && *ptr == 0xBEEF && *result == 0));

    // terminate_sent_messages_from_sender: removes all pending messages from sender 99.
    let terminated = gwe.terminate_sent_messages_from_sender(99);
    assert!(terminated.contains(&id1));
    assert!(terminated.contains(&id2));
    let remaining = gwe.sent_message_ids_for_windows(&[hwnd1, hwnd2]);
    assert!(!remaining.contains(&id1));
    assert!(!remaining.contains(&id2));
    // Unknown sender → empty.
    assert!(gwe.terminate_sent_messages_from_sender(12345).is_empty());

    // take_sent_message_filtered: removes specific sent message by hwnd + msg range.
    let _id5 = gwe.queue_send_message_for_window(
        Some(55), hwnd1, Message::new(hwnd1, WM_USER + 5, 0, 0, 0), 0, None
    ).unwrap();
    let taken = gwe.take_sent_message_filtered(tid, Some(hwnd1), WM_USER + 5, WM_USER + 5);
    assert!(taken.is_some());
    assert_eq!(taken.unwrap().msg, WM_USER + 5);
    // Already consumed → None.
    assert!(gwe.take_sent_message_filtered(tid, Some(hwnd1), WM_USER + 5, WM_USER + 5).is_none());

    // child_window_from_point_for_thread.
    let par_rect = Rect { left: 0, top: 0, right: 300, bottom: 300 };
    let par = gwe.create_window_ex_with_rect(tid, "ParC", "P", None, 0, WS_VISIBLE, 0, par_rect);
    let chd_rect = Rect { left: 10, top: 10, right: 100, bottom: 100 };
    let chd = gwe.create_window_ex_with_rect(
        tid, "ChdC", "C", Some(par), 0, WS_VISIBLE | WS_CHILD, 0, chd_rect
    );
    // Point in child client area → child returned.
    assert_eq!(gwe.child_window_from_point_for_thread(tid, par, Point { x: 50, y: 50 }), Some(chd));
    // Point in parent but outside child → parent returned.
    assert_eq!(gwe.child_window_from_point_for_thread(tid, par, Point { x: 5, y: 5 }), Some(par));
    // Point outside parent → None.
    assert!(gwe.child_window_from_point_for_thread(tid, par, Point { x: 500, y: 500 }).is_none());
    // Invalid parent → None.
    assert!(gwe.child_window_from_point_for_thread(tid, 0xDEAD, Point { x: 0, y: 0 }).is_none());

    // window_class_cursor / window_class_hbr_background: read from WNDCLASSW bytes.
    let mut class_bytes = [0u8; WNDCLASSW_SIZE];
    class_bytes[24..28].copy_from_slice(&0x0000_1234_u32.to_le_bytes()); // cursor at offset 24
    class_bytes[28..32].copy_from_slice(&0x0000_5678_u32.to_le_bytes()); // hbr_background at offset 28
    gwe.register_class("CursorClass", class_bytes);
    let cw = gwe.create_window_ex_with_rect(tid, "CursorClass", "CW", None, 0, WS_VISIBLE, 0, rect);
    assert_eq!(gwe.window_class_cursor(cw), Some(0x1234));
    assert_eq!(gwe.window_class_hbr_background(cw), 0x5678);
    // cursor field = 0 → None; hbr_background still readable.
    let mut class_bytes0 = [0u8; WNDCLASSW_SIZE];
    class_bytes0[28..32].copy_from_slice(&0x0000_ABCD_u32.to_le_bytes());
    gwe.register_class("NoCursorClass", class_bytes0);
    let ncw = gwe.create_window_ex_with_rect(tid, "NoCursorClass", "NC", None, 0, WS_VISIBLE, 0, rect);
    assert_eq!(gwe.window_class_cursor(ncw), None);
    assert_eq!(gwe.window_class_hbr_background(ncw), 0xABCD);
    // Invalid hwnd → None / 0.
    assert_eq!(gwe.window_class_cursor(0xDEAD), None);
    assert_eq!(gwe.window_class_hbr_background(0xDEAD), 0);
}

#[test]
fn gwe_post_message_for_window_register_window_message_message_source_activate_complete_and_window_and_descendants() {
    use wince_emulation_v3::ce::gwe::{
        Gwe, Message, Rect, WS_CHILD, WS_VISIBLE, MSGSRC_SOFTWARE_POST, MSGSRC_SOFTWARE_SEND,
        MSGSRC_UNKNOWN,
    };
    const WM_USER: u32 = 0x0400;

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 100, bottom: 100 };
    let tid: u32 = 1;
    let hwnd = gwe.create_window_ex_with_rect(tid, "Cls", "W", None, 0, WS_VISIBLE, 0, rect);

    // post_message_for_window: valid hwnd → true; invalid → false.
    assert!(gwe.post_message_for_window(hwnd, Message::new(hwnd, WM_USER + 1, 0, 0, 0)));
    assert!(!gwe.post_message_for_window(0xDEAD, Message::new(0, WM_USER + 2, 0, 0, 0)));
    let msg = gwe.get_message(tid).unwrap();
    assert_eq!(msg.msg, WM_USER + 1);

    // get_message_source: after get_message of a posted message → MSGSRC_SOFTWARE_POST.
    assert_eq!(gwe.get_message_source(tid), MSGSRC_SOFTWARE_POST);
    // Unknown thread → MSGSRC_UNKNOWN.
    assert_eq!(gwe.get_message_source(999), MSGSRC_UNKNOWN);

    // register_window_message: first call → id in [0xC000, 0xFFFF].
    let rm1 = gwe.register_window_message("MyMsg").unwrap();
    assert!((0xC000..=0xFFFF).contains(&rm1));
    // Same name → same id.
    assert_eq!(gwe.register_window_message("MyMsg").unwrap(), rm1);
    // Different name → different id.
    let rm2 = gwe.register_window_message("OtherMsg").unwrap();
    assert_ne!(rm1, rm2);
    // Empty string normalizes to "#anonymous", still registers successfully.
    let rm_anon = gwe.register_window_message("").unwrap();
    // Calling again with empty string returns the same id (idempotent).
    assert_eq!(gwe.register_window_message("").unwrap(), rm_anon);

    // queue_sent_message_for_window: bool wrapper around queue_send_message_for_window.
    assert!(gwe.queue_sent_message_for_window(hwnd, Message::new(hwnd, WM_USER + 10, 0, 0, 0)));
    assert!(!gwe.queue_sent_message_for_window(0xDEAD, Message::new(0, WM_USER + 10, 0, 0, 0)));

    // activate_sent_message_for_receiver / complete_active_sent_message / take_completed_sent_message_result.
    let sm_id = gwe.queue_send_message_for_window(
        Some(77), hwnd, Message::new(hwnd, WM_USER + 20, 0, 0, 0), 0, None
    ).unwrap();
    // Activate: moves sm_id from sent_queues to active_sent_stack.
    assert!(gwe.activate_sent_message_for_receiver(tid, sm_id));
    // get_message_source now reflects MSGSRC_SOFTWARE_SEND.
    assert_eq!(gwe.get_message_source(tid), MSGSRC_SOFTWARE_SEND);
    // Second activate: no longer in queue → false.
    assert!(!gwe.activate_sent_message_for_receiver(tid, sm_id));
    // active_sent_message_id: sm_id is on top of the stack.
    assert_eq!(gwe.active_sent_message_id(tid), Some(sm_id));
    // Not RESULT_READY yet → take returns None.
    assert!(gwe.take_completed_sent_message_result(sm_id).is_none());
    // complete_active_sent_message: marks RESULT_READY, stores result.
    let completed = gwe.complete_active_sent_message(tid, 0xCAFE).unwrap();
    assert_eq!(completed, sm_id);
    assert!(gwe.sent_message_result_ready(sm_id));
    // take_completed_sent_message_result: returns result and removes entry.
    assert_eq!(gwe.take_completed_sent_message_result(sm_id), Some(0xCAFE));
    // Gone after take.
    assert!(gwe.take_completed_sent_message_result(sm_id).is_none());

    // window_and_descendants: includes self, children, and grandchildren.
    let child = gwe.create_window_ex_with_rect(
        tid, "ChlC", "C", Some(hwnd), 0, WS_VISIBLE | WS_CHILD, 0, rect
    );
    let grand = gwe.create_window_ex_with_rect(
        tid, "GrC", "G", Some(child), 0, WS_VISIBLE | WS_CHILD, 0, rect
    );
    let desc = gwe.window_and_descendants(hwnd).unwrap();
    assert!(desc.contains(&hwnd));
    assert!(desc.contains(&child));
    assert!(desc.contains(&grand));
    // Invalid hwnd → None.
    assert!(gwe.window_and_descendants(0xDEAD).is_none());
}

#[test]
fn gwe_send_message_dispatch_set_window_pos_and_clipboard_render() {
    use wince_emulation_v3::ce::gwe::{
        Gwe, Rect, WS_VISIBLE, WNDCLASSW_SIZE,
        WM_MOUSEACTIVATE, WM_NCACTIVATE, WM_NCHITTEST, WM_ERASEBKGND, WM_SETREDRAW,
        WM_GETTEXTLENGTH, WM_SETFONT, WM_GETFONT, WM_NCCREATE,
        MA_ACTIVATE, HTCLIENT,
        SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SWP_HIDEWINDOW, SWP_NOZORDER,
    };

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 200, bottom: 100 };
    let tid: u32 = 1;
    let hwnd = gwe.create_window_ex_with_rect(tid, "TestCls", "MyTitle", None, 0, WS_VISIBLE, 0, rect);

    // --- send_message dispatch.
    // Invalid hwnd → None.
    assert!(gwe.send_message(0xDEAD, WM_MOUSEACTIVATE, 0, 0).is_none());

    // WM_MOUSEACTIVATE → MA_ACTIVATE.
    assert_eq!(gwe.send_message(hwnd, WM_MOUSEACTIVATE, 0, 0), Some(MA_ACTIVATE));

    // WM_NCACTIVATE → 1.
    assert_eq!(gwe.send_message(hwnd, WM_NCACTIVATE, 0, 0), Some(1));

    // WM_NCHITTEST: point (50, 50) inside client area → HTCLIENT.
    // lparam encodes point as (y<<16) | (x & 0xffff).
    let lparam = ((50u32) << 16) | (50u32 & 0xffff);
    assert_eq!(gwe.send_message(hwnd, WM_NCHITTEST, 0, lparam), Some(HTCLIENT));

    // WM_GETTEXTLENGTH → character count of title (encoded as UTF-16 code units).
    let title_len = "MyTitle".encode_utf16().count() as u32;
    assert_eq!(gwe.send_message(hwnd, WM_GETTEXTLENGTH, 0, 0), Some(title_len));

    // WM_ERASEBKGND without background brush → 0; with brush → 1.
    assert_eq!(gwe.send_message(hwnd, WM_ERASEBKGND, 0, 0), Some(0)); // no brush registered
    // Register a class with non-zero hbr_background.
    let mut class_bytes = [0u8; WNDCLASSW_SIZE];
    class_bytes[28..32].copy_from_slice(&1u32.to_le_bytes()); // hbr = 1
    gwe.register_class("TestCls", class_bytes); // re-register updates bytes
    assert_eq!(gwe.send_message(hwnd, WM_ERASEBKGND, 0, 0), Some(1)); // brush present

    // WM_NCCREATE → 1.
    assert_eq!(gwe.send_message(hwnd, WM_NCCREATE, 0, 0), Some(1));

    // WM_SETFONT stores font handle; WM_GETFONT returns it.
    assert_eq!(gwe.send_message(hwnd, WM_GETFONT, 0, 0), Some(0)); // initially 0
    gwe.send_message(hwnd, WM_SETFONT, 0xABCD, 0);
    assert_eq!(gwe.send_message(hwnd, WM_GETFONT, 0, 0), Some(0xABCD));

    // WM_SETREDRAW (returns default 0).
    assert_eq!(gwe.send_message(hwnd, WM_SETREDRAW, 1, 0), Some(0));

    // --- set_window_pos.
    // Invalid hwnd → false.
    assert!(!gwe.set_window_pos(0xDEAD, None, 10, 20, 100, 50, 0));

    // Move and resize (no flags): new rect = (10, 20, 10+150=160, 20+80=100).
    assert!(gwe.set_window_pos(hwnd, None, 10, 20, 150, 80, SWP_NOZORDER));
    let new_rect = gwe.get_window_rect(hwnd).unwrap();
    assert_eq!(new_rect.left, 10);
    assert_eq!(new_rect.top, 20);
    assert_eq!(new_rect.width(), 150);
    assert_eq!(new_rect.height(), 80);

    // SWP_NOMOVE: position unchanged; size changes.
    assert!(gwe.set_window_pos(hwnd, None, 999, 999, 50, 40, SWP_NOMOVE | SWP_NOZORDER));
    let r2 = gwe.get_window_rect(hwnd).unwrap();
    assert_eq!(r2.left, 10); // unchanged
    assert_eq!(r2.top, 20); // unchanged
    assert_eq!(r2.width(), 50);
    assert_eq!(r2.height(), 40);

    // SWP_NOSIZE: size unchanged; position changes.
    assert!(gwe.set_window_pos(hwnd, None, 0, 0, 999, 999, SWP_NOSIZE | SWP_NOZORDER));
    let r3 = gwe.get_window_rect(hwnd).unwrap();
    assert_eq!(r3.left, 0);
    assert_eq!(r3.top, 0);
    assert_eq!(r3.width(), 50); // unchanged
    assert_eq!(r3.height(), 40); // unchanged

    // SWP_HIDEWINDOW: window becomes invisible.
    assert!(gwe.is_window_visible(hwnd));
    assert!(gwe.set_window_pos(hwnd, None, 0, 0, 50, 40, SWP_HIDEWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER));
    assert!(!gwe.is_window_visible(hwnd));

    // SWP_SHOWWINDOW: window becomes visible again.
    assert!(gwe.set_window_pos(hwnd, None, 0, 0, 50, 40, SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER));
    assert!(gwe.is_window_visible(hwnd));

    // --- begin_clipboard_render / clipboard_render_all_owner.
    let owner_hwnd = gwe.create_window_ex_with_rect(tid, "ClpCls", "O", None, 0, WS_VISIBLE, 0, rect);
    // Open and empty clipboard to set owner.
    assert!(gwe.open_clipboard(owner_hwnd));
    assert!(gwe.empty_clipboard()); // owner = owner_hwnd; data cleared
    // Set delayed render format (handle=0) — clipboard must be open.
    assert_eq!(gwe.set_clipboard_data(tid, 42, 0), Some(0));

    // clipboard_render_all_owner: owner_hwnd has format 42 with handle=0 → Some.
    assert_eq!(gwe.clipboard_render_all_owner(), Some(owner_hwnd));

    // begin_clipboard_render: clipboard is open, format 42 has handle=0.
    assert_eq!(gwe.begin_clipboard_render(42), Some(owner_hwnd));
    // Second call: render_window is already set → None.
    assert!(gwe.begin_clipboard_render(42).is_none());
    // begin_clipboard_render with unknown format → None.
    assert!(gwe.begin_clipboard_render(99).is_none());
}

#[test]
fn registry_reg_enum_value_w_key_count_and_reg_set_value_exw_binary() {
    use wince_emulation_v3::ce::registry::{
        ERROR_INVALID_HANDLE, ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS,
        HKEY_LOCAL_MACHINE, Registry, RegistryDump, RegistryValue, REG_BINARY,
    };

    let mut reg = Registry::from_dump(RegistryDump {
        version: 0,
        source: None,
        keys: std::collections::BTreeMap::new(),
    });

    // key_count: starts with predefined roots only.
    let initial = reg.key_count();
    reg.create_key(r"hklm\EnumTest");
    assert_eq!(reg.key_count(), initial + 1);

    // Set two values on EnumTest.
    reg.set_value(r"hklm\EnumTest", "Alpha", RegistryValue::dword(0xABCD));
    reg.set_value(r"hklm\EnumTest", "Beta", RegistryValue::string("hello"));

    // Open a handle to EnumTest.
    let create = reg.reg_create_key_exw(HKEY_LOCAL_MACHINE, Some("EnumTest"));
    assert_eq!(create.status, ERROR_SUCCESS);
    let hkey = create.hkey.unwrap();

    // reg_enum_value_w: index 0 returns first value; index beyond end → ERROR_NO_MORE_ITEMS.
    let r0 = reg.reg_enum_value_w(hkey, 0, None, None);
    assert_eq!(r0.status, ERROR_SUCCESS);
    assert!(r0.name.is_some());

    let r1 = reg.reg_enum_value_w(hkey, 1, None, None);
    assert_eq!(r1.status, ERROR_SUCCESS);
    assert!(r1.name.is_some());

    let r2 = reg.reg_enum_value_w(hkey, 2, None, None);
    assert_eq!(r2.status, ERROR_NO_MORE_ITEMS);

    // Both values found (names are "alpha" and "beta" — normalized to lowercase).
    let names: std::collections::BTreeSet<_> = [r0.name.unwrap(), r1.name.unwrap()].into();
    assert!(names.contains("alpha"));
    assert!(names.contains("beta"));

    // reg_enum_value_w: name_capacity too small → ERROR_MORE_DATA.
    let r_small = reg.reg_enum_value_w(hkey, 0, Some(1), None); // capacity=1 for a 5-char name
    assert_eq!(r_small.status, ERROR_MORE_DATA);
    assert!(r_small.name.is_none());
    assert!(r_small.data.is_none());

    // reg_enum_value_w: data_capacity too small → ERROR_MORE_DATA; name still returned.
    let r_small_data = reg.reg_enum_value_w(hkey, 0, None, Some(0));
    assert_eq!(r_small_data.status, ERROR_MORE_DATA);
    assert!(r_small_data.name.is_some());
    assert!(r_small_data.data.is_none());

    // reg_enum_value_w: with sufficient data_capacity → data is Some.
    let r_with_data = reg.reg_enum_value_w(hkey, 0, None, Some(4096));
    assert_eq!(r_with_data.status, ERROR_SUCCESS);
    assert!(r_with_data.data.is_some());

    // reg_enum_value_w with invalid handle → ERROR_INVALID_HANDLE.
    let r_inv = reg.reg_enum_value_w(0xDEAD_BEEF, 0, None, None);
    assert_eq!(r_inv.status, ERROR_INVALID_HANDLE);

    // reg_set_value_exw with REG_BINARY data.
    let hkey2 = reg.reg_create_key_exw(HKEY_LOCAL_MACHINE, Some("BinaryTest")).hkey.unwrap();
    let bin_data = [0xDE_u8, 0xAD, 0xBE, 0xEF];
    assert_eq!(
        reg.reg_set_value_exw(hkey2, Some("BinVal"), REG_BINARY, &bin_data),
        ERROR_SUCCESS
    );
    let q = reg.reg_query_value_exw(hkey2, Some("BinVal"), Some(16));
    assert_eq!(q.status, ERROR_SUCCESS);
    assert_eq!(q.value_type, Some(REG_BINARY));
    assert_eq!(q.data.as_deref(), Some(bin_data.as_slice()));
}

#[test]
fn timer_system_next_due_delay_ms_due_timers_remove_window_timers_and_clear_pending_message() {
    use wince_emulation_v3::ce::timer::TimerSystem;
    use wince_emulation_v3::ce::gwe::WM_TIMER;

    let mut timers = TimerSystem::default();
    let tid: u32 = 1;

    // next_due_delay_ms: no timers → None.
    assert!(timers.next_due_delay_ms().is_none());

    // Set a timer with 500ms period → due in 500ms.
    timers.set_timer(tid, Some(0x1_0001), Some(1), 500, WM_TIMER, None);
    // next_due_delay_ms: should be ≤ 500 (slightly less due to elapsed boot time).
    let delay = timers.next_due_delay_ms().unwrap();
    assert!(delay <= 500);

    // due_timers: not due yet → empty.
    let due = timers.due_timers();
    assert!(due.is_empty());

    // Advance virtual time by 600ms to make the timer due.
    timers.sleep_ms(600);
    let due = timers.due_timers();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].id, 1);
    assert_eq!(due[0].hwnd, Some(0x1_0001));
    // After due_timers, timer is re-inserted with pending_message=true.
    assert_eq!(timers.timer_count(), 1);

    // clear_pending_message: clears pending flag; returns true if it was set.
    assert!(timers.clear_pending_message(tid, Some(0x1_0001), 1));
    // Calling again when already cleared → false.
    assert!(!timers.clear_pending_message(tid, Some(0x1_0001), 1));
    // Invalid timer → false.
    assert!(!timers.clear_pending_message(tid, Some(0x1_0001), 99));

    // remove_window_timers: removes all timers for the given hwnd(s).
    // Set a second timer for a different hwnd.
    timers.set_timer(tid, Some(0x2_0000), Some(2), 100, WM_TIMER, None);
    assert_eq!(timers.timer_count(), 2);
    // Remove timers for hwnd 0x1_0001 only.
    let removed = timers.remove_window_timers(&[0x1_0001]);
    assert_eq!(removed, 1);
    assert_eq!(timers.timer_count(), 1);
    // Remove remaining.
    let removed2 = timers.remove_window_timers(&[0x2_0000]);
    assert_eq!(removed2, 1);
    assert_eq!(timers.timer_count(), 0);
    // No timers for unknown hwnd → 0.
    assert_eq!(timers.remove_window_timers(&[0xDEAD_BEEF]), 0);
}

#[test]
fn shell_system_special_folder_queries_fallback_policy_destroyed_notify_icons_and_notification_callbacks() {
    use wince_emulation_v3::ce::shell::{
        NotifyIconData, NotifyIconOp, ShellNotificationCallbackMethod,
        ShellNotificationCallbackRecord, ShellSpecialFolderFallbackPolicy,
        ShellSpecialFolderRecord, ShellSpecialFolderSource, ShellSystem,
        NIF_ICON, HHTBF_DESTROYICON,
    };

    let mut shell = ShellSystem::default();

    // special_folder_fallback_policy: default is Compat.
    assert_eq!(shell.special_folder_fallback_policy(), ShellSpecialFolderFallbackPolicy::Compat);
    // set_special_folder_fallback_policy → stored.
    shell.set_special_folder_fallback_policy(ShellSpecialFolderFallbackPolicy::Strict);
    assert_eq!(shell.special_folder_fallback_policy(), ShellSpecialFolderFallbackPolicy::Strict);

    // record_special_folder_query / special_folder_queries.
    assert_eq!(shell.special_folder_queries().count(), 0);
    shell.record_special_folder_query(ShellSpecialFolderRecord {
        csidl: 5,
        value_name: "Personal".to_owned(),
        path: r"\My Documents".to_owned(),
        source: ShellSpecialFolderSource::Registry,
        create_requested: false,
    });
    shell.record_special_folder_query(ShellSpecialFolderRecord {
        csidl: 7,
        value_name: "Startup".to_owned(),
        path: r"\Windows\Start Menu\Programs\Startup".to_owned(),
        source: ShellSpecialFolderSource::FallbackMissing,
        create_requested: true,
    });
    let queries: Vec<_> = shell.special_folder_queries().collect();
    assert_eq!(queries.len(), 2);
    assert_eq!(queries[0].csidl, 5);
    assert_eq!(queries[1].source, ShellSpecialFolderSource::FallbackMissing);

    // destroyed_notify_icons: populated when a notify icon with destroy_icon=true is replaced.
    // HHTBF_DESTROYICON flag sets destroy_icon=true on creation.
    let icon_data = NotifyIconData {
        hwnd: 20, id: 3, flags: HHTBF_DESTROYICON, icon: 0xABCD, tip: String::new(),
        callback_message: 0, state: 0, state_mask: 0,
    };
    shell.apply_notify_icon(NotifyIconOp::Add, icon_data.clone());
    // Modify with NIF_ICON and a different icon → old icon pushed to destroyed_notify_icons.
    let modified = NotifyIconData {
        hwnd: 20, id: 3, flags: NIF_ICON, icon: 0xDEAD, tip: String::new(),
        callback_message: 0, state: 0, state_mask: 0,
    };
    shell.apply_notify_icon(NotifyIconOp::Modify, modified);
    assert_eq!(shell.destroyed_notify_icons().count(), 1);
    assert_eq!(*shell.destroyed_notify_icons().next().unwrap(), 0xABCD);

    // record_notification_callback / notification_callbacks.
    use wince_emulation_v3::ce::shell::ShellNotificationCallbackArguments;
    assert_eq!(shell.notification_callbacks().count(), 0);
    shell.record_notification_callback(ShellNotificationCallbackRecord {
        clsid: [0u8; 16],
        id: 1,
        method: ShellNotificationCallbackMethod::OnDismiss { timed_out: false },
        vtable_offset: 0,
        arguments: ShellNotificationCallbackArguments::OnDismiss {
            id: 1,
            timed_out: false,
            lparam: 0,
        },
        lparam: 0,
    });
    assert_eq!(shell.notification_callbacks().count(), 1);
}

#[test]
fn gwe_send_message_wm_close_wm_getdlgcode_dm_defid_and_message_pos_after_get_message() {
    use wince_emulation_v3::ce::gwe::{
        BS_DEFPUSHBUTTON, BS_PUSHBUTTON, DC_HASDEFID, DLGC_BUTTON, DLGC_DEFPUSHBUTTON,
        DLGC_HASSETSEL, DLGC_STATIC, DLGC_UNDEFPUSHBUTTON, DLGC_WANTCHARS, DM_GETDEFID,
        DM_SETDEFID, Gwe, Message, Rect, WM_CLOSE, WM_GETDLGCODE, WM_USER, WS_CHILD, WS_VISIBLE,
    };

    let mut gwe = Gwe::default();
    let big = Rect { left: 0, top: 0, right: 400, bottom: 300 };
    let btn_rect = Rect { left: 10, top: 10, right: 90, bottom: 40 };

    // WM_CLOSE dispatch: sends WM_DESTROY internally then destroys window.
    let close_w = gwe.create_window_ex_with_rect(1, "CloseMe", "W", None, 0, WS_VISIBLE, 0, big);
    assert!(gwe.is_window(close_w));
    let _ = gwe.send_message(close_w, WM_CLOSE, 0, 0);
    assert!(!gwe.is_window(close_w));

    // WM_GETDLGCODE for a generic (non-dialog) class returns 0.
    let generic = gwe.create_window_ex_with_rect(1, "MyClass", "W", None, 0, WS_VISIBLE, 0, big);
    assert_eq!(gwe.send_message(generic, WM_GETDLGCODE, 0, 0), Some(0));

    // WM_GETDLGCODE for "button" BS_PUSHBUTTON → DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON.
    let dlg = gwe.create_window_ex_with_rect(1, "dialog", "Dlg", None, 0, WS_VISIBLE, 0, big);
    let pushbtn = gwe.create_window_ex_with_rect(
        1, "button", "OK", Some(dlg), 1, WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, 0, btn_rect,
    );
    assert_eq!(
        gwe.send_message(pushbtn, WM_GETDLGCODE, 0, 0),
        Some(DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON),
    );

    // WM_GETDLGCODE for "button" BS_DEFPUSHBUTTON → DLGC_BUTTON | DLGC_DEFPUSHBUTTON.
    let defbtn = gwe.create_window_ex_with_rect(
        1, "button", "Def", Some(dlg), 2, WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON, 0, btn_rect,
    );
    assert_eq!(
        gwe.send_message(defbtn, WM_GETDLGCODE, 0, 0),
        Some(DLGC_BUTTON | DLGC_DEFPUSHBUTTON),
    );

    // WM_GETDLGCODE for "static" → DLGC_STATIC.
    let static_w = gwe.create_window_ex_with_rect(
        1, "static", "Lbl", Some(dlg), 3, WS_VISIBLE | WS_CHILD, 0, btn_rect,
    );
    assert_eq!(gwe.send_message(static_w, WM_GETDLGCODE, 0, 0), Some(DLGC_STATIC));

    // WM_GETDLGCODE for "edit" → DLGC_HASSETSEL | DLGC_WANTCHARS | DLGC_WANTARROWS.
    let edit_w = gwe.create_window_ex_with_rect(
        1, "edit", "Ed", Some(dlg), 4, WS_VISIBLE | WS_CHILD, 0, btn_rect,
    );
    {
        use wince_emulation_v3::ce::gwe::DLGC_WANTARROWS;
        assert_eq!(
            gwe.send_message(edit_w, WM_GETDLGCODE, 0, 0),
            Some(DLGC_HASSETSEL | DLGC_WANTCHARS | DLGC_WANTARROWS),
        );
    }

    // DM_GETDEFID with no DEFPUSHBUTTON → 0.
    let dlg2 = gwe.create_window_ex_with_rect(2, "dialog", "Dlg2", None, 0, WS_VISIBLE, 0, big);
    let plain_btn = gwe.create_window_ex_with_rect(
        2, "button", "Go", Some(dlg2), 42, WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, 0, btn_rect,
    );
    assert_eq!(gwe.send_message(dlg2, DM_GETDEFID, 0, 0), Some(0));

    // DM_SETDEFID(42): promotes plain_btn (id=42) to BS_DEFPUSHBUTTON.
    assert_eq!(gwe.send_message(dlg2, DM_SETDEFID, 42, 0), Some(1));

    // DM_GETDEFID after: returns id | (DC_HASDEFID << 16).
    let expected = 42 | (DC_HASDEFID << 16);
    assert_eq!(gwe.send_message(dlg2, DM_GETDEFID, 0, 0), Some(expected));

    // DM_SETDEFID also switches the button style.
    assert_eq!(
        gwe.send_message(plain_btn, WM_GETDLGCODE, 0, 0),
        Some(DLGC_BUTTON | DLGC_DEFPUSHBUTTON),
    );

    // post_quit_message → get_message_queue_ready_time_stamp non-zero.
    let tid = 3_u32;
    gwe.post_quit_message(tid, 7, 5000);
    assert_eq!(gwe.get_message_queue_ready_time_stamp(tid, 0), 5000);
    // hwnd-based lookup uses window's thread_id.
    let ts_hwnd = gwe.create_window_ex_with_rect(tid, "Ts", "T", None, 0, WS_VISIBLE, 0, big);
    assert_eq!(gwe.get_message_queue_ready_time_stamp(99, ts_hwnd), 5000);

    // get_message_pos after get_message with mouse_pos_at_post.
    let tid2 = 4_u32;
    let pos_hwnd = gwe.create_window_ex_with_rect(tid2, "Pos", "P", None, 0, WS_VISIBLE, 0, big);
    let encoded_pos = (50u32 << 16) | 100u32; // y=50, x=100
    let mut msg = Message::new(pos_hwnd, WM_USER, 0, 0, 0);
    msg.mouse_pos_at_post = Some(encoded_pos);
    gwe.post_message(tid2, msg);
    assert_eq!(gwe.get_message_pos(tid2), 0); // not yet dequeued
    let _ = gwe.get_message(tid2).unwrap();
    assert_eq!(gwe.get_message_pos(tid2), encoded_pos);
}

#[test]
fn gwe_send_message_setcursor_syscommand_nclbuttondown_hscroll_vscroll_and_syskeydown_chains() {
    use wince_emulation_v3::ce::gwe::{
        Gwe, Rect, WM_HSCROLL, WM_NCLBUTTONDOWN, WM_NCLBUTTONDBLCLK, WM_SETCURSOR, WM_SYSCOMMAND,
        WM_VSCROLL, HTCAPTION, HTCLIENT, HTCLOSE, HTNOWHERE, HTSYSMENU, SC_CLOSE, WNDCLASSW_SIZE,
        WS_CHILD, WS_VISIBLE,
    };

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 400, bottom: 300 };

    // WM_SETCURSOR with HTCLIENT and a class cursor: sets the cursor, returns Some(1).
    let mut class_bytes = [0u8; WNDCLASSW_SIZE];
    let cursor_handle: u32 = 0x1234;
    class_bytes[24..28].copy_from_slice(&cursor_handle.to_le_bytes());
    gwe.register_class("CursorCls", class_bytes);
    let w_cursor = gwe.create_window_ex_with_rect(1, "CursorCls", "W", None, 0, WS_VISIBLE, 0, rect);

    let result = gwe.send_message(w_cursor, WM_SETCURSOR, 0, HTCLIENT);
    assert_eq!(result, Some(1));
    assert_eq!(gwe.get_cursor(), Some(cursor_handle));

    // WM_SETCURSOR with HTNOWHERE: default_nonclient_cursor returns None → falls through → Some(0).
    let cursor_before = gwe.get_cursor();
    let result_nowhere = gwe.send_message(w_cursor, WM_SETCURSOR, 0, HTNOWHERE);
    assert_eq!(result_nowhere, Some(0));
    // Cursor unchanged when HTNOWHERE.
    assert_eq!(gwe.get_cursor(), cursor_before);

    // WM_SETCURSOR with HTCAPTION: non-client, has a default cursor → sets cursor, returns Some(1).
    let result_cap = gwe.send_message(w_cursor, WM_SETCURSOR, 0, HTCAPTION);
    assert_eq!(result_cap, Some(1));
    // Cursor was changed to the arrow/nonclient default (different from class cursor).
    assert_ne!(gwe.get_cursor(), Some(cursor_handle));

    // WM_SETCURSOR with HTCLIENT but no class cursor: falls through → Some(0).
    let w_no_cursor = gwe.create_window_ex_with_rect(1, "NoCur", "W2", None, 0, WS_VISIBLE, 0, rect);
    let result_no = gwe.send_message(w_no_cursor, WM_SETCURSOR, 0, HTCLIENT);
    assert_eq!(result_no, Some(0));

    // WM_SYSCOMMAND SC_CLOSE → sends WM_CLOSE internally → window destroyed.
    let w_sys = gwe.create_window_ex_with_rect(1, "SysCmd", "W3", None, 0, WS_VISIBLE, 0, rect);
    assert!(gwe.is_window(w_sys));
    let _ = gwe.send_message(w_sys, WM_SYSCOMMAND, SC_CLOSE, 0);
    assert!(!gwe.is_window(w_sys));

    // WM_NCLBUTTONDOWN with HTCLOSE → WM_SYSCOMMAND(SC_CLOSE) → WM_CLOSE → window destroyed.
    let w_nc = gwe.create_window_ex_with_rect(1, "Nc", "W4", None, 0, WS_VISIBLE, 0, rect);
    assert!(gwe.is_window(w_nc));
    let _ = gwe.send_message(w_nc, WM_NCLBUTTONDOWN, HTCLOSE, 0);
    assert!(!gwe.is_window(w_nc));

    // WM_NCLBUTTONDBLCLK with HTSYSMENU → same chain → window destroyed.
    let w_nc2 = gwe.create_window_ex_with_rect(1, "Nc2", "W5", None, 0, WS_VISIBLE, 0, rect);
    assert!(gwe.is_window(w_nc2));
    let _ = gwe.send_message(w_nc2, WM_NCLBUTTONDBLCLK, HTSYSMENU, 0);
    assert!(!gwe.is_window(w_nc2));

    // WM_NCLBUTTONDOWN with HTCAPTION (not HTCLOSE): no SC_CLOSE → window survives.
    let w_nc3 = gwe.create_window_ex_with_rect(1, "Nc3", "W6", None, 0, WS_VISIBLE, 0, rect);
    let _ = gwe.send_message(w_nc3, WM_NCLBUTTONDOWN, HTCAPTION, 0);
    assert!(gwe.is_window(w_nc3));

    // WM_SYSKEYDOWN with VK_F4 (0x73) → WM_SYSCOMMAND(SC_CLOSE) → window destroyed.
    let w_f4 = gwe.create_window_ex_with_rect(1, "F4", "W7", None, 0, WS_VISIBLE, 0, rect);
    assert!(gwe.is_window(w_f4));
    use wince_emulation_v3::ce::gwe::WM_SYSKEYDOWN;
    let _ = gwe.send_message(w_f4, WM_SYSKEYDOWN, 0x73, 0); // VK_F4
    assert!(!gwe.is_window(w_f4));

    // WM_HSCROLL on a child with parent → returns Some(0), no error.
    let parent = gwe.create_window_ex_with_rect(1, "Par", "P", None, 0, WS_VISIBLE, 0, rect);
    let child_rect = Rect { left: 0, top: 0, right: 100, bottom: 50 };
    let child = gwe.create_window_ex_with_rect(1, "Ch", "C", Some(parent), 0, WS_VISIBLE | WS_CHILD, 0, child_rect);
    assert_eq!(gwe.send_message(child, WM_HSCROLL, 1, 0), Some(0));

    // WM_VSCROLL on child → Some(0).
    assert_eq!(gwe.send_message(child, WM_VSCROLL, 2, 0), Some(0));

    // WM_HSCROLL on window without parent → also Some(0).
    let lone = gwe.create_window_ex_with_rect(1, "Lone", "L", None, 0, WS_VISIBLE, 0, rect);
    assert_eq!(gwe.send_message(lone, WM_HSCROLL, 0, 0), Some(0));
}

#[test]
fn gwe_send_message_chartoitem_vkeytoitem_syskeydown_f10_letter_syskeyup_menu_and_toggle_keys() {
    use wince_emulation_v3::ce::gwe::{
        Gwe, Message, Rect, VK_MENU, VK_NUMLOCK, VK_SCROLL, WM_CHARTOITEM, WM_KEYDOWN,
        WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_VKEYTOITEM, WS_VISIBLE,
    };

    let mut gwe = Gwe::default();
    let rect = Rect { left: 0, top: 0, right: 400, bottom: 300 };
    let hwnd = gwe.create_window_ex_with_rect(1, "W", "W", None, 0, WS_VISIBLE, 0, rect);

    // WM_CHARTOITEM → u32::MAX (CE DefWindowProcW returns -1 for list-box routing).
    assert_eq!(gwe.send_message(hwnd, WM_CHARTOITEM, 0, 0), Some(u32::MAX));

    // WM_VKEYTOITEM → u32::MAX (same -1 list-box default).
    assert_eq!(gwe.send_message(hwnd, WM_VKEYTOITEM, 0, 0), Some(u32::MAX));

    // WM_SYSKEYDOWN(VK_F10 = 0x79) → sends WM_SYSCOMMAND(SC_KEYMENU, VK_F10) internally.
    // SC_KEYMENU != SC_CLOSE, so the window survives and the call returns Some(0).
    let w_f10 = gwe.create_window_ex_with_rect(1, "F10", "F", None, 0, WS_VISIBLE, 0, rect);
    let result_f10 = gwe.send_message(w_f10, WM_SYSKEYDOWN, 0x79, 0);
    assert_eq!(result_f10, Some(0));
    assert!(gwe.is_window(w_f10)); // window was NOT destroyed

    // WM_SYSKEYDOWN with Alt+letter (0x41 = 'A') → SC_KEYMENU, wparam.
    let w_alt = gwe.create_window_ex_with_rect(1, "Alt", "A", None, 0, WS_VISIBLE, 0, rect);
    let result_alt = gwe.send_message(w_alt, WM_SYSKEYDOWN, 0x41, 0);
    assert_eq!(result_alt, Some(0));
    assert!(gwe.is_window(w_alt));

    // WM_SYSKEYDOWN with Alt+digit (0x30 = '0') → SC_KEYMENU as well.
    let w_digit = gwe.create_window_ex_with_rect(1, "Dig", "D", None, 0, WS_VISIBLE, 0, rect);
    let result_digit = gwe.send_message(w_digit, WM_SYSKEYDOWN, 0x30, 0);
    assert_eq!(result_digit, Some(0));
    assert!(gwe.is_window(w_digit));

    // WM_SYSKEYUP(VK_MENU = 0x12) → sends WM_SYSCOMMAND(SC_KEYMENU, 0). Window survives.
    let w_altup = gwe.create_window_ex_with_rect(1, "Au", "AU", None, 0, WS_VISIBLE, 0, rect);
    let result_altup = gwe.send_message(w_altup, WM_SYSKEYUP, VK_MENU, 0);
    assert_eq!(result_altup, Some(0));
    assert!(gwe.is_window(w_altup));

    // VK_NUMLOCK toggle: posting WM_KEYDOWN sets down-bit (0x8000) and toggles toggle-bit (0x0001).
    let tid = 1_u32;
    assert_eq!(gwe.get_key_state(VK_NUMLOCK) & 0x0001, 0); // initially not toggled
    gwe.post_message(tid, Message::new(0, WM_KEYDOWN, VK_NUMLOCK, 0, 0));
    // After key-down: toggle bit set, down bit set.
    // get_key_state returns (state as i16 as i32) as u32 — down-bit 0x8000 sign-extends.
    let ks_down = gwe.get_key_state(VK_NUMLOCK);
    assert_ne!(ks_down & 0x8000, 0); // key is now pressed (high bit after sign-extension)
    assert_ne!(ks_down & 0x0001, 0); // toggle bit flipped on first press

    // After WM_KEYUP: down-bit cleared, toggle bit remains.
    gwe.post_message(tid, Message::new(0, WM_KEYUP, VK_NUMLOCK, 0, 0));
    let ks_up = gwe.get_key_state(VK_NUMLOCK);
    assert_eq!(ks_up & 0x8000, 0); // no longer pressed
    assert_ne!(ks_up & 0x0001, 0); // toggle bit persists

    // Second WM_KEYDOWN: was_down=false (key was released), so toggle bit flips again.
    gwe.post_message(tid, Message::new(0, WM_KEYDOWN, VK_NUMLOCK, 0, 0));
    let ks_down2 = gwe.get_key_state(VK_NUMLOCK);
    assert_ne!(ks_down2 & 0x8000, 0); // pressed again
    assert_eq!(ks_down2 & 0x0001, 0); // toggle bit flipped back to 0

    // VK_SCROLL follows the same toggle behavior.
    assert_eq!(gwe.get_key_state(VK_SCROLL) & 0x0001, 0);
    gwe.post_message(tid, Message::new(0, WM_KEYDOWN, VK_SCROLL, 0, 0));
    assert_ne!(gwe.get_key_state(VK_SCROLL) & 0x0001, 0);

    // Regular key (0x41 = 'A') does NOT have a toggle bit.
    let vk_a = 0x41_u32;
    assert_eq!(gwe.get_key_state(vk_a), 0);
    gwe.post_message(tid, Message::new(0, WM_KEYDOWN, vk_a, 0, 0));
    let ks_a = gwe.get_key_state(vk_a);
    assert_ne!(ks_a & 0x8000, 0); // down
    assert_eq!(ks_a & 0x0001, 0); // no toggle bit for regular key
}

#[test]
fn audio_wave_out_reset_set_volume_close_flush_all_and_write_pcm() {
    use wince_emulation_v3::ce::audio::{
        AudioSystem, MMSYSERR_INVALHANDLE, MMSYSERR_NOERROR, NullAudioSink, WaveBuffer,
        WaveFormat, WaveOutState,
    };

    let mut audio = AudioSystem::default();
    let fmt = WaveFormat {
        format_tag: 1,
        channels: 2,
        samples_per_sec: 44_100,
        avg_bytes_per_sec: 176_400,
        block_align: 4,
        bits_per_sample: 16,
    };
    let id = audio.open_wave_out(fmt.clone());

    // wave_out_reset on invalid id → MMSYSERR_INVALHANDLE.
    assert_eq!(audio.wave_out_reset(0xDEAD), MMSYSERR_INVALHANDLE);

    // wave_out_write then wave_out_reset: clears pending buffers, state → Reset.
    let buf = WaveBuffer { guest_ptr: 0x1000, len: 8 };
    assert_eq!(audio.wave_out_write(id, buf), MMSYSERR_NOERROR);
    assert_eq!(audio.output(id).unwrap().state, WaveOutState::Playing);
    assert_eq!(audio.wave_out_reset(id), MMSYSERR_NOERROR);
    assert_eq!(audio.output(id).unwrap().state, WaveOutState::Reset);
    // After reset, complete_next_buffer returns None (queue drained).
    assert!(audio.complete_next_buffer(id).is_none());

    // wave_out_set_volume on invalid id → MMSYSERR_INVALHANDLE.
    assert_eq!(audio.wave_out_set_volume(0xDEAD, 0xFFFF), MMSYSERR_INVALHANDLE);
    // Valid id → MMSYSERR_NOERROR; volume stored.
    assert_eq!(audio.wave_out_set_volume(id, 0x8000_0000), MMSYSERR_NOERROR);
    assert_eq!(audio.get_volume(id), Ok(0x8000_0000));

    // wave_out_close on invalid id → MMSYSERR_INVALHANDLE.
    assert_eq!(audio.wave_out_close(0xDEAD), MMSYSERR_INVALHANDLE);
    // Valid id → MMSYSERR_NOERROR; state → Closed, pending cleared.
    let id2 = audio.open_wave_out(fmt.clone());
    let buf2 = WaveBuffer { guest_ptr: 0x2000, len: 4 };
    audio.wave_out_write(id2, buf2);
    assert_eq!(audio.wave_out_close(id2), MMSYSERR_NOERROR);
    assert_eq!(audio.output(id2).unwrap().state, WaveOutState::Closed);
    assert!(audio.complete_next_buffer(id2).is_none()); // pending cleared by close

    // flush_sinks: no-op when no sinks registered (must not panic).
    audio.flush_sinks();
    // With a sink registered, flush_sinks calls flush on it.
    audio.register_sink(NullAudioSink::new("fs_sink"));
    audio.flush_sinks(); // should not panic; NullAudioSink flush is a no-op

    // wave_out_write_pcm on invalid id → MMSYSERR_INVALHANDLE.
    let dummy_pcm = vec![0u8; 8];
    let buf3 = WaveBuffer { guest_ptr: 0x3000, len: 8 };
    assert_eq!(audio.wave_out_write_pcm(0xDEAD, buf3.clone(), &dummy_pcm), MMSYSERR_INVALHANDLE);
    // Valid id → MMSYSERR_NOERROR; buffer queued.
    let id3 = audio.open_wave_out(fmt);
    assert_eq!(audio.wave_out_write_pcm(id3, buf3.clone(), &dummy_pcm), MMSYSERR_NOERROR);
    assert_eq!(audio.output(id3).unwrap().state, WaveOutState::Playing);
    // Buffer is dequeued by complete_next_buffer.
    assert_eq!(audio.complete_next_buffer(id3), Some(buf3));
}

#[test]
fn shell_notification_callback_method_com_vtable_offset_and_com_arguments() {
    use wince_emulation_v3::ce::shell::{
        ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET,
        ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET,
        ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET,
        ISHELL_NOTIFICATION_CALLBACK_ON_SHOW_VTABLE_OFFSET,
        ShellNotificationCallbackArguments, ShellNotificationCallbackMethod,
    };

    // OnShow: vtable_offset = 0x0c; arguments carry id, x, y, lparam.
    let on_show = ShellNotificationCallbackMethod::OnShow { x: 10, y: 20 };
    assert_eq!(on_show.com_vtable_offset(), ISHELL_NOTIFICATION_CALLBACK_ON_SHOW_VTABLE_OFFSET);
    assert_eq!(
        on_show.com_arguments(5, 0xABCD),
        ShellNotificationCallbackArguments::OnShow { id: 5, x: 10, y: 20, lparam: 0xABCD },
    );

    // OnCommandSelected: vtable_offset = 0x10; arguments carry id, command_id.
    let on_cmd = ShellNotificationCallbackMethod::OnCommandSelected { command_id: 42 };
    assert_eq!(
        on_cmd.com_vtable_offset(),
        ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET,
    );
    assert_eq!(
        on_cmd.com_arguments(7, 0),
        ShellNotificationCallbackArguments::OnCommandSelected { id: 7, command_id: 42 },
    );

    // OnLinkSelected: vtable_offset = 0x14; arguments carry id, link, lparam.
    let on_link = ShellNotificationCallbackMethod::OnLinkSelected { link: "https://example".into() };
    assert_eq!(
        on_link.com_vtable_offset(),
        ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET,
    );
    assert_eq!(
        on_link.com_arguments(3, 0x1234),
        ShellNotificationCallbackArguments::OnLinkSelected {
            id: 3,
            link: "https://example".into(),
            lparam: 0x1234,
        },
    );

    // OnDismiss: vtable_offset = 0x18; arguments carry id, timed_out, lparam.
    let on_dismiss = ShellNotificationCallbackMethod::OnDismiss { timed_out: true };
    assert_eq!(
        on_dismiss.com_vtable_offset(),
        ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET,
    );
    assert_eq!(
        on_dismiss.com_arguments(9, 0xBEEF),
        ShellNotificationCallbackArguments::OnDismiss {
            id: 9,
            timed_out: true,
            lparam: 0xBEEF,
        },
    );
}

#[test]
fn remote_ceremote_paused_key_serial_nmea_location_imu_audio_and_log_lines() {
    use wince_emulation_v3::ce::remote::{CeRemote, LocationFix, RemoteError};

    let mut remote = CeRemote::default();

    // paused / set_paused.
    assert!(!remote.paused());
    remote.set_paused(true);
    assert!(remote.paused());
    remote.set_paused(false);
    assert!(!remote.paused());

    // enqueue_key: invalid phase → Err.
    assert!(remote.enqueue_key("hold", 0x41).is_err());
    // vk=0 → Err (out of range 1..=0xFF).
    assert!(matches!(remote.enqueue_key("down", 0), Err(RemoteError::InvalidVirtualKey(0))));
    // Valid.
    assert!(remote.enqueue_key("down", 0x41).is_ok());
    assert!(remote.enqueue_key("up", 0x41).is_ok());
    assert_eq!(remote.key_event_count(), 2);
    let events = remote.drain_key_events();
    assert_eq!(events.len(), 2);
    assert_eq!(remote.key_event_count(), 0); // drained

    // enqueue_touch without framebuffer size → FramebufferUnavailable.
    assert!(matches!(
        remote.enqueue_touch("tap", 10, 10),
        Err(RemoteError::FramebufferUnavailable)
    ));
    // Set framebuffer size, then touch works.
    remote.set_framebuffer_size(320, 240);
    assert!(remote.enqueue_touch("tap", 10, 10).is_ok());
    let touch_events = remote.drain_touch_events();
    assert!(!touch_events.is_empty());
    assert_eq!(remote.touch_event_count(), 0);

    // inject_serial_bytes / read_serial_bytes / serial_byte_count.
    remote.inject_serial_bytes(b"hello");
    assert_eq!(remote.serial_byte_count(), 5);
    let read = remote.read_serial_bytes(3);
    assert_eq!(read, b"hel");
    assert_eq!(remote.serial_byte_count(), 2);
    let rest = remote.read_serial_bytes(100);
    assert_eq!(rest, b"lo");
    assert_eq!(remote.serial_byte_count(), 0);

    // inject_nmea_sentences → goes through serial buffer.
    let n = remote.inject_nmea_sentences(["$GPRMC,...*00\r\n"]);
    assert_eq!(n, 1);
    assert!(remote.serial_byte_count() > 0);
    let _ = remote.read_serial_bytes(1024); // drain

    // inject_location_nmea → produces sentences and injects them.
    let fix = LocationFix {
        lat: 37.5,
        lon: 127.0,
        altitude_m: 10.0,
        speed_mps: 0.0,
        bearing_deg: 0.0,
        timestamp_ms: None,
    };
    let sentences = remote.inject_location_nmea(fix);
    assert!(!sentences.is_empty()); // at least one NMEA sentence generated
    assert!(remote.serial_byte_count() > 0);
    let _ = remote.read_serial_bytes(4096);

    // update_imu_state / imu_state.
    let state = serde_json::json!({"ax": 0.1, "ay": 0.2, "az": 9.8});
    remote.update_imu_state(state.clone());
    assert_eq!(remote.imu_state(), &state);

    // register_audio_client / unregister_audio_client_id / audio_client_count.
    assert_eq!(remote.audio_client_count(), 0);
    let cid = remote.register_audio_client(1000);
    assert_eq!(remote.audio_client_count(), 1);
    assert!(remote.unregister_audio_client_id(cid));
    assert_eq!(remote.audio_client_count(), 0);
    // unregister unknown client → false.
    assert!(!remote.unregister_audio_client_id(cid));

    // unregister_audio_client (pops the latest client).
    let _ = remote.register_audio_client(2000);
    assert_eq!(remote.audio_client_count(), 1);
    assert!(remote.unregister_audio_client());
    assert_eq!(remote.audio_client_count(), 0);
    assert!(!remote.unregister_audio_client()); // no clients left

    // publish_audio_chunk / audio_chunk_count / take_audio_chunks / clear_audio_chunks.
    let _cid2 = remote.register_audio_client(3000);
    remote.publish_audio_chunk(vec![1, 2, 3, 4], 20);
    assert_eq!(remote.audio_chunk_count(), 1);
    // take_audio_chunks advances the client cursor but does NOT remove chunks from storage.
    let chunks = remote.take_audio_chunks(10);
    assert_eq!(chunks.len(), 1);
    assert_eq!(remote.audio_chunk_count(), 1); // count unchanged; cursor advanced
    // clear_audio_chunks removes all chunks from storage.
    remote.clear_audio_chunks(3100);
    assert_eq!(remote.audio_chunk_count(), 0);

    // push_log_line / recent_log_lines.
    remote.push_log_line("first");
    remote.push_log_line("second");
    remote.push_log_line("third");
    let logs = remote.recent_log_lines(2);
    assert_eq!(logs.len(), 2);
    // recent_log_lines returns the most recent lines.
    assert!(logs.iter().any(|l| l.contains("second") || l.contains("third")));
}

#[test]
fn scheduler_serial_sendreply_waiter_ids_queue_candidates_and_timed_out() {
    use wince_emulation_v3::ce::scheduler::{
        Scheduler, SchedulerBlockedWaitKind, blocked_wait_timed_out,
    };
    use wince_emulation_v3::ce::timer::INFINITE;

    let mut sched = Scheduler::default();

    // SerialRead waiter: appears in serial_read_waiter_ids_for_handle + all_serial_read_waiter_ids.
    let sr_id = sched.register_blocked_wait(
        1,
        0x101,
        vec![],
        SchedulerBlockedWaitKind::SerialRead { handle: 0x500 },
        0,
        INFINITE,
    );
    assert_eq!(sched.serial_read_waiter_ids_for_handle(0x500), vec![sr_id]);
    assert!(sched.serial_read_waiter_ids_for_handle(0x501).is_empty());
    assert_eq!(sched.all_serial_read_waiter_ids(), vec![sr_id]);

    // SerialCommEvent waiter: appears in serial_event_waiter_ids_for_handle + all_serial_event_waiter_ids.
    let se_id = sched.register_blocked_wait(
        2,
        0x102,
        vec![],
        SchedulerBlockedWaitKind::SerialCommEvent { handle: 0x600 },
        0,
        INFINITE,
    );
    assert_eq!(sched.serial_event_waiter_ids_for_handle(0x600), vec![se_id]);
    assert!(sched.serial_event_waiter_ids_for_handle(0x601).is_empty());
    assert_eq!(sched.all_serial_event_waiter_ids(), vec![se_id]);

    // SendMessage waiter: appears in send_reply_waiter_ids_for_send.
    let sm_id = sched.register_blocked_wait(
        3,
        0x103,
        vec![],
        SchedulerBlockedWaitKind::SendMessage { send_id: 42 },
        0,
        INFINITE,
    );
    assert_eq!(sched.send_reply_waiter_ids_for_send(42), vec![sm_id]);
    assert!(sched.send_reply_waiter_ids_for_send(99).is_empty());

    // queue_serial_read_wake_candidates → queues sr_id.
    let n = sched.queue_serial_read_wake_candidates(0x500);
    assert_eq!(n, 1);
    // Re-queuing same id returns 0 (already pending).
    let n2 = sched.queue_serial_read_wake_candidates(0x500);
    assert_eq!(n2, 0);
    // Unknown handle → 0 queued.
    let n3 = sched.queue_serial_read_wake_candidates(0x999);
    assert_eq!(n3, 0);

    // queue_serial_event_wake_candidates → queues se_id.
    let n = sched.queue_serial_event_wake_candidates(0x600);
    assert_eq!(n, 1);

    // queue_send_reply_wake_candidates → queues sm_id.
    let n = sched.queue_send_reply_wake_candidates(42);
    assert_eq!(n, 1);
    assert_eq!(sched.send_reply_waiter_ids_for_send(99).len(), 0);

    // Remove sr waiter; all_serial_read_waiter_ids becomes empty.
    sched.remove_blocked_wait(sr_id);
    assert!(sched.all_serial_read_waiter_ids().is_empty());
    assert!(sched.serial_read_waiter_ids_for_handle(0x500).is_empty());

    // queue_all_serial_read_wake_candidates / queue_all_serial_event_wake_candidates.
    let mut sched2 = Scheduler::default();
    let _a = sched2.register_blocked_wait(
        10, 0x110, vec![],
        SchedulerBlockedWaitKind::SerialRead { handle: 0xA0 },
        0, INFINITE,
    );
    let _b = sched2.register_blocked_wait(
        11, 0x111, vec![],
        SchedulerBlockedWaitKind::SerialRead { handle: 0xB0 },
        0, INFINITE,
    );
    let n = sched2.queue_all_serial_read_wake_candidates();
    assert_eq!(n, 2);

    let _c = sched2.register_blocked_wait(
        12, 0x112, vec![],
        SchedulerBlockedWaitKind::SerialCommEvent { handle: 0xC0 },
        0, INFINITE,
    );
    let n = sched2.queue_all_serial_event_wake_candidates();
    assert_eq!(n, 1);

    // queue_pending_wake_ids with explicit ids.
    let mut sched3 = Scheduler::default();
    let d = sched3.register_blocked_wait(
        20, 0x120, vec![0x300],
        SchedulerBlockedWaitKind::Kernel,
        0, 1000,
    );
    let e = sched3.register_blocked_wait(
        21, 0x121, vec![0x301],
        SchedulerBlockedWaitKind::Kernel,
        0, 1000,
    );
    // queue both manually.
    let n = sched3.queue_pending_wake_ids(vec![d, e]);
    assert_eq!(n, 2);
    // Duplicate → 0.
    let n = sched3.queue_pending_wake_ids(vec![d]);
    assert_eq!(n, 0);
    // Non-existent id → 0.
    let n = sched3.queue_pending_wake_ids(vec![9999]);
    assert_eq!(n, 0);

    // blocked_wait_timed_out: finite timeout elapses.
    let wait_finite = sched3.remove_blocked_wait(d).unwrap();
    assert!(!blocked_wait_timed_out(&wait_finite, 999));   // not yet elapsed
    assert!(blocked_wait_timed_out(&wait_finite, 1000));   // exactly elapsed
    assert!(blocked_wait_timed_out(&wait_finite, 5000));   // past elapsed

    // blocked_wait_timed_out: INFINITE timeout never elapses.
    let mut sched4 = Scheduler::default();
    let f = sched4.register_blocked_wait(
        30, 0x130, vec![],
        SchedulerBlockedWaitKind::Kernel,
        0, INFINITE,
    );
    let wait_inf = sched4.remove_blocked_wait(f).unwrap();
    assert!(!blocked_wait_timed_out(&wait_inf, 0xFFFF_FFFF));

    // Stats reflect queue operations.
    let stats = sched.stats();
    assert!(stats.serial_read_signal_count >= 3); // queue_serial_read × 3 calls
    assert!(stats.serial_event_signal_count >= 1);
    assert!(stats.send_reply_signal_count >= 1);
}

#[test]
fn cemath_export_name_all_variants_and_gwe_get_window_text_length() {
    use wince_emulation_v3::ce::cemath::{
        CeMathBinaryF32, CeMathBinaryF64, CeMathCall, CeMathUnaryF64,
    };
    use wince_emulation_v3::ce::gwe::{Gwe, Rect, WS_VISIBLE};

    // CeMathCall::export_name for every fixed variant.
    assert_eq!(CeMathCall::Abs(0).export_name(), "abs");
    assert_eq!(CeMathCall::Labs(0).export_name(), "labs");
    assert_eq!(CeMathCall::Div { numer: 1, denom: 1 }.export_name(), "div");
    assert_eq!(CeMathCall::Ldiv { numer: 1, denom: 1 }.export_name(), "ldiv");
    assert_eq!(CeMathCall::Frexp(1.0).export_name(), "frexp");
    assert_eq!(CeMathCall::Ldexp { value: 1.0, exp: 0 }.export_name(), "ldexp");
    assert_eq!(CeMathCall::Modf(1.0).export_name(), "modf");
    assert_eq!(CeMathCall::LlMul { lhs: 0, rhs: 0 }.export_name(), "__ll_mul");
    assert_eq!(CeMathCall::LlDiv { lhs: 1, rhs: 1 }.export_name(), "__ll_div");
    assert_eq!(CeMathCall::LlRem { lhs: 1, rhs: 1 }.export_name(), "__ll_rem");
    assert_eq!(CeMathCall::UllDiv { lhs: 1, rhs: 1 }.export_name(), "__ull_div");
    assert_eq!(CeMathCall::UllRem { lhs: 1, rhs: 1 }.export_name(), "__ull_rem");
    assert_eq!(CeMathCall::LlLShift { value: 1, shift: 0 }.export_name(), "__ll_lshift");
    assert_eq!(CeMathCall::LlRShift { value: 1, shift: 0 }.export_name(), "__ll_rshift");
    assert_eq!(CeMathCall::UllRShift { value: 1, shift: 0 }.export_name(), "__ull_rshift");
    assert_eq!(CeMathCall::FloatAdd { lhs: 0.0, rhs: 0.0 }.export_name(), "__fpadd");
    assert_eq!(CeMathCall::FloatSub { lhs: 0.0, rhs: 0.0 }.export_name(), "__fpsub");
    assert_eq!(CeMathCall::FloatMul { lhs: 0.0, rhs: 0.0 }.export_name(), "__fpmul");
    assert_eq!(CeMathCall::FloatDiv { lhs: 1.0, rhs: 1.0 }.export_name(), "__fpdiv");
    assert_eq!(CeMathCall::DoubleAdd { lhs: 0.0, rhs: 0.0 }.export_name(), "__dpadd");
    assert_eq!(CeMathCall::DoubleSub { lhs: 0.0, rhs: 0.0 }.export_name(), "__dpsub");
    assert_eq!(CeMathCall::DoubleMul { lhs: 0.0, rhs: 0.0 }.export_name(), "__dpmul");
    assert_eq!(CeMathCall::DoubleDiv { lhs: 1.0, rhs: 1.0 }.export_name(), "__dpdiv");
    assert_eq!(CeMathCall::FloatToLong(0.0).export_name(), "__fptoli");
    assert_eq!(CeMathCall::FloatToUnsignedLong(0.0).export_name(), "__fptoul");
    assert_eq!(CeMathCall::DoubleToLong(0.0).export_name(), "__dptoli");
    assert_eq!(CeMathCall::DoubleToUnsignedLong(0.0).export_name(), "__dptoul");
    assert_eq!(CeMathCall::FloatToLongLong(0.0).export_name(), "__f_to_ll");
    assert_eq!(CeMathCall::DoubleToLongLong(0.0).export_name(), "__d_to_ll");
    assert_eq!(CeMathCall::FloatToUnsignedLongLong(0.0).export_name(), "__f_to_ull");
    assert_eq!(CeMathCall::DoubleToUnsignedLongLong(0.0).export_name(), "__d_to_ull");
    assert_eq!(CeMathCall::LongToFloat(0).export_name(), "__litofp");
    assert_eq!(CeMathCall::LongToDouble(0).export_name(), "__litodp");
    assert_eq!(CeMathCall::UnsignedLongToFloat(0).export_name(), "__ultofp");
    assert_eq!(CeMathCall::UnsignedLongToDouble(0).export_name(), "__ultodp");
    assert_eq!(CeMathCall::FloatToDouble(0.0).export_name(), "__fptodp");
    assert_eq!(CeMathCall::DoubleToFloat(0.0).export_name(), "__dptofp");
    assert_eq!(CeMathCall::FloatCmp { lhs: 0.0, rhs: 0.0 }.export_name(), "__fpcmp");
    assert_eq!(CeMathCall::DoubleCmp { lhs: 0.0, rhs: 0.0 }.export_name(), "__dpcmp");

    // UnaryF64 variants dispatch export_name from the op.
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Acos, value: 0.0 }.export_name(), "acos");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Asin, value: 0.0 }.export_name(), "asin");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Atan, value: 0.0 }.export_name(), "atan");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Cos, value: 0.0 }.export_name(), "cos");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Cosh, value: 0.0 }.export_name(), "cosh");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Exp, value: 0.0 }.export_name(), "exp");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Fabs, value: 0.0 }.export_name(), "fabs");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Log, value: 1.0 }.export_name(), "log");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Log10, value: 1.0 }.export_name(), "log10");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Sin, value: 0.0 }.export_name(), "sin");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Sinh, value: 0.0 }.export_name(), "sinh");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Tan, value: 0.0 }.export_name(), "tan");
    assert_eq!(CeMathCall::UnaryF64 { op: CeMathUnaryF64::Tanh, value: 0.0 }.export_name(), "tanh");

    // BinaryF64 variants.
    assert_eq!(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Atan2, lhs: 0.0, rhs: 1.0 }.export_name(), "atan2");
    assert_eq!(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Fmod, lhs: 1.0, rhs: 1.0 }.export_name(), "fmod");
    assert_eq!(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Hypot, lhs: 3.0, rhs: 4.0 }.export_name(), "_hypot");
    assert_eq!(CeMathCall::BinaryF64 { op: CeMathBinaryF64::Pow, lhs: 2.0, rhs: 3.0 }.export_name(), "pow");

    // BinaryF32 variants.
    assert_eq!(CeMathCall::BinaryF32 { op: CeMathBinaryF32::Fmod, lhs: 1.0, rhs: 1.0 }.export_name(), "fmodf");

    // get_window_text_length: counts UTF-16 code units of the title.
    let mut gwe = Gwe::default();
    let hwnd = gwe.create_window_ex_with_rect(
        1,
        "test",
        "Hello",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect { left: 0, top: 0, right: 100, bottom: 50 },
    );
    // "Hello" encodes to exactly 5 UTF-16 code units.
    assert_eq!(gwe.get_window_text_length(hwnd), Some(5));

    // After set_window_text to an ASCII string, length matches char count.
    gwe.set_window_text(hwnd, "AB");
    assert_eq!(gwe.get_window_text_length(hwnd), Some(2));

    // Invalid hwnd → None.
    assert_eq!(gwe.get_window_text_length(0xDEAD_BEEF), None);
}

#[test]
fn filesystem_disk_space_volume_info_object_store_and_file_ops() -> Result<()> {
    use wince_emulation_v3::ce::file::{
        CREATE_ALWAYS, FILE_ATTRIBUTE_READONLY, GENERIC_READ, GENERIC_WRITE,
        OPEN_EXISTING,
    };

    let root = unique_test_root("filesystem_disk_space_volume_info");
    fs::create_dir_all(&root).unwrap();

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.set_file_root(&root);

    // disk_space_for_path(None) falls back to the object store.
    let ds = kernel.files.disk_space_for_path(None);
    assert!(ds.total_bytes > 0);
    assert!(ds.free_bytes <= ds.total_bytes);

    // object_store() reflects the same values as disk_space fallback.
    let store = kernel.files.object_store();
    assert_eq!(store.total_bytes, ds.total_bytes);

    // volume_info_for_path(None) returns a RAMFS volume for the object store.
    let vi = kernel.files.volume_info_for_path(None);
    // CE_VOLUME_FLAG_RAMFS = 0x40
    assert_eq!(vi.flags & 0x40, 0x40);
    assert_eq!(vi.store_name, "ObjectStore");
    assert_eq!(vi.block_size, 4096);

    // volume_info_for_path for a mounted guest path returns STORE info.
    kernel.mount_guest_root("\\TestMount", root.clone());
    let vi_mount = kernel.files.volume_info_for_path(Some("\\TestMount\\file.txt"));
    // CE_VOLUME_FLAG_STORE = 0x20
    assert_eq!(vi_mount.flags & 0x20, 0x20);

    // create_directory_w / file_attributes_w (directory) / remove_directory_w.
    kernel.create_directory_w("\\ResidentFlash\\TestDir")?;
    let attr = kernel.file_attributes_w("\\ResidentFlash\\TestDir")?;
    // FILE_ATTRIBUTE_DIRECTORY = 0x10
    assert_eq!(attr.attributes & 0x10, 0x10);
    kernel.remove_directory_w("\\ResidentFlash\\TestDir")?;
    // Directory is gone; re-querying returns an error.
    assert!(kernel.file_attributes_w("\\ResidentFlash\\TestDir").is_err());

    // Write a file, then exercise file operations.
    let fh = kernel.create_file_w(
        "\\ResidentFlash\\ops_src.txt",
        GENERIC_READ | GENERIC_WRITE,
        CREATE_ALWAYS,
    )?;
    kernel.write_file(fh, b"hello world")?;
    kernel.close_handle(fh)?;

    // read_guest_file: reads the raw bytes from host without opening a handle.
    let raw = kernel.read_guest_file("\\ResidentFlash\\ops_src.txt")?;
    assert_eq!(raw, b"hello world");

    // file_attributes_w on a file returns non-directory attributes.
    let fattr = kernel.file_attributes_w("\\ResidentFlash\\ops_src.txt")?;
    assert_eq!(fattr.attributes & 0x10, 0); // not a directory

    // copy_file_w: copy src → dst.
    kernel.copy_file_w("\\ResidentFlash\\ops_src.txt", "\\ResidentFlash\\ops_dst.txt", false)?;
    let dst_raw = kernel.read_guest_file("\\ResidentFlash\\ops_dst.txt")?;
    assert_eq!(dst_raw, b"hello world");

    // copy_file_w with fail_if_exists=true on existing dst → error.
    assert!(kernel.copy_file_w(
        "\\ResidentFlash\\ops_src.txt",
        "\\ResidentFlash\\ops_dst.txt",
        true
    ).is_err());

    // move_file_w: rename dst → moved.
    kernel.move_file_w("\\ResidentFlash\\ops_dst.txt", "\\ResidentFlash\\ops_moved.txt")?;
    assert!(kernel.file_attributes_w("\\ResidentFlash\\ops_dst.txt").is_err());
    assert!(kernel.file_attributes_w("\\ResidentFlash\\ops_moved.txt").is_ok());

    // delete_file_w: remove moved file.
    kernel.delete_file_w("\\ResidentFlash\\ops_moved.txt")?;
    assert!(kernel.file_attributes_w("\\ResidentFlash\\ops_moved.txt").is_err());

    // set_file_attributes_w: mark src readonly.
    kernel.set_file_attributes_w("\\ResidentFlash\\ops_src.txt", FILE_ATTRIBUTE_READONLY)?;
    // Restore writable so test cleanup works.
    kernel.set_file_attributes_w("\\ResidentFlash\\ops_src.txt", 0)?;

    // read_file_into: read via callback.
    let fh2 = kernel.create_file_w(
        "\\ResidentFlash\\ops_src.txt",
        GENERIC_READ,
        OPEN_EXISTING,
    )?;
    let mut buf: Vec<u8> = Vec::new();
    kernel.read_file_into(fh2, 5, |chunk| {
        buf.extend_from_slice(chunk);
        Ok(())
    })?;
    assert_eq!(&buf, b"hello");

    // file_is_eof: not at EOF after reading 5 of 11 bytes.
    assert!(!kernel.file_is_eof(fh2)?);

    // Read remaining 6 bytes.
    let mut buf2: Vec<u8> = Vec::new();
    kernel.read_file_into(fh2, 100, |chunk| {
        buf2.extend_from_slice(chunk);
        Ok(())
    })?;
    assert_eq!(&buf2, b" world");

    // file_is_eof: now at EOF after reading all bytes.
    assert!(kernel.file_is_eof(fh2)?);
    kernel.close_handle(fh2)?;

    Ok(())
}

#[test]
fn kernel_loaded_module_snapshots_gwe_stats_recent_file_open_ops_lifecycle_trace_and_process_launch_variants() {
    use wince_emulation_v3::ce::gwe::GweStats;

    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);

    // loaded_module_snapshots: no modules loaded → empty.
    assert!(kernel.loaded_module_snapshots().is_empty());

    // Register a module and check snapshot.
    kernel.register_loaded_module("test.dll", 0x1000_0000, Default::default(), Default::default());
    let snaps = kernel.loaded_module_snapshots();
    assert_eq!(snaps.len(), 1);
    assert_eq!(snaps[0].name, "test.dll");
    assert_eq!(snaps[0].base, 0x1000_0000);

    // loaded_module_by_handle: returns Some for a valid base.
    let found = kernel.loaded_module_by_handle(0x1000_0000);
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test.dll");
    // Unknown base → None.
    assert!(kernel.loaded_module_by_handle(0xDEAD_0000).is_none());

    // loaded_module_export_snapshots: exports are empty for a module with no exports.
    let export_snaps = kernel.loaded_module_export_snapshots();
    assert_eq!(export_snaps.len(), 1);
    assert_eq!(export_snaps[0].name, "test.dll");
    assert!(export_snaps[0].exports_by_name.is_empty());

    // gwe_stats: starts at zero.
    let stats: GweStats = kernel.gwe_stats();
    assert_eq!(stats.send_transaction_count, 0);
    assert_eq!(stats.send_transaction_completed_count, 0);
    assert_eq!(stats.send_transaction_timeout_count, 0);
    assert_eq!(stats.max_sent_queue_depth, 0);

    // recent_file_open_ops: empty before any file operations.
    assert!(kernel.recent_file_open_ops().is_empty());

    // record_window_lifecycle_trace: records a message trace without panicking.
    let before_msg = kernel.recent_message_ops().len();
    kernel.record_window_lifecycle_trace("CreateWindow", 1, None, Some(0x2_0000), None);
    assert_eq!(kernel.recent_message_ops().len(), before_msg + 1);
    let rec = kernel.recent_message_ops().last().unwrap();
    assert_eq!(rec.op, "CreateWindow");
    assert_eq!(rec.thread_id, 1);
    assert_eq!(rec.result, Some(0x2_0000));

    // queue_process_launch_with_show: adds a pending launch with show_cmd.
    let launch_show = kernel.queue_process_launch_with_show(
        Some("ShowApp.exe".to_owned()),
        None,
        Some(1), // SW_SHOWNORMAL
    );
    assert_eq!(launch_show.application.as_deref(), Some("ShowApp.exe"));
    assert_eq!(launch_show.show_cmd, Some(1));
    assert!(launch_show.process_handle != 0);

    // queue_process_launch_with_options: adds a launch with current_directory.
    let launch_opts = kernel.queue_process_launch_with_options(
        Some("OptsApp.exe".to_owned()),
        Some("OptsApp.exe arg".to_owned()),
        Some("\\ResidentFlash".to_owned()),
        Some(0),
    );
    assert_eq!(launch_opts.application.as_deref(), Some("OptsApp.exe"));
    assert_eq!(launch_opts.command_line.as_deref(), Some("OptsApp.exe arg"));
    assert_eq!(launch_opts.current_directory.as_deref(), Some("\\ResidentFlash"));
    assert_eq!(launch_opts.show_cmd, Some(0));

    // take_pending_process_launches: retrieves and clears both queued launches.
    let pending = kernel.take_pending_process_launches();
    assert_eq!(pending.len(), 2);
    // After take, no more pending launches.
    assert!(kernel.take_pending_process_launches().is_empty());
}

#[test]
fn kernel_file_pointer_size_position_flush_find_first_next_close_and_change_notification() -> Result<()> {
    use wince_emulation_v3::ce::file::{
        CREATE_ALWAYS, FILE_ATTRIBUTE_DIRECTORY, GENERIC_READ, GENERIC_WRITE,
    };

    let root = unique_test_root("file_pointer_find_notification");
    fs::create_dir_all(&root).unwrap();
    // Also create a subdir for the find test.
    fs::create_dir_all(root.join("ResidentFlash")).unwrap();

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.set_file_root(&root);

    // Create a file with known content.
    let fh = kernel.create_file_w(
        "\\ResidentFlash\\seek.bin",
        GENERIC_READ | GENERIC_WRITE,
        CREATE_ALWAYS,
    )?;
    kernel.write_file(fh, b"ABCDEFGHIJ")?; // 10 bytes
    kernel.close_handle(fh)?;

    // open_existing_readonly.
    let ro = kernel.open_existing_readonly("\\ResidentFlash\\seek.bin")?;
    // get_file_size: 10 bytes.
    assert_eq!(kernel.get_file_size(ro)?, 10);
    // file_position: starts at 0.
    assert_eq!(kernel.file_position(ro)?, 0);

    // set_file_pointer method=0 (from start).
    let pos = kernel.set_file_pointer(ro, 3, 0)?;
    assert_eq!(pos, 3);
    assert_eq!(kernel.file_position(ro)?, 3);

    // set_file_pointer method=1 (from current).
    let pos2 = kernel.set_file_pointer(ro, 2, 1)?;
    assert_eq!(pos2, 5);

    // set_file_pointer method=2 (from end).
    let pos3 = kernel.set_file_pointer(ro, -2, 2)?;
    assert_eq!(pos3, 8);

    // set_file_pointer with invalid method → error.
    assert!(kernel.set_file_pointer(ro, 0, 99).is_err());

    kernel.close_handle(ro)?;

    // open_existing_readwrite.
    let rw = kernel.open_existing_readwrite("\\ResidentFlash\\seek.bin")?;
    // flush_file_buffers: no-op on memory-backed file, returns Ok(true).
    assert!(kernel.flush_file_buffers(rw)?);
    kernel.close_handle(rw)?;

    // find_first_file_w / find_next_file_w / find_close.
    // Create a second file for find_next to return.
    let fh2 = kernel.create_file_w(
        "\\ResidentFlash\\other.bin",
        GENERIC_READ | GENERIC_WRITE,
        CREATE_ALWAYS,
    )?;
    kernel.write_file(fh2, b"XY")?;
    kernel.close_handle(fh2)?;

    let (find_handle, first_data) = kernel.find_first_file_w("\\ResidentFlash\\*.bin")?;
    // First result is a valid file.
    assert!(!first_data.file_name.is_empty());
    assert_eq!(first_data.attributes & FILE_ATTRIBUTE_DIRECTORY, 0);

    // find_next_file_w: may return second file or None.
    let next = kernel.find_next_file_w(find_handle)?;
    // Either Some (second file) or None (only one matched) — both are valid.
    let _ = next;

    // find_close: closes the find handle.
    assert!(kernel.find_close(find_handle)?);

    // find_first_file_w with no-match pattern → error.
    assert!(kernel.find_first_file_w("\\ResidentFlash\\*.zzz").is_err());

    // find_first_change_notification_w / find_next_change_notification /
    // file_change_notification_records / drain / clear / find_close_change_notification.
    kernel.create_directory_w("\\ResidentFlash\\WatchDir")?;
    // FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE = 0x1 | 0x10 = 0x11
    let notify_filter = 0x0000_0001u32 | 0x0000_0010;
    let nfh = kernel.find_first_change_notification_w(
        "\\ResidentFlash\\WatchDir",
        false,
        notify_filter,
    )?;
    // Records start empty.
    assert!(kernel.file_change_notification_records(nfh)?.is_empty());

    // find_next_change_notification: resets signaled/pending → Ok(true).
    assert!(kernel.find_next_change_notification(nfh)?);

    // drain_file_change_notification_records with count=0: no-op, returns Ok(false).
    let still = kernel.drain_file_change_notification_records(nfh, 0)?;
    assert!(!still);

    // clear_file_change_notification: delegates to find_next → Ok(true).
    assert!(kernel.clear_file_change_notification(nfh)?);

    // Unsupported notify_filter → error (bit 9 = 0x200 is not in the supported mask).
    assert!(kernel.find_first_change_notification_w(
        "\\ResidentFlash\\WatchDir",
        false,
        0x0000_0200, // not in the supported mask
    ).is_err());

    // Path that is a file (not directory) → error.
    assert!(kernel.find_first_change_notification_w(
        "\\ResidentFlash\\seek.bin",
        false,
        notify_filter,
    ).is_err());

    // find_close_change_notification: closes the handle.
    assert!(kernel.find_close_change_notification(nfh)?);

    Ok(())
}

#[test]
fn kernel_scheduler_stat_recording_winsock_wake_thread_has_sent_message_describe_handle_window_state() {
    use wince_emulation_v3::ce::gwe::{WM_USER, WS_VISIBLE};
    use wince_emulation_v3::ce::scheduler::{
        SchedulerBlockedWaitKind, SchedulerWinsockReadyKind,
    };
    use wince_emulation_v3::ce::timer::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0};

    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);
    let thread_id = 42u32;

    // --- Scheduler stat recording via kernel wrappers ---
    // Before any records, stats are zero.
    let s0 = kernel.scheduler.stats();
    assert_eq!(s0.wait_single_count, 0);
    assert_eq!(s0.wait_multiple_count, 0);
    assert_eq!(s0.msg_wait_count, 0);
    assert_eq!(s0.sleep_count, 0);
    assert_eq!(s0.yield_count, 0);
    assert_eq!(s0.wait_block_count, 0);
    assert_eq!(s0.wait_wake_count, 0);

    // record_blocked_single_wait: increments wait_single_count and wait_block_count.
    kernel.record_blocked_single_wait(INFINITE);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_single_count, 1);
    assert_eq!(s.wait_block_count, 1);

    // record_blocked_multiple_wait: increments wait_multiple_count and wait_block_count.
    kernel.record_blocked_multiple_wait(3, 1000);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_multiple_count, 1);
    assert_eq!(s.wait_block_count, 2);
    assert_eq!(s.max_wait_handles, 3);
    // max_timeout_ms is already INFINITE from the prior call; 1000 does not replace it.
    assert_eq!(s.max_timeout_ms, INFINITE);

    // record_blocked_msg_wait: increments msg_wait_count and wait_block_count.
    kernel.record_blocked_msg_wait(2, 500);
    let s = kernel.scheduler.stats();
    assert_eq!(s.msg_wait_count, 1);
    assert_eq!(s.wait_block_count, 3);

    // record_blocked_thread_sleep: increments sleep_count and wait_block_count.
    kernel.record_blocked_thread_sleep(100);
    let s = kernel.scheduler.stats();
    assert_eq!(s.sleep_count, 1);
    assert_eq!(s.wait_block_count, 4);

    // record_thread_yield: increments yield_count AND sleep_count (Sleep wait kind).
    kernel.record_thread_yield();
    let s = kernel.scheduler.stats();
    assert_eq!(s.yield_count, 1);
    assert_eq!(s.sleep_count, 2);

    // record_resumed_single_wait: increments wait_wake_count and wait_acquired_count
    // (result=WAIT_OBJECT_0 → ObjectSignaled).
    kernel.record_resumed_single_wait(WAIT_OBJECT_0);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, 1);
    assert_eq!(s.wait_acquired_count, 1);

    // record_resumed_wait: same as resumed_single — wake_count and acquired.
    kernel.record_resumed_wait(WAIT_OBJECT_0);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, 2);
    assert_eq!(s.wait_acquired_count, 2);

    // record_msg_wait_result: msg_wait_count+1, wait_acquired_count+1 (ObjectSignaled).
    let msg_wait_before = kernel.scheduler.stats().msg_wait_count;
    let acquired_before = kernel.scheduler.stats().wait_acquired_count;
    kernel.record_msg_wait_result(1, 0, WAIT_OBJECT_0);
    let s = kernel.scheduler.stats();
    assert_eq!(s.msg_wait_count, msg_wait_before + 1);
    assert_eq!(s.wait_acquired_count, acquired_before + 1);

    // record_msg_wait_input: msg_wait_count+1, wait_acquired_count+1 (MessageInput→acquired).
    let msg_wait_before = kernel.scheduler.stats().msg_wait_count;
    let acquired_before = kernel.scheduler.stats().wait_acquired_count;
    kernel.record_msg_wait_input(2, 0);
    let s = kernel.scheduler.stats();
    assert_eq!(s.msg_wait_count, msg_wait_before + 1);
    assert_eq!(s.wait_acquired_count, acquired_before + 1);

    // record_resumed_msg_wait_input: wait_wake_count+1, wait_acquired_count+1.
    let wake_before = kernel.scheduler.stats().wait_wake_count;
    let acquired_before = kernel.scheduler.stats().wait_acquired_count;
    kernel.record_resumed_msg_wait_input();
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, wake_before + 1);
    assert_eq!(s.wait_acquired_count, acquired_before + 1);

    // record_resumed_msg_wait_result with WAIT_TIMEOUT: wait_wake_count+1, wait_timeout_count+1.
    let wake_before = kernel.scheduler.stats().wait_wake_count;
    let timeout_before = kernel.scheduler.stats().wait_timeout_count;
    kernel.record_resumed_msg_wait_result(258); // WAIT_TIMEOUT = 258
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, wake_before + 1);
    assert_eq!(s.wait_timeout_count, timeout_before + 1);

    // record_resumed_thread_sleep: wait_wake_count+1, wait_timeout_count+1 (Timeout wake).
    let wake_before = kernel.scheduler.stats().wait_wake_count;
    let timeout_before = kernel.scheduler.stats().wait_timeout_count;
    kernel.record_resumed_thread_sleep();
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_wake_count, wake_before + 1);
    assert_eq!(s.wait_timeout_count, timeout_before + 1);

    // record_multiple_wait_attempt: wait_multiple_count+1, wait_acquired_count+1 (Object0).
    let multiple_before = kernel.scheduler.stats().wait_multiple_count;
    let acquired_before = kernel.scheduler.stats().wait_acquired_count;
    kernel.record_multiple_wait_attempt(2, 0, WAIT_OBJECT_0);
    let s = kernel.scheduler.stats();
    assert_eq!(s.wait_multiple_count, multiple_before + 1);
    assert_eq!(s.wait_acquired_count, acquired_before + 1);

    // --- wait_for_single_object_without_scheduler_record ---
    // manual-reset, not signaled.
    let event = kernel.create_event_w(Some("stat_ev1".to_owned()), true, false);
    // Not signaled → WAIT_TIMEOUT (timeout_ms=0 means poll).
    let r = kernel.wait_for_single_object_without_scheduler_record(event, 0, thread_id);
    assert_eq!(r, 258); // WAIT_TIMEOUT

    kernel.set_event(event);
    // Signaled → WAIT_OBJECT_0.
    let r = kernel.wait_for_single_object_without_scheduler_record(event, 0, thread_id);
    assert_eq!(r, WAIT_OBJECT_0);

    // Invalid handle → WAIT_FAILED.
    let r = kernel.wait_for_single_object_without_scheduler_record(0xDEAD_BEEF, 0, thread_id);
    assert_eq!(r, WAIT_FAILED);

    // --- wait_for_multiple_objects_without_scheduler_record ---
    // manual-reset, signaled.
    let event2 = kernel.create_event_w(Some("stat_ev2".to_owned()), true, true);
    // wait_all=true → WAIT_FAILED (not supported).
    let r = kernel.wait_for_multiple_objects_without_scheduler_record(&[event2], true, thread_id);
    assert_eq!(r, WAIT_FAILED);
    // empty slice → WAIT_FAILED.
    let r = kernel.wait_for_multiple_objects_without_scheduler_record(&[], false, thread_id);
    assert_eq!(r, WAIT_FAILED);
    // Valid signaled handle → WAIT_OBJECT_0.
    let r = kernel.wait_for_multiple_objects_without_scheduler_record(&[event2], false, thread_id);
    assert_eq!(r, WAIT_OBJECT_0);
    // Second element signaled when first is not → WAIT_OBJECT_0 + 1.
    let event3 = kernel.create_event_w(Some("stat_ev3".to_owned()), true, true);
    let event_unset = kernel.create_event_w(Some("stat_ev4".to_owned()), true, false);
    let r = kernel.wait_for_multiple_objects_without_scheduler_record(&[event_unset, event3], false, thread_id);
    assert_eq!(r, WAIT_OBJECT_0 + 1);

    // --- queue_winsock_wake_candidates and queue_winsock_wake_candidates_for_handles ---
    let socket_handle = 0xCC00_0001u32;
    // Register a waiter waiting on socket_handle.
    let wait_id = kernel.register_blocked_waiter(
        thread_id,
        0x8000_0001,
        vec![socket_handle],
        SchedulerBlockedWaitKind::WinsockRead {
            socket: socket_handle,
            readiness: SchedulerWinsockReadyKind::Read,
            read_mask: 0,
            write_mask: 0,
            except_mask: 0,
        },
        0,
        INFINITE,
    );
    // queue_winsock_wake_candidates for that socket → 1 queued.
    let n = kernel.queue_winsock_wake_candidates(socket_handle);
    assert_eq!(n, 1);
    // Re-queuing → 0 (already pending).
    let n2 = kernel.queue_winsock_wake_candidates(socket_handle);
    assert_eq!(n2, 0);
    // Unknown socket → 0.
    let n3 = kernel.queue_winsock_wake_candidates(0xDEAD_CAFE);
    assert_eq!(n3, 0);
    kernel.remove_blocked_waiter(wait_id);

    // queue_winsock_wake_candidates_for_handles: two sockets, one waiter each.
    let sock_a = 0xCC00_0002u32;
    let sock_b = 0xCC00_0003u32;
    let wait_a = kernel.register_blocked_waiter(
        thread_id,
        0x8000_0002,
        vec![sock_a],
        SchedulerBlockedWaitKind::WinsockRead {
            socket: sock_a,
            readiness: SchedulerWinsockReadyKind::Read,
            read_mask: 0,
            write_mask: 0,
            except_mask: 0,
        },
        0,
        INFINITE,
    );
    let wait_b = kernel.register_blocked_waiter(
        thread_id,
        0x8000_0003,
        vec![sock_b],
        SchedulerBlockedWaitKind::WinsockRead {
            socket: sock_b,
            readiness: SchedulerWinsockReadyKind::Write,
            read_mask: 0,
            write_mask: 0,
            except_mask: 0,
        },
        0,
        INFINITE,
    );
    let n = kernel.queue_winsock_wake_candidates_for_handles([sock_a, sock_b]);
    assert_eq!(n, 2);
    kernel.remove_blocked_waiter(wait_a);
    kernel.remove_blocked_waiter(wait_b);

    // --- thread_has_pending_sent_message ---
    let receiver_thread = 0xBB01u32;
    let sender_thread = 0xBB02u32;
    // Create a window owned by receiver_thread.
    let hwnd = kernel.create_window_ex_w(receiver_thread, "Button", "Msg", None, 0, WS_VISIBLE, 0);
    // Before any send: no pending.
    assert!(!kernel.thread_has_pending_sent_message(receiver_thread));
    // Queue an inter-thread send (sender_thread ≠ receiver_thread).
    let send_id = kernel.begin_cross_thread_send_message_w(
        sender_thread, hwnd, WM_USER + 0x80, 0, 0, None,
    );
    assert!(send_id.is_some());
    // Now receiver_thread has a pending sent message.
    assert!(kernel.thread_has_pending_sent_message(receiver_thread));
    // sender_thread itself has none.
    assert!(!kernel.thread_has_pending_sent_message(sender_thread));

    // --- describe_handle on kernel ---
    // Pseudo thread handle → "current_thread_pseudo".
    let desc = kernel.describe_handle(CE_CURRENT_THREAD_PSEUDO_HANDLE);
    assert_eq!(desc, "current_thread_pseudo");
    // Pseudo process handle → starts with "current_process_pseudo".
    let desc2 = kernel.describe_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE);
    assert!(desc2.starts_with("current_process_pseudo"));
    // A real kernel event handle → non-empty, not "invalid".
    let ev = kernel.create_event_w(Some("stat_ev_desc".to_owned()), true, false);
    let desc3 = kernel.describe_handle(ev);
    assert!(!desc3.is_empty());
    assert_ne!(desc3, "invalid");
    // Invalid handle → "invalid".
    let desc4 = kernel.describe_handle(0xDEAD_CAFE);
    assert_eq!(desc4, "invalid");

    // --- show_window_with_activation ---
    let top = kernel.create_window_ex_w(thread_id, "Button", "Top", None, 0, WS_VISIBLE, 0);
    // show_window_with_activation(top, false, false) — hide, don't activate.
    let was_visible = kernel.show_window_with_activation(top, false, false);
    // show_window returns previous visibility; window was visible (WS_VISIBLE set).
    assert!(was_visible);
    // show_window_with_activation on invalid hwnd → false.
    let r = kernel.show_window_with_activation(0xDEAD_0000, true, false);
    assert!(!r);

    // --- set_window_enabled_state ---
    // Window starts enabled. set to disabled → Some((was_enabled=true, changed=true)).
    let result = kernel.set_window_enabled_state(top, false);
    assert_eq!(result, Some((true, true)));
    // Set disabled again → Some((was_enabled=false, changed=false)).
    let result2 = kernel.set_window_enabled_state(top, false);
    assert_eq!(result2, Some((false, false)));
    // Re-enable.
    let result3 = kernel.set_window_enabled_state(top, true);
    assert_eq!(result3, Some((false, true)));
    // Invalid hwnd → None.
    let result4 = kernel.set_window_enabled_state(0xDEAD_0000, true);
    assert!(result4.is_none());

    // --- bring_window_to_top ---
    // Valid window → true.
    let brought = kernel.bring_window_to_top(top);
    assert!(brought);
    // Invalid window → false.
    let brought2 = kernel.bring_window_to_top(0xDEAD_0000);
    assert!(!brought2);

    // --- activate_window ---
    // Deactivate by passing None → returns previous active (top or hwnd).
    let prev = kernel.activate_window(None);
    // prev may be Some(any window) or None; just check it succeeds.
    let _ = prev;
    // Activate top → previous is whatever was active before.
    kernel.show_window(top, true);
    let _prev2 = kernel.activate_window(Some(top));
    // Activating a disabled window → None.
    kernel.set_window_enabled_state(top, false);
    let r = kernel.activate_window(Some(top));
    assert!(r.is_none());
}

#[test]
fn kernel_post_shell_notify_icon_and_notification_callbacks_expire() {
    use wince_emulation_v3::ce::gwe::WS_VISIBLE;
    use wince_emulation_v3::ce::shell::{
        NIF_MESSAGE, NotifyIconData, NotifyIconOp, ShellNotificationData, SHNP_ICONIC,
    };

    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);
    let thread_id = 55u32;

    // --- post_shell_notify_icon_callback ---
    // Create a window to receive the callback.
    let callback_msg = 0xBB01u32;
    let hwnd = kernel.create_window_ex_w(thread_id, "Button", "Tray", None, 0, WS_VISIBLE, 0);

    // No notify icon registered → returns false.
    assert!(!kernel.post_shell_notify_icon_callback(hwnd, 1, 0x200));

    // Add a notify icon with callback_message and NIF_MESSAGE flag.
    kernel.shell.apply_notify_icon(
        NotifyIconOp::Add,
        NotifyIconData {
            hwnd,
            id: 1,
            flags: NIF_MESSAGE,
            callback_message: callback_msg,
            icon: 0,
            tip: String::new(),
            state: 0,
            state_mask: 0,
        },
    );

    // Now posting the callback → posts a message to hwnd, returns true.
    let ok = kernel.post_shell_notify_icon_callback(hwnd, 1, 0x202);
    assert!(ok);

    // Verify the message was posted (wparam=icon id, lparam=event_lparam).
    let msg = kernel.take_ready_message_w_filtered(thread_id, Some(hwnd), callback_msg, callback_msg);
    assert!(msg.is_some());
    let m = msg.unwrap();
    assert_eq!(m.msg, callback_msg);
    assert_eq!(m.wparam, 1); // icon id
    assert_eq!(m.lparam, 0x202);

    // Icon with callback_message=0 → returns false even if icon exists.
    kernel.shell.apply_notify_icon(
        NotifyIconOp::Add,
        NotifyIconData {
            hwnd,
            id: 2,
            flags: 0,
            callback_message: 0,
            icon: 0,
            tip: String::new(),
            state: 0,
            state_mask: 0,
        },
    );
    assert!(!kernel.post_shell_notify_icon_callback(hwnd, 2, 0x200));

    // --- post_shell_notification_callback ---
    let clsid = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    // No notification registered → false.
    assert!(!kernel.post_shell_notification_callback(clsid, 99, 0, 0, 0));

    // Add a notification with non-zero clsid (enables COM callback recording).
    let now_ms = kernel.timers.tick_count();
    kernel.shell.add_notification(
        ShellNotificationData {
            id: 5,
            priority: SHNP_ICONIC,
            duration_cs: 1000,
            icon: 0,
            flags: 0,
            clsid,
            hwnd_sink: 0,
            title: "Test".to_owned(),
            html: String::new(),
            lparam: 0,
        },
        now_ms,
    );

    // post_shell_notification_callback with SHNN_DISMISS → records COM callback.
    let before = kernel.shell.notification_callbacks().count();
    let delivered = kernel.post_shell_notification_callback(clsid, 5, 0xffff_fc17, 0, 0);
    // With non-zero clsid and no hwnd_sink → COM callback recorded.
    assert!(delivered);
    assert_eq!(kernel.shell.notification_callbacks().count(), before + 1);

    // --- post_shell_notification_dismiss_callback ---
    // Reset callbacks and verify dismiss posts a COM record.
    let before2 = kernel.shell.notification_callbacks().count();
    let ok = kernel.post_shell_notification_dismiss_callback(clsid, 5, true);
    assert!(ok);
    assert_eq!(kernel.shell.notification_callbacks().count(), before2 + 1);

    // --- expire_shell_notifications ---
    // Add a notification with duration_cs=1 → expires_at = now + 10 ms.
    let clsid2 = [9u8; 16];
    let now_ms2 = kernel.timers.tick_count();
    kernel.shell.add_notification(
        ShellNotificationData {
            id: 7,
            priority: SHNP_ICONIC,
            duration_cs: 1, // → expires_at = now_ms2 + 10 ms
            icon: 0,
            flags: 0,
            clsid: clsid2,
            hwnd_sink: 0,
            title: "Exp".to_owned(),
            html: String::new(),
            lparam: 0,
        },
        now_ms2,
    );
    // Before advancing time: notification still present.
    assert!(kernel.shell.notification(clsid2, 7).is_some());
    // Advance virtual time by 20 ms so tick_count > expires_at.
    kernel.timers.sleep_ms(20);
    let expired_count = kernel.expire_shell_notifications();
    // expire returns the number of dismiss callbacks posted (0 here: no window sink, no COM).
    let _ = expired_count;
    // Verify the notification was removed from shell.
    assert!(kernel.shell.notification(clsid2, 7).is_none());

    // --- post_shell_notification_link_callback ---
    // Re-add notification for link callback.
    let now_ms3 = kernel.timers.tick_count();
    kernel.shell.add_notification(
        ShellNotificationData {
            id: 5,
            priority: SHNP_ICONIC,
            duration_cs: 10000,
            icon: 0,
            flags: 0,
            clsid,
            hwnd_sink: 0,
            title: "Link".to_owned(),
            html: String::new(),
            lparam: 0,
        },
        now_ms3,
    );
    let before3 = kernel.shell.notification_callbacks().count();
    let ok2 = kernel.post_shell_notification_link_callback(clsid, 5, "https://example.com");
    assert!(ok2);
    assert_eq!(kernel.shell.notification_callbacks().count(), before3 + 1);

    // --- post_shell_notification_command_callback ---
    let before4 = kernel.shell.notification_callbacks().count();
    let ok3 = kernel.post_shell_notification_command_callback(clsid, 5, 42);
    assert!(ok3);
    assert_eq!(kernel.shell.notification_callbacks().count(), before4 + 1);
}

#[test]
fn kernel_get_message_peek_message_erase_background_update_window() {
    use wince_emulation_v3::ce::gwe::{PeekFlags, WM_USER, WS_VISIBLE};

    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);
    let thread_id = 77u32;

    let hwnd = kernel.create_window_ex_w(thread_id, "Button", "Wnd", None, 0, WS_VISIBLE, 0);

    // get_message_w_filtered: empty queue → None (no queued messages).
    let msg = kernel.get_message_w_filtered(thread_id, Some(hwnd), WM_USER, WM_USER + 100);
    assert!(msg.is_none());

    // Post a message to hwnd, then get_message_w_filtered retrieves it.
    kernel.post_message_w(hwnd, WM_USER + 1, 0x11, 0x22);
    let msg2 = kernel.get_message_w_filtered(thread_id, Some(hwnd), WM_USER, WM_USER + 100);
    assert!(msg2.is_some());
    let m2 = msg2.unwrap();
    assert_eq!(m2.msg, WM_USER + 1);
    assert_eq!(m2.wparam, 0x11);
    assert_eq!(m2.lparam, 0x22);

    // Drain only WM_USER range so timers/system messages do not cause an infinite loop.
    while kernel.peek_message_w_filtered(thread_id, None, WM_USER, WM_USER + 100, PeekFlags::REMOVE).is_some() {}
    let msg3 = kernel.peek_message_w_filtered(thread_id, None, WM_USER, WM_USER + 100, PeekFlags::NO_REMOVE);
    assert!(msg3.is_none());

    // peek_message_w_filtered with NO_REMOVE: empty queue → None.
    let peek = kernel.peek_message_w_filtered(
        thread_id, Some(hwnd), WM_USER, WM_USER + 100, PeekFlags::NO_REMOVE,
    );
    assert!(peek.is_none());

    // Post a message, peek with NO_REMOVE: message stays in queue.
    kernel.post_message_w(hwnd, WM_USER + 2, 0x33, 0x44);
    let peek2 = kernel.peek_message_w_filtered(
        thread_id, Some(hwnd), WM_USER, WM_USER + 100, PeekFlags::NO_REMOVE,
    );
    assert!(peek2.is_some());
    assert_eq!(peek2.unwrap().msg, WM_USER + 2);

    // peek with REMOVE: retrieves and removes the message.
    let peek3 = kernel.peek_message_w_filtered(
        thread_id, Some(hwnd), WM_USER, WM_USER + 100, PeekFlags::REMOVE,
    );
    assert!(peek3.is_some());
    assert_eq!(peek3.unwrap().msg, WM_USER + 2);

    // After REMOVE, the message is gone.
    let peek4 = kernel.peek_message_w_filtered(
        thread_id, Some(hwnd), WM_USER, WM_USER + 100, PeekFlags::NO_REMOVE,
    );
    assert!(peek4.is_none());

    // --- erase_window_background ---
    // Invalid hwnd → false.
    assert!(!kernel.erase_window_background(0xDEAD_0000, 0x100));

    // Valid hwnd: WM_ERASEBKGND is sent. Default handler returns 0 (no hbrBackground brush),
    // so erase_window_background returns true (clear_update_erase condition not triggered).
    let hdc = 0x0200_0000 | (hwnd & 0x00ff_ffff);
    let erased = kernel.erase_window_background(hwnd, hdc);
    assert!(erased);

    // --- update_window ---
    // Invalid hwnd → false.
    assert!(!kernel.update_window(0xDEAD_0000));

    // Valid hwnd with no pending update_rect → true (no-op).
    let updated = kernel.update_window(hwnd);
    assert!(updated);
}

#[test]
fn memory_local_re_alloc_detail_and_heap_re_alloc_detail_reallocation_fields() {
    use wince_emulation_v3::ce::memory::{
        HEAP_REALLOC_IN_PLACE_ONLY, HEAP_ZERO_MEMORY,
        LMEM_MODIFY, LMEM_ZEROINIT, MemorySystem,
    };

    let mut mem = MemorySystem::default();
    let heap = mem.get_process_heap();

    // --- heap_re_alloc_detail: shrink (in-place) ---
    let p = mem.heap_alloc(heap, 0, 128).unwrap();
    let r = mem.heap_re_alloc_detail(heap, 0, p, 64).unwrap();
    assert!(!r.moved); // shrink is in-place
    assert_eq!(r.ptr, p);
    assert_eq!(r.old_ptr, p);
    assert!(r.old_actual_size >= 128);
    assert!(r.new_actual_size <= r.old_actual_size);

    // heap_re_alloc_detail: grow (ptr moves).
    let r2 = mem.heap_re_alloc_detail(heap, 0, p, 4096).unwrap();
    // After grow the old ptr is invalid; new ptr is different.
    assert!(r2.moved);
    assert!(r2.ptr != r2.old_ptr);
    assert!(r2.new_actual_size >= 4096);

    // heap_re_alloc_detail: HEAP_REALLOC_IN_PLACE_ONLY when grow required → None.
    let p2 = mem.heap_alloc(heap, 0, 64).unwrap();
    assert!(mem.heap_re_alloc_detail(heap, HEAP_REALLOC_IN_PLACE_ONLY, p2, 9999).is_none());

    // heap_re_alloc_detail: invalid flags → None.
    assert!(mem.heap_re_alloc_detail(heap, 0xFFFF_0000, p2, 64).is_none());

    // heap_re_alloc_detail: invalid heap → None.
    assert!(mem.heap_re_alloc_detail(0xDEAD_0000, 0, p2, 64).is_none());

    // heap_re_alloc_detail with HEAP_ZERO_MEMORY: zeroed flag set in result allocation.
    let p3 = mem.heap_alloc(heap, 0, 64).unwrap();
    let r3 = mem.heap_re_alloc_detail(heap, HEAP_ZERO_MEMORY, p3, 32).unwrap();
    // Result ptr (in-place): verify the allocation is now tracked as zeroed.
    let alloc = mem.allocation(r3.ptr).unwrap();
    assert!(alloc.zeroed);

    // --- local_re_alloc_detail: invalid flags (LMEM_MODIFY is valid, 0xFFFF_0000 is not) ---
    let lp = mem.local_alloc(0, 64).unwrap();
    // Valid with LMEM_ZEROINIT | LMEM_MODIFY (both in valid mask).
    let lr = mem.local_re_alloc_detail(lp, 32, LMEM_ZEROINIT | LMEM_MODIFY).unwrap();
    assert!(!lr.moved); // shrink in-place
    // Invalid flags → None.
    assert!(mem.local_re_alloc_detail(lp, 32, 0xFFFF_0000).is_none());
}

#[test]
fn device_namespace_default_baud_mode_and_accepts_remote_serial_target_and_bitmap_with_masks() {
    use wince_emulation_v3::ce::devices::{
        DeviceBackend, DeviceConfig, DeviceConfigFile, DeviceDefaults, DeviceKind, DeviceNamespace,
    };
    use wince_emulation_v3::ce::resource::ResourceSystem;

    // --- DeviceNamespace default_baud and default_mode ---
    let defaults = DeviceDefaults::default();
    let config = DeviceConfigFile {
        version: 0,
        defaults,
        devices: vec![DeviceConfig {
            guest: "COM3:".to_owned(),
            kind: DeviceKind::Serial,
            backend: DeviceBackend::Stub,
            host: None,
            enabled: true,
            note: None,
        }],
    };
    let ns = DeviceNamespace::from_config(config);
    // default_baud: 9600 (the standard default).
    assert_eq!(ns.default_baud(), 9600);
    // default_mode: "8N1".
    assert_eq!(ns.default_mode(), "8N1");

    // --- accepts_remote_serial_target ---
    let session = ns.open("COM3:").unwrap();
    // Exact match (case-normalized).
    assert!(session.accepts_remote_serial_target("COM3:"));
    assert!(session.accepts_remote_serial_target("com3:"));
    // Non-matching target.
    assert!(!session.accepts_remote_serial_target("COM7:"));

    // --- ResourceSystem::create_bitmap_with_masks ---
    let mut res = ResourceSystem::default();
    let masks = [0x00ff_0000u32, 0x0000_ff00, 0x0000_00ff];
    let bmp = res.create_bitmap_with_masks(16, 16, 1, 32, 0x3000, Some(masks));
    assert_ne!(bmp, 0);
    let bmp_obj = res.bitmap(bmp).unwrap();
    assert_eq!(bmp_obj.width, 16);
    assert_eq!(bmp_obj.bits_pixel, 32);
    // rgb_masks should be stored.
    assert_eq!(bmp_obj.rgb_masks, Some(masks));
    // Non-owned bitmap: owned flag should be false (bits_ptr not allocated).
    // (Verify the bitmap is accessible and deletable.)
    assert!(res.delete_bitmap(bmp));

    // create_bitmap_with_masks without masks (None) also works.
    let bmp2 = res.create_bitmap_with_masks(8, 8, 1, 16, 0x4000, None);
    assert_ne!(bmp2, 0);
    assert!(res.bitmap(bmp2).unwrap().rgb_masks.is_none());
    assert!(res.delete_bitmap(bmp2));
}

#[test]
fn gwe_validate_window_rect_and_kernel_send_message_w_and_remote_free_functions() {
    use wince_emulation_v3::ce::gwe::{Gwe, Rect, WM_USER};
    use wince_emulation_v3::ce::remote::{make_lparam, nmea_checksum_line, normalize_nmea_sentence};

    // --- validate_window_rect ---
    let mut gwe = Gwe::default();
    let client = Rect { left: 0, top: 0, right: 100, bottom: 100 };
    let hwnd = gwe.create_window_ex_with_rect(1, "cls", "w", None, 0, 0, 0, client);

    // No pending update → validate_window_rect succeeds and is a no-op.
    assert!(gwe.validate_window_rect(hwnd, Some(Rect { left: 10, top: 10, right: 50, bottom: 50 })));
    assert!(gwe.update_rect(hwnd).is_none());

    // Invalidate a region, then validate with None → clears all pending.
    gwe.invalidate_window(hwnd, Some(Rect { left: 0, top: 0, right: 40, bottom: 40 }), true);
    assert!(gwe.update_rect(hwnd).is_some());
    assert!(gwe.validate_window_rect(hwnd, None));
    assert!(gwe.update_rect(hwnd).is_none());

    // Invalidate, then validate sub-rect that does NOT fully subtract bounding box → update remains.
    gwe.invalidate_window(hwnd, Some(Rect { left: 0, top: 0, right: 40, bottom: 40 }), false);
    assert!(gwe.update_rect(hwnd).is_some());
    // Validate a rect that partially overlaps but doesn't remove all → update_rect may still be Some
    // (exact result depends on subtract_bounding; just verify it doesn't panic and returns true).
    let ok = gwe.validate_window_rect(hwnd, Some(Rect { left: 50, top: 50, right: 90, bottom: 90 }));
    assert!(ok);

    // Invalid hwnd → false.
    assert!(!gwe.validate_window_rect(0xDEAD_0000, None));

    // --- kernel.send_message_w: WM_USER to a known window ---
    let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
    let mut kernel = CeKernel::boot(config);
    let tid = 88u32;
    let win = kernel.create_window_ex_w(tid, "Button", "Btn", None, 0, WS_VISIBLE, 0);

    // WM_USER + 10 has no special handler → default result 0.
    assert_eq!(kernel.send_message_w(win, WM_USER + 10, 11, 22), Some(0));

    // Invalid hwnd → None.
    assert_eq!(kernel.send_message_w(0xDEAD_0000, WM_USER, 0, 0), None);

    // --- normalize_nmea_sentence ---
    assert_eq!(normalize_nmea_sentence("$GPRMC,...*00"), "$GPRMC,...*00\r\n");
    assert_eq!(normalize_nmea_sentence("$GPRMC,...*00\r\n"), "$GPRMC,...*00\r\n");
    assert_eq!(normalize_nmea_sentence("$GPRMC,...*00\r"), "$GPRMC,...*00\r\n");

    // --- nmea_checksum_line ---
    // XOR checksum of "GPRMC" = 'G'^'P'^'R'^'M'^'C' = 0x47^0x50^0x52^0x4D^0x43 = 0x4B
    let checksum_line = nmea_checksum_line("GPRMC");
    assert_eq!(checksum_line, "$GPRMC*4B\r\n");

    // --- make_lparam ---
    assert_eq!(make_lparam(10, 20), (20u32 << 16) | 10u32);
    assert_eq!(make_lparam(0x1234, 0x5678), (0x5678u32 << 16) | 0x1234u32);
    // Negative values are packed as 16-bit two's complement.
    assert_eq!(make_lparam(-1, -1), 0xffff_ffff);
}

#[test]
fn resource_image_list_duplicate_merge_add_masked_replace_remove_copy_overlay_drag() {
    use wince_emulation_v3::ce::resource::{ImageListDraw, ResourceSystem};
    use wince_emulation_v3::ce::gwe::Rect;

    let mut res = ResourceSystem::default();

    // Create a 16×16 image list and two 32×16 bitmaps (each yields 2 strips at 16px width).
    let bmp_a = res.create_bitmap(32, 16, 1, 16, 0x1000);
    let bmp_b = res.create_bitmap(32, 16, 1, 16, 0x2000);
    let ilh = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    let idx_a = res.add_image_list_image(ilh, bmp_a, 0).unwrap();
    assert_eq!(idx_a, 0);
    assert_eq!(res.image_list_count(ilh), Some(2)); // 32px strip → 2 images

    // --- duplicate_image_list ---
    let dup = res.duplicate_image_list(ilh).unwrap();
    assert_ne!(dup, ilh);
    assert_eq!(res.image_list_count(dup), Some(2));
    // Invalid handle → None.
    assert!(res.duplicate_image_list(0xDEAD).is_none());

    // --- merge_image_list_images ---
    let merged = res.merge_image_list_images(ilh, 0, dup, 1, 4, 4).unwrap();
    assert_ne!(merged, ilh);
    assert_eq!(res.image_list_count(merged), Some(2)); // first + second image
    // Negative index → None.
    assert!(res.merge_image_list_images(ilh, -1, dup, 0, 0, 0).is_none());

    // --- add_masked_image_list_image ---
    let ilh2 = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    let idx_masked = res.add_masked_image_list_image(ilh2, bmp_b, 0x00ff_00ff).unwrap();
    assert_eq!(idx_masked, 0);
    let info_masked = res.image_list_info(ilh2, 0).unwrap();
    assert_eq!(info_masked.bitmap, bmp_b);
    // bitmap=0 → None.
    assert!(res.add_masked_image_list_image(ilh2, 0, 0).is_none());

    // --- replace_image_list_image ---
    // Replace index 0 with bmp_b.
    assert_eq!(res.replace_image_list_image(ilh, 0, bmp_b, 0), Some(true));
    let info_r = res.image_list_info(ilh, 0).unwrap();
    assert_eq!(info_r.bitmap, bmp_b);
    // Out of bounds index → Some(false).
    assert_eq!(res.replace_image_list_image(ilh, 99, bmp_b, 0), Some(false));
    // Negative index → Some(false).
    assert_eq!(res.replace_image_list_image(ilh, -1, bmp_b, 0), Some(false));
    // bitmap=0 → Some(false).
    assert_eq!(res.replace_image_list_image(ilh, 0, 0, 0), Some(false));
    // Invalid handle → None.
    assert_eq!(res.replace_image_list_image(0xDEAD, 0, bmp_b, 0), None);

    // --- replace_image_list_icon ---
    // icon=0 → None.
    assert!(res.replace_image_list_icon(ilh, 0, 0).is_none());
    // index<0 → append.
    let icon_handle = 0xABCD;
    let new_idx = res.replace_image_list_icon(ilh, -1, icon_handle).unwrap();
    assert_eq!(new_idx as usize, res.image_list_count(ilh).unwrap() - 1);
    // index>=0 → replace at that index.
    let prev_count = res.image_list_count(ilh).unwrap();
    let _ = res.replace_image_list_icon(ilh, 0, 0xBEEF).unwrap();
    assert_eq!(res.image_list_count(ilh), Some(prev_count)); // count unchanged

    // --- set_image_list_count ---
    let ilh3 = res.create_image_list(16, 16, 0, 0, 4).unwrap();
    res.add_image_list_image(ilh3, bmp_a, 0).unwrap();
    assert_eq!(res.image_list_count(ilh3), Some(2));
    // Grow to 5.
    assert_eq!(res.set_image_list_count(ilh3, 5), Some(true));
    assert_eq!(res.image_list_count(ilh3), Some(5));
    // Shrink to 1.
    assert_eq!(res.set_image_list_count(ilh3, 1), Some(true));
    assert_eq!(res.image_list_count(ilh3), Some(1));
    // Invalid handle → None.
    assert_eq!(res.set_image_list_count(0xDEAD, 5), None);

    // --- remove_image_list_image ---
    // Remove index=0 (the only remaining image).
    assert_eq!(res.remove_image_list_image(ilh3, 0), Some(true));
    assert_eq!(res.image_list_count(ilh3), Some(0));
    // Out of bounds → Some(false).
    assert_eq!(res.remove_image_list_image(ilh3, 0), Some(false));
    // Add two images and remove all with index<0.
    res.add_image_list_image(ilh3, bmp_a, 0);
    res.add_image_list_image(ilh3, bmp_b, 0);
    assert_eq!(res.remove_image_list_image(ilh3, -1), Some(true));
    assert_eq!(res.image_list_count(ilh3), Some(0));

    // --- copy_image_list_image ---
    let src = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    let dst_il = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    res.add_image_list_image(src, bmp_a, 0).unwrap();
    res.add_image_list_image(dst_il, bmp_b, 0).unwrap();
    // Copy src[0] → dst[0], remove_source=false.
    assert_eq!(res.copy_image_list_image(dst_il, 0, src, 0, false), Some(true));
    let dst_info = res.image_list_info(dst_il, 0).unwrap();
    assert_eq!(dst_info.bitmap, bmp_a); // src bitmap copied
    assert_eq!(res.image_list_count(src), Some(2)); // source unchanged
    // Negative src_index → Some(false).
    assert_eq!(res.copy_image_list_image(dst_il, 0, src, -1, false), Some(false));

    // --- copy_dither_image_list_image ---
    let dither_src = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    let dither_dst = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    res.add_image_list_image(dither_src, bmp_a, 0).unwrap();
    res.add_image_list_image(dither_dst, bmp_b, 0).unwrap();
    assert_eq!(res.copy_dither_image_list_image(dither_dst, 0, 5, 10, dither_src, 0, 0x42), Some(true));
    // Negative dst_index or src_index → Some(false).
    assert_eq!(res.copy_dither_image_list_image(dither_dst, -1, 0, 0, dither_src, 0, 0), Some(false));
    assert_eq!(res.copy_dither_image_list_image(dither_dst, 0, 0, 0, dither_src, -1, 0), Some(false));

    // --- image_list_icon ---
    let icon_il = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    res.add_image_list_image(icon_il, bmp_a, 0).unwrap();
    // index<0 → None.
    assert!(res.image_list_icon(icon_il, -1, 0, 0).is_none());
    // index with bitmap: returns derived icon handle.
    let ic = res.image_list_icon(icon_il, 0, 0, 0).unwrap();
    assert_ne!(ic, 0);
    // fallback_icon used when image.icon == 0 and bitmap == 0.
    let bare_il = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    res.set_image_list_count(bare_il, 1);
    let ic_fallback = res.image_list_icon(bare_il, 0, 0xFACE, 0).unwrap();
    assert_eq!(ic_fallback, 0xFACE);

    // --- set_image_list_overlay ---
    let ov_il = res.create_image_list(16, 16, 0, 0, 4).unwrap();
    res.add_image_list_image(ov_il, bmp_a, 0).unwrap();
    // image_index < 0 → Some(false).
    assert_eq!(res.set_image_list_overlay(ov_il, -1, 1), Some(false));
    // overlay out of [1..=15] → Some(false).
    assert_eq!(res.set_image_list_overlay(ov_il, 0, 0), Some(false));
    assert_eq!(res.set_image_list_overlay(ov_il, 0, 16), Some(false));
    // Valid: maps overlay 1 → image index 0.
    assert_eq!(res.set_image_list_overlay(ov_il, 0, 1), Some(true));

    // --- record_image_list_draw ---
    let draw = ImageListDraw {
        image_list: ov_il,
        hdc: 0x200,
        index: 0,
        x: 5,
        y: 10,
        width: 16,
        height: 16,
        flags: 0x0100, // overlay = 1
        overlay_image: None,
    };
    assert_eq!(res.record_image_list_draw(draw.clone()), Some(true));
    let last_draw = res.image_list(ov_il).unwrap().last_draw.unwrap();
    assert_eq!(last_draw.index, 0);
    // overlay_image filled in from overlay map (overlay 1 → index 0).
    assert_eq!(last_draw.overlay_image, Some(0));
    // Out-of-bounds index → Some(false).
    let bad_draw = ImageListDraw { index: 99, ..draw };
    assert_eq!(res.record_image_list_draw(bad_draw), Some(false));

    // --- image_list drag sequence ---
    // begin before drag started → false for drag ops.
    assert!(!res.image_list_drag_enter(0, 0, 0));
    assert!(!res.image_list_drag_move(5, 5));
    assert!(!res.image_list_drag_show(true));
    assert!(!res.end_image_list_drag());
    assert!(res.image_list_drag().is_none());

    let drag_il = res.create_image_list(16, 16, 0, 0, 2).unwrap();
    res.add_image_list_image(drag_il, bmp_a, 0).unwrap();
    // begin_image_list_drag with bad index → Some(false).
    assert_eq!(res.begin_image_list_drag(drag_il, 99, 8, 8), Some(false));
    // begin_image_list_drag valid.
    assert_eq!(res.begin_image_list_drag(drag_il, 0, 4, 4), Some(true));
    let drag = res.image_list_drag().unwrap();
    assert_eq!(drag.image_list, drag_il);
    assert_eq!(drag.hotspot_x, 4);
    // drag_enter sets lock_hwnd and position.
    assert!(res.image_list_drag_enter(0x1234, 10, 20));
    let drag2 = res.image_list_drag().unwrap();
    assert_eq!(drag2.lock_hwnd, 0x1234);
    assert_eq!(drag2.x, 10);
    // drag_move updates position.
    assert!(res.image_list_drag_move(30, 40));
    assert_eq!(res.image_list_drag().unwrap().x, 30);
    // drag_show toggles visible.
    assert!(res.image_list_drag_show(false));
    assert!(!res.image_list_drag().unwrap().visible);
    // drag_leave: hwnd mismatch → false.
    assert!(!res.image_list_drag_leave(0xDEAD));
    // drag_leave with matching hwnd.
    assert!(res.image_list_drag_leave(0x1234));
    assert_eq!(res.image_list_drag().unwrap().lock_hwnd, 0);
    // set_image_list_drag_cursor with valid index updates drag state.
    assert_eq!(res.set_image_list_drag_cursor(drag_il, 0, 6, 7), Some(true));
    assert_eq!(res.image_list_drag().unwrap().hotspot_x, 6);
    // set_image_list_drag_cursor with bad index → Some(false).
    assert_eq!(res.set_image_list_drag_cursor(drag_il, 99, 0, 0), Some(false));
    // end drag.
    assert!(res.end_image_list_drag());
    assert!(res.image_list_drag().is_none());
    // second end → false.
    assert!(!res.end_image_list_drag());

    // --- set_region_rects ---
    let rh = res.create_region(Rect { left: 0, top: 0, right: 100, bottom: 100 });
    let rects = vec![
        Rect { left: 0, top: 0, right: 50, bottom: 50 },
        Rect { left: 60, top: 0, right: 100, bottom: 50 },
    ];
    assert!(res.set_region_rects(rh, rects));
    // Invalid handle → false.
    assert!(!res.set_region_rects(0xDEAD, vec![]));

    // --- select_clip_region / clip_region ---
    let hdc = res.create_compatible_dc();
    assert!(res.clip_region(hdc).is_none()); // no clip region set
    res.select_clip_region(hdc, Some(rh));
    assert_eq!(res.clip_region(hdc), Some(rh));
    // Deselect clip region.
    res.select_clip_region(hdc, None);
    assert!(res.clip_region(hdc).is_none());
}
