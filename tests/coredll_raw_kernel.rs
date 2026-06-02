use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_CLOSE_HANDLE, ORD_CREATE_EVENT_W, ORD_CREATE_SEMAPHORE_W, ORD_CREATE_THREAD,
            ORD_EVENT_MODIFY, ORD_GET_EXIT_CODE_PROCESS, ORD_GET_EXIT_CODE_THREAD,
            ORD_GET_LAST_ERROR, ORD_GET_PROCESS_ID, ORD_GET_PROCESS_VERSION,
            ORD_GET_STORE_INFORMATION, ORD_GET_THREAD_ID, ORD_GET_THREAD_PRIORITY,
            ORD_GET_THREAD_TIMES, ORD_GET_TICK_COUNT, ORD_GET_VERSION_EX_W,
            ORD_INITIALIZE_CRITICAL_SECTION, ORD_INTERLOCKED_COMPARE_EXCHANGE,
            ORD_INTERLOCKED_EXCHANGE_ADD, ORD_INTERLOCKED_INCREMENT, ORD_LEAVE_CRITICAL_SECTION,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX, ORD_QUERY_PERFORMANCE_COUNTER,
            ORD_QUERY_PERFORMANCE_FREQUENCY, ORD_RELEASE_SEMAPHORE, ORD_RESUME_THREAD,
            ORD_SET_LAST_ERROR, ORD_SET_THREAD_PRIORITY, ORD_SLEEP, ORD_SUSPEND_THREAD,
            ORD_TERMINATE_PROCESS, ORD_TLS_GET_VALUE, ORD_TLS_SET_VALUE,
            ORD_TRY_ENTER_CRITICAL_SECTION, ORD_WAIT_FOR_SINGLE_OBJECT,
        },
        gwe::Message,
        kernel::CeKernel,
        registry::ERROR_SUCCESS,
        thread::{ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER},
        timer::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
};

mod support;
use support::TestGuestMemory;

