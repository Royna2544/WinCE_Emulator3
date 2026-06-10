use std::fs;

use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_CE_GET_THREAD_PRIORITY, ORD_CE_SET_THREAD_PRIORITY, ORD_CLEAR_COMM_ERROR,
            ORD_CLOSE_CLIPBOARD, ORD_CLOSE_HANDLE, ORD_COUNT_CLIPBOARD_FORMATS,
            ORD_CREATE_COMPATIBLE_DC, ORD_CREATE_DIBSECTION, ORD_CREATE_DIRECTORY_W,
            ORD_CREATE_EVENT_W, ORD_CREATE_FILE_W, ORD_CREATE_PROCESS_W, ORD_CREATE_SEMAPHORE_W,
            ORD_CREATE_THREAD, ORD_DELETE_CRITICAL_SECTION, ORD_DISABLE_THREAD_LIBRARY_CALLS,
            ORD_DISPATCH_MESSAGE_W, ORD_EMPTY_CLIPBOARD, ORD_ENTER_CRITICAL_SECTION,
            ORD_ENUM_CLIPBOARD_FORMATS, ORD_EVENT_MODIFY, ORD_EXTRACT_ICON_EX_W,
            ORD_FILE_TIME_TO_SYSTEM_TIME, ORD_FREE_LIBRARY, ORD_GET_CALLER_PROCESS_INDEX,
            ORD_GET_CLIPBOARD_DATA, ORD_GET_CLIPBOARD_DATA_ALLOC, ORD_GET_CLIPBOARD_FORMAT_NAME_W,
            ORD_GET_CLIPBOARD_OWNER, ORD_GET_COMM_MASK, ORD_GET_COMM_STATE, ORD_GET_COMM_TIMEOUTS,
            ORD_GET_DC, ORD_GET_EXIT_CODE_PROCESS, ORD_GET_EXIT_CODE_THREAD, ORD_GET_LAST_ERROR,
            ORD_GET_LOCAL_TIME, ORD_GET_MODULE_HANDLE_W, ORD_GET_OPEN_CLIPBOARD_WINDOW,
            ORD_GET_PRIORITY_CLIPBOARD_FORMAT, ORD_GET_PROC_ADDRESS_A, ORD_GET_PROC_ADDRESS_W,
            ORD_GET_PROCESS_ID, ORD_GET_PROCESS_IDFROM_INDEX, ORD_GET_PROCESS_INDEX_FROM_ID,
            ORD_GET_PROCESS_VERSION, ORD_GET_STORE_INFORMATION, ORD_GET_SYSTEM_TIME,
            ORD_GET_SYSTEM_TIME_AS_FILE_TIME, ORD_GET_THREAD_ID, ORD_GET_THREAD_PRIORITY,
            ORD_GET_THREAD_TIMES, ORD_GET_TICK_COUNT, ORD_GET_TIME_ZONE_INFORMATION,
            ORD_GET_VERSION_EX_W, ORD_IMAGE_LIST_ADD, ORD_IMAGE_LIST_ADD_MASKED,
            ORD_IMAGE_LIST_BEGIN_DRAG, ORD_IMAGE_LIST_COPY, ORD_IMAGE_LIST_COPY_DITHER_IMAGE,
            ORD_IMAGE_LIST_CREATE, ORD_IMAGE_LIST_DESTROY, ORD_IMAGE_LIST_DRAG_ENTER,
            ORD_IMAGE_LIST_DRAG_LEAVE, ORD_IMAGE_LIST_DRAG_MOVE, ORD_IMAGE_LIST_DRAG_SHOW_NOLOCK,
            ORD_IMAGE_LIST_DRAW_EX, ORD_IMAGE_LIST_DRAW_INDIRECT, ORD_IMAGE_LIST_DUPLICATE,
            ORD_IMAGE_LIST_END_DRAG, ORD_IMAGE_LIST_GET_BK_COLOR, ORD_IMAGE_LIST_GET_DRAG_IMAGE,
            ORD_IMAGE_LIST_GET_ICON, ORD_IMAGE_LIST_GET_ICON_SIZE, ORD_IMAGE_LIST_GET_IMAGE_COUNT,
            ORD_IMAGE_LIST_GET_IMAGE_INFO, ORD_IMAGE_LIST_LOAD_IMAGE, ORD_IMAGE_LIST_MERGE,
            ORD_IMAGE_LIST_REMOVE, ORD_IMAGE_LIST_REPLACE, ORD_IMAGE_LIST_REPLACE_ICON,
            ORD_IMAGE_LIST_SET_BK_COLOR, ORD_IMAGE_LIST_SET_DRAG_CURSOR_IMAGE,
            ORD_IMAGE_LIST_SET_ICON_SIZE, ORD_IMAGE_LIST_SET_IMAGE_COUNT,
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE, ORD_INITIALIZE_CRITICAL_SECTION,
            ORD_INPUT_DEBUG_CHAR_W, ORD_INTERLOCKED_COMPARE_EXCHANGE, ORD_INTERLOCKED_EXCHANGE_ADD,
            ORD_INTERLOCKED_INCREMENT, ORD_IS_CLIPBOARD_FORMAT_AVAILABLE, ORD_KERNEL_IO_CONTROL,
            ORD_LEAVE_CRITICAL_SECTION, ORD_LOAD_IMAGE_W, ORD_LOAD_LIBRARY_EX_W,
            ORD_LOAD_LIBRARY_W, ORD_MBSTOWCS, ORD_MESSAGE_BOX_W, ORD_MOVE_FILE_W,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX, ORD_MULTI_BYTE_TO_WIDE_CHAR, ORD_OPEN_CLIPBOARD,
            ORD_OPEN_EVENT_W, ORD_PEEK_MESSAGE_W, ORD_PURGE_COMM, ORD_QUERY_PERFORMANCE_COUNTER,
            ORD_QUERY_PERFORMANCE_FREQUENCY, ORD_REGISTER_CLIPBOARD_FORMAT_W, ORD_RELEASE_MUTEX,
            ORD_RELEASE_SEMAPHORE, ORD_RESUME_THREAD, ORD_SELECT_OBJECT, ORD_SET_CLIPBOARD_DATA,
            ORD_SET_COMM_MASK, ORD_SET_COMM_STATE, ORD_SET_COMM_TIMEOUTS, ORD_SET_LAST_ERROR,
            ORD_SET_THREAD_PRIORITY, ORD_SHADD_TO_RECENT_DOCS, ORD_SHCHANGE_NOTIFY_REGISTER_I,
            ORD_SHCREATE_SHORTCUT, ORD_SHCREATE_SHORTCUT_EX, ORD_SHELL_EXECUTE_EX,
            ORD_SHELL_NOTIFY_ICON, ORD_SHFILE_NOTIFY_FREE_I, ORD_SHFILE_NOTIFY_REMOVE_I,
            ORD_SHGET_FILE_INFO, ORD_SHGET_SHORTCUT_TARGET, ORD_SHGET_SPECIAL_FOLDER_PATH,
            ORD_SHNOTIFICATION_ADD_I, ORD_SHNOTIFICATION_GET_DATA_I, ORD_SHNOTIFICATION_REMOVE_I,
            ORD_SHNOTIFICATION_UPDATE_I, ORD_SLEEP, ORD_SLEEP_TILL_TICK, ORD_SUSPEND_THREAD,
            ORD_SYSTEM_TIME_TO_FILE_TIME, ORD_TERMINATE_PROCESS, ORD_TLS_GET_VALUE,
            ORD_TLS_SET_VALUE, ORD_TRY_ENTER_CRITICAL_SECTION, ORD_WAIT_COMM_EVENT,
            ORD_WAIT_FOR_MULTIPLE_OBJECTS, ORD_WAIT_FOR_SINGLE_OBJECT, ORD_WCSTOMBS,
            ORD_WIDE_CHAR_TO_MULTI_BYTE,
        },
        devices::CommDcb,
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        framebuffer::{Framebuffer, PixelFormat, VirtualFramebuffer},
        gwe::{
            Message, PeekFlags, QS_POSTMESSAGE, QS_TIMER, Rect, SC_CLOSE, WM_CHAR, WM_CLOSE,
            WM_COMMAND, WM_DESTROYCLIPBOARD, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_NOTIFY,
            WM_PAINT, WM_RENDERALLFORMATS, WM_RENDERFORMAT, WM_SYSCOMMAND, WM_TIMER, WM_USER,
            WS_VISIBLE,
        },
        kernel::{
            CE_CURRENT_PROCESS_PSEUDO_HANDLE, CE_CURRENT_THREAD_PSEUDO_HANDLE, CeKernel,
            LoadedModuleMetadata,
        },
        memory::PROCESS_HEAP_HANDLE,
        object::MAX_SUSPEND_COUNT,
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE, RegistryValue},
        resource::ResourceId,
        scheduler::SchedulerBlockedWaitKind,
        shell::{
            ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET,
            ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET,
            ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET,
            ISHELL_NOTIFICATION_CALLBACK_ON_SHOW_VTABLE_OFFSET, MessageBoxButtonLabel,
            MessageBoxButtonSlot, MessageBoxIcon, ShellNotificationCallbackArguments,
            ShellNotificationCallbackMethod, ShellSpecialFolderFallbackPolicy,
            ShellSpecialFolderSource,
        },
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
    assert_eq!(memory.read_u32(version_info + 4)?, 6); // CE 6.0 dwMajorVersion
    assert_eq!(memory.read_u32(version_info + 8)?, 0); // CE 6.0 dwMinorVersion
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
    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "Programs",
        RegistryValue::dword(2),
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
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].csidl, 0x0024);
    assert_eq!(queries[0].value_name, "Windows");
    assert_eq!(queries[0].path, r"\ResidentFlash\Windows");
    assert_eq!(queries[0].source, ShellSpecialFolderSource::Registry);
    assert!(!queries[0].create_requested);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, 0x0028, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(path_ptr, 260), r"\Windows\Profiles");
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 2);
    assert_eq!(queries[1].csidl, 0x0028);
    assert_eq!(queries[1].value_name, "Profile");
    assert_eq!(queries[1].source, ShellSpecialFolderSource::FallbackMissing);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, 0x0002, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(path_ptr, 260), r"\Windows\Programs");
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 3);
    assert_eq!(queries[2].csidl, 0x0002);
    assert_eq!(queries[2].value_name, "Programs");
    assert_eq!(
        queries[2].source,
        ShellSpecialFolderSource::FallbackNonString
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, 0x0025, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(path_ptr, 260), r"\Windows");
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 4);
    assert_eq!(queries[3].csidl, 0x0025);
    assert_eq!(queries[3].value_name, "System");
    assert_eq!(queries[3].source, ShellSpecialFolderSource::FallbackMissing);

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
    assert_eq!(kernel.shell.special_folder_queries().count(), 4);
    Ok(())
}

#[test]
fn coredll_raw_shget_special_folder_path_covers_ce_supported_fallbacks() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x5040;
    memory.map_halfwords(path_ptr, 260);
    assert_eq!(
        kernel
            .registry
            .reg_delete_key_w(HKEY_LOCAL_MACHINE, Some(r"System\Explorer\Shell Folders")),
        ERROR_SUCCESS
    );

    let folders = [
        (0x0000, "Desktop", r"\Windows\Desktop"),
        (0x0002, "Programs", r"\Windows\Programs"),
        (0x0005, "Personal", r"\My Documents"),
        (0x0006, "Favorites", r"\Windows\Favorites"),
        (0x0007, "Startup", r"\Windows\Startup"),
        (0x0008, "Recent", r"\Windows\Recent"),
        (0x000b, "Start Menu", r"\Windows\Start Menu"),
        (0x0010, "Desktop", r"\Windows\Desktop"),
        (0x0014, "Fonts", r"\Windows\Fonts"),
        (0x001a, "AppData", r"\Application Data"),
        (0x0024, "Windows", r"\Windows"),
        (0x0026, "Program Files", r"\Program Files"),
        (0x0028, "Profile", r"\Windows\Profiles"),
        (0x0025, "System", r"\Windows"),
    ];

    for (csidl, _value_name, fallback) in folders {
        memory.write_wide_z(path_ptr, "stale");
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SHGET_SPECIAL_FOLDER_PATH,
                [0, path_ptr, csidl, 0],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
        assert_eq!(memory.read_wide_z(path_ptr, 260), fallback);
        assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    }

    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), folders.len());
    for (query, (csidl, value_name, fallback)) in queries.iter().zip(folders) {
        assert_eq!(query.csidl, csidl);
        assert_eq!(query.value_name, value_name);
        assert_eq!(query.path, fallback);
        assert_eq!(query.source, ShellSpecialFolderSource::FallbackMissing);
        assert!(!query.create_requested);
    }

    Ok(())
}

#[test]
fn coredll_raw_shget_special_folder_path_strict_policy_rejects_fallbacks() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x5080;
    memory.map_halfwords(path_ptr, 260);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, 0x0028, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(path_ptr, 260), r"\Windows\Profiles");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    kernel
        .shell
        .set_special_folder_fallback_policy(ShellSpecialFolderFallbackPolicy::Strict);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, 0x0028, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 2);
    assert_eq!(queries[0].source, ShellSpecialFolderSource::FallbackMissing);
    assert_eq!(queries[1].csidl, 0x0028);
    assert_eq!(queries[1].value_name, "Profile");
    assert_eq!(queries[1].path, r"\Windows\Profiles");
    assert_eq!(queries[1].source, ShellSpecialFolderSource::FallbackMissing);
    assert!(!queries[1].create_requested);

    Ok(())
}

#[test]
fn coredll_raw_shget_special_folder_path_rejects_overlong_registry_path() -> Result<()> {
    const ERROR_FILENAME_EXCED_RANGE: u32 = 206;
    const CSIDL_WINDOWS: u32 = 0x0024;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x50c0;
    memory.map_halfwords(path_ptr, 260);
    memory.write_wide_z(path_ptr, "unchanged");
    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "Windows",
        RegistryValue::string(format!(r"\Windows\{}", "A".repeat(270))),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, CSIDL_WINDOWS, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILENAME_EXCED_RANGE
    );
    assert_eq!(memory.read_wide_z(path_ptr, 260), "unchanged");
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].csidl, CSIDL_WINDOWS);
    assert_eq!(queries[0].value_name, "Windows");
    assert_eq!(queries[0].source, ShellSpecialFolderSource::Registry);
    assert!(!queries[0].create_requested);

    Ok(())
}

