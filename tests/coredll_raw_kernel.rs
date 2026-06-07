use std::fs;

use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_CE_GET_THREAD_PRIORITY, ORD_CE_SET_THREAD_PRIORITY, ORD_CLEAR_COMM_ERROR,
            ORD_CLOSE_CLIPBOARD, ORD_CLOSE_HANDLE, ORD_COUNT_CLIPBOARD_FORMATS, ORD_CREATE_EVENT_W,
            ORD_CREATE_SEMAPHORE_W, ORD_CREATE_THREAD, ORD_DISABLE_THREAD_LIBRARY_CALLS,
            ORD_EMPTY_CLIPBOARD, ORD_ENUM_CLIPBOARD_FORMATS, ORD_EVENT_MODIFY,
            ORD_FILE_TIME_TO_SYSTEM_TIME, ORD_FREE_LIBRARY, ORD_GET_CALLER_PROCESS_INDEX,
            ORD_GET_CLIPBOARD_DATA, ORD_GET_CLIPBOARD_FORMAT_NAME_W, ORD_GET_CLIPBOARD_OWNER,
            ORD_GET_COMM_MASK, ORD_GET_COMM_STATE, ORD_GET_COMM_TIMEOUTS,
            ORD_GET_EXIT_CODE_PROCESS, ORD_GET_EXIT_CODE_THREAD, ORD_GET_LAST_ERROR,
            ORD_GET_LOCAL_TIME, ORD_GET_MODULE_HANDLE_W, ORD_GET_OPEN_CLIPBOARD_WINDOW,
            ORD_GET_PRIORITY_CLIPBOARD_FORMAT, ORD_GET_PROC_ADDRESS_A, ORD_GET_PROC_ADDRESS_W,
            ORD_GET_PROCESS_ID, ORD_GET_PROCESS_IDFROM_INDEX, ORD_GET_PROCESS_INDEX_FROM_ID,
            ORD_GET_PROCESS_VERSION, ORD_GET_STORE_INFORMATION, ORD_GET_SYSTEM_TIME,
            ORD_GET_SYSTEM_TIME_AS_FILE_TIME, ORD_GET_THREAD_ID, ORD_GET_THREAD_PRIORITY,
            ORD_GET_THREAD_TIMES, ORD_GET_TICK_COUNT, ORD_GET_TIME_ZONE_INFORMATION,
            ORD_GET_VERSION_EX_W, ORD_INITIALIZE_CRITICAL_SECTION, ORD_INPUT_DEBUG_CHAR_W,
            ORD_INTERLOCKED_COMPARE_EXCHANGE, ORD_INTERLOCKED_EXCHANGE_ADD,
            ORD_INTERLOCKED_INCREMENT, ORD_IS_CLIPBOARD_FORMAT_AVAILABLE, ORD_KERNEL_IO_CONTROL,
            ORD_LEAVE_CRITICAL_SECTION, ORD_LOAD_LIBRARY_EX_W, ORD_LOAD_LIBRARY_W, ORD_MBSTOWCS,
            ORD_MESSAGE_BOX_W, ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX, ORD_MULTI_BYTE_TO_WIDE_CHAR,
            ORD_OPEN_CLIPBOARD, ORD_OPEN_EVENT_W, ORD_PURGE_COMM, ORD_QUERY_PERFORMANCE_COUNTER,
            ORD_QUERY_PERFORMANCE_FREQUENCY, ORD_REGISTER_CLIPBOARD_FORMAT_W, ORD_RELEASE_MUTEX,
            ORD_RELEASE_SEMAPHORE, ORD_RESUME_THREAD, ORD_SET_CLIPBOARD_DATA, ORD_SET_COMM_MASK,
            ORD_SET_COMM_STATE, ORD_SET_COMM_TIMEOUTS, ORD_SET_LAST_ERROR, ORD_SET_THREAD_PRIORITY,
            ORD_SHELL_EXECUTE_EX, ORD_SHELL_NOTIFY_ICON, ORD_SHGET_FILE_INFO,
            ORD_SHGET_SPECIAL_FOLDER_PATH, ORD_SHNOTIFICATION_ADD_I, ORD_SHNOTIFICATION_GET_DATA_I,
            ORD_SHNOTIFICATION_REMOVE_I, ORD_SHNOTIFICATION_UPDATE_I, ORD_SLEEP,
            ORD_SLEEP_TILL_TICK, ORD_SUSPEND_THREAD, ORD_SYSTEM_TIME_TO_FILE_TIME,
            ORD_TERMINATE_PROCESS, ORD_TLS_GET_VALUE, ORD_TLS_SET_VALUE,
            ORD_TRY_ENTER_CRITICAL_SECTION, ORD_WAIT_COMM_EVENT, ORD_WAIT_FOR_MULTIPLE_OBJECTS,
            ORD_WAIT_FOR_SINGLE_OBJECT, ORD_WCSTOMBS, ORD_WIDE_CHAR_TO_MULTI_BYTE,
        },
        devices::CommDcb,
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        gwe::{Message, QS_POSTMESSAGE, QS_TIMER, WM_TIMER, WM_USER},
        kernel::{
            CE_CURRENT_PROCESS_PSEUDO_HANDLE, CE_CURRENT_THREAD_PSEUDO_HANDLE, CeKernel,
            LoadedModuleMetadata,
        },
        object::MAX_SUSPEND_COUNT,
        registry::{ERROR_SUCCESS, RegistryValue},
        scheduler::SchedulerBlockedWaitKind,
        thread::{
            ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE,
            ERROR_INVALID_PARAMETER, ERROR_INVALID_WINDOW_HANDLE, ERROR_NOT_OWNER,
            ERROR_NOT_SUPPORTED, ERROR_SIGNAL_REFUSED,
        },
        timer::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
};