#[test]
fn coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let word = 0x1000;
    let critical_section = 0x2000;
    memory.write_word(word, 41);
    memory.map_words(critical_section, 5);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INITIALIZE_CRITICAL_SECTION,
            [critical_section]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(critical_section)?, 0);
    assert_eq!(memory.read_u32(critical_section + 4)?, 0);
    assert!(memory.read_u32(critical_section + 8)? >= 0x100);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRY_ENTER_CRITICAL_SECTION,
            [critical_section]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(critical_section)?, 1);
    assert_eq!(memory.read_u32(critical_section + 4)?, thread_id);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LEAVE_CRITICAL_SECTION,
            [critical_section]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(critical_section + 4)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INTERLOCKED_INCREMENT,
            [word],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(42),
            ..
        }
    ));
    assert_eq!(memory.read_u32(word)?, 42);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INTERLOCKED_EXCHANGE_ADD,
            [word, 8]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(42),
            ..
        }
    ));
    assert_eq!(memory.read_u32(word)?, 50);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INTERLOCKED_COMPARE_EXCHANGE,
            [word, 0xfeed, 50]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(50),
            ..
        }
    ));
    assert_eq!(memory.read_u32(word)?, 0xfeed);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TLS_SET_VALUE,
            [5, 0xbeef]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROCESS_VERSION,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x0004_0014),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TLS_GET_VALUE,
            [5],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0xbeef),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_LAST_ERROR,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TLS_GET_VALUE,
            [64],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_LAST_ERROR,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_LAST_ERROR,
            [1234],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_LAST_ERROR,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1234),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_TICK_COUNT,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(_),
            ..
        }
    ));
    memory.map_words(0x3000, 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_QUERY_PERFORMANCE_FREQUENCY,
            [0x3000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(0x3000)?, 1_000);
    assert_eq!(memory.read_u32(0x3004)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_QUERY_PERFORMANCE_COUNTER,
            [0x3008],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(0x300c)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_QUERY_PERFORMANCE_FREQUENCY,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    assert!(matches!(
        table
            .dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_SLEEP, [0],),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    let version_info = 0x6000;
    memory.map_words(version_info, 69);
    memory.map_halfwords(version_info + 20, 128);
    memory.write_word(version_info, 276);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_VERSION_EX_W,
            [version_info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(version_info + 4)?, 4);
    assert_eq!(memory.read_u32(version_info + 8)?, 20);
    assert_eq!(memory.read_u32(version_info + 16)?, 3);
    assert_eq!(memory.read_wide_z(version_info + 20, 128), "");

    let store_info = 0x5100;
    memory.map_words(store_info, 2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_STORE_INFORMATION,
            [store_info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let store_size = memory.read_u32(store_info)?;
    let free_size = memory.read_u32(store_info + 4)?;
    assert!(store_size > 0);
    assert!(free_size <= store_size);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_STORE_INFORMATION,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    let thread_id_ptr = 0x5000;
    memory.map_words(thread_id_ptr, 1);
    let worker_thread = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_THREAD,
        [0, 0, 0x1234_5678, 0xabcd_0000, 0, thread_id_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateThread did not return a raw thread handle: {other:?}"),
    };
    assert_ne!(worker_thread, 0);
    assert_eq!(memory.read_u32(thread_id_ptr)?, 2);
    let exit_code_ptr = 0x5040;
    memory.map_words(exit_code_ptr, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_THREAD_ID,
            [worker_thread],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_EXIT_CODE_THREAD,
            [worker_thread, exit_code_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(exit_code_ptr)?, 259);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_EXIT_CODE_THREAD,
            [worker_thread, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    let thread_times_ptr = 0x50c0;
    memory.map_words(thread_times_ptr, 8);
    memory.write_word(thread_times_ptr, 0xdead_beef);
    memory.write_word(thread_times_ptr + 4, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_THREAD_TIMES,
            [
                worker_thread,
                thread_times_ptr,
                thread_times_ptr + 8,
                thread_times_ptr + 16,
                thread_times_ptr + 24,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    for offset in (0..32).step_by(4) {
        assert_eq!(memory.read_u32(thread_times_ptr + offset)?, 0);
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SUSPEND_THREAD,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(kernel.guest_thread_start(worker_thread).is_none());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESUME_THREAD,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert!(kernel.guest_thread_start(worker_thread).is_some());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_THREAD_PRIORITY,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_THREAD_PRIORITY,
            [worker_thread, 1]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_THREAD_PRIORITY,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [worker_thread, 0]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_TIMEOUT),
            ..
        }
    ));
    assert!(kernel.mark_guest_thread_exited(worker_thread, 0x55));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_EXIT_CODE_THREAD,
            [worker_thread, exit_code_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(exit_code_ptr)?, 0x55);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [worker_thread, 0]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [worker_thread],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let launch = kernel.queue_process_launch(Some("raw-child.exe".to_owned()), None);
    let process_exit_ptr = 0x5080;
    memory.map_words(process_exit_ptr, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROCESS_ID,
            [launch.process_handle],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(id),
            ..
        } if id == launch.process_id
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_EXIT_CODE_PROCESS,
            [launch.process_handle, process_exit_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(process_exit_ptr)?, 259);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TERMINATE_PROCESS,
            [launch.process_handle, 0x66],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_EXIT_CODE_PROCESS,
            [launch.process_handle, process_exit_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(process_exit_ptr)?, 0x66);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_EXIT_CODE_PROCESS,
            [worker_thread, process_exit_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_HANDLE
    );

    let event_name_ptr = 0x4000;
    memory.write_wide_z(event_name_ptr, "raw-event");
    let event = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_EVENT_W,
        [0, 0, 0, event_name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateEventW did not return a raw event handle: {other:?}"),
    };
    assert_ne!(event, 0);
    let handles_ptr = 0x6000;
    memory.map_words(handles_ptr, 1);
    memory.write_word(handles_ptr, event);
    kernel.gwe.post_message(
        thread_id,
        Message::new(0, 0x400 + 42, 77, 0, kernel.timers.tick_count()),
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [0, 0, 1000, 0x04ff, 0x0004],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));
    assert!(kernel.gwe.get_message(thread_id).is_some());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EVENT_MODIFY,
            [event, 3]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [1, handles_ptr, 1000, 0x04ff, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EVENT_MODIFY,
            [event, 3]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [event, 0]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [event, 0]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_TIMEOUT),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [event],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let sem_name_ptr = 0x6200;
    let sem_prev_ptr = 0x6240;
    memory.write_wide_z(sem_name_ptr, "raw-semaphore");
    memory.map_words(sem_prev_ptr, 1);
    let semaphore = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_SEMAPHORE_W,
        [0, 1, 2, sem_name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateSemaphoreW did not return a raw semaphore handle: {other:?}"),
    };
    assert_ne!(semaphore, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [semaphore, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [semaphore, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_TIMEOUT),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_SEMAPHORE,
            [semaphore, 2, sem_prev_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(sem_prev_ptr)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_SEMAPHORE,
            [semaphore, 1, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [semaphore],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}