#[test]
fn coredll_raw_shget_special_folder_path_honors_create_flags() -> Result<()> {
    const CSIDL_FLAG_CREATE: u32 = 0x0000_8000;
    const CSIDL_DESKTOPDIRECTORY: u32 = 0x0010;
    const CSIDL_FAVORITES: u32 = 0x0006;
    const CSIDL_RECENT: u32 = 0x0008;
    const CSIDL_PERSONAL: u32 = 0x0005;
    const CSIDL_APPDATA: u32 = 0x001a;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shget_special_folder_create");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("SDMMC")).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root(r"\SDMMC Disk", root.join("SDMMC"));
    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "Favorites",
        RegistryValue::string(r"\Windows\Favorites"),
    );
    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "Recent",
        RegistryValue::string(r"\Windows\Recent"),
    );
    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "Personal",
        RegistryValue::string(r"\SDMMC Disk\My Documents"),
    );
    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "AppData",
        RegistryValue::string(r"\SDMMC Disk\Profiles\guest\Application Data"),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x5100;
    memory.map_halfwords(path_ptr, 260);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, CSIDL_FAVORITES, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(path_ptr, 260), r"\Windows\Favorites");
    assert!(!root.join("Windows").join("Favorites").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, CSIDL_FAVORITES, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(root.join("Windows").join("Favorites").is_dir());
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 2);
    assert_eq!(queries[0].source, ShellSpecialFolderSource::Registry);
    assert!(!queries[0].create_requested);
    assert_eq!(queries[1].source, ShellSpecialFolderSource::Registry);
    assert!(queries[1].create_requested);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, CSIDL_RECENT | CSIDL_FLAG_CREATE, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(path_ptr, 260), r"\Windows\Recent");
    assert!(root.join("Windows").join("Recent").is_dir());
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 3);
    assert_eq!(queries[2].csidl, CSIDL_RECENT);
    assert_eq!(queries[2].source, ShellSpecialFolderSource::Registry);
    assert!(queries[2].create_requested);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, CSIDL_DESKTOPDIRECTORY | CSIDL_FLAG_CREATE, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let desktop_path = memory.read_wide_z(path_ptr, 260);
    assert!(desktop_path.starts_with(r"\Windows\"));
    let desktop_name = desktop_path.trim_start_matches(r"\Windows\");
    assert!(root.join("Windows").join(desktop_name).is_dir());
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 4);
    assert_eq!(queries[3].csidl, CSIDL_DESKTOPDIRECTORY);
    assert_eq!(queries[3].value_name, "Desktop");
    assert_eq!(queries[3].source, ShellSpecialFolderSource::Registry);
    assert!(queries[3].create_requested);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, CSIDL_PERSONAL | CSIDL_FLAG_CREATE, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_wide_z(path_ptr, 260),
        r"\SDMMC Disk\My Documents"
    );
    assert!(root.join("SDMMC").join("My Documents").is_dir());
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 5);
    assert_eq!(queries[4].csidl, CSIDL_PERSONAL);
    assert_eq!(queries[4].value_name, "Personal");
    assert_eq!(queries[4].source, ShellSpecialFolderSource::Registry);
    assert!(queries[4].create_requested);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SPECIAL_FOLDER_PATH,
            [0, path_ptr, CSIDL_APPDATA | CSIDL_FLAG_CREATE, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_wide_z(path_ptr, 260),
        r"\SDMMC Disk\Profiles\guest\Application Data"
    );
    assert!(
        root.join("SDMMC")
            .join("Profiles")
            .join("guest")
            .join("Application Data")
            .is_dir()
    );
    let queries = kernel
        .shell
        .special_folder_queries()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(queries.len(), 6);
    assert_eq!(queries[5].csidl, CSIDL_APPDATA);
    assert_eq!(queries[5].value_name, "AppData");
    assert_eq!(queries[5].source, ShellSpecialFolderSource::Registry);
    assert!(queries[5].create_requested);

    let _ = fs::remove_dir_all(root);
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
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    fs::write(root.join("Docs").join("route.nav"), b"route").unwrap();
    kernel.set_file_root(&root);
    kernel
        .registry
        .set_value(r"HKCR\.nav", "", RegistryValue::string("navfile"));
    kernel.registry.set_value(
        r"HKCR\navfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" "%1" %*"#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 43;
    let info = 0x2_0000;
    let file_ptr = 0x2_0100;
    let directory_ptr = 0x2_0200;
    let params_ptr = 0x2_0300;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 4, 0x0000_0040);
    memory.write_word(info + 16, file_ptr);
    memory.write_word(info + 20, params_ptr);
    memory.write_word(info + 24, directory_ptr);
    memory.write_wide_z(file_ptr, "route.nav");
    memory.write_wide_z(params_ptr, "-safe mode");
    memory.write_wide_z(directory_ptr, r"\Docs\");

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
        Some(r#""\Windows\viewer.exe" "\Docs\route.nav" -safe mode"#)
    );
    assert_eq!(launches[0].current_directory.as_deref(), Some(r"\Docs"));

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_appends_parameters_without_template_placeholder() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_assoc_append");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    fs::write(root.join("Docs").join("route.nav"), b"route").unwrap();
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
    let thread_id = 46;
    let info = 0x2_5000;
    let file_ptr = 0x2_5100;
    let params_ptr = 0x2_5200;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 16, file_ptr);
    memory.write_word(info + 20, params_ptr);
    memory.write_wide_z(file_ptr, r"\Docs\route.nav");
    memory.write_wide_z(params_ptr, "viewer.exe");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\viewer.exe" "\Docs\route.nav" viewer.exe"#)
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_expands_long_filename_template_placeholders() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_assoc_long_placeholders");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    fs::write(root.join("Docs").join("Route 42.nav"), b"route").unwrap();
    kernel.set_file_root(&root);
    kernel
        .registry
        .set_value(r"HKCR\.nav", "", RegistryValue::string("navfile"));
    kernel.registry.set_value(
        r"HKCR\navfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" /long=%L /lower=%l %*"#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5800;
    let file_ptr = 0x2_5900;
    let params_ptr = 0x2_5a00;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 16, file_ptr);
    memory.write_word(info + 20, params_ptr);
    memory.write_wide_z(file_ptr, r"\Docs\Route 42.nav");
    memory.write_wide_z(params_ptr, "-safe");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(
            r#""\Windows\viewer.exe" /long="\Docs\Route 42.nav" /lower="\Docs\Route 42.nav" -safe"#
        )
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_expands_urlfile_embedded_target_placeholder() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_urlfile_placeholder");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(
        root.join("Docs").join("Route.url"),
        b"[InternetShortcut]\r\n",
    )
    .unwrap();
    kernel.set_file_root(&root);
    kernel
        .registry
        .set_value(r"HKCR\.url", "", RegistryValue::string("urlfile"));
    kernel.registry.set_value(
        r"HKCR\urlfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#"explorer.exe -u%1"#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5b00;
    let file_ptr = 0x2_5b80;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(file_ptr, r"\Docs\Route.url");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(launches[0].application.as_deref(), Some("explorer.exe"));
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""explorer.exe" -u\Docs\Route.url"#)
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_resolves_hklm_software_classes_association() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_assoc_hklm");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    fs::write(root.join("Docs").join("HKLM Route.nav"), b"route").unwrap();
    kernel.set_file_root(&root);
    kernel.registry.set_value(
        r"HKLM\Software\Classes\.nav",
        "",
        RegistryValue::string("navfile"),
    );
    kernel.registry.set_value(
        r"HKLM\Software\Classes\navfile\Shell\Preview\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" /preview "%1""#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5c00;
    let verb_ptr = 0x2_5d00;
    let file_ptr = 0x2_5e00;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 12, verb_ptr);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(verb_ptr, "preview");
    memory.write_wide_z(file_ptr, r"\Docs\HKLM Route.nav");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\viewer.exe" /preview "\Docs\HKLM Route.nav""#)
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_resolves_relative_paths_from_process_current_directory() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_process_cwd");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    fs::write(root.join("Docs").join("route.nav"), b"route").unwrap();
    kernel.set_file_root(&root);
    kernel.set_process_current_directory(Some(r"\Docs".to_owned()));
    kernel
        .registry
        .set_value(r"HKCR\.nav", "", RegistryValue::string("navfile"));
    kernel.registry.set_value(
        r"HKCR\navfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" "%1" %*"#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5e80;
    let file_ptr = 0x2_5f00;
    let params_ptr = 0x2_5f80;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 16, file_ptr);
    memory.write_word(info + 20, params_ptr);
    memory.write_wide_z(file_ptr, "route.nav");
    memory.write_wide_z(params_ptr, "-safe");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].application.as_deref(),
        Some(r"\Windows\viewer.exe")
    );
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\viewer.exe" "\Docs\route.nav" -safe"#)
    );
    assert_eq!(launches[0].current_directory.as_deref(), Some(r"\Docs"));

    kernel.set_process_current_directory(Some(r"\Windows".to_owned()));
    memory.write_word(info + 20, 0);
    memory.write_wide_z(file_ptr, "viewer.exe");
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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].application.as_deref(),
        Some(r"\Windows\viewer.exe")
    );
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\viewer.exe""#)
    );
    assert_eq!(launches[0].current_directory.as_deref(), Some(r"\Windows"));

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_resolves_generic_file_association_for_extensionless_document() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_generic_file_assoc");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("explorer.exe"), b"fake explorer").unwrap();
    fs::write(root.join("Docs").join("README"), b"route notes").unwrap();
    kernel.set_file_root(&root);
    kernel.registry.set_value(
        r"HKCR\file\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\explorer.exe" %1"#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5f00;
    let file_ptr = 0x2_5f80;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(file_ptr, r"\Docs\README");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].application.as_deref(),
        Some(r"\Windows\explorer.exe")
    );
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\explorer.exe" "\Docs\README""#)
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_reports_precise_missing_exe_and_no_association_errors() -> Result<()> {
    const SE_ERR_FNF: u32 = 2;
    const SE_ERR_PNF: u32 = 3;
    const SE_ERR_NOASSOC: u32 = 31;
    const ERROR_PATH_NOT_FOUND: u32 = 3;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_errors");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Docs").join("route.nav"), b"route").unwrap();
    kernel.set_file_root(&root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_6000;
    let file_ptr = 0x2_6100;
    memory.map_words(info, 16);
    memory.map_halfwords(file_ptr, 120);
    memory.write_word(info, 60);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(file_ptr, r"\Windows\missing.exe");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_EXECUTE_EX,
            [info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_FNF);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    memory.write_word(info + 32, 0);
    memory.write_wide_z(file_ptr, r"\Docs\missing");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_EXECUTE_EX,
            [info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_FNF);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    memory.write_word(info + 32, 0);
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
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_NOASSOC);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    kernel
        .registry
        .set_value(r"HKCR\.nav", "", RegistryValue::string("navfile"));
    kernel.registry.set_value(
        r"HKCR\navfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" "%1""#),
    );
    memory.write_word(info + 32, 0);
    memory.write_wide_z(file_ptr, r"\Docs\missing.nav");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_EXECUTE_EX,
            [info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_FNF);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    kernel
        .registry
        .set_value(r"HKCR\.nav", "", RegistryValue::string("navfile"));
    kernel.registry.set_value(
        r"HKCR\navfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\missing-viewer.exe" "%1""#),
    );
    memory.write_word(info + 32, 0);
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
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_FNF);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    kernel.registry.set_value(
        r"HKCR\navfile\Shell\Open\Command",
        "",
        RegistryValue::string(""),
    );
    memory.write_word(info + 32, 0);
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
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_NOASSOC);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    // A file in a directory that does not exist is SE_ERR_PNF (path not found),
    // distinct from SE_ERR_FNF for a missing file in an existing directory.
    memory.write_word(info + 32, 0);
    memory.write_wide_z(file_ptr, r"\NoSuchDir\missing.exe");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_EXECUTE_EX,
            [info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_PNF);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_PATH_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    // A document whose containing directory is missing reports PNF as well.
    memory.write_word(info + 32, 0);
    memory.write_wide_z(file_ptr, r"\NoSuchDir\route.nav");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_EXECUTE_EX,
            [info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_PNF);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_PATH_NOT_FOUND
    );
    assert!(kernel.take_pending_process_launches().is_empty());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_verb_print_falls_back_to_open_when_print_not_registered() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_verb_fallback");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    fs::write(root.join("Docs").join("report.txt"), b"text").unwrap();
    kernel.set_file_root(&root);
    kernel
        .registry
        .set_value(r"HKCR\.txt", "", RegistryValue::string("txtfile"));
    kernel.registry.set_value(
        r"HKCR\txtfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" "%1""#),
    );
    // No Print command registered for txtfile — should fall back to Open.

    let mut memory = TestGuestMemory::default();
    let thread_id = 47_u32;
    let info = 0x2_7000;
    let file_ptr = 0x2_7100;
    let verb_ptr = 0x2_7200;
    memory.map_words(info, 16);
    memory.map_halfwords(file_ptr, 120);
    memory.map_halfwords(verb_ptr, 20);
    memory.write_word(info, 60);
    memory.write_word(info + 12, verb_ptr);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(verb_ptr, "print");
    memory.write_wide_z(file_ptr, r"\Docs\report.txt");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].application.as_deref(),
        Some(r"\Windows\viewer.exe"),
        "print verb with no Print command must fall back to Open"
    );
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\viewer.exe" "\Docs\report.txt""#)
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_verb_print_uses_print_command_when_registered() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_verb_print");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    fs::write(root.join("Windows").join("printer.exe"), b"fake printer").unwrap();
    fs::write(root.join("Docs").join("report.txt"), b"text").unwrap();
    kernel.set_file_root(&root);
    kernel
        .registry
        .set_value(r"HKCR\.txt", "", RegistryValue::string("txtfile"));
    kernel.registry.set_value(
        r"HKCR\txtfile\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\viewer.exe" "%1""#),
    );
    kernel.registry.set_value(
        r"HKCR\txtfile\Shell\Print\Command",
        "",
        RegistryValue::string(r#""\Windows\printer.exe" /p "%1""#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 48_u32;
    let info = 0x2_8000;
    let file_ptr = 0x2_8100;
    let verb_ptr = 0x2_8200;
    memory.map_words(info, 16);
    memory.map_halfwords(file_ptr, 120);
    memory.map_halfwords(verb_ptr, 20);
    memory.write_word(info, 60);
    memory.write_word(info + 12, verb_ptr);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(verb_ptr, "print");
    memory.write_wide_z(file_ptr, r"\Docs\report.txt");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].application.as_deref(),
        Some(r"\Windows\printer.exe"),
        "print verb must use Print command when registered"
    );
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\printer.exe" /p "\Docs\report.txt""#)
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_opens_directory_via_folder_class() -> Result<()> {
    // Directories are routed through HKCR\folder\Shell\Open\Command (the "folder" class),
    // not through the file-extension association, matching CE's Explorer dispatch.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_folder_class");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Windows").join("explorer.exe"), b"fake exe").unwrap();
    kernel.set_file_root(&root);
    kernel.registry.set_value(
        r"HKCR\folder\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\explorer.exe" "%1""#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5c00;
    let verb_ptr = 0x2_5d00;
    let file_ptr = 0x2_5e00;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 12, verb_ptr);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(verb_ptr, "");
    memory.write_wide_z(file_ptr, r"\Docs");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\explorer.exe" "\Docs""#),
        "directory must route through folder class open command"
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_directory_with_no_folder_class_returns_noassoc() -> Result<()> {
    // When no HKCR\folder\Shell\Open\Command is registered, opening a directory
    // returns SE_ERR_NOASSOC (31).
    const SE_ERR_NOASSOC: u32 = 31;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_folder_noassoc");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Docs")).unwrap();
    kernel.set_file_root(&root);
    // No folder open command registered.

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5c00;
    let verb_ptr = 0x2_5d00;
    let file_ptr = 0x2_5e00;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 12, verb_ptr);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(verb_ptr, "");
    memory.write_wide_z(file_ptr, r"\Docs");

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHELL_EXECUTE_EX,
        [info],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_NOASSOC);
    assert!(kernel.take_pending_process_launches().is_empty());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_execute_ex_opens_url_via_protocol_handler() -> Result<()> {
    // URL targets (containing "://") are routed through HKCR\{scheme}\Shell\Open\Command
    // without a filesystem existence check, matching CE's URL protocol-handler dispatch.
    const SE_ERR_NOASSOC: u32 = 31;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_execute_ex_url_protocol");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::write(root.join("Windows").join("iexplore.exe"), b"fake ie").unwrap();
    kernel.set_file_root(&root);
    kernel.registry.set_value(
        r"HKCR\http\Shell\Open\Command",
        "",
        RegistryValue::string(r#""\Windows\iexplore.exe" "%1""#),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let info = 0x2_5c00;
    let verb_ptr = 0x2_5d00;
    let file_ptr = 0x2_5e00;
    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 12, verb_ptr);
    memory.write_word(info + 16, file_ptr);
    memory.write_wide_z(verb_ptr, "");
    memory.write_wide_z(file_ptr, "http://example.com/route.nav");

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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\iexplore.exe" "http://example.com/route.nav""#),
        "http URL must route through HKCR\\http\\Shell\\Open\\Command"
    );

    // URL with no registered scheme handler returns SE_ERR_NOASSOC.
    memory.write_wide_z(file_ptr, "ftp://files.example.com/data");
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHELL_EXECUTE_EX,
        [info],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info + 32)?, SE_ERR_NOASSOC);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_create_process_preserves_current_directory() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;
    let app_ptr = 0x2_4000;
    let dir_ptr = 0x2_4200;
    let process_info = 0x2_4400;
    memory.map_words(process_info, 4);
    memory.write_wide_z(app_ptr, "child.exe");
    memory.write_wide_z(dir_ptr, r"\SDMMC Disk\INavi\bin\");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_PROCESS_W,
            [app_ptr, 0, 0, 0, 0, 0, 0, dir_ptr, 0, process_info],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(launches[0].application.as_deref(), Some("child.exe"));
    assert_eq!(
        launches[0].current_directory.as_deref(),
        Some(r"\SDMMC Disk\INavi\bin")
    );
    assert_eq!(memory.read_u32(process_info)?, launches[0].process_handle);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    Ok(())
}