mod support;
use support::{TestGuestMemory, unique_test_root};

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
    let launch_process_index = launch.process_id.saturating_sub(0x42).saturating_add(1);
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
            value: CoredllValue::U32(index),
            ..
        } if index == launch_process_index
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
            ORD_GET_PROCESS_INDEX_FROM_ID,
            [launch.process_handle],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(index),
            ..
        } if index == launch_process_index
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROCESS_IDFROM_INDEX,
            [launch_process_index],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(id),
            ..
        } if id == launch.process_id
    ));
    kernel.set_current_process_id(launch.process_id);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROCESS_INDEX_FROM_ID,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(index),
            ..
        } if index == launch_process_index
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALLER_PROCESS_INDEX,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(index),
            ..
        } if index == launch_process_index
    ));
    kernel.set_current_process_id(1);
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
    let stats = kernel.runtime_loader_stats();
    assert_eq!(stats.export_lookup_count, 3);
    assert_eq!(stats.export_lookup_miss_count, 0);

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
fn coredll_raw_loadlibrary_refcounts_dynamic_modules_and_ex_flags_reuse_loaded_modules()
-> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 42;
    let module_name_ptr = 0x1_9000;
    let module_base = 0x6300_0000;

    kernel.register_loaded_module_with_metadata(
        "dynamic.dll",
        module_base,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\dynamic.dll".to_owned()),
            image_size: 0x12000,
            ..LoadedModuleMetadata::default()
        },
    );
    memory.write_wide_z(module_name_ptr, "dynamic.dll");

    for _ in 0..2 {
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
    }
    assert_eq!(kernel.loaded_module_snapshots()[0].ref_count, 3);

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
    assert_eq!(kernel.loaded_module_snapshots()[0].ref_count, 2);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_EX_W,
            [module_name_ptr, 0, 0x0000_0002],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == module_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(kernel.loaded_module_snapshots()[0].ref_count, 3);

    let missing_name_ptr = 0x1_9040;
    memory.write_wide_z(missing_name_ptr, "missing.dll");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_EX_W,
            [missing_name_ptr, 0, 0x0000_0003],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_EX_W,
            [module_name_ptr, 0, 0x0000_0040],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    Ok(())
}

