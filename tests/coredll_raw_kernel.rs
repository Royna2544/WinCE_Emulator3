use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_CE_GET_THREAD_PRIORITY, ORD_CE_SET_THREAD_PRIORITY, ORD_CLOSE_HANDLE,
            ORD_CREATE_EVENT_W, ORD_CREATE_SEMAPHORE_W, ORD_CREATE_THREAD, ORD_EVENT_MODIFY,
            ORD_FILE_TIME_TO_SYSTEM_TIME, ORD_FREE_LIBRARY, ORD_GET_EXIT_CODE_PROCESS,
            ORD_GET_EXIT_CODE_THREAD, ORD_GET_LAST_ERROR, ORD_GET_LOCAL_TIME,
            ORD_GET_MODULE_HANDLE_W, ORD_GET_PROC_ADDRESS_A, ORD_GET_PROC_ADDRESS_W,
            ORD_GET_PROCESS_ID, ORD_GET_PROCESS_VERSION, ORD_GET_STORE_INFORMATION,
            ORD_GET_SYSTEM_TIME, ORD_GET_SYSTEM_TIME_AS_FILE_TIME, ORD_GET_THREAD_ID,
            ORD_GET_THREAD_PRIORITY, ORD_GET_THREAD_TIMES, ORD_GET_TICK_COUNT,
            ORD_GET_TIME_ZONE_INFORMATION, ORD_GET_VERSION_EX_W, ORD_INITIALIZE_CRITICAL_SECTION,
            ORD_INPUT_DEBUG_CHAR_W, ORD_INTERLOCKED_COMPARE_EXCHANGE, ORD_INTERLOCKED_EXCHANGE_ADD,
            ORD_INTERLOCKED_INCREMENT, ORD_KERNEL_IO_CONTROL, ORD_LEAVE_CRITICAL_SECTION,
            ORD_LOAD_LIBRARY_W, ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX, ORD_MULTI_BYTE_TO_WIDE_CHAR,
            ORD_OPEN_EVENT_W, ORD_QUERY_PERFORMANCE_COUNTER, ORD_QUERY_PERFORMANCE_FREQUENCY,
            ORD_RELEASE_MUTEX, ORD_RELEASE_SEMAPHORE, ORD_RESUME_THREAD, ORD_SET_LAST_ERROR,
            ORD_SET_THREAD_PRIORITY, ORD_SHGET_SPECIAL_FOLDER_PATH, ORD_SLEEP, ORD_SLEEP_TILL_TICK,
            ORD_SUSPEND_THREAD, ORD_SYSTEM_TIME_TO_FILE_TIME, ORD_TERMINATE_PROCESS,
            ORD_TLS_GET_VALUE, ORD_TLS_SET_VALUE, ORD_TRY_ENTER_CRITICAL_SECTION,
            ORD_WAIT_FOR_MULTIPLE_OBJECTS, ORD_WAIT_FOR_SINGLE_OBJECT,
        },
        gwe::{Message, QS_POSTMESSAGE},
        kernel::{CE_CURRENT_PROCESS_PSEUDO_HANDLE, CE_CURRENT_THREAD_PSEUDO_HANDLE, CeKernel},
        object::MAX_SUSPEND_COUNT,
        registry::{ERROR_SUCCESS, RegistryValue},
        scheduler::SchedulerBlockedWaitKind,
        thread::{
            ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_NOT_OWNER,
            ERROR_SIGNAL_REFUSED,
        },
        timer::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
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
    let system_time = 0x2f00;
    memory.map_halfwords(system_time, 8);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_TIME,
            [system_time],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u16(system_time)?, 2024);
    assert_eq!(memory.read_u16(system_time + 2)?, 1);
    assert_eq!(memory.read_u16(system_time + 4)?, 1);
    assert_eq!(memory.read_u16(system_time + 6)?, 1);
    assert!(memory.read_u16(system_time + 8)? <= 23);
    assert!(memory.read_u16(system_time + 10)? <= 59);
    assert!(memory.read_u16(system_time + 12)? <= 59);
    assert!(memory.read_u16(system_time + 14)? <= 999);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    let local_time = 0x2f20;
    memory.map_halfwords(local_time, 8);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_LOCAL_TIME,
            [local_time],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u16(local_time)?, 2024);
    assert_eq!(memory.read_u16(local_time + 2)?, 1);

    let file_time = 0x2f40;
    memory.map_words(file_time, 2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_TIME_AS_FILE_TIME,
            [file_time],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_ne!(memory.read_u32(file_time + 4)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    let time_zone = 0x2f60;
    memory.map_bytes(time_zone, 172);
    memory.map_halfwords(time_zone, 86);
    memory.map_words(time_zone, 43);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_TIME_ZONE_INFORMATION,
            [time_zone],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(time_zone)?, 0);
    assert_eq!(memory.read_u16(time_zone + 4)?, b'U' as u16);
    assert_eq!(memory.read_u16(time_zone + 6)?, b'T' as u16);
    assert_eq!(memory.read_u16(time_zone + 8)?, b'C' as u16);
    assert_eq!(memory.read_u16(time_zone + 10)?, 0);
    assert_eq!(memory.read_u32(time_zone + 84)?, 0);
    assert_eq!(memory.read_u32(time_zone + 168)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INPUT_DEBUG_CHAR_W,
            []
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_TIME,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
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
    let stats = kernel.scheduler_stats();
    assert_eq!(stats.sleep_count, 1);
    assert_eq!(stats.yield_count, 1);
    assert_eq!(stats.wait_block_count, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SLEEP,
            [INFINITE],
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
            ORD_SLEEP_TILL_TICK,
            [],
        ),
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
            ORD_GET_THREAD_ID,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(id),
            ..
        } if id == thread_id
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_EXIT_CODE_THREAD,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE, exit_code_ptr],
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
            ORD_GET_THREAD_TIMES,
            [
                CE_CURRENT_THREAD_PSEUDO_HANDLE,
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
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_THREAD_PRIORITY,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(3),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_THREAD_PRIORITY,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE, 1],
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
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
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
            ORD_CE_GET_THREAD_PRIORITY,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(249),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_SET_THREAD_PRIORITY,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE, 251],
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
            ORD_SUSPEND_THREAD,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
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
            ORD_RESUME_THREAD,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
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
            ORD_RESUME_THREAD,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_THREAD_PRIORITY,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(3),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_THREAD_PRIORITY,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(251),
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
            ORD_CE_GET_THREAD_PRIORITY,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(249),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_SET_THREAD_PRIORITY,
            [worker_thread, 42]
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
            ORD_CE_GET_THREAD_PRIORITY,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(42),
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
            [worker_thread, 8]
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

    for expected in 0..MAX_SUSPEND_COUNT {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SUSPEND_THREAD,
                [worker_thread]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(value),
                ..
            } if value == expected
        ));
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
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_SIGNAL_REFUSED
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESUME_THREAD,
            [worker_thread]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(MAX_SUSPEND_COUNT),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
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
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE],
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
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0],
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
    let process_wait = kernel.register_blocked_waiter(
        8,
        0x108,
        vec![launch.process_handle],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
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
    assert_eq!(
        kernel.select_ready_blocked_waiter(thread_id, 0, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(process_wait)
    );
    kernel.remove_blocked_waiter(process_wait).unwrap();
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
    let current_process_wait = kernel.register_blocked_waiter(
        9,
        0x109,
        vec![CE_CURRENT_PROCESS_PSEUDO_HANDLE],
        SchedulerBlockedWaitKind::Kernel,
        0,
        INFINITE,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TERMINATE_PROCESS,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x77],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.select_ready_blocked_waiter(thread_id, 0, |blocked, kernel| {
            blocked
                .wait_handles
                .iter()
                .any(|handle| kernel.is_wait_ready(*handle, blocked.thread_id) == Some(true))
        }),
        Some(current_process_wait)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0],
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
            ORD_GET_EXIT_CODE_PROCESS,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, process_exit_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(process_exit_ptr)?, 0x77);
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

    let foreign_mutex = kernel.create_mutex_w(None, Some(99));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_MUTEX,
            [foreign_mutex],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_NOT_OWNER);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_MUTEX,
            [0xdead_beef],
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
    memory.map_words(handles_ptr, 2);
    memory.write_word(handles_ptr, event);
    memory.write_word(handles_ptr + 4, 0xdead_beef);
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
            ORD_WAIT_FOR_MULTIPLE_OBJECTS,
            [2, handles_ptr, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_FAILED),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_HANDLE
    );
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