#[test]
fn shell_shortcut_ordinals_create_read_and_launch_ce_lnk_files() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_shortcut_ordinals");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    kernel.set_file_root(&root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 44;
    let shortcut_ptr = 0x2_2000;
    let target_ptr = 0x2_2200;
    let out_ptr = 0x2_2600;
    let info = 0x2_3000;
    let file_ptr = 0x2_3100;
    let params_ptr = 0x2_3200;
    memory.map_halfwords(shortcut_ptr, 80);
    memory.map_halfwords(target_ptr, 120);
    memory.map_halfwords(out_ptr, 120);
    memory.map_halfwords(params_ptr, 40);
    memory.write_wide_z(shortcut_ptr, r"\RouteSearch.lnk");
    memory.write_wide_z(target_ptr, r#""\Windows\viewer.exe" -safe"#);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCREATE_SHORTCUT,
            [shortcut_ptr, target_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let raw = fs::read(root.join("RouteSearch.lnk")).unwrap();
    assert!(raw.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(
        String::from_utf8_lossy(&raw).contains(r#"27#"\Windows\viewer.exe" -safe"#),
        "unexpected shortcut text: {:?}",
        String::from_utf8_lossy(&raw)
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SHORTCUT_TARGET,
            [shortcut_ptr, out_ptr, 120],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_wide_z(out_ptr, 120),
        r#""\Windows\viewer.exe" -safe"#
    );

    memory.map_words(info, 16);
    memory.write_word(info, 60);
    memory.write_word(info + 4, 0x0000_0040);
    memory.write_word(info + 16, file_ptr);
    memory.write_word(info + 20, params_ptr);
    memory.write_word(info + 28, 5);
    memory.write_wide_z(file_ptr, r"\RouteSearch.lnk");
    memory.write_wide_z(params_ptr, "-extra");
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
    let launches = kernel.take_pending_process_launches();
    assert_eq!(launches.len(), 1);
    assert_eq!(
        launches[0].application.as_deref(),
        Some(r"\Windows\viewer.exe")
    );
    assert_eq!(
        launches[0].command_line.as_deref(),
        Some(r#""\Windows\viewer.exe" -safe -extra"#)
    );
    assert_eq!(launches[0].show_cmd, Some(5));
    assert_eq!(memory.read_u32(info + 56)?, launches[0].process_handle);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCREATE_SHORTCUT,
            [shortcut_ptr, target_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 80);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SHORTCUT_TARGET,
            [shortcut_ptr, out_ptr, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 122);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_create_shortcut_ex_returns_unique_name_and_checks_output_capacity() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_shortcut_ex");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::write(root.join("Existing.lnk"), b"old shortcut").unwrap();
    fs::write(root.join("Windows").join("viewer.exe"), b"fake exe").unwrap();
    kernel.set_file_root(&root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let requested_ptr = 0x2_8000;
    let target_ptr = 0x2_8200;
    let out_ptr = 0x2_8400;
    let out_capacity_ptr = 0x2_8800;
    let readback_ptr = 0x2_8a00;
    memory.map_halfwords(requested_ptr, 80);
    memory.map_halfwords(target_ptr, 120);
    memory.map_halfwords(out_ptr, 120);
    memory.map_halfwords(readback_ptr, 120);
    memory.map_words(out_capacity_ptr, 1);
    memory.write_wide_z(requested_ptr, r"\Existing.lnk");
    memory.write_wide_z(target_ptr, r#""\Windows\viewer.exe" -route"#);

    memory.write_word(out_capacity_ptr, 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCREATE_SHORTCUT_EX,
            [requested_ptr, target_ptr, out_ptr, out_capacity_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 122);
    assert_eq!(memory.read_u32(out_capacity_ptr)?, 18);
    assert!(!root.join("Existing (2).lnk").exists());

    memory.write_word(out_capacity_ptr, 120);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCREATE_SHORTCUT_EX,
            [requested_ptr, target_ptr, out_ptr, out_capacity_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_wide_z(out_ptr, 120), r"\Existing (2).lnk");
    assert!(root.join("Existing (2).lnk").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_SHORTCUT_TARGET,
            [out_ptr, readback_ptr, 120],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_wide_z(readback_ptr, 120),
        r#""\Windows\viewer.exe" -route"#
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn shell_add_to_recent_docs_creates_and_clears_recent_shortcuts() -> Result<()> {
    const SHARD_PIDL: u32 = 1;
    const SHARD_PATH: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shell_recent_docs");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    kernel.set_file_root(&root);
    kernel.registry.set_value(
        r"HKLM\System\Explorer\Shell Folders",
        "Recent",
        RegistryValue::string(r"\Windows\Recent"),
    );

    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let target_ptr = 0x2_9000;
    let pidl_ptr = 0x2_a000;
    memory.map_halfwords(target_ptr, 120);
    memory.write_wide_z(target_ptr, r"\Docs\Morning Route.nav");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHADD_TO_RECENT_DOCS,
            [SHARD_PATH, target_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let recent_link = root
        .join("Windows")
        .join("Recent")
        .join("Morning Route.lnk");
    assert!(recent_link.exists());
    let raw = fs::read(&recent_link).unwrap();
    assert!(
        String::from_utf8_lossy(&raw).contains(r#"25#"\Docs\Morning Route.nav""#),
        "unexpected recent shortcut text: {:?}",
        String::from_utf8_lossy(&raw)
    );
    let recent: Vec<_> = kernel.shell.recent_documents().cloned().collect();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].target_path, r"\Docs\Morning Route.nav");
    assert_eq!(
        recent[0].shortcut_path,
        r"\Windows\Recent\Morning Route.lnk"
    );
    let display_entries: Vec<_> = kernel.shell.recent_document_display_entries().collect();
    assert_eq!(display_entries.len(), 1);
    assert_eq!(display_entries[0].label, "Morning Route");
    assert_eq!(display_entries[0].target_path, r"\Docs\Morning Route.nav");
    assert_eq!(
        display_entries[0].shortcut_path,
        r"\Windows\Recent\Morning Route.lnk"
    );
    assert_eq!(display_entries[0].flags, SHARD_PATH);
    assert!(!display_entries[0].has_namespace_pidl);

    let pidl_path = r"\Docs\Evening Route.nav";
    let pidl_units: Vec<u16> = pidl_path.encode_utf16().chain(std::iter::once(0)).collect();
    let pidl_cb = 2 + (pidl_units.len() * 2) as u16;
    let mut pidl_bytes = Vec::with_capacity(usize::from(pidl_cb) + 2);
    pidl_bytes.extend_from_slice(&pidl_cb.to_le_bytes());
    for unit in &pidl_units {
        pidl_bytes.extend_from_slice(&unit.to_le_bytes());
    }
    pidl_bytes.extend_from_slice(&0u16.to_le_bytes());
    memory.map_bytes(pidl_ptr, pidl_bytes.len() as u32);
    memory.map_halfwords(pidl_ptr, (pidl_bytes.len() as u32).div_ceil(2));
    memory.write_bytes(pidl_ptr, &pidl_bytes);
    memory.write_halfword(pidl_ptr, pidl_cb);
    memory.write_halfword(pidl_ptr + u32::from(pidl_cb), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHADD_TO_RECENT_DOCS,
            [SHARD_PIDL, pidl_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let pidl_link = root
        .join("Windows")
        .join("Recent")
        .join("Evening Route.lnk");
    assert!(pidl_link.exists());
    let recent: Vec<_> = kernel.shell.recent_documents().cloned().collect();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].flags, SHARD_PIDL);
    assert_eq!(recent[0].target_path, pidl_path);
    assert_eq!(recent[0].display_name, "Evening Route");
    assert!(recent[0].pidl_bytes.is_none());
    assert_eq!(
        recent[0].shortcut_path,
        r"\Windows\Recent\Evening Route.lnk"
    );

    let opaque_pidl = [0xde, 0xad, 0xbe, 0xef];
    let opaque_pidl_cb = 2 + opaque_pidl.len() as u16;
    let mut opaque_pidl_bytes = Vec::with_capacity(usize::from(opaque_pidl_cb) + 2);
    opaque_pidl_bytes.extend_from_slice(&opaque_pidl_cb.to_le_bytes());
    opaque_pidl_bytes.extend_from_slice(&opaque_pidl);
    opaque_pidl_bytes.extend_from_slice(&0u16.to_le_bytes());
    memory.write_bytes(pidl_ptr, &opaque_pidl_bytes);
    memory.write_halfword(pidl_ptr, opaque_pidl_cb);
    memory.write_halfword(pidl_ptr + u32::from(opaque_pidl_cb), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHADD_TO_RECENT_DOCS,
            [SHARD_PIDL, pidl_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let opaque_pidl_link = root
        .join("Windows")
        .join("Recent")
        .join("Namespace PIDL deadbeef.lnk");
    assert!(opaque_pidl_link.exists());
    let opaque_raw = fs::read(&opaque_pidl_link).unwrap();
    assert!(
        String::from_utf8_lossy(&opaque_raw).contains(r#"::pidl:deadbeef"#),
        "unexpected opaque PIDL shortcut text: {:?}",
        String::from_utf8_lossy(&opaque_raw)
    );
    let recent: Vec<_> = kernel.shell.recent_documents().cloned().collect();
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].flags, SHARD_PIDL);
    assert_eq!(recent[0].target_path, "::pidl:deadbeef");
    assert_eq!(recent[0].display_name, "Namespace PIDL deadbeef");
    assert_eq!(recent[0].pidl_bytes.as_deref(), Some(&opaque_pidl[..]));
    assert_eq!(
        recent[0].shortcut_path,
        r"\Windows\Recent\Namespace PIDL deadbeef.lnk"
    );
    let display_entries: Vec<_> = kernel.shell.recent_document_display_entries().collect();
    assert_eq!(display_entries.len(), 3);
    assert_eq!(display_entries[0].label, "Namespace PIDL deadbeef");
    assert_eq!(display_entries[0].target_path, "::pidl:deadbeef");
    assert_eq!(display_entries[0].flags, SHARD_PIDL);
    assert!(display_entries[0].has_namespace_pidl);
    assert_eq!(display_entries[1].label, "Evening Route");
    assert_eq!(display_entries[1].target_path, pidl_path);
    assert!(!display_entries[1].has_namespace_pidl);
    assert_eq!(display_entries[2].label, "Morning Route");

    for index in 2..=11 {
        memory.write_wide_z(target_ptr, &format!(r"\Docs\Route {index:02}.nav"));
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SHADD_TO_RECENT_DOCS,
                [SHARD_PATH, target_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ));
    }
    let recent: Vec<_> = kernel.shell.recent_documents().cloned().collect();
    assert_eq!(recent.len(), 10);
    assert_eq!(recent[0].target_path, r"\Docs\Route 11.nav");
    assert_eq!(recent[9].target_path, r"\Docs\Route 02.nav");
    let display_labels: Vec<_> = kernel
        .shell
        .recent_document_display_entries()
        .map(|entry| entry.label.to_owned())
        .collect();
    assert_eq!(
        display_labels,
        vec![
            "Route 11", "Route 10", "Route 09", "Route 08", "Route 07", "Route 06", "Route 05",
            "Route 04", "Route 03", "Route 02"
        ]
    );
    assert!(!recent_link.exists());
    assert!(!pidl_link.exists());
    assert!(!opaque_pidl_link.exists());
    assert!(
        root.join("Windows")
            .join("Recent")
            .join("Route 02.lnk")
            .exists()
    );
    let newest_link = root.join("Windows").join("Recent").join("Route 11.lnk");
    assert!(newest_link.exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHADD_TO_RECENT_DOCS,
            [SHARD_PATH, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(!recent_link.exists());
    assert!(!newest_link.exists());
    assert_eq!(kernel.shell.recent_documents().count(), 0);
    assert_eq!(kernel.shell.recent_document_display_entries().count(), 0);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn sh_get_file_info_uses_registry_associations_and_attributes() -> Result<()> {
    const ERROR_MOD_NOT_FOUND: u32 = 126;
    const SHFILEINFO_SIZE_W: u32 = 692;
    const SHFILEINFO_HICON_OFFSET: u32 = 0;
    const SHFILEINFO_IICON_OFFSET: u32 = 4;
    const SHFILEINFO_ATTRIBUTES_OFFSET: u32 = 8;
    const SHFILEINFO_DISPLAY_NAME_OFFSET: u32 = 12;
    const SHFILEINFO_TYPE_NAME_OFFSET: u32 = 532;
    const SHGFI_ICON: u32 = 0x0000_0100;
    const SHGFI_DISPLAYNAME: u32 = 0x0000_0200;
    const SHGFI_TYPENAME: u32 = 0x0000_0400;
    const SHGFI_ATTRIBUTES: u32 = 0x0000_0800;
    const SHGFI_SYSICONINDEX: u32 = 0x0000_4000;
    const SHGFI_SMALLICON: u32 = 0x0000_0001;
    const SHGFI_USEFILEATTRIBUTES: u32 = 0x0000_0010;
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
    const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;
    const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x0000_0100;
    const SFGAO_FOLDER: u32 = 0x2000_0000;
    const SFGAO_FILESYSTEM: u32 = 0x4000_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shget_file_info");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Docs").join("morning.nav"), b"route").unwrap();
    fs::write(root.join("Docs").join("viewer.exe"), b"MZ").unwrap();
    fs::write(
        root.join("Docs").join("viewer.lnk"),
        br#"23#"\Docs\viewer.exe""#,
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("viewer-loop.lnk"),
        br#"28#"\Docs\viewer-loop.lnk""#,
    )
    .unwrap();
    kernel.set_file_root(&root);
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
    let large_icon_ptr = 0x2_7000;
    let small_icon_ptr = 0x2_7010;
    memory.map_halfwords(path_ptr, 64);
    memory.write_wide_z(path_ptr, r"\Docs\morning.nav");
    memory.map_words(info_ptr, SHFILEINFO_SIZE_W / 4);
    memory.map_halfwords(info_ptr + SHFILEINFO_DISPLAY_NAME_OFFSET, 260);
    memory.map_halfwords(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80);
    memory.map_words(large_icon_ptr, 2);
    memory.map_words(small_icon_ptr, 2);

    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_DISPLAYNAME
                | SHGFI_TYPENAME
                | SHGFI_ATTRIBUTES
                | SHGFI_SYSICONINDEX
                | SHGFI_ICON
                | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f000),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET)?,
        FILE_ATTRIBUTE_ARCHIVE
    );
    let expected_nav_icon = expected_default_icon_index(r"\Windows\navicons.dll", -7);
    assert_eq!(
        memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?,
        expected_nav_icon
    );
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle(expected_nav_icon)
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_DISPLAY_NAME_OFFSET, 260),
        "morning.nav"
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
        "Route Plan"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0xffff_ffff, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    memory.write_u32(large_icon_ptr, 0)?;
    memory.write_u32(small_icon_ptr, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0, large_icon_ptr, small_icon_ptr, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(large_icon_ptr)?,
        shell_pseudo_icon_handle(expected_nav_icon)
    );
    assert_eq!(
        memory.read_u32(small_icon_ptr)?,
        shell_pseudo_icon_handle(expected_nav_icon)
    );

    memory.write_wide_z(path_ptr, r"\Docs\viewer.exe");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_SYSICONINDEX | SHGFI_ICON | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f000),
            ..
        }
    ));
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 2);
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle(2)
    );
    memory.write_u32(large_icon_ptr, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0, large_icon_ptr, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(large_icon_ptr)?,
        shell_pseudo_icon_handle(2)
    );

    memory.write_wide_z(path_ptr, r"\Docs\viewer.lnk");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_DISPLAYNAME | SHGFI_TYPENAME | SHGFI_SYSICONINDEX | SHGFI_ICON | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f000),
            ..
        }
    ));
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 2);
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle_with_overlay(2, 1)
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_DISPLAY_NAME_OFFSET, 260),
        "viewer.lnk"
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
        "Shortcut"
    );
    memory.write_u32(large_icon_ptr, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0, large_icon_ptr, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(large_icon_ptr)?,
        shell_pseudo_icon_handle_with_overlay(2, 1)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 1, large_icon_ptr, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\viewer-loop.lnk");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_TYPENAME | SHGFI_SYSICONINDEX | SHGFI_ICON | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f000),
            ..
        }
    ));
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 0);
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle_with_overlay(0, 1)
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
        "Shortcut"
    );

    memory.write_wide_z(path_ptr, r"\Docs\Uncreated");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            FILE_ATTRIBUTE_DIRECTORY,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_USEFILEATTRIBUTES | SHGFI_SYSICONINDEX | SHGFI_ICON | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f000),
            ..
        }
    ));
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 1);
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle(1)
    );

    kernel.mount_guest_root(r"\SDMMC Disk", root.join("SDMMC"));
    memory.write_wide_z(path_ptr, r"\SDMMC Disk");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET, 0)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_DISPLAYNAME
                | SHGFI_TYPENAME
                | SHGFI_ATTRIBUTES
                | SHGFI_SYSICONINDEX
                | SHGFI_ICON
                | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f000),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET)?,
        FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY
    );
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 3);
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle(3)
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_DISPLAY_NAME_OFFSET, 260),
        "SDMMC Disk"
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
        "Storage Card"
    );

    for network_folder in [r"\\nas\drop", r"\release", r"\network"] {
        memory.write_wide_z(path_ptr, network_folder);
        memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
        memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0)?;
        let ret = table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_FILE_INFO,
            [
                path_ptr,
                FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY,
                info_ptr,
                SHFILEINFO_SIZE_W,
                SHGFI_USEFILEATTRIBUTES
                    | SHGFI_TYPENAME
                    | SHGFI_SYSICONINDEX
                    | SHGFI_ICON
                    | SHGFI_SMALLICON,
            ],
        );
        assert!(matches!(
            ret,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0x000b_f000),
                ..
            }
        ));
        assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 4);
        assert_eq!(
            memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
            shell_pseudo_icon_handle(4)
        );
        assert_eq!(
            memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
            "Network Folder"
        );
    }

    memory.write_wide_z(path_ptr, r"\\nas\drop");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET, 0)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_TYPENAME | SHGFI_ATTRIBUTES | SHGFI_SYSICONINDEX | SHGFI_ICON | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f000),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET)?,
        SFGAO_FILESYSTEM | SFGAO_FOLDER
    );
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 4);
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle(4)
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
        "Network Folder"
    );

    memory.write_wide_z(path_ptr, r"\Docs\missing.nav");
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_DISPLAYNAME | SHGFI_TYPENAME,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_MOD_NOT_FOUND
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0, large_icon_ptr, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn expected_default_icon_index(location: &str, icon_index: i32) -> i32 {
    let location_hash = location.bytes().fold(0u32, |acc, byte| {
        acc.wrapping_mul(33)
            .wrapping_add(byte.to_ascii_lowercase() as u32)
    });
    100 + ((location_hash ^ icon_index.unsigned_abs()) % 4096) as i32
}