#[test]
fn coredll_raw_disable_thread_library_calls_validates_module_handles() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 42;
    let module_base = 0x6400_0000;

    kernel.register_loaded_module_with_metadata(
        "loaded.dll",
        module_base,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            ..LoadedModuleMetadata::default()
        },
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISABLE_THREAD_LIBRARY_CALLS,
            [module_base],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISABLE_THREAD_LIBRARY_CALLS,
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
    Ok(())
}

#[test]
fn shell_execute_ex_resolves_registry_association_and_queues_process() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_assoc");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    kernel.set_file_root(&root);
    kernel
        .registry
        .set_value(r"HKCR\.nav", "", RegistryValue::string("navfile"));
    kernel.registry.set_value(
        r"HKCR\navfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" "%1""#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 43;
    let info = 0x2_0000;
    let file_ptr = 0x2_0100;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 4, 0x0000_0040);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(file_ptr, r"\Docs\route.nav");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_EXECUTE_EX,
            [info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, 33);
    assert_ne!(memory.read_u32(info + 56)?, 0);
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].application.as_deref(),
        Some(r"\Windows\viewer.exe")
    );
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\viewer.exe" "\Docs\route.nav""#)
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn sh_get_file_info_uses_registry_associations_and_attributes() -> Result<()> {
    const SHFILEINFO_SIZE_W: u32 = 692;
    const SHFILEINFO_IICON_OFFSET: u32 = 4;
    const SHFILEINFO_ATTRIBUTES_OFFSET: u32 = 8;
    const SHFILEINFO_DISPLAY_NAME_OFFSET: u32 = 12;
    const SHFILEINFO_TYPE_NAME_OFFSET: u32 = 532;
    const SHGFI_DISPLAYNAME: u32 = 0x0000_0200;
    const SHGFI_TYPENAME: u32 = 0x0000_0400;
    const SHGFI_ATTRIBUTES: u32 = 0x0000_0800;
    const SHGFI_ICONLOCATION: u32 = 0x0000_1000;
    const SHGFI_USEFILEATTRIBUTES: u32 = 0x0000_0010;
    const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel
        .registry
        .set_value(r"HKCR\.nav", "", RegistryValue::string("navfile"));
    kernel
        .registry
        .set_value(r"HKCR\navfile", "", RegistryValue::string("Route Plan"));
    kernel.registry.set_value(
        r"HKCR\navfile\DefaultIcon",
        "",
        RegistryValue::string(r"\Windows\navicons.dll,-7"),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 45;
    let path_ptr = 0x2_4000;
    let info_ptr = 0x2_5000;
    memory.map_halfwords(path_ptr, 64);
    memory.write_wide_z(path_ptr, r"\Docs\morning.nav");
    memory.map_words(info_ptr, SHFILEINFO_SIZE_W / 4);
    memory.map_halfwords(info_ptr + SHFILEINFO_DISPLAY_NAME_OFFSET, 260);
    memory.map_halfwords(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80);

    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            FILE_ATTRIBUTE_ARCHIVE,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_USEFILEATTRIBUTES
                | SHGFI_DISPLAYNAME
                | SHGFI_TYPENAME
                | SHGFI_ATTRIBUTES
                | SHGFI_ICONLOCATION,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET)?,
        FILE_ATTRIBUTE_ARCHIVE
    );
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, -7);
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_DISPLAY_NAME_OFFSET, 260),
        r"\Windows\navicons.dll"
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
        "Route Plan"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn shell_notify_icon_tracks_add_modify_delete_state() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIM_MODIFY: u32 = 1;
    const NIM_DELETE: u32 = 2;
    const NIF_MESSAGE: u32 = 0x0000_0001;
    const NIF_ICON: u32 = 0x0000_0002;
    const NIF_TIP: u32 = 0x0000_0004;
    const NIF_STATE: u32 = 0x0000_0008;
    const NID_SIZE: u32 = 160;
    const NID_TIP_OFFSET: u32 = 24;
    const NID_STATE_OFFSET: u32 = 152;
    const NID_STATE_MASK_OFFSET: u32 = 156;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHELL_NOTIFY", "", None, 0, 0, 0);
    let data = 0x2_9000;
    memory.map_words(data, NID_SIZE / 4);
    memory.map_halfwords(data + NID_TIP_OFFSET, 64);
    memory.write_word(data, NID_SIZE);
    memory.write_word(data + 4, hwnd);
    memory.write_word(data + 8, 77);
    memory.write_word(data + 12, NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_STATE);
    memory.write_word(data + 16, WM_USER + 88);
    memory.write_word(data + 20, 0x000b_8001);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Route ready");
    memory.write_word(data + NID_STATE_OFFSET, 0x2);
    memory.write_word(data + NID_STATE_MASK_OFFSET, 0xffff_ffff);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, data],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let icon = kernel.shell.notify_icon(hwnd, 77).expect("notify icon");
    assert_eq!(icon.callback_message, WM_USER + 88);
    assert_eq!(icon.icon, 0x000b_8001);
    assert_eq!(icon.tip, "Route ready");
    assert_eq!(icon.state, 0x2);

    memory.write_word(data + 12, NIF_TIP | NIF_STATE);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Route updated");
    memory.write_word(data + NID_STATE_OFFSET, 0);
    memory.write_word(data + NID_STATE_MASK_OFFSET, 0x2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_MODIFY, data],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let icon = kernel.shell.notify_icon(hwnd, 77).expect("notify icon");
    assert_eq!(icon.callback_message, WM_USER + 88);
    assert_eq!(icon.icon, 0x000b_8001);
    assert_eq!(icon.tip, "Route updated");
    assert_eq!(icon.state, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_DELETE, data],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.shell.notify_icon(hwnd, 77).is_none());
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn shnotification_i_tracks_query_update_and_remove_state() -> Result<()> {
    const ERROR_INVALID_DATA: u32 = 13;
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_INFORM: u32 = 0x1b1;
    const SHNP_ICONIC: u32 = 0x1b2;
    const SHNUM_PRIORITY: u32 = 0x0001;
    const SHNUM_DURATION: u32 = 0x0002;
    const SHNUM_ICON: u32 = 0x0004;
    const SHNUM_HTML: u32 = 0x0008;
    const SHNUM_TITLE: u32 = 0x0010;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHN_SINK", "", None, 0, 0, 0);
    let data = 0x3002_a000;
    let title = 0x3002_b000;
    let html = 0x3002_c000;
    let out = 0x3002_d000;
    let out_title = 0x3002_e000;
    let out_html = 0x3002_f000;
    let html_len = 0x3002_f800;
    let clsid = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x44, 0x33, 0x22, 0x11, 0xaa, 0xbb, 0xcc,
        0xdd,
    ];
    memory.map_words(data, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(title, 64);
    memory.map_bytes(html, 128);
    memory.map_words(out, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(out_title, 64);
    memory.map_bytes(out_html, 128);
    memory.map_words(html_len, 1);
    memory.write_word(data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(data + 4, 301);
    memory.write_word(data + 8, SHNP_INFORM);
    memory.write_word(data + 12, 0);
    memory.write_word(data + 16, 0x000b_9001);
    memory.write_word(data + 20, 0x10);
    memory.write_bytes(data + 24, &clsid);
    memory.write_word(data + 40, hwnd);
    memory.write_word(data + 52, 0xCAFE_BABE);
    memory.write_wide_z(title, "Route alert");
    memory.write_wide_z(html, "<b>Drive now</b>");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_ADD_I,
            [data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.title, "Route alert");
    assert_eq!(record.html, "<b>Drive now</b>");
    assert_eq!(record.duration_cs, 5);

    memory.write_word(html_len, 8);
    let get_result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHNOTIFICATION_GET_DATA_I,
        [
            data + 24,
            16,
            301,
            out,
            SHNOTIFICATIONDATA_SIZE,
            out_title,
            64,
            out_html,
            128,
            html_len,
        ],
    );
    match get_result {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } => assert_eq!(value, ERROR_SUCCESS),
        other => panic!("unexpected SHNotificationGetDataI result: {other:?}"),
    }
    assert_eq!(memory.read_u32(out)?, SHNOTIFICATIONDATA_SIZE);
    assert_eq!(memory.read_u32(out + 4)?, 301);
    assert_eq!(memory.read_u32(out + 8)?, SHNP_INFORM);
    assert_eq!(memory.read_u32(out + 12)?, 5);
    assert_eq!(memory.read_u32(out + 16)?, 0x000b_9001);
    assert_eq!(memory.read_u32(out + 40)?, hwnd);
    assert_eq!(memory.read_u32(out + 52)?, 0xCAFE_BABE);
    assert_eq!(memory.read_wide_z(out_title, 32), "Route alert");
    assert_eq!(memory.read_wide_z(out_html, 32), "<b>Driv");

    memory.write_word(data + 8, SHNP_ICONIC);
    memory.write_word(data + 12, 3);
    memory.write_word(data + 16, 0x000b_9002);
    memory.write_wide_z(title, "Route changed");
    memory.write_wide_z(html, "<i>Later</i>");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [
                SHNUM_PRIORITY | SHNUM_DURATION | SHNUM_ICON | SHNUM_TITLE | SHNUM_HTML,
                data,
                SHNOTIFICATIONDATA_SIZE,
                title,
                html
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.priority, SHNP_ICONIC);
    assert_eq!(record.duration_cs, 3);
    assert_eq!(record.icon, 0x000b_9002);
    assert_eq!(record.title, "Route changed");
    assert_eq!(record.html, "<i>Later</i>");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_REMOVE_I,
            [data + 24, 16, 301],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert!(kernel.shell.notification(clsid, 301).is_none());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_GET_DATA_I,
            [
                data + 24,
                16,
                301,
                out,
                SHNOTIFICATIONDATA_SIZE,
                0,
                0,
                0,
                0,
                html_len
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_DATA),
            ..
        }
    ));

    Ok(())
}