#[test]
fn coredll_raw_open_event_w_opens_existing_named_event_only() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let event_name = 0x4000;
    let event_all_access = 0x001f_0003;
    memory.map_halfwords(event_name, 32);
    memory.write_wide_z(event_name, "WAIT_EXPLORER");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_EVENT_W,
            [event_all_access, 0, event_name],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );

    let created = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_EVENT_W,
        [0, 1, 0, event_name],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateEventW did not return a handle: {other:?}"),
    };
    assert_ne!(created, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_EVENT_W,
            [event_all_access, 0, event_name],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == created
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_EVENT_W,
            [event_all_access, 1, event_name],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    Ok(())
}

#[test]
fn coredll_raw_shget_special_folder_path_returns_ce_paths() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x5000;
    memory.map_halfwords(path_ptr, 260);

    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "Windows",
        RegistryValue::string(r"\ResidentFlash\Windows"),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, 0x0024, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(path_ptr, 260), r"\ResidentFlash\Windows");
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, 0xffff, 0],
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
    Ok(())
}

#[test]
fn coredll_raw_module_apis_resolve_preloaded_search_dll_exports() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 41;
    let module_name_ptr = 0x1_8000;
    let proc_w_ptr = 0x1_8040;
    let proc_a_ptr = 0x1_8080;
    let module_base = 0x6200_0000;
    let proc_by_name = 0x6200_1234;
    let proc_by_ordinal = 0x6200_5678;

    let mut exports_by_name = std::collections::BTreeMap::new();
    exports_by_name.insert("InitCommonControlsEx".to_owned(), proc_by_name);
    let mut exports_by_ordinal = std::collections::BTreeMap::new();
    exports_by_ordinal.insert(17, proc_by_ordinal);
    kernel.register_loaded_module(
        "commctrl.dll",
        module_base,
        exports_by_name,
        exports_by_ordinal,
    );
    memory.write_wide_z(module_name_ptr, "commctrl.dll");
    memory.write_wide_z(proc_w_ptr, "InitCommonControlsEx");
    memory.write_bytes(proc_a_ptr, b"InitCommonControlsEx\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_W,
            [module_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == module_base
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MODULE_HANDLE_W,
            [module_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == module_base
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_W,
            [module_base, proc_w_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == proc_by_name
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_A,
            [module_base, proc_a_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == proc_by_name
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_A,
            [module_base, 17],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == proc_by_ordinal
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FREE_LIBRARY,
            [module_base],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_file_time_to_system_time_round_trips_guest_fields() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 17;
    let source_system_time = 0x1000;
    let file_time = 0x1100;
    let dest_system_time = 0x1200;
    memory.map_halfwords(source_system_time, 8);
    memory.map_words(file_time, 2);
    memory.map_halfwords(dest_system_time, 8);
    memory.write_halfword(source_system_time, 2024);
    memory.write_halfword(source_system_time + 2, 2);
    memory.write_halfword(source_system_time + 4, 4);
    memory.write_halfword(source_system_time + 6, 29);
    memory.write_halfword(source_system_time + 8, 12);
    memory.write_halfword(source_system_time + 10, 34);
    memory.write_halfword(source_system_time + 12, 56);
    memory.write_halfword(source_system_time + 14, 789);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SYSTEM_TIME_TO_FILE_TIME,
            [source_system_time, file_time],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(memory.read_u32(file_time + 4)? != 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FILE_TIME_TO_SYSTEM_TIME,
            [file_time, dest_system_time],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u16(dest_system_time)?, 2024);
    assert_eq!(memory.read_u16(dest_system_time + 2)?, 2);
    assert_eq!(memory.read_u16(dest_system_time + 4)?, 4);
    assert_eq!(memory.read_u16(dest_system_time + 6)?, 29);
    assert_eq!(memory.read_u16(dest_system_time + 8)?, 12);
    assert_eq!(memory.read_u16(dest_system_time + 10)?, 34);
    assert_eq!(memory.read_u16(dest_system_time + 12)?, 56);
    assert_eq!(memory.read_u16(dest_system_time + 14)?, 789);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_kernel_io_control_returns_device_id() -> Result<()> {
    const IOCTL_HAL_GET_DEVICEID: u32 = 0x0101_207c;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 18;
    let out = 0x1400;
    let returned = 0x1500;
    memory.map_words(out, 5);
    memory.map_bytes(out + 20, 64);
    memory.map_words(returned, 1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERNEL_IO_CONTROL,
            [IOCTL_HAL_GET_DEVICEID, 0, 0, out, 4, returned],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(memory.read_u32(out)? > 20);

    let required_size = memory.read_u32(out)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERNEL_IO_CONTROL,
            [IOCTL_HAL_GET_DEVICEID, 0, 0, out, required_size, returned],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(out)?, required_size);
    assert_eq!(memory.read_u32(out + 4)?, 20);
    assert_eq!(memory.read_u32(out + 8)?, 10);
    assert_eq!(memory.read_u32(out + 12)?, 30);
    assert_eq!(memory.read_u32(out + 16)?, 11);
    assert_eq!(memory.read_bytes(out + 20, 10), b"WINCE_EMU\0");
    assert_eq!(memory.read_bytes(out + 30, 11), b"INAVI_HOST\0");
    assert_eq!(memory.read_u32(returned)?, required_size);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_multibyte_to_wide_char_uses_korean_acp() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let input = 0x1000;
    let output = 0x2000;
    memory.write_bytes(input, &[0xb0, 0xa1, 0]);
    memory.map_halfwords(output, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MULTI_BYTE_TO_WIDE_CHAR,
            [0, 0, input, u32::MAX, output, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(output, 4), "가");

    Ok(())
}

#[test]
fn coredll_input_debug_char_w_formats_when_called_like_wsprintf() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let dest = 0x1000;
    let format = 0x2000;
    let source = 0x3000;
    memory.map_halfwords(dest, 32);
    memory.write_wide_z(format, "%s");
    memory.write_wide_z(source, "\\SDMMC Disk\\INavi\\iNavi.exe");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INPUT_DEBUG_CHAR_W,
            [dest, format, source],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(27),
            ..
        }
    ));
    assert_eq!(
        memory.read_wide_z(dest, 32),
        "\\SDMMC Disk\\INavi\\iNavi.exe"
    );

    Ok(())
}

#[test]
fn coredll_raw_msgwait_requires_new_input_unless_inputavailable() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 77;

    kernel.gwe.post_message(
        thread_id,
        Message::new(0, 0x400 + 77, 0x77, 0, kernel.timers.tick_count()),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [0, 0, 1000, QS_POSTMESSAGE, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));
    assert!(kernel.gwe.has_queue_input(thread_id, QS_POSTMESSAGE));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [0, 0, 0, QS_POSTMESSAGE, 0],
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
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [0, 0, 0, QS_POSTMESSAGE, 0x0004],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_adb_account_setter_is_not_exported_by_current_map() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            1943,
            [0x0054_310e, 0x0054_310c, 0x7ffd_efd0, 0x0079_4a3c],
        ),
        CoredllDispatch::UnresolvedOrdinal(1943)
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}