fn shell_pseudo_icon_handle(index: i32) -> u32 {
    shell_pseudo_icon_handle_with_overlay(index, 0)
}

fn shell_pseudo_icon_handle_with_overlay(index: i32, overlay: u32) -> u32 {
    let stock = match index {
        0 => 32512,
        1 => 32513,
        2 => 32514,
        3 => 32515,
        4 => 32516,
        5 => 32517,
        value => 0x0000_c000 | ((value as u32) & 0x3fff),
    };
    0x000b_8000 | stock | ((overlay & 0x0f) << 24)
}

fn bmp_1x1_24bpp() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&58u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54u32.to_le_bytes());
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&4u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0x33, 0x66, 0x99, 0x00]);
    bytes
}

fn bmp_2x1_magenta_green_24bpp() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&62u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54u32.to_le_bytes());
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&8u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0x00]);
    bytes
}

fn bmp_2x1_white_black_24bpp() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&62u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54u32.to_le_bytes());
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&8u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00]);
    bytes
}

fn bmp_2x1_red_red_24bpp() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&62u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54u32.to_le_bytes());
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&8u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0x00, 0x00, 0xff, 0x00, 0x00, 0xff, 0x00, 0x00]);
    bytes
}

fn create_selected_rgb565_dib(
    table: &CoredllExportTable,
    kernel: &mut CeKernel,
    memory: &mut TestGuestMemory,
    thread_id: u32,
    width: i32,
    height: i32,
) -> (u32, u32, u32) {
    let mem_dc = match table.dispatch_raw_ordinal_with_memory(
        kernel,
        memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateCompatibleDC(NULL) did not return a handle: {other:?}"),
    };
    let info = 0x1_f000;
    let bits_out = 0x1_f100;
    memory.map_bytes(info, 40);
    memory.map_words(bits_out, 1);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&width.to_le_bytes());
    header[8..12].copy_from_slice(&(-height).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&16u16.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);

    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        kernel,
        memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [mem_dc, info, 0, bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection did not return a bitmap: {other:?}"),
    };
    assert_ne!(bitmap, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            kernel,
            memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    let bits_ptr = memory
        .read_u32(bits_out)
        .expect("CreateDIBSection should write bits pointer");
    let stride = (((width as u32 * 16) + 31) / 32) * 4;
    (mem_dc, bits_ptr, stride)
}

fn rgb565_at(memory: &TestGuestMemory, bits_ptr: u32, stride: u32, x: u32, y: u32) -> u16 {
    memory
        .read_u16(bits_ptr + y * stride + x * 2)
        .expect("pixel should be readable")
}

#[test]
fn sh_get_file_info_system_image_list_supports_icon_queries_and_draw() -> Result<()> {
    const SHFILEINFO_SIZE_W: u32 = 692;
    const SHFILEINFO_IICON_OFFSET: u32 = 4;
    const SHGFI_SYSICONINDEX: u32 = 0x0000_4000;
    const SHGFI_SMALLICON: u32 = 0x0000_0001;
    const SHGFI_USEFILEATTRIBUTES: u32 = 0x0000_0010;
    const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;
    const SHELL_SYSTEM_IMAGE_LIST_HANDLE: u32 = 0x000b_f000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;
    let path_ptr = 0x2_4000;
    let info_ptr = 0x2_5000;
    let size_ptr = 0x2_6000;
    let image_info_ptr = 0x2_7000;
    memory.map_halfwords(path_ptr, 64);
    memory.write_wide_z(path_ptr, "");
    memory.map_words(info_ptr, SHFILEINFO_SIZE_W / 4);
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            0,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_SYSICONINDEX | SHGFI_SMALLICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(SHELL_SYSTEM_IMAGE_LIST_HANDLE),
            ..
        }
    ));
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\ghost.nav");
    memory.map_words(size_ptr, 2);
    memory.map_words(image_info_ptr, 8);

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
            SHGFI_USEFILEATTRIBUTES | SHGFI_SYSICONINDEX,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(SHELL_SYSTEM_IMAGE_LIST_HANDLE),
            ..
        }
    ));
    let icon_index = memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_IMAGE_COUNT,
            [SHELL_SYSTEM_IMAGE_LIST_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(8192),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON_SIZE,
            [SHELL_SYSTEM_IMAGE_LIST_HANDLE, size_ptr, size_ptr + 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(size_ptr)?, 16);
    assert_eq!(memory.read_i32(size_ptr + 4)?, 16);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON,
            [SHELL_SYSTEM_IMAGE_LIST_HANDLE, icon_index as u32, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == shell_pseudo_icon_handle(icon_index)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON,
            [SHELL_SYSTEM_IMAGE_LIST_HANDLE, icon_index as u32, 0x0100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == shell_pseudo_icon_handle_with_overlay(icon_index, 1)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_IMAGE_INFO,
            [
                SHELL_SYSTEM_IMAGE_LIST_HANDLE,
                icon_index as u32,
                image_info_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_ne!(memory.read_u32(image_info_ptr)?, 0);
    assert_eq!(memory.read_i32(image_info_ptr + 20)?, 0);
    assert_eq!(memory.read_i32(image_info_ptr + 24)?, icon_index * 16 + 16);
    assert_eq!(memory.read_i32(image_info_ptr + 28)?, 16);
    let hdc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetDC did not return a desktop HDC: {other:?}"),
    };
    let mut framebuffer = VirtualFramebuffer::new(40, 40, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [
                SHELL_SYSTEM_IMAGE_LIST_HANDLE,
                icon_index as u32,
                hdc,
                4,
                8,
                16,
                16,
                0,
                0,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!framebuffer.dirty_rects().is_empty());
    let pixel = |x: usize, y: usize| {
        let offset = y * framebuffer.stride() + x * PixelFormat::Rgb565.bytes_per_pixel();
        &framebuffer.pixels()[offset..offset + PixelFormat::Rgb565.bytes_per_pixel()]
    };
    assert_eq!(pixel(0, 0), &[0, 0]);
    assert_ne!(pixel(5, 9), &[0, 0]);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    Ok(())
}

#[test]
fn image_list_ordinals_track_created_lists_and_icons() -> Result<()> {
    const IMAGE_BITMAP: u32 = 0;
    const LR_LOADFROMFILE: u32 = 0x0000_0010;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("image_list_load_image");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Images")).unwrap();
    fs::write(root.join("Images").join("one.bmp"), bmp_1x1_24bpp()).unwrap();
    fs::write(
        root.join("Images").join("masked.bmp"),
        bmp_2x1_magenta_green_24bpp(),
    )
    .unwrap();
    fs::write(
        root.join("Images").join("mask.bmp"),
        bmp_2x1_white_black_24bpp(),
    )
    .unwrap();
    fs::write(root.join("Images").join("red.bmp"), bmp_2x1_red_red_24bpp()).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;
    let size_ptr = 0x2_6000;
    let image_info_ptr = 0x2_7000;
    let drag_point_ptr = 0x2_8000;
    let drag_hotspot_ptr = 0x2_9000;
    let bitmap_path_ptr = 0x2_a000;
    let resource_dib_ptr = 0x2_b000;
    memory.map_words(size_ptr, 2);
    memory.map_words(image_info_ptr, 8);
    memory.map_words(drag_point_ptr, 2);
    memory.map_words(drag_hotspot_ptr, 2);
    memory.map_halfwords(bitmap_path_ptr, 64);
    memory.write_wide_z(bitmap_path_ptr, r"\Images\one.bmp");
    let resource_bmp = bmp_1x1_24bpp();
    let resource_dib = &resource_bmp[14..];
    memory.map_bytes(resource_dib_ptr, resource_dib.len() as u32);
    memory.write_bytes(resource_dib_ptr, resource_dib);
    kernel.resources.register(
        0x0040_0000,
        ResourceId::Integer(77),
        ResourceId::Integer(2),
        resource_dib_ptr,
        resource_dib.len() as u32,
    );

    let image_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [24, 20, 0, 1, 2],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create did not return a handle: {other:?}"),
    };
    assert_ne!(image_list, 0);
    assert_eq!(kernel.resources.image_list(image_list).unwrap().width, 24);
    assert_eq!(kernel.resources.image_list(image_list).unwrap().height, 20);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [image_list, 0x000a_1111, 0x000a_2222],
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
            ORD_IMAGE_LIST_REPLACE_ICON,
            [image_list, 0xffff_ffff, 0x000b_8123],
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
            ORD_IMAGE_LIST_GET_IMAGE_COUNT,
            [image_list],
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
            ORD_IMAGE_LIST_GET_ICON_SIZE,
            [image_list, size_ptr, size_ptr + 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(size_ptr)?, 24);
    assert_eq!(memory.read_i32(size_ptr + 4)?, 20);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_ICON_SIZE,
            [image_list, 32, 18],
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
            ORD_IMAGE_LIST_GET_ICON_SIZE,
            [image_list, size_ptr, size_ptr + 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(size_ptr)?, 32);
    assert_eq!(memory.read_i32(size_ptr + 4)?, 18);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON,
            [image_list, 1, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x000b_8123),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_REPLACE,
            [image_list, 0, 0x000a_3333, 0x000a_4444],
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
            ORD_IMAGE_LIST_GET_IMAGE_INFO,
            [image_list, 0, image_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(image_info_ptr)?, 0x000a_3333);
    assert_eq!(memory.read_u32(image_info_ptr + 4)?, 0x000a_4444);
    assert_eq!(memory.read_i32(image_info_ptr + 24)?, 32);
    assert_eq!(memory.read_i32(image_info_ptr + 28)?, 18);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_BK_COLOR,
            [image_list, 0x00ff_00ff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0xffff_ffff),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_BK_COLOR,
            [image_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x00ff_00ff),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            [image_list, 1, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(image_list)
            .unwrap()
            .overlays
            .get(&2),
        Some(&1)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON,
            [image_list, 1, 0x0200],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x020b_8123),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON,
            [image_list, 1, 0x0300],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x000b_8123),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [image_list, 1, 0x200, 3, 5, 24, 20, 0, 0, 0x0201],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let draw = kernel
        .resources
        .image_list(image_list)
        .unwrap()
        .last_draw
        .unwrap();
    assert_eq!(draw.index, 1);
    assert_eq!(draw.hdc, 0x200);
    assert_eq!(draw.x, 3);
    assert_eq!(draw.y, 5);
    assert_eq!(draw.flags, 0x0201);
    assert_eq!(draw.overlay_image, Some(1));

    let loaded = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [
            0,
            bitmap_path_ptr,
            12,
            1,
            0x00ff_00ff,
            IMAGE_BITMAP,
            LR_LOADFROMFILE,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_LoadImage did not return a handle: {other:?}"),
    };
    assert_ne!(loaded, 0);
    let loaded_list = kernel.resources.image_list(loaded).unwrap();
    assert_eq!(loaded_list.width, 12);
    assert_eq!(loaded_list.height, 1);
    assert_eq!(loaded_list.images.len(), 1);
    assert_ne!(loaded_list.images[0].bitmap, 0);
    assert_eq!(loaded_list.images[0].mask, 0x00ff_00ff);
    let hdc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetDC did not return a desktop HDC: {other:?}"),
    };
    let mut framebuffer = VirtualFramebuffer::new(20, 20, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [loaded, 0, hdc, 6, 7, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(!framebuffer.dirty_rects().is_empty());
    let pixel_offset = 7 * framebuffer.stride() + 6 * PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[pixel_offset..pixel_offset + 2],
        &[0x26, 0x9b],
        "ImageList_DrawEx should blit the loaded bitmap pixel, not just the pseudo-icon placeholder"
    );

    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 12, 12);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [loaded, 0, mem_dc, 4, 5, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 4, 5),
        0x9b26,
        "ImageList_DrawEx should blit bitmap-backed entries into selected memory DIBs"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 3, 5),
        0,
        "ImageList_DrawEx should leave memory DIB pixels outside the target rect untouched"
    );

    let resource_loaded = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [0x0040_0000, 77, 1, 1, 0xffff_ffff, IMAGE_BITMAP, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_LoadImage(resource bitmap) did not return a handle: {other:?}"),
    };
    assert_ne!(resource_loaded, 0);
    let resource_loaded_list = kernel.resources.image_list(resource_loaded).unwrap();
    assert_eq!(resource_loaded_list.width, 1);
    assert_eq!(resource_loaded_list.height, 1);
    assert_ne!(resource_loaded_list.images[0].bitmap, 0);
    let resource_bitmap = kernel
        .resources
        .bitmap(resource_loaded_list.images[0].bitmap)
        .unwrap();
    assert_ne!(
        resource_bitmap.bits_ptr, resource_dib_ptr,
        "resource-backed DIB pixels should be copied into owned bitmap storage"
    );
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [resource_loaded, 0, hdc, 8, 7, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let resource_pixel_offset =
        7 * framebuffer.stride() + 8 * PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[resource_pixel_offset..resource_pixel_offset + 2],
        &[0x26, 0x9b],
        "ImageList_LoadImage should blit RT_BITMAP DIB rows, not the DIB header bytes"
    );

    memory.write_wide_z(bitmap_path_ptr, r"\Images\masked.bmp");
    let strip_loaded = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [
            0,
            bitmap_path_ptr,
            1,
            1,
            0xffff_ffff,
            IMAGE_BITMAP,
            LR_LOADFROMFILE,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_LoadImage(strip bitmap) did not return a handle: {other:?}"),
    };
    let strip_loaded_list = kernel.resources.image_list(strip_loaded).unwrap();
    assert_eq!(strip_loaded_list.width, 1);
    assert_eq!(strip_loaded_list.images.len(), 2);
    assert_eq!(strip_loaded_list.images[0].source_x, 0);
    assert_eq!(strip_loaded_list.images[1].source_x, 1);
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [strip_loaded, 1, hdc, 10, 7, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let strip_pixel_offset = 7 * framebuffer.stride() + 10 * PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[strip_pixel_offset..strip_pixel_offset + 2],
        &[0xe0, 0x07],
        "ImageList_LoadImage should split bitmap strips and draw the selected source column"
    );

    memory.write_wide_z(bitmap_path_ptr, r"\Images\masked.bmp");
    let masked_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [0, bitmap_path_ptr, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("LoadImageW(masked bitmap) did not return a bitmap: {other:?}"),
    };
    assert_ne!(masked_bitmap, 0);
    let masked_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, 0, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(masked) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD_MASKED,
            [masked_list, masked_bitmap, 0x00ff_00ff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.resources.image_list(masked_list).unwrap().images[0].transparent_color,
        Some(0x00ff_00ff)
    );
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [masked_list, 0, hdc, 2, 3, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let masked_left = 3 * framebuffer.stride() + 2 * PixelFormat::Rgb565.bytes_per_pixel();
    let masked_right = masked_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[masked_left..masked_left + 2],
        &[0, 0],
        "ImageList_AddMasked should leave transparent source pixels untouched"
    );
    assert_eq!(
        &framebuffer.pixels()[masked_right..masked_right + 2],
        &[0xe0, 0x07],
        "ImageList_AddMasked should still blit non-transparent source pixels"
    );

    memory.write_wide_z(bitmap_path_ptr, r"\Images\mask.bmp");
    let mask_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [0, bitmap_path_ptr, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("LoadImageW(mask bitmap) did not return a bitmap: {other:?}"),
    };
    assert_ne!(mask_bitmap, 0);
    let mask_handle_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, 0, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(mask handle) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [mask_handle_list, masked_bitmap, mask_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let mask_image = &kernel
        .resources
        .image_list(mask_handle_list)
        .unwrap()
        .images[0];
    assert_eq!(mask_image.mask, mask_bitmap);
    assert_eq!(mask_image.transparent_color, None);
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [mask_handle_list, 0, hdc, 4, 5, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let mask_left = 5 * framebuffer.stride() + 4 * PixelFormat::Rgb565.bytes_per_pixel();
    let mask_right = mask_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[mask_left..mask_left + 2],
        &[0, 0],
        "ImageList_Add should leave pixels masked by a non-black hbmMask untouched"
    );
    assert_eq!(
        &framebuffer.pixels()[mask_right..mask_right + 2],
        &[0xe0, 0x07],
        "ImageList_Add should blit pixels allowed by a black hbmMask"
    );

    memory.write_wide_z(bitmap_path_ptr, r"\Images\red.bmp");
    let red_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [0, bitmap_path_ptr, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("LoadImageW(red bitmap) did not return a bitmap: {other:?}"),
    };
    assert_ne!(red_bitmap, 0);
    let overlay_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, 0, 2, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(overlay) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [overlay_list, red_bitmap, 0],
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
            ORD_IMAGE_LIST_ADD,
            [overlay_list, masked_bitmap, mask_bitmap],
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
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            [overlay_list, 1, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [overlay_list, 0, hdc, 6, 7, 2, 1, 0, 0, 0x0100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let overlay_left = 7 * framebuffer.stride() + 6 * PixelFormat::Rgb565.bytes_per_pixel();
    let overlay_right = overlay_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[overlay_left..overlay_left + 2],
        &[0x00, 0xf8],
        "ImageList overlay draw should preserve base pixels masked out by the overlay image"
    );
    assert_eq!(
        &framebuffer.pixels()[overlay_right..overlay_right + 2],
        &[0xe0, 0x07],
        "ImageList overlay draw should composite the registered overlay image"
    );

    let merged = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_MERGE,
        [image_list, 0, loaded, 0, 5, 7],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Merge did not return a handle: {other:?}"),
    };
    assert_ne!(merged, 0);
    let merged_list = kernel.resources.image_list(merged).unwrap();
    assert_eq!(merged_list.width, 37);
    assert_eq!(merged_list.height, 25);
    assert_eq!(merged_list.images.len(), 2);

    let duplicate = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_DUPLICATE,
        [image_list],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Duplicate did not return a handle: {other:?}"),
    };
    assert_ne!(duplicate, 0);
    assert_ne!(duplicate, image_list);
    assert_eq!(
        kernel.resources.image_list(duplicate).unwrap().images.len(),
        2
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY_DITHER_IMAGE,
            [image_list, 0, 6, 9, duplicate, 1, 0x0201],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let image_list_after_dither = kernel.resources.image_list(image_list).unwrap();
    assert_eq!(image_list_after_dither.images[0].icon, 0x000b_8123);
    let dither_copy = image_list_after_dither.last_dither_copy.unwrap();
    assert_eq!(dither_copy.dst_image_list, image_list);
    assert_eq!(dither_copy.dst_index, 0);
    assert_eq!(dither_copy.x, 6);
    assert_eq!(dither_copy.y, 9);
    assert_eq!(dither_copy.src_image_list, duplicate);
    assert_eq!(dither_copy.src_index, 1);
    assert_eq!(dither_copy.flags, 0x0201);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [image_list, 0, duplicate, 1, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.resources.image_list(duplicate).unwrap().images.len(),
        2
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON,
            [image_list, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x000b_8123),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_BEGIN_DRAG,
            [image_list, 0, 2, 3],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let drag = kernel.resources.image_list_drag().unwrap();
    assert_eq!(drag.image_list, image_list);
    assert_eq!(drag.index, 0);
    assert_eq!(drag.hotspot_x, 2);
    assert_eq!(drag.hotspot_y, 3);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAG_ENTER,
            [0x0007_0000, 10, 11],
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
            ORD_IMAGE_LIST_DRAG_MOVE,
            [20, 21],
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
            ORD_IMAGE_LIST_DRAG_SHOW_NOLOCK,
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
            ORD_IMAGE_LIST_SET_DRAG_CURSOR_IMAGE,
            [duplicate, 1, 4, 5],
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
            ORD_IMAGE_LIST_GET_DRAG_IMAGE,
            [drag_point_ptr, drag_hotspot_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == duplicate
    ));
    assert_eq!(memory.read_i32(drag_point_ptr)?, 20);
    assert_eq!(memory.read_i32(drag_point_ptr + 4)?, 21);
    assert_eq!(memory.read_i32(drag_hotspot_ptr)?, 4);
    assert_eq!(memory.read_i32(drag_hotspot_ptr + 4)?, 5);
    assert!(!kernel.resources.image_list_drag().unwrap().visible);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAG_LEAVE,
            [0x0007_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.resources.image_list_drag().unwrap().lock_hwnd, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_END_DRAG,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.image_list_drag().is_none());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_IMAGE_COUNT,
            [image_list, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(image_list)
            .unwrap()
            .images
            .len(),
        1
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_REMOVE,
            [image_list, 0xffff_ffff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(image_list)
            .unwrap()
            .images
            .len(),
        0
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DESTROY,
            [image_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.image_list(image_list).is_none());
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn image_list_draw_indirect_applies_x_bitmap_offset_and_rgb_bk_fill() -> Result<()> {
    // IMAGELISTDRAWPARAMS layout (4 bytes each, 14 fields = 56 bytes):
    // @0 cbSize @4 himl @8 i @12 hdcDst @16 x @20 y @24 cx @28 cy
    // @32 xBitmap @36 yBitmap @40 rgbBk @44 rgbFg @48 fStyle @52 dwRop
    const IMAGE_BITMAP: u32 = 0;
    const LR_LOADFROMFILE: u32 = 0x0000_0010;
    const ILC_COLOR16: u32 = 0x0010;
    const ILC_MASK: u32 = 0x0001;
    const CLR_NONE: u32 = 0xffff_ffff;
    const CLR_DEFAULT: u32 = 0xff00_0000;
    const IMLDP_WORDS: u32 = 14; // 56-byte struct / 4 = 14 words
    const PARAMS_PTR: u32 = 0x3_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("image_list_draw_indirect");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    // 2x1 BMP: pixel 0 = magenta (0xFF00FF), pixel 1 = green (0x00FF00).
    fs::write(root.join("mg.bmp"), bmp_2x1_magenta_green_24bpp()).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;

    // --- xBitmap offset test ---
    // Create a width=2 image list (each image is 2 pixels wide).
    let il = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, ILC_COLOR16, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("ImageList_Create failed: {other:?}"),
    };

    let path_ptr = 0x3_1000;
    memory.map_halfwords(path_ptr, 64);
    memory.write_wide_z(path_ptr, r"\mg.bmp");
    // Load the 2x1 bitmap from file (cx=2, cy=1, no transparent mask).
    let bmp_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [0, path_ptr, IMAGE_BITMAP, 2, 1, LR_LOADFROMFILE],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("LoadImage failed: {other:?}"),
    };

    // ImageList_Add: adds the bitmap as a single image at source_x=0.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [il, bmp_handle, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    // Create 4x2 memory DC to draw into.
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 4, 2);

    // Allocate IMAGELISTDRAWPARAMS in guest memory (14 words = 56 bytes).
    memory.map_words(PARAMS_PTR, IMLDP_WORDS);

    // Draw with xBitmap=0: reads from source_x=0 → magenta pixel blitted to dest (0,0).
    memory.write_word(PARAMS_PTR, 56); // cbSize
    memory.write_word(PARAMS_PTR + 4, il); // himl
    memory.write_word(PARAMS_PTR + 8, 0); // i=0
    memory.write_word(PARAMS_PTR + 12, mem_dc); // hdcDst
    memory.write_word(PARAMS_PTR + 16, 0); // x=0
    memory.write_word(PARAMS_PTR + 20, 0); // y=0
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 32, 0); // xBitmap=0
    memory.write_word(PARAMS_PTR + 36, 0); // yBitmap=0
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk
    memory.write_word(PARAMS_PTR + 44, CLR_DEFAULT); // rgbFg
    memory.write_word(PARAMS_PTR + 48, 0); // fStyle
    memory.write_word(PARAMS_PTR + 52, 0); // dwRop
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_INDIRECT,
            [PARAMS_PTR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let magenta_rgb565 = 0xF81F_u16;
    let green_rgb565 = 0x07E0_u16;
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 0, 0),
        magenta_rgb565,
        "xBitmap=0 should draw magenta (first pixel of the 2x1 bitmap)"
    );

    // Draw with xBitmap=1: source_x shifts by 1 → green pixel blitted to dest (1,0).
    memory.write_word(PARAMS_PTR + 16, 1); // x=1
    memory.write_word(PARAMS_PTR + 32, 1); // xBitmap=1
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_INDIRECT,
            [PARAMS_PTR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 1, 0),
        green_rgb565,
        "xBitmap=1 should draw green (second pixel of the 2x1 bitmap)"
    );

    // --- rgb_bk fill test ---
    // Create a 1x1 image list with magenta as the transparent mask color.
    let il2 = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLOR16 | ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("ImageList_Create #2 failed: {other:?}"),
    };

    // Load the 2x1 BMP at 1x1 crop to get just the magenta pixel.
    let bmp_1x1_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [0, path_ptr, IMAGE_BITMAP, 1, 1, LR_LOADFROMFILE],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("LoadImage 1x1 failed: {other:?}"),
    };
    // ImageList_AddMasked with magenta (0x00FF00FF) → whole 1x1 image is transparent.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD_MASKED,
            [il2, bmp_1x1_handle, 0x00ff_00ff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.resources.image_list(il2).unwrap().images[0].transparent_color,
        Some(0x00ff_00ff),
        "image should carry the magenta transparent_color"
    );

    // Draw with rgb_bk=CLR_NONE (ILD_TRANSPARENT forced): transparent pixel stays 0.
    memory.write_word(PARAMS_PTR + 4, il2); // himl=il2
    memory.write_word(PARAMS_PTR + 16, 2); // x=2
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 32, 0); // xBitmap=0
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk=CLR_NONE
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_INDIRECT,
            [PARAMS_PTR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 2, 0),
        0,
        "rgb_bk=CLR_NONE: transparent draw should leave dest pixel untouched (no bg fill)"
    );

    // Draw with rgb_bk=green (0x0000FF00): transparent area should be filled with green.
    memory.write_word(PARAMS_PTR + 16, 3); // x=3
    memory.write_word(PARAMS_PTR + 40, 0x0000_ff00); // rgbBk=green
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_INDIRECT,
            [PARAMS_PTR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 3, 0),
        green_rgb565,
        "rgb_bk=green: transparent area should be pre-filled with background color"
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn sh_get_file_info_rejects_unsupported_and_colliding_flags() -> Result<()> {
    const ERROR_INVALID_FLAGS: u32 = 1004;
    const SHFILEINFO_SIZE_W: u32 = 692;
    const SHGFI_DISPLAYNAME: u32 = 0x0000_0200;
    const SHGFI_ATTRIBUTES: u32 = 0x0000_0800;
    const SHGFI_ICONLOCATION: u32 = 0x0000_1000;
    const SHGFI_SMALLICON: u32 = 0x0000_0001;
    const SHGFI_USEFILEATTRIBUTES: u32 = 0x0000_0010;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;
    let path_ptr = 0x2_7000;
    let info_ptr = 0x2_8000;
    memory.map_halfwords(path_ptr, 64);
    memory.write_wide_z(path_ptr, r"\Docs\morning.nav");
    memory.map_words(info_ptr, SHFILEINFO_SIZE_W / 4);

    for flags in [
        SHGFI_ICONLOCATION,
        SHGFI_ATTRIBUTES | SHGFI_USEFILEATTRIBUTES,
        SHGFI_DISPLAYNAME | SHGFI_SMALLICON,
    ] {
        let result = table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHGET_FILE_INFO,
            [path_ptr, 0, info_ptr, SHFILEINFO_SIZE_W, flags],
        );
        assert!(matches!(
            result,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ));
        assert_eq!(
            kernel.threads.get_last_error(thread_id),
            ERROR_INVALID_FLAGS
        );
    }

    Ok(())
}