#[test]
fn shell_window_destroy_removes_notify_icon_and_notification_state() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIF_MESSAGE: u32 = 0x0000_0001;
    const NIF_ICON: u32 = 0x0000_0002;
    const NIF_TIP: u32 = 0x0000_0004;
    const NID_SIZE: u32 = 160;
    const NID_TIP_OFFSET: u32 = 24;
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_INFORM: u32 = 0x1b1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 48;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHN_CLEANUP", "", None, 0, 0, 0);
    let notify_icon = 0x3003_0000;
    let notify_data = 0x3003_1000;
    let title = 0x3003_2000;
    let html = 0x3003_3000;
    let clsid = [
        0x44, 0x33, 0x22, 0x11, 0xaa, 0xbb, 0xcc, 0xdd, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x20, 0x30,
        0x40,
    ];
    memory.write_word(notify_icon, NID_SIZE);
    memory.write_word(notify_icon + 4, hwnd);
    memory.write_word(notify_icon + 8, 11);
    memory.write_word(notify_icon + 12, NIF_MESSAGE | NIF_ICON | NIF_TIP);
    memory.write_word(notify_icon + 16, WM_USER + 11);
    memory.write_word(notify_icon + 20, 0x000b_8002);
    memory.write_wide_z(notify_icon + NID_TIP_OFFSET, "cleanup");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, notify_icon],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    memory.write_word(notify_data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(notify_data + 4, 402);
    memory.write_word(notify_data + 8, SHNP_INFORM);
    memory.write_word(notify_data + 12, 0);
    memory.write_word(notify_data + 16, 0x000b_9003);
    memory.write_word(notify_data + 20, 0);
    memory.write_bytes(notify_data + 24, &clsid);
    memory.write_word(notify_data + 40, hwnd);
    memory.write_word(notify_data + 52, 0);
    memory.write_wide_z(title, "Cleanup");
    memory.write_wide_z(html, "<p>Cleanup</p>");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_ADD_I,
            [notify_data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert!(kernel.shell.notify_icon(hwnd, 11).is_some());
    assert!(kernel.shell.notification(clsid, 402).is_some());

    assert!(kernel.destroy_window(hwnd));

    assert!(kernel.shell.notify_icon(hwnd, 11).is_none());
    assert!(kernel.shell.notification(clsid, 402).is_none());

    Ok(())
}

#[test]
fn message_box_w_records_text_owner_and_returns_default_button() -> Result<()> {
    const MB_YESNOCANCEL: u32 = 0x0000_0003;
    const MB_DEFBUTTON2: u32 = 0x0000_0100;
    const MB_ICONQUESTION: u32 = 0x0000_0020;
    const IDNO: u32 = 7;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 49;
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGBOX_OWNER", "", None, 0, 0, 0);
    let text = 0x3004_0000;
    let caption = 0x3004_1000;
    memory.write_wide_z(text, "Route search failed");
    memory.write_wide_z(caption, "iNavi");

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [
            hwnd,
            text,
            caption,
            MB_YESNOCANCEL | MB_DEFBUTTON2 | MB_ICONQUESTION,
        ],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    let record = kernel.shell.last_message_box().expect("message box record");
    assert_eq!(record.thread_id, thread_id);
    assert_eq!(record.owner_hwnd, hwnd);
    assert_eq!(record.text, "Route search failed");
    assert_eq!(record.caption, "iNavi");
    assert_eq!(
        record.style,
        MB_YESNOCANCEL | MB_DEFBUTTON2 | MB_ICONQUESTION
    );
    assert_eq!(record.result, IDNO);
    assert_eq!(record.owner_was_enabled, Some(true));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn message_box_w_rejects_destroyed_owner_without_recording() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 50;
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGBOX_DEAD_OWNER", "", None, 0, 0, 0);
    assert!(kernel.destroy_window_with_reason(hwnd, "test"));
    memory.write_wide_z(0x3005_0000, "dead owner");

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, 0x3005_0000, 0, 0],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(kernel.shell.last_message_box().is_none());
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_WINDOW_HANDLE
    );

    Ok(())
}