#[test]
fn shell_notify_icon_tracks_add_modify_delete_and_posts_callback() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIM_MODIFY: u32 = 1;
    const NIM_DELETE: u32 = 2;
    const NIF_MESSAGE: u32 = 0x0000_0001;
    const NIF_ICON: u32 = 0x0000_0002;
    const NIF_TIP: u32 = 0x0000_0004;
    const NIF_STATE: u32 = 0x0000_0008;
    const HHTBF_DESTROYICON: u32 = 0x1000_0000;
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
    memory.write_word(
        data + 12,
        NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_STATE | HHTBF_DESTROYICON,
    );
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
    assert!(icon.destroy_icon);
    assert!(kernel.post_shell_notify_icon_callback(hwnd, 77, WM_LBUTTONDOWN));
    let callback = kernel
        .gwe
        .peek_message_filtered(
            thread_id,
            Some(hwnd),
            WM_USER + 88,
            WM_USER + 88,
            PeekFlags::REMOVE,
        )
        .expect("notify callback message");
    assert_eq!(callback.hwnd, hwnd);
    assert_eq!(callback.msg, WM_USER + 88);
    assert_eq!(callback.wparam, 77);
    assert_eq!(callback.lparam, WM_LBUTTONDOWN);

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
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        Vec::<u32>::new()
    );

    memory.write_word(data + 12, NIF_ICON | HHTBF_DESTROYICON);
    memory.write_word(data + 20, 0x000b_8002);
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
    assert_eq!(icon.icon, 0x000b_8002);
    assert!(icon.destroy_icon);
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_8001]
    );

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
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_8001, 0x000b_8002]
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(!kernel.post_shell_notify_icon_callback(hwnd, 77, WM_LBUTTONDOWN));

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
    const SHNUM_UNKNOWN: u32 = 0x8000;
    const SHNN_SHOW: u32 = 0xffff_fc16;
    const SHNN_DISMISS: u32 = 0xffff_fc17;
    const SHNN_LINKSEL: u32 = 0xffff_fc18;

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
    let msg_ptr = 0x3002_fc00;
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
    memory.map_words(msg_ptr, 7);
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

    memory.write_word(data + 40, 0xDEAD_BEEF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_ADD_I,
            [data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
            ..
        }
    ));
    assert!(kernel.shell.notification(clsid, 301).is_none());
    memory.write_word(data + 40, hwnd);

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
    assert!(kernel.post_shell_notification_callback(clsid, 301, SHNN_SHOW, 0, 0));
    let callbacks = kernel
        .shell
        .notification_callbacks()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(callbacks.len(), 1);
    assert_eq!(callbacks[0].clsid, clsid);
    assert_eq!(callbacks[0].id, 301);
    assert_eq!(callbacks[0].lparam, 0xCAFE_BABE);
    assert_eq!(
        callbacks[0].method,
        ShellNotificationCallbackMethod::OnShow { x: 0, y: 0 }
    );
    assert_eq!(
        callbacks[0].vtable_offset,
        ISHELL_NOTIFICATION_CALLBACK_ON_SHOW_VTABLE_OFFSET
    );
    assert_eq!(
        callbacks[0].arguments,
        ShellNotificationCallbackArguments::OnShow {
            id: 301,
            x: 0,
            y: 0,
            lparam: 0xCAFE_BABE
        }
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_NOTIFY, WM_NOTIFY, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_NOTIFY);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 301);
    let nmshn_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(nmshn_ptr, 0);
    assert_eq!(memory.read_u32(nmshn_ptr)?, 0);
    assert_eq!(memory.read_u32(nmshn_ptr + 4)?, 301);
    assert_eq!(memory.read_u32(nmshn_ptr + 8)?, SHNN_SHOW);
    assert_eq!(memory.read_u32(nmshn_ptr + 12)?, 0xCAFE_BABE);
    assert_eq!(memory.read_u32(nmshn_ptr + 16)?, 0);
    assert_eq!(memory.read_u32(nmshn_ptr + 20)?, 0);
    assert_eq!(memory.read_u32(nmshn_ptr + 24)?, 0);
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, nmshn_ptr)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, nmshn_ptr)
            .is_none()
    );

    assert!(kernel.post_shell_notification_link_callback(clsid, 301, "cmd:route"));
    let callbacks = kernel
        .shell
        .notification_callbacks()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(callbacks.len(), 2);
    assert_eq!(
        callbacks[1].method,
        ShellNotificationCallbackMethod::OnLinkSelected {
            link: "cmd:route".to_owned()
        }
    );
    assert_eq!(
        callbacks[1].vtable_offset,
        ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET
    );
    assert_eq!(
        callbacks[1].arguments,
        ShellNotificationCallbackArguments::OnLinkSelected {
            id: 301,
            link: "cmd:route".to_owned(),
            lparam: 0xCAFE_BABE
        }
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_NOTIFY, WM_NOTIFY, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let link_nmshn_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(link_nmshn_ptr, 0);
    assert_eq!(memory.read_u32(link_nmshn_ptr)?, 0);
    assert_eq!(memory.read_u32(link_nmshn_ptr + 4)?, 301);
    assert_eq!(memory.read_u32(link_nmshn_ptr + 8)?, SHNN_LINKSEL);
    assert_eq!(memory.read_u32(link_nmshn_ptr + 12)?, 0xCAFE_BABE);
    assert_eq!(memory.read_u32(link_nmshn_ptr + 16)?, 0);
    let link_ptr = memory.read_u32(link_nmshn_ptr + 20)?;
    assert_eq!(link_ptr, link_nmshn_ptr + 28);
    assert_eq!(memory.read_u32(link_nmshn_ptr + 24)?, 0);
    assert_eq!(memory.read_wide_z(link_ptr, 32), "cmd:route");
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, link_nmshn_ptr)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, link_nmshn_ptr)
            .is_none()
    );

    assert!(kernel.post_shell_notification_dismiss_callback(clsid, 301, true));
    let callbacks = kernel
        .shell
        .notification_callbacks()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(callbacks.len(), 3);
    assert_eq!(
        callbacks[2].method,
        ShellNotificationCallbackMethod::OnDismiss { timed_out: true }
    );
    assert_eq!(
        callbacks[2].vtable_offset,
        ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET
    );
    assert_eq!(
        callbacks[2].arguments,
        ShellNotificationCallbackArguments::OnDismiss {
            id: 301,
            timed_out: true,
            lparam: 0xCAFE_BABE
        }
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_NOTIFY, WM_NOTIFY, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let dismiss_nmshn_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(dismiss_nmshn_ptr, 0);
    assert_eq!(memory.read_u32(dismiss_nmshn_ptr)?, 0);
    assert_eq!(memory.read_u32(dismiss_nmshn_ptr + 4)?, 301);
    assert_eq!(memory.read_u32(dismiss_nmshn_ptr + 8)?, SHNN_DISMISS);
    assert_eq!(memory.read_u32(dismiss_nmshn_ptr + 12)?, 0xCAFE_BABE);
    assert_eq!(memory.read_u32(dismiss_nmshn_ptr + 16)?, 0);
    assert_eq!(memory.read_u32(dismiss_nmshn_ptr + 20)?, 1);
    assert_eq!(memory.read_u32(dismiss_nmshn_ptr + 24)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, dismiss_nmshn_ptr)
            .is_none()
    );
    assert!(kernel.shell.notification(clsid, 301).is_some());

    assert!(kernel.post_shell_notification_command_callback(clsid, 301, 0x1234));
    let callbacks = kernel
        .shell
        .notification_callbacks()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(callbacks.len(), 4);
    assert_eq!(
        callbacks[3].method,
        ShellNotificationCallbackMethod::OnCommandSelected { command_id: 0x1234 }
    );
    assert_eq!(
        callbacks[3].vtable_offset,
        ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET
    );
    assert_eq!(
        callbacks[3].arguments,
        ShellNotificationCallbackArguments::OnCommandSelected {
            id: 301,
            command_id: 0x1234
        }
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_COMMAND, WM_COMMAND, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_COMMAND);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x1234);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 301);

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

    memory.write_word(data + 40, 0xDEAD_BEEF);
    memory.write_wide_z(title, "Ignored sink");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_TITLE, data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.hwnd_sink, hwnd);
    assert_eq!(record.title, "Route alert");
    memory.write_word(data + 40, hwnd);

    memory.write_wide_z(title, "Mask ignored");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [0, data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_DATA),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_UNKNOWN, data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_DATA),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.title, "Route alert");
    assert_eq!(record.icon, 0x000b_9001);

    memory.write_word(data + 8, SHNP_ICONIC);
    memory.write_word(data + 12, 3);
    memory.write_word(data + 16, 0x000b_9002);
    memory.write_word(data + 20, 0x24);
    memory.write_word(data + 52, 0xDEAD_C0DE);
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
    assert_eq!(record.flags, 0x10);
    assert_eq!(record.hwnd_sink, hwnd);
    assert_eq!(record.lparam, 0xCAFE_BABE);
    assert_eq!(record.title, "Route changed");
    assert_eq!(record.html, "<i>Later</i>");

    memory.write_word(data + 16, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_ICON, data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.icon, 0x000b_9002);

    memory.write_word(data + 12, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_DURATION, data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.duration_cs, 0);
    assert_eq!(record.expires_at_ms, Some(record.added_at_ms));

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
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(out + 20)?, 0x10);
    assert_eq!(memory.read_u32(out + 40)?, hwnd);
    assert_eq!(memory.read_u32(out + 52)?, 0xCAFE_BABE);

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
fn shnotification_i_posts_timeout_dismiss_and_removes_expired_record() -> Result<()> {
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_INFORM: u32 = 0x1b1;
    const SHNN_DISMISS: u32 = 0xffff_fc17;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHN_TIMEOUT", "", None, 0, 0, 0);
    let data = 0x3003_0000;
    let title = 0x3003_1000;
    let html = 0x3003_2000;
    let msg_ptr = 0x3003_3000;
    let clsid = [
        0xaa, 0xbb, 0xcc, 0xdd, 0x10, 0x20, 0x30, 0x40, 0x55, 0x66, 0x77, 0x88, 0x90, 0xa0, 0xb0,
        0xc0,
    ];
    memory.map_words(data, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(title, 64);
    memory.map_bytes(html, 128);
    memory.map_words(msg_ptr, 7);
    memory.write_word(data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(data + 4, 401);
    memory.write_word(data + 8, SHNP_INFORM);
    memory.write_word(data + 12, 1);
    memory.write_word(data + 16, 0x000b_9001);
    memory.write_word(data + 20, 0x10);
    memory.write_bytes(data + 24, &clsid);
    memory.write_word(data + 40, hwnd);
    memory.write_word(data + 52, 0xFACE_CAFE);
    memory.write_wide_z(title, "Auto dismiss");
    memory.write_wide_z(html, "<b>Timed route</b>");

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
    let record = kernel.shell.notification(clsid, 401).expect("notification");
    assert_eq!(record.duration_cs, 1);
    assert!(record.expires_at_ms.is_some());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_NOTIFY, WM_NOTIFY, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_SLEEP, [9]),
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_NOTIFY, WM_NOTIFY, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_NOTIFY);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 401);
    let nmshn_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(nmshn_ptr, 0);
    assert_eq!(memory.read_u32(nmshn_ptr + 4)?, 401);
    assert_eq!(memory.read_u32(nmshn_ptr + 8)?, SHNN_DISMISS);
    assert_eq!(memory.read_u32(nmshn_ptr + 12)?, 0xFACE_CAFE);
    assert_eq!(memory.read_u32(nmshn_ptr + 20)?, 1);
    assert!(kernel.shell.notification(clsid, 401).is_none());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, nmshn_ptr)
            .is_none()
    );

    Ok(())
}