#[test]
fn clipboard_raw_ordinals_track_lock_owner_formats_and_names() -> Result<()> {
    const CF_TEXT: u32 = 1;
    const CF_UNICODETEXT: u32 = 13;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 51;
    let owner = kernel.create_window_ex_w(thread_id, "CLIP_OWNER", "", None, 0, 0, 0);
    let other = kernel.create_window_ex_w(thread_id, "CLIP_OTHER", "", None, 0, 0, 0);
    let formats = 0x3006_0000;
    let format_name = 0x3006_1000;
    let out_name = 0x3006_2000;
    memory.write_word(formats, CF_TEXT);
    memory.write_word(formats + 4, CF_UNICODETEXT);
    memory.write_wide_z(format_name, "iNavi.Route");
    memory.map_halfwords(out_name, 64);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_CLIPBOARD,
            [owner],
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
            ORD_OPEN_CLIPBOARD,
            [other],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ACCESS_DENIED
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_OPEN_CLIPBOARD_WINDOW,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == owner
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EMPTY_CLIPBOARD,
            [],
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
            ORD_GET_CLIPBOARD_OWNER,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == owner
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CLIPBOARD_DATA,
            [CF_UNICODETEXT, 0x7000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7000),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CLIPBOARD_DATA,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7000),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_CLIPBOARD_FORMAT_AVAILABLE,
            [CF_UNICODETEXT],
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
            ORD_COUNT_CLIPBOARD_FORMATS,
            [],
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
            ORD_ENUM_CLIPBOARD_FORMATS,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(CF_UNICODETEXT),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PRIORITY_CLIPBOARD_FORMAT,
            [formats, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(CF_UNICODETEXT),
            ..
        }
    ));

    let custom = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTER_CLIPBOARD_FORMAT_W,
        [format_name],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(format),
            ..
        } => format,
        other => panic!("unexpected RegisterClipboardFormatW result: {other:?}"),
    };
    assert!(custom >= 0xc000);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_CLIPBOARD_FORMAT_W,
            [format_name],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(format),
            ..
        } if format == custom
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CLIPBOARD_FORMAT_NAME_W,
            [custom, out_name, 64],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(11),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(out_name, 64), "iNavi.Route");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CLIPBOARD_DATA,
            [custom, 0x9000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x9000),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_COUNT_CLIPBOARD_FORMATS,
            [],
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
            ORD_CLOSE_CLIPBOARD,
            [],
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
            ORD_GET_CLIPBOARD_DATA,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ACCESS_DENIED
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_CLIPBOARD,
            [0],
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
            ORD_GET_OPEN_CLIPBOARD_WINDOW,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CLIPBOARD_DATA,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7000),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_CLIPBOARD,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}

#[test]
fn clipboard_raw_ordinals_reject_invalid_open_and_missing_lock() -> Result<()> {
    const CF_UNICODETEXT: u32 = 13;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 52;
    let hwnd = kernel.create_window_ex_w(thread_id, "CLIP_DEAD", "", None, 0, 0, 0);
    assert!(kernel.destroy_window_with_reason(hwnd, "test"));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_CLIPBOARD,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_WINDOW_HANDLE
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EMPTY_CLIPBOARD,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ACCESS_DENIED
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CLIPBOARD_DATA,
            [CF_UNICODETEXT, 0x7000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ACCESS_DENIED
    );

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
fn coredll_wide_char_to_multi_byte_uses_korean_acp() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let input = 0x1000;
    let output = 0x2000;
    let used_default = 0x3000;
    memory.write_wide_z(input, "운전");
    memory.map_bytes(output, 8);
    memory.map_words(used_default, 1);
    memory.write_word(used_default, 0xffff_ffff);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WIDE_CHAR_TO_MULTI_BYTE,
            [0, 0, input, u32::MAX, output, 8, 0, used_default],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(5),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(output, 5), [0xbf, 0xee, 0xc0, 0xfc, 0]);
    assert_eq!(memory.read_u32(used_default)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_crt_mbstowcs_and_wcstombs_use_korean_acp() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let narrow = 0x1000;
    let wide = 0x2000;
    let round_trip = 0x3000;
    memory.write_bytes(narrow, &[0xbf, 0xee, 0xc0, 0xfc, 0]);
    memory.map_halfwords(wide, 4);
    memory.map_bytes(round_trip, 8);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MBSTOWCS,
            [0, narrow, 0],
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
            ORD_MBSTOWCS,
            [wide, narrow, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(wide, 4), "운전");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSTOMBS,
            [0, wide, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(4),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSTOMBS,
            [round_trip, wide, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(4),
            ..
        }
    ));
    assert_eq!(
        memory.read_bytes(round_trip, 5),
        [0xbf, 0xee, 0xc0, 0xfc, 0]
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

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
fn coredll_raw_msgwait_wakes_for_timer_due_inside_timeout() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 78;
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGWAIT_TIMER", "", None, 0, 0, 0);

    assert_eq!(
        kernel.set_timer_for_thread(thread_id, Some(hwnd), Some(78), 5, None),
        78
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [0, 0, 1000, QS_TIMER, 0x0004],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));

    let timer = kernel.gwe.get_message(thread_id).unwrap();
    assert_eq!(timer.hwnd, hwnd);
    assert_eq!(timer.msg, WM_TIMER);
    assert_eq!(timer.wparam, 78);

    let mut kernel = CeKernel::boot(RuntimeConfig::load("regs.json", "serial_devices.json")?);
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGWAIT_TIMER_LATE", "", None, 0, 0, 0);
    assert_eq!(
        kernel.set_timer_for_thread(thread_id, Some(hwnd), Some(79), 1000, None),
        79
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [0, 0, 5, QS_TIMER, 0x0004],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_TIMEOUT),
            ..
        }
    ));
    assert!(kernel.gwe.get_message(thread_id).is_none());

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