#[test]
fn sh_change_notify_i_tracks_register_remove_and_free_state() -> Result<()> {
    const ERROR_INVALID_WINDOW_HANDLE: u32 = 1400;
    const SHCNE_CREATE: u32 = 0x0000_0002;
    const SHCNE_DELETE: u32 = 0x0000_0004;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHCNE_SINK", "", None, 0, 0, 0);
    let entry = 0x3003_1000;
    let watch_dir = 0x3003_2000;
    let packed = 0x3003_3000;
    memory.map_words(entry, 3);
    memory.map_halfwords(watch_dir, 64);
    memory.map_words(packed, 2);
    memory.write_word(entry, SHCNE_CREATE | SHCNE_DELETE);
    memory.write_word(entry + 4, watch_dir);
    memory.write_word(entry + 8, 1);
    memory.write_wide_z(watch_dir, r"\Windows");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCHANGE_NOTIFY_REGISTER_I,
            [hwnd, entry],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let registration = kernel
        .shell
        .change_notification(hwnd)
        .expect("change notification registration");
    assert_eq!(registration.event_mask, SHCNE_CREATE | SHCNE_DELETE);
    assert_eq!(registration.watch_dir.as_deref(), Some(r"\Windows"));
    assert!(registration.recursive);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_word(entry, SHCNE_DELETE);
    memory.write_word(entry + 8, 0);
    memory.write_word(packed, hwnd);
    memory.write_word(packed + 4, entry);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCHANGE_NOTIFY_REGISTER_I,
            [packed, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let registrations: Vec<_> = kernel.shell.change_notifications().cloned().collect();
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].event_mask, SHCNE_DELETE);
    assert!(!registrations[0].recursive);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_FREE_I,
            [0x3003_f000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel
            .shell
            .freed_file_notifications()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x3003_f000]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_REMOVE_I,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.shell.change_notification(hwnd).is_none());
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_REMOVE_I,
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

    Ok(())
}

#[test]
fn sh_change_notify_i_posts_filechangeinfo_for_matching_file_operations() -> Result<()> {
    const WM_FILECHANGEINFO: u32 = 0x8000 + 0x101;
    const SHCNE_RENAMEITEM: u32 = 0x0000_0001;
    const SHCNE_MKDIR: u32 = 0x0000_0008;
    const SHCNE_DRIVEREMOVED: u32 = 0x0000_0080;
    const SHCNE_DRIVEADD: u32 = 0x0000_0100;
    const SHCNE_CREATE: u32 = 0x0000_0002;
    const SHCNF_IDLIST: u32 = 0x0000_0000;
    const SHCNF_PATHW: u32 = 0x0000_0005;
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
    const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x0000_0100;
    const FILECHANGEINFO_SIZE: u32 = 36;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let root = unique_test_root("shell-filechange");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    kernel.set_file_root(&root);

    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHCNE_DELIVERY", "", None, 0, 0, 0);
    let idlist_hwnd =
        kernel.create_window_ex_w(thread_id, "SHCNE_IDLIST_DELIVERY", "", None, 0, 0, 0);
    let entry = 0x3004_1000;
    let watch_dir = 0x3004_2000;
    let path1 = 0x3004_3000;
    let path2 = 0x3004_4000;
    let msg_ptr = 0x3004_5000;
    let idlist_entry = 0x3004_6000;
    let idlist_watch_dir = 0x3004_7000;
    let idlist_msg_ptr = 0x3004_8000;
    memory.map_words(entry, 3);
    memory.map_halfwords(watch_dir, 64);
    memory.map_halfwords(path1, 128);
    memory.map_halfwords(path2, 128);
    memory.map_words(msg_ptr, 7);
    memory.map_words(idlist_entry, 4);
    memory.map_halfwords(idlist_watch_dir, 64);
    memory.map_words(idlist_msg_ptr, 7);
    memory.write_word(
        entry,
        SHCNE_MKDIR | SHCNE_RENAMEITEM | SHCNE_DRIVEADD | SHCNE_DRIVEREMOVED,
    );
    memory.write_word(entry + 4, watch_dir);
    memory.write_word(entry + 8, 1);
    memory.write_wide_z(watch_dir, r"\");
    memory.write_word(idlist_entry, SHCNE_CREATE);
    memory.write_word(idlist_entry + 4, idlist_watch_dir);
    memory.write_word(idlist_entry + 8, 1);
    memory.write_word(idlist_entry + 12, SHCNF_IDLIST);
    memory.write_wide_z(idlist_watch_dir, r"\Windows");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCHANGE_NOTIFY_REGISTER_I,
            [hwnd, entry],
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
            ORD_SHCHANGE_NOTIFY_REGISTER_I,
            [idlist_hwnd, idlist_entry],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_wide_z(path1, r"\Windows\Routes");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_DIRECTORY_W,
            [path1, 0],
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_FILECHANGEINFO, WM_FILECHANGEINFO, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_FILECHANGEINFO);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, SHCNE_MKDIR);
    let mkdir_notify = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(mkdir_notify, 0);
    assert_eq!(memory.read_u32(mkdir_notify)?, 1);
    assert_eq!(memory.read_u32(mkdir_notify + 4)?, FILECHANGEINFO_SIZE);
    assert_eq!(memory.read_u32(mkdir_notify + 8)?, SHCNE_MKDIR);
    assert_eq!(memory.read_u32(mkdir_notify + 12)?, SHCNF_PATHW);
    let mkdir_path = memory.read_u32(mkdir_notify + 16)?;
    assert_eq!(memory.read_u32(mkdir_notify + 20)?, 0);
    assert_eq!(memory.read_wide_z(mkdir_path, 64), r"\Windows\Routes");
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, mkdir_notify)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_FREE_I,
            [mkdir_notify],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, mkdir_notify)
            .is_none()
    );

    memory.write_wide_z(path1, r"\Windows\PidlRoutes.txt");
    let create_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path1, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("unexpected CreateFileW result: {other:?}"),
    };
    assert_ne!(create_handle, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [create_handle],
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
            ORD_PEEK_MESSAGE_W,
            [
                idlist_msg_ptr,
                idlist_hwnd,
                WM_FILECHANGEINFO,
                WM_FILECHANGEINFO,
                1
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(idlist_msg_ptr)?, idlist_hwnd);
    assert_eq!(memory.read_u32(idlist_msg_ptr + 4)?, WM_FILECHANGEINFO);
    assert_eq!(memory.read_u32(idlist_msg_ptr + 8)?, SHCNE_CREATE);
    let idlist_notify = memory.read_u32(idlist_msg_ptr + 12)?;
    assert_ne!(idlist_notify, 0);
    assert_eq!(memory.read_u32(idlist_notify + 8)?, SHCNE_CREATE);
    assert_eq!(memory.read_u32(idlist_notify + 12)?, SHCNF_IDLIST);
    let pidl_ptr = memory.read_u32(idlist_notify + 16)?;
    assert_ne!(pidl_ptr, 0);
    assert_eq!(memory.read_u32(idlist_notify + 20)?, 0);
    let pidl_cb = u32::from(memory.read_u16(pidl_ptr)?);
    assert!(pidl_cb > 4);
    let mut pidl_units = Vec::new();
    for index in 0..64 {
        let unit = memory.read_u16(pidl_ptr + 2 + index * 2)?;
        if unit == 0 {
            break;
        }
        pidl_units.push(unit);
    }
    assert_eq!(
        String::from_utf16_lossy(&pidl_units),
        r"\Windows\PidlRoutes.txt"
    );
    assert_eq!(memory.read_u16(pidl_ptr + pidl_cb)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_FREE_I,
            [idlist_notify],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    fs::write(root.join("Windows").join("old.txt"), b"old").unwrap();
    memory.write_wide_z(path1, r"\Windows\old.txt");
    memory.write_wide_z(path2, r"\Windows\new.txt");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [path1, path2],
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_FILECHANGEINFO, WM_FILECHANGEINFO, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_FILECHANGEINFO);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, SHCNE_RENAMEITEM);
    let rename_notify = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(rename_notify, 0);
    assert_eq!(memory.read_u32(rename_notify + 8)?, SHCNE_RENAMEITEM);
    assert_eq!(memory.read_u32(rename_notify + 12)?, SHCNF_PATHW);
    let old_path = memory.read_u32(rename_notify + 16)?;
    let new_path = memory.read_u32(rename_notify + 20)?;
    assert_ne!(old_path, 0);
    assert_ne!(new_path, 0);
    assert_eq!(memory.read_wide_z(old_path, 64), r"\Windows\old.txt");
    assert_eq!(memory.read_wide_z(new_path, 64), r"\Windows\new.txt");
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, rename_notify)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_FREE_I,
            [rename_notify],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, rename_notify)
            .is_none()
    );
    assert_eq!(
        kernel
            .shell
            .freed_file_notifications()
            .copied()
            .collect::<Vec<_>>(),
        vec![mkdir_notify, idlist_notify, rename_notify]
    );

    fs::create_dir_all(root.join("SDMMC")).unwrap();
    kernel.mount_guest_root(r"\SDMMC Disk", root.join("SDMMC"));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_FILECHANGEINFO, WM_FILECHANGEINFO, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr + 8)?, SHCNE_DRIVEADD);
    let drive_add_notify = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(drive_add_notify, 0);
    assert_eq!(memory.read_u32(drive_add_notify + 8)?, SHCNE_DRIVEADD);
    assert_eq!(memory.read_u32(drive_add_notify + 12)?, SHCNF_PATHW);
    let drive_add_path = memory.read_u32(drive_add_notify + 16)?;
    assert_eq!(memory.read_u32(drive_add_notify + 20)?, 0);
    assert_eq!(memory.read_wide_z(drive_add_path, 64), r"\SDMMC Disk");
    assert_eq!(
        memory.read_u32(drive_add_notify + 24)?,
        FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_TEMPORARY
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_FREE_I,
            [drive_add_notify],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    assert!(kernel.unmount_guest_root(r"\SDMMC Disk"));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_FILECHANGEINFO, WM_FILECHANGEINFO, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr + 8)?, SHCNE_DRIVEREMOVED);
    let drive_remove_notify = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(drive_remove_notify, 0);
    assert_eq!(
        memory.read_u32(drive_remove_notify + 8)?,
        SHCNE_DRIVEREMOVED
    );
    assert_eq!(memory.read_u32(drive_remove_notify + 12)?, SHCNF_PATHW);
    let drive_remove_path = memory.read_u32(drive_remove_notify + 16)?;
    assert_eq!(memory.read_wide_z(drive_remove_path, 64), r"\SDMMC Disk");
    assert_eq!(memory.read_u32(drive_remove_notify + 24)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHFILE_NOTIFY_FREE_I,
            [drive_remove_notify],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel
            .shell
            .freed_file_notifications()
            .copied()
            .collect::<Vec<_>>(),
        vec![
            mkdir_notify,
            idlist_notify,
            rename_notify,
            drive_add_notify,
            drive_remove_notify
        ]
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn mount_unmount_broadcasts_wm_devicechange_to_top_level_windows() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let msg_ptr = 0x1_0000u32;
    memory.map_words(msg_ptr, 7);
    const WM_DEVICECHANGE: u32 = 0x0219;
    const DBT_DEVICEARRIVAL: u32 = 0x8000;
    const DBT_DEVICEREMOVECOMPLETE: u32 = 0x8004;

    let hwnd = kernel.create_window_ex_w(thread_id, "DEVCHG", "", None, 0, 0, 0);

    let root = unique_test_root("wm_devicechange_mount");
    let sdmmc = root.join("SDMMC");
    fs::create_dir_all(&sdmmc).unwrap();

    // Mount broadcasts WM_DEVICECHANGE(DBT_DEVICEARRIVAL) to the top-level window.
    kernel.mount_guest_root(r"\SDMMC Disk", &sdmmc);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_DEVICECHANGE, WM_DEVICECHANGE, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    // MSG layout: hwnd@0, msg@4, wparam@8, lparam@12
    assert_eq!(memory.read_u32(msg_ptr + 0)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_DEVICECHANGE);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, DBT_DEVICEARRIVAL);

    // Unmount broadcasts WM_DEVICECHANGE(DBT_DEVICEREMOVECOMPLETE).
    assert!(kernel.unmount_guest_root(r"\SDMMC Disk"));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_DEVICECHANGE, WM_DEVICECHANGE, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr + 0)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_DEVICECHANGE);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, DBT_DEVICEREMOVECOMPLETE);

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn shell_window_destroy_removes_notify_icon_and_notification_state() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIF_MESSAGE: u32 = 0x0000_0001;
    const NIF_ICON: u32 = 0x0000_0002;
    const NIF_TIP: u32 = 0x0000_0004;
    const HHTBF_DESTROYICON: u32 = 0x1000_0000;
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
    let change_entry = 0x3003_4000;
    let clsid = [
        0x44, 0x33, 0x22, 0x11, 0xaa, 0xbb, 0xcc, 0xdd, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x20, 0x30,
        0x40,
    ];
    memory.write_word(notify_icon, NID_SIZE);
    memory.write_word(notify_icon + 4, hwnd);
    memory.write_word(notify_icon + 8, 11);
    memory.write_word(
        notify_icon + 12,
        NIF_MESSAGE | NIF_ICON | NIF_TIP | HHTBF_DESTROYICON,
    );
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
    memory.map_words(change_entry, 3);
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
    memory.write_word(change_entry, 0x0000_0002);
    memory.write_word(change_entry + 4, 0);
    memory.write_word(change_entry + 8, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCHANGE_NOTIFY_REGISTER_I,
            [hwnd, change_entry],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.shell.change_notification(hwnd).is_some());

    assert!(kernel.destroy_window(hwnd));

    assert!(kernel.shell.notify_icon(hwnd, 11).is_none());
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_8002]
    );
    assert!(kernel.shell.notification(clsid, 402).is_none());
    assert!(kernel.shell.change_notification(hwnd).is_none());

    let process_id = 0x2200;
    kernel.set_current_process_id(process_id);
    let process_hwnd =
        kernel.create_window_ex_w(thread_id, "SHN_PROCESS_CLEANUP", "", None, 0, 0, 0);
    let process_clsid = [
        0x21, 0x43, 0x65, 0x87, 0xa9, 0xcb, 0xed, 0x0f, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22,
        0x11,
    ];
    memory.write_word(notify_icon, NID_SIZE);
    memory.write_word(notify_icon + 4, process_hwnd);
    memory.write_word(notify_icon + 8, 12);
    memory.write_word(
        notify_icon + 12,
        NIF_MESSAGE | NIF_ICON | NIF_TIP | HHTBF_DESTROYICON,
    );
    memory.write_word(notify_icon + 16, WM_USER + 12);
    memory.write_word(notify_icon + 20, 0x000b_8003);
    memory.write_wide_z(notify_icon + NID_TIP_OFFSET, "process cleanup");
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
    memory.write_word(notify_data + 4, 403);
    memory.write_word(notify_data + 8, SHNP_INFORM);
    memory.write_word(notify_data + 12, 0);
    memory.write_word(notify_data + 16, 0x000b_9004);
    memory.write_word(notify_data + 20, 0);
    memory.write_bytes(notify_data + 24, &process_clsid);
    memory.write_word(notify_data + 40, process_hwnd);
    memory.write_word(notify_data + 52, 0);
    memory.write_wide_z(title, "Process Cleanup");
    memory.write_wide_z(html, "<p>Process cleanup</p>");
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
    memory.write_word(change_entry, 0x0000_0002);
    memory.write_word(change_entry + 4, 0);
    memory.write_word(change_entry + 8, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHCHANGE_NOTIFY_REGISTER_I,
            [process_hwnd, change_entry],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.shell.notify_icon(process_hwnd, 12).is_some());
    assert!(kernel.shell.notification(process_clsid, 403).is_some());
    assert!(kernel.shell.change_notification(process_hwnd).is_some());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TERMINATE_PROCESS,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x55],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window(process_hwnd));
    assert!(kernel.shell.notify_icon(process_hwnd, 12).is_none());
    assert!(kernel.shell.notification(process_clsid, 403).is_none());
    assert!(kernel.shell.change_notification(process_hwnd).is_none());
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_8002, 0x000b_8003]
    );

    Ok(())
}

#[test]
fn message_box_w_records_text_owner_and_returns_default_button() -> Result<()> {
    const MB_YESNOCANCEL: u32 = 0x0000_0003;
    const MB_YESALL: u32 = 0x0000_0006;
    const MB_CANCEL: u32 = 0x0000_0007;
    const MB_DEFBUTTON2: u32 = 0x0000_0100;
    const MB_ICONQUESTION: u32 = 0x0000_0020;
    const IDCANCEL: u32 = 2;
    const IDYES: u32 = 6;
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
    let mut framebuffer = VirtualFramebuffer::new(260, 140, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();

    let result = table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel,
        &mut memory,
        Some(&mut framebuffer),
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
    assert_ne!(record.dialog_hwnd, 0);
    assert_ne!(record.text_hwnd, 0);
    assert_eq!(record.text, "Route search failed");
    assert_eq!(record.caption, "iNavi");
    assert_eq!(
        record.style,
        MB_YESNOCANCEL | MB_DEFBUTTON2 | MB_ICONQUESTION
    );
    assert_eq!(record.buttons, vec![IDYES, IDNO, IDCANCEL]);
    assert_eq!(record.default_button_index, 1);
    assert_eq!(record.icon, Some(MessageBoxIcon::Question));
    assert_eq!(record.result, IDNO);
    assert!(record.rendered);
    assert_eq!(record.button_hwnds.len(), 3);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));
    assert!(!kernel.gwe.is_window(record.dialog_hwnd));
    assert!(!kernel.gwe.is_window(record.text_hwnd));
    assert!(
        record
            .button_hwnds
            .iter()
            .all(|button_hwnd| !kernel.gwe.is_window(*button_hwnd))
    );
    assert_eq!(record.owner_was_enabled, Some(true));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(!framebuffer.dirty_rects().is_empty());
    let pixel = |x: usize, y: usize| {
        let offset = y * framebuffer.stride() + x * PixelFormat::Rgb565.bytes_per_pixel();
        &framebuffer.pixels()[offset..offset + PixelFormat::Rgb565.bytes_per_pixel()]
    };
    assert_ne!(pixel(30, 30), &[0, 0]);
    assert_ne!(pixel(45, 32), &[0, 0]);
    assert_ne!(pixel(46, 55), &[0, 0]);
    assert_ne!(pixel(58, 87), &[0, 0]);

    let yes_all = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESALL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        yes_all,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    let record = kernel.shell.last_message_box().expect("yes-all record");
    assert_eq!(record.style, MB_YESALL | MB_DEFBUTTON2);
    assert_eq!(record.buttons, vec![IDYES, IDNO, IDCANCEL]);
    assert_eq!(record.button_hwnds.len(), 3);
    assert_eq!(
        record
            .button_layout
            .iter()
            .map(|button| (button.id, button.label, button.slot))
            .collect::<Vec<_>>(),
        vec![
            (
                IDYES,
                MessageBoxButtonLabel::Yes,
                MessageBoxButtonSlot::Left
            ),
            (
                IDNO,
                MessageBoxButtonLabel::No,
                MessageBoxButtonSlot::Center
            ),
            (
                IDCANCEL,
                MessageBoxButtonLabel::YesAll,
                MessageBoxButtonSlot::Right
            ),
        ]
    );
    assert_eq!(record.default_button_index, 1);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));
    assert_eq!(record.icon, None);
    assert!(!record.rendered);

    let cancel_only = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_CANCEL],
    );
    assert!(matches!(
        cancel_only,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    let record = kernel.shell.last_message_box().expect("cancel-only record");
    assert_eq!(record.style, MB_CANCEL);
    assert_eq!(record.buttons, vec![IDCANCEL]);
    assert_eq!(record.button_hwnds.len(), 1);
    assert_eq!(
        record
            .button_layout
            .iter()
            .map(|button| (button.id, button.label, button.slot))
            .collect::<Vec<_>>(),
        vec![(
            IDCANCEL,
            MessageBoxButtonLabel::Cancel,
            MessageBoxButtonSlot::Center
        )]
    );
    assert_eq!(record.default_button_index, 0);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert_eq!(record.icon, None);

    assert_eq!(kernel.enable_window(hwnd, false), Some(true));
    let disabled_owner = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_CANCEL],
    );
    assert!(matches!(
        disabled_owner,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("disabled-owner record");
    assert_eq!(record.owner_was_enabled, Some(false));
    assert!(!kernel.gwe.is_window_enabled(hwnd));

    Ok(())
}

#[test]
fn message_box_w_uses_queued_modal_key_and_button_input() -> Result<()> {
    const MB_YESNOCANCEL: u32 = 0x0000_0003;
    const MB_DEFBUTTON2: u32 = 0x0000_0100;
    const IDCANCEL: u32 = 2;
    const IDYES: u32 = 6;
    const IDNO: u32 = 7;
    const VK_ESCAPE: u32 = 0x1b;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 51;
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGBOX_INPUT_OWNER", "", None, 0, 0, 0);
    let text = 0x3006_0000;
    let caption = 0x3006_1000;
    memory.write_wide_z(text, "Choose a route");
    memory.write_wide_z(caption, "iNavi");

    let dialog_hwnd = hwnd + 4;
    kernel.gwe.post_message(
        thread_id,
        Message::new(dialog_hwnd, WM_KEYDOWN, VK_ESCAPE, 0, 10),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    let result_value = match result {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } => value,
        other => panic!("unexpected MessageBoxW escape result: {other:?}"),
    };
    let record = kernel
        .shell
        .last_message_box()
        .expect("escape-input record");
    assert_eq!(result_value, IDCANCEL, "record={record:?}");
    assert_eq!(record.dialog_hwnd, dialog_hwnd);
    assert_eq!(record.buttons, vec![IDYES, IDNO, IDCANCEL]);
    assert_eq!(record.default_button_index, 1);
    assert_eq!(record.result, IDCANCEL);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(dialog_hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let next_dialog_hwnd = hwnd + 24;
    let yes_button_hwnd = next_dialog_hwnd + 8;
    kernel.gwe.post_message(
        thread_id,
        Message::new(yes_button_hwnd, WM_LBUTTONUP, 1, 0, 20),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDYES),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("button-input record");
    assert_eq!(record.dialog_hwnd, next_dialog_hwnd);
    assert_eq!(record.button_hwnds[0], yes_button_hwnd);
    assert_eq!(record.result, IDYES);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDYES));

    let command_dialog_hwnd = hwnd + 44;
    kernel.gwe.post_message(
        thread_id,
        Message::new(command_dialog_hwnd, WM_COMMAND, IDNO, 0, 30),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("command-input record");
    assert_eq!(record.dialog_hwnd, command_dialog_hwnd);
    assert_eq!(record.result, IDNO);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(command_dialog_hwnd),
                WM_COMMAND,
                WM_COMMAND,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let close_dialog_hwnd = hwnd + 64;
    kernel.gwe.post_message(
        thread_id,
        Message::new(close_dialog_hwnd, WM_CLOSE, 0, 0, 40),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    let record = kernel.shell.last_message_box().expect("close-input record");
    assert_eq!(record.dialog_hwnd, close_dialog_hwnd);
    assert_eq!(record.result, IDCANCEL);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(close_dialog_hwnd),
                WM_CLOSE,
                WM_CLOSE,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let sysclose_dialog_hwnd = hwnd + 84;
    kernel.gwe.post_message(
        thread_id,
        Message::new(sysclose_dialog_hwnd, WM_SYSCOMMAND, SC_CLOSE, 0, 50),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("sysclose-input record");
    assert_eq!(record.dialog_hwnd, sysclose_dialog_hwnd);
    assert_eq!(record.result, IDCANCEL);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(sysclose_dialog_hwnd),
                WM_SYSCOMMAND,
                WM_SYSCOMMAND,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let char_return_dialog_hwnd = hwnd + 104;
    kernel.gwe.post_message(
        thread_id,
        Message::new(char_return_dialog_hwnd, WM_CHAR, 0x0d, 0, 60),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("char-return-input record");
    assert_eq!(record.dialog_hwnd, char_return_dialog_hwnd);
    assert_eq!(record.result, IDNO);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(char_return_dialog_hwnd),
                WM_CHAR,
                WM_CHAR,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let char_escape_dialog_hwnd = hwnd + 124;
    kernel.gwe.post_message(
        thread_id,
        Message::new(char_escape_dialog_hwnd, WM_CHAR, 0x1b, 0, 70),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("char-escape-input record");
    assert_eq!(record.dialog_hwnd, char_escape_dialog_hwnd);
    assert_eq!(record.result, IDCANCEL);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(char_escape_dialog_hwnd),
                WM_CHAR,
                WM_CHAR,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let down_only_dialog_hwnd = hwnd + 144;
    let down_only_yes_button_hwnd = down_only_dialog_hwnd + 8;
    kernel.gwe.post_message(
        thread_id,
        Message::new(down_only_yes_button_hwnd, WM_LBUTTONDOWN, 1, 0, 80),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("button-down-only record");
    assert_eq!(record.dialog_hwnd, down_only_dialog_hwnd);
    assert_eq!(record.result, IDNO);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(down_only_yes_button_hwnd),
                WM_LBUTTONDOWN,
                WM_LBUTTONDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let dialog_click_dialog_hwnd = hwnd + 164;
    let no_button_lparam = make_lparam(112, 70);
    kernel.gwe.post_message(
        thread_id,
        Message::new(
            dialog_click_dialog_hwnd,
            WM_LBUTTONUP,
            0,
            no_button_lparam,
            90,
        ),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("dialog-click-button record");
    assert_eq!(record.dialog_hwnd, dialog_click_dialog_hwnd);
    assert_eq!(record.result, IDNO);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));

    let outside_click_dialog_hwnd = hwnd + 184;
    kernel.gwe.post_message(
        thread_id,
        Message::new(
            outside_click_dialog_hwnd,
            WM_LBUTTONUP,
            0,
            make_lparam(4, 4),
            100,
        ),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("dialog-outside-click record");
    assert_eq!(record.dialog_hwnd, outside_click_dialog_hwnd);
    assert_eq!(record.result, IDNO);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));

    let nested_other =
        kernel.create_window_ex_w(thread_id, "MSGBOX_NESTED_OTHER", "", None, 0, 0, 0);
    let nested_dialog_hwnd = nested_other + 4;
    kernel
        .gwe
        .post_message(thread_id, Message::new(nested_other, WM_CLOSE, 0, 0, 110));
    kernel.gwe.post_message(
        thread_id,
        Message::new(nested_dialog_hwnd, WM_KEYDOWN, VK_ESCAPE, 0, 111),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    assert!(
        !kernel.gwe.is_window(nested_other),
        "nested modal pump should dispatch unrelated same-thread WM_CLOSE before dialog input"
    );
    let record = kernel
        .shell
        .last_message_box()
        .expect("nested-pump input record");
    assert_eq!(record.dialog_hwnd, nested_dialog_hwnd);
    assert_eq!(record.result, IDCANCEL);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(nested_dialog_hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let sent_other = kernel.create_window_ex_w(thread_id, "MSGBOX_SENT_OTHER", "", None, 0, 0, 0);
    let sent_dialog_hwnd = sent_other + 4;
    let send_id = kernel
        .begin_cross_thread_send_message_w(777, sent_other, WM_CLOSE, 0, 0, None)
        .expect("cross-thread sent close should queue");
    kernel.gwe.post_message(
        thread_id,
        Message::new(sent_dialog_hwnd, WM_KEYDOWN, VK_ESCAPE, 0, 121),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    assert!(
        !kernel.gwe.is_window(sent_other),
        "nested modal pump should dispatch unrelated sent WM_CLOSE before dialog input"
    );
    assert_eq!(kernel.take_completed_send_message_result(send_id), Some(0));
    let record = kernel
        .shell
        .last_message_box()
        .expect("nested-sent-pump input record");
    assert_eq!(record.dialog_hwnd, sent_dialog_hwnd);
    assert_eq!(record.result, IDCANCEL);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(sent_dialog_hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let paint_other = kernel.create_window_ex_w_with_rect(
        thread_id,
        "MSGBOX_PAINT_OTHER",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(320, 200, 96, 48),
    );
    assert!(
        kernel.gwe.update_rect(paint_other).is_some(),
        "visible non-dialog window starts with pending generated paint"
    );
    let pending_paint = kernel
        .gwe
        .peek_message_filtered(thread_id, None, WM_PAINT, WM_PAINT, PeekFlags::NO_REMOVE)
        .expect("visible non-dialog window should expose generated paint");
    assert_eq!(pending_paint.hwnd, paint_other);
    let paint_dialog_hwnd = paint_other + 4;
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [0, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));
    assert!(
        kernel.gwe.update_rect(paint_other).is_none(),
        "modal MessageBox pump should dispatch generated non-dialog WM_PAINT before default fallback"
    );
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(paint_other),
                WM_PAINT,
                WM_PAINT,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let record = kernel
        .shell
        .last_message_box()
        .expect("generated-paint modal record");
    assert_eq!(record.dialog_hwnd, paint_dialog_hwnd);
    assert_eq!(record.result, IDNO);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDNO));

    Ok(())
}

#[test]
fn message_box_w_tab_navigation_changes_activated_button() -> Result<()> {
    // The CE dialog manager focuses the default button and lets Tab/Shift+Tab and
    // arrow keys move focus between the box's buttons; a following Enter or Space
    // activates the navigated button rather than the original default.
    const MB_YESNOCANCEL: u32 = 0x0000_0003;
    const MB_DEFBUTTON2: u32 = 0x0000_0100;
    const IDCANCEL: u32 = 2;
    const IDYES: u32 = 6;
    const VK_TAB: u32 = 0x09;
    const VK_RETURN: u32 = 0x0d;
    const VK_SPACE: u32 = 0x20;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 53;
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGBOX_TAB_OWNER", "", None, 0, 0, 0);
    let text = 0x3007_0000;
    let caption = 0x3007_1000;
    memory.write_wide_z(text, "Pick a route");
    memory.write_wide_z(caption, "iNavi");

    // First box: focus starts on the default No button (index 1). Tab forward twice
    // (No -> Cancel -> wraps to Yes) then Enter activates the focused Yes button.
    let tab_dialog_hwnd = hwnd + 4;
    kernel.gwe.post_message(
        thread_id,
        Message::new(tab_dialog_hwnd, WM_KEYDOWN, VK_TAB, 0, 10),
    );
    kernel.gwe.post_message(
        thread_id,
        Message::new(tab_dialog_hwnd, WM_KEYDOWN, VK_TAB, 0, 11),
    );
    kernel.gwe.post_message(
        thread_id,
        Message::new(tab_dialog_hwnd, WM_KEYDOWN, VK_RETURN, 0, 12),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDYES),
            ..
        }
    ));
    let record = kernel.shell.last_message_box().expect("tab-return record");
    assert_eq!(record.dialog_hwnd, tab_dialog_hwnd);
    assert_eq!(record.result, IDYES);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDYES));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(tab_dialog_hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    // Second box: Tab forward once (No -> Cancel) then Space activates the Cancel button.
    let space_dialog_hwnd = hwnd + 24;
    kernel.gwe.post_message(
        thread_id,
        Message::new(space_dialog_hwnd, WM_KEYDOWN, VK_TAB, 0, 20),
    );
    kernel.gwe.post_message(
        thread_id,
        Message::new(space_dialog_hwnd, WM_KEYDOWN, VK_SPACE, 0, 21),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL | MB_DEFBUTTON2],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    let record = kernel.shell.last_message_box().expect("tab-space record");
    assert_eq!(record.dialog_hwnd, space_dialog_hwnd);
    assert_eq!(record.result, IDCANCEL);
    assert_eq!(kernel.gwe.dialog_result(record.dialog_hwnd), Some(IDCANCEL));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(space_dialog_hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    Ok(())
}

#[test]
fn message_box_w_mnemonic_key_activates_matching_button() -> Result<()> {
    // CE dialog manager: WM_CHAR with the mnemonic letter of a button activates it.
    // Mnemonics from CE ownerdrawlib odlib.rc &-markers: OK→'o', Cancel→'c', Yes→'y',
    // No→'n', Quit(Abort)→'q', Retry→'r', Ignore→'i', YesAll(Y&es to All)→'e'.
    // WM_SYSCHAR (Alt+letter) is also accepted.
    const MB_YESNOCANCEL: u32 = 0x0000_0003;
    const MB_ABORTRETRYIGNORE: u32 = 0x0000_0002;
    const IDABORT: u32 = 3;
    const IDCANCEL: u32 = 2;
    const IDNO: u32 = 7;
    const WM_SYSCHAR: u32 = 0x0106;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 53;
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGBOX_MNEMONIC_OWNER", "", None, 0, 0, 0);
    let text = 0x3007_0000;
    let caption = 0x3007_1000;
    memory.write_wide_z(text, "Question");
    memory.write_wide_z(caption, "Test");

    // First box (MB_YESNOCANCEL): WM_CHAR 'n' activates No (IDNO=7).
    // Windows created: dialog (+4), static (+8), Yes (+12), No (+16), Cancel (+20).
    let no_dialog_hwnd = hwnd + 4;
    kernel.gwe.post_message(
        thread_id,
        Message::new(no_dialog_hwnd, WM_CHAR, 'n' as u32, 0, 10),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDNO),
            ..
        }
    ));

    // Second box (MB_YESNOCANCEL): WM_SYSCHAR 'C' (uppercase) activates Cancel.
    // Dialog at hwnd+24 (5 windows consumed per MB call).
    let cancel_dialog_hwnd = hwnd + 24;
    kernel.gwe.post_message(
        thread_id,
        Message::new(cancel_dialog_hwnd, WM_SYSCHAR, 'C' as u32, 0, 20),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_YESNOCANCEL],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));

    // Third box (MB_ABORTRETRYIGNORE): WM_CHAR 'q' activates Quit/Abort (IDABORT=3).
    // CE ownerdrawlib displays Abort as "Quit" with mnemonic 'q'.
    // Dialog at hwnd+44.
    let abort_dialog_hwnd = hwnd + 44;
    kernel.gwe.post_message(
        thread_id,
        Message::new(abort_dialog_hwnd, WM_CHAR, 'q' as u32, 0, 30),
    );
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_ABORTRETRYIGNORE],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDABORT),
            ..
        }
    ));

    Ok(())
}