#[test]
fn coredll_raw_comm_timeouts_round_trip_on_serial_handle() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let com = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
    let timeouts_ptr = 0x3100_0000;
    memory.map_words(timeouts_ptr, 5);
    memory.write_word(timeouts_ptr, 50);
    memory.write_word(timeouts_ptr + 4, 2);
    memory.write_word(timeouts_ptr + 8, 10);
    memory.write_word(timeouts_ptr + 12, 3);
    memory.write_word(timeouts_ptr + 16, 11);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_COMM_TIMEOUTS,
            [com, timeouts_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    for offset in [0, 4, 8, 12, 16] {
        memory.write_word(timeouts_ptr + offset, 0);
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_COMM_TIMEOUTS,
            [com, timeouts_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(timeouts_ptr)?, 50);
    assert_eq!(memory.read_u32(timeouts_ptr + 4)?, 2);
    assert_eq!(memory.read_u32(timeouts_ptr + 8)?, 10);
    assert_eq!(memory.read_u32(timeouts_ptr + 12)?, 3);
    assert_eq!(memory.read_u32(timeouts_ptr + 16)?, 11);
    assert_eq!(kernel.serial_empty_read_timeout_ms(com, 4), Some(18));

    Ok(())
}

#[test]
fn coredll_raw_comm_state_mask_wait_and_purge_are_stateful() -> Result<()> {
    const EV_RXCHAR: u32 = 0x0001;
    const EV_ERR: u32 = 0x0080;
    const PURGE_TXCLEAR: u32 = 0x0004;
    const PURGE_RXCLEAR: u32 = 0x0008;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let com = kernel.create_file_w("COM7:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
    let dcb_ptr = 0x3100_1000;
    memory.map_bytes(dcb_ptr, CommDcb::SIZE as u32);
    let mut dcb = [0u8; CommDcb::SIZE];
    dcb[0..4].copy_from_slice(&(CommDcb::SIZE as u32).to_le_bytes());
    dcb[4..8].copy_from_slice(&57600u32.to_le_bytes());
    dcb[8..12].copy_from_slice(&1u32.to_le_bytes());
    dcb[18] = 7;
    dcb[19] = 1;
    dcb[20] = 2;
    memory.write_bytes(dcb_ptr, &dcb);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_COMM_STATE,
            [com, dcb_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    memory.write_bytes(dcb_ptr, &[0; CommDcb::SIZE]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_COMM_STATE,
            [com, dcb_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(dcb_ptr, CommDcb::SIZE), dcb);

    let mask_ptr = 0x3100_2000;
    memory.map_words(mask_ptr, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_COMM_MASK,
            [com, EV_RXCHAR | EV_ERR]
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
            ORD_GET_COMM_MASK,
            [com, mask_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(mask_ptr)?, EV_RXCHAR | EV_ERR);

    kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "nmea",
        "sentences": ["$GPRMC,raw*00"]
    }));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_COMM_EVENT,
            [com, mask_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(mask_ptr)?, EV_RXCHAR);

    assert_eq!(kernel.read_file(com, 4)?.len(), 4);
    assert_eq!(kernel.write_file(com, b"$PUBX")?.bytes_transferred, 5);
    let stat_ptr = 0x3100_3000;
    let errors_ptr = 0x3100_4000;
    memory.map_words(stat_ptr, 3);
    memory.map_words(errors_ptr, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLEAR_COMM_ERROR,
            [com, errors_ptr, stat_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(errors_ptr)?, 0);
    assert!(memory.read_u32(stat_ptr + 4)? > 0);
    assert_eq!(memory.read_u32(stat_ptr + 8)?, 5);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PURGE_COMM,
            [com, PURGE_RXCLEAR | PURGE_TXCLEAR]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.comm_queue_lengths(com)?, (0, 0));

    Ok(())
}