fn make_lparam(x: i32, y: i32) -> u32 {
    ((y as u16 as u32) << 16) | (x as u16 as u32)
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
fn clipboard_delayed_render_requests_owner_and_accepts_rendered_handle() -> Result<()> {
    const CF_TEXT: u32 = 1;
    const CF_UNICODETEXT: u32 = 13;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let owner_thread = 52;
    let consumer_thread = 53;
    let owner = kernel.create_window_ex_w(owner_thread, "CLIP_RENDER_OWNER", "", None, 0, 0, 0);
    let consumer =
        kernel.create_window_ex_w(consumer_thread, "CLIP_RENDER_CONSUMER", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
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
            owner_thread,
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
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_UNICODETEXT, 0],
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
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_TEXT, 0],
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
            owner_thread,
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
            consumer_thread,
            ORD_OPEN_CLIPBOARD,
            [consumer],
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
            consumer_thread,
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
            consumer_thread,
            ORD_GET_CLIPBOARD_DATA,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(consumer_thread), 0);

    let render_message = kernel
        .gwe
        .peek_message_filtered(
            owner_thread,
            Some(owner),
            WM_RENDERFORMAT,
            WM_RENDERFORMAT,
            PeekFlags::REMOVE,
        )
        .expect("delayed render should queue WM_RENDERFORMAT to owner");
    assert_eq!(render_message.hwnd, owner);
    assert_eq!(render_message.wparam, CF_UNICODETEXT);
    assert_eq!(render_message.lparam, 0);
    assert!(kernel.gwe.in_send_message(owner_thread));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            consumer_thread,
            ORD_GET_CLIPBOARD_DATA,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                owner_thread,
                Some(owner),
                WM_RENDERFORMAT,
                WM_RENDERFORMAT,
                PeekFlags::NO_REMOVE,
            )
            .is_none(),
        "nested delayed render reads should not queue duplicate WM_RENDERFORMAT"
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_UNICODETEXT, 0x7000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7000),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .complete_active_sent_message(owner_thread, 0)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            consumer_thread,
            ORD_GET_CLIPBOARD_DATA,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7000),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(consumer_thread), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            consumer_thread,
            ORD_GET_CLIPBOARD_DATA,
            [CF_TEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    let stale_render_message = kernel
        .gwe
        .peek_message_filtered(
            owner_thread,
            Some(owner),
            WM_RENDERFORMAT,
            WM_RENDERFORMAT,
            PeekFlags::REMOVE,
        )
        .expect("second delayed format should queue WM_RENDERFORMAT");
    assert_eq!(stale_render_message.hwnd, owner);
    assert_eq!(stale_render_message.wparam, CF_TEXT);
    assert!(
        kernel
            .gwe
            .complete_active_sent_message(owner_thread, 0)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_TEXT, 0x7100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(owner_thread),
        ERROR_ACCESS_DENIED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            consumer_thread,
            ORD_GET_CLIPBOARD_DATA,
            [CF_TEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    let retry_render_message = kernel
        .gwe
        .peek_message_filtered(
            owner_thread,
            Some(owner),
            WM_RENDERFORMAT,
            WM_RENDERFORMAT,
            PeekFlags::REMOVE,
        )
        .expect("abandoned delayed render should be requestable again");
    assert_eq!(retry_render_message.hwnd, owner);
    assert_eq!(retry_render_message.wparam, CF_TEXT);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_TEXT, 0x7100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7100),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .complete_active_sent_message(owner_thread, 0)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            consumer_thread,
            ORD_GET_CLIPBOARD_DATA,
            [CF_TEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7100),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(consumer_thread), 0);

    Ok(())
}

#[test]
fn empty_clipboard_notifies_previous_owner_and_reassigns_owner() -> Result<()> {
    const CF_TEXT: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let owner_thread = 54;
    let replacer_thread = 55;
    let owner = kernel.create_window_ex_w(owner_thread, "CLIP_DESTROY_OWNER", "", None, 0, 0, 0);
    let replacer =
        kernel.create_window_ex_w(replacer_thread, "CLIP_DESTROY_REPLACER", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
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
            owner_thread,
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
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_TEXT, 0x7100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7100),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
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
            replacer_thread,
            ORD_OPEN_CLIPBOARD,
            [replacer],
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
            replacer_thread,
            ORD_EMPTY_CLIPBOARD,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let destroy_message = kernel
        .gwe
        .peek_message_filtered(
            owner_thread,
            Some(owner),
            WM_DESTROYCLIPBOARD,
            WM_DESTROYCLIPBOARD,
            PeekFlags::REMOVE,
        )
        .expect("emptying live clipboard data should notify previous owner");
    assert_eq!(destroy_message.hwnd, owner);
    assert_eq!(destroy_message.wparam, 0);
    assert_eq!(destroy_message.lparam, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            replacer_thread,
            ORD_GET_CLIPBOARD_OWNER,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == replacer
    ));

    Ok(())
}

#[test]
fn destroying_clipboard_owner_clears_open_owner_and_unrendered_formats() -> Result<()> {
    const CF_UNICODETEXT: u32 = 13;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 56;
    let owner = kernel.create_window_ex_w(thread_id, "CLIP_DESTROYED_OWNER", "", None, 0, 0, 0);

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
            ORD_SET_CLIPBOARD_DATA,
            [CF_UNICODETEXT, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));

    assert!(kernel.destroy_window_with_reason(owner, "clipboard-owner-test"));
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
            ORD_GET_CLIPBOARD_OWNER,
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
            ORD_IS_CLIPBOARD_FORMAT_AVAILABLE,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn destroying_clipboard_owner_requests_render_all_and_keeps_rendered_data() -> Result<()> {
    const CF_TEXT: u32 = 1;
    const CF_UNICODETEXT: u32 = 13;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let owner_thread = 57;
    let consumer_thread = 58;
    let owner = kernel.create_window_ex_w(owner_thread, "CLIP_RENDER_ALL_OWNER", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
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
            owner_thread,
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
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_TEXT, 0x7200],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7200),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_SET_CLIPBOARD_DATA,
            [CF_UNICODETEXT, 0],
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
            owner_thread,
            ORD_CLOSE_CLIPBOARD,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert!(kernel.destroy_window_with_reason(owner, "clipboard-render-all-test"));
    assert!(kernel.recent_message_ops().iter().any(|record| {
        record.op == "send_message"
            && record.thread_id == owner_thread
            && record.hwnd == Some(owner)
            && record.msg == Some(WM_RENDERALLFORMATS)
            && record.detail.as_deref() == Some("clipboard_render_all")
    }));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            consumer_thread,
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
            consumer_thread,
            ORD_GET_CLIPBOARD_DATA,
            [CF_TEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x7200),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            consumer_thread,
            ORD_IS_CLIPBOARD_FORMAT_AVAILABLE,
            [CF_UNICODETEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
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
fn clipboard_data_alloc_copies_known_local_handle_contents() -> Result<()> {
    const CF_TEXT: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 53;
    let owner = kernel.create_window_ex_w(thread_id, "CLIP_ALLOC", "", None, 0, 0, 0);
    let source = kernel.memory.local_alloc(0, 6).unwrap();
    memory.write_bytes(source, b"abc\0xy");

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
            ORD_SET_CLIPBOARD_DATA,
            [CF_TEXT, source],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == source
    ));

    let copy = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_CLIPBOARD_DATA_ALLOC,
        [CF_TEXT],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("unexpected GetClipboardDataAlloc result: {other:?}"),
    };
    assert_ne!(copy, 0);
    assert_ne!(copy, source);
    assert_eq!(kernel.memory.local_size(copy), Some(6));
    assert_eq!(memory.read_bytes(copy, 6), b"abc\0xy");

    memory.write_bytes(source, b"zzz\0xy");
    assert_eq!(memory.read_bytes(copy, 6), b"abc\0xy");
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn clipboard_data_alloc_rejects_unknown_source_handle() -> Result<()> {
    const CF_TEXT: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 54;
    let owner = kernel.create_window_ex_w(thread_id, "CLIP_BAD_ALLOC", "", None, 0, 0, 0);

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
            ORD_SET_CLIPBOARD_DATA,
            [CF_TEXT, 0x7000],
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
            ORD_GET_CLIPBOARD_DATA_ALLOC,
            [CF_TEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
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

#[test]
fn coredll_raw_enter_and_delete_critical_section_complete_lifecycle() -> Result<()> {
    const CS_LOCK_COUNT_OFFSET: u32 = 0;
    const CS_OWNER_THREAD_OFFSET: u32 = 4;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7_u32;
    let cs_ptr = 0x2500_u32;
    memory.map_words(cs_ptr, 5);

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_INITIALIZE_CRITICAL_SECTION,
        [cs_ptr],
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_ENTER_CRITICAL_SECTION,
                [cs_ptr]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ),
        "EnterCriticalSection must return 0"
    );
    assert_eq!(
        memory.read_u32(cs_ptr + CS_LOCK_COUNT_OFFSET)?,
        1,
        "lock count must be 1 after first enter"
    );
    assert_eq!(
        memory.read_u32(cs_ptr + CS_OWNER_THREAD_OFFSET)?,
        thread_id,
        "owner must be the current thread"
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_ENTER_CRITICAL_SECTION,
        [cs_ptr],
    );
    assert_eq!(
        memory.read_u32(cs_ptr + CS_LOCK_COUNT_OFFSET)?,
        2,
        "lock count must be 2 after re-entrant enter"
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LEAVE_CRITICAL_SECTION,
        [cs_ptr],
    );
    assert_eq!(
        memory.read_u32(cs_ptr + CS_LOCK_COUNT_OFFSET)?,
        1,
        "lock count must drop to 1 after first leave"
    );
    assert_eq!(
        memory.read_u32(cs_ptr + CS_OWNER_THREAD_OFFSET)?,
        thread_id,
        "owner must still be set after first leave"
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LEAVE_CRITICAL_SECTION,
        [cs_ptr],
    );
    assert_eq!(
        memory.read_u32(cs_ptr + CS_OWNER_THREAD_OFFSET)?,
        0,
        "owner must be cleared after final leave"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_DELETE_CRITICAL_SECTION,
                [cs_ptr]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ),
        "DeleteCriticalSection must return 0"
    );
    for word_index in 0..5_u32 {
        assert_eq!(
            memory.read_u32(cs_ptr + word_index * 4)?,
            0,
            "DeleteCriticalSection must zero all CS fields"
        );
    }

    Ok(())
}
