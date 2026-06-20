use std::fs;

use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_ACTIVATE_DEVICE, ORD_ACTIVATE_DEVICE_EX, ORD_BATTERY_DRVR_GET_LEVELS,
            ORD_BATTERY_DRVR_SUPPORTS_CHANGE_NOTIFICATION, ORD_BATTERY_GET_LIFE_TIME_INFO,
            ORD_BATTERY_NOTIFY_OF_TIME_CHANGE, ORD_CE_FIND_CLOSE_REG_CHANGE,
            ORD_CE_FIND_FIRST_REG_CHANGE, ORD_CE_FIND_NEXT_REG_CHANGE, ORD_CE_GET_MODULE_INFO,
            ORD_CE_GET_THREAD_PRIORITY, ORD_CE_GET_THREAD_QUANTUM, ORD_CE_OPEN_FILE_HANDLE,
            ORD_CE_SET_THREAD_PRIORITY, ORD_CE_SET_THREAD_QUANTUM, ORD_CLEAR_COMM_ERROR,
            ORD_CLOSE_CLIPBOARD, ORD_CLOSE_HANDLE, ORD_CLOSE_MSG_QUEUE,
            ORD_COUNT_CLIPBOARD_FORMATS, ORD_CREATE_COMPATIBLE_DC, ORD_CREATE_DIBSECTION,
            ORD_CREATE_DIRECTORY_W, ORD_CREATE_EVENT_W, ORD_CREATE_FILE_W, ORD_CREATE_MSG_QUEUE,
            ORD_CREATE_PROCESS_W, ORD_CREATE_SEMAPHORE_W, ORD_CREATE_THREAD, ORD_DEACTIVATE_DEVICE,
            ORD_DELETE_CRITICAL_SECTION, ORD_DELETE_OBJECT, ORD_DEREGISTER_DEVICE,
            ORD_DESTROY_CURSOR, ORD_DESTROY_ICON, ORD_DEVICE_POWER_NOTIFY,
            ORD_DISABLE_THREAD_LIBRARY_CALLS, ORD_DISPATCH_MESSAGE_W, ORD_DRAW_ICON_EX,
            ORD_EMPTY_CLIPBOARD, ORD_ENTER_CRITICAL_SECTION, ORD_ENUM_CLIPBOARD_FORMATS,
            ORD_ENUM_DEVICE_INTERFACES, ORD_ENUM_DEVICES, ORD_ENUM_PNP_IDS, ORD_EVENT_MODIFY,
            ORD_EXTRACT_ICON_EX_W, ORD_FILE_TIME_TO_SYSTEM_TIME, ORD_FREE_LIBRARY,
            ORD_GET_CALL_STACK_SNAPSHOT, ORD_GET_CALLER_PROCESS_INDEX, ORD_GET_CLIPBOARD_DATA,
            ORD_GET_CLIPBOARD_DATA_ALLOC, ORD_GET_CLIPBOARD_FORMAT_NAME_W, ORD_GET_CLIPBOARD_OWNER,
            ORD_GET_COMM_MASK, ORD_GET_COMM_MODEM_STATUS, ORD_GET_COMM_PROPERTIES,
            ORD_GET_COMM_STATE, ORD_GET_COMM_TIMEOUTS, ORD_GET_DC, ORD_GET_DEVICE_KEYS,
            ORD_GET_DEVICE_POWER, ORD_GET_EXIT_CODE_PROCESS, ORD_GET_EXIT_CODE_THREAD,
            ORD_GET_FILE_VERSION_INFO_SIZE_W, ORD_GET_FILE_VERSION_INFO_W, ORD_GET_HEAP_SNAPSHOT,
            ORD_GET_ICON_INFO, ORD_GET_LAST_ERROR, ORD_GET_LOCAL_TIME, ORD_GET_MODULE_FILE_NAME_W,
            ORD_GET_MODULE_HANDLE_W, ORD_GET_MODULE_INFORMATION, ORD_GET_MSG_QUEUE_INFO,
            ORD_GET_OPEN_CLIPBOARD_WINDOW, ORD_GET_PRIORITY_CLIPBOARD_FORMAT,
            ORD_GET_PROC_ADDRESS_A, ORD_GET_PROC_ADDRESS_IN_PROCESS, ORD_GET_PROC_ADDRESS_W,
            ORD_GET_PROC_NAME, ORD_GET_PROCESS_ID, ORD_GET_PROCESS_IDFROM_INDEX,
            ORD_GET_PROCESS_INDEX_FROM_ID, ORD_GET_PROCESS_VERSION, ORD_GET_STORE_INFORMATION,
            ORD_GET_SYSTEM_MEMORY_DIVISION, ORD_GET_SYSTEM_POWER_STATE,
            ORD_GET_SYSTEM_POWER_STATUS_EX, ORD_GET_SYSTEM_POWER_STATUS_EX2, ORD_GET_SYSTEM_TIME,
            ORD_GET_SYSTEM_TIME_AS_FILE_TIME, ORD_GET_THREAD_ID, ORD_GET_THREAD_PRIORITY,
            ORD_GET_THREAD_TIMES, ORD_GET_TICK_COUNT, ORD_GET_TIME_ZONE_INFORMATION,
            ORD_GET_VERSION_EX_W, ORD_IMAGE_LIST_ADD, ORD_IMAGE_LIST_ADD_MASKED,
            ORD_IMAGE_LIST_BEGIN_DRAG, ORD_IMAGE_LIST_COPY, ORD_IMAGE_LIST_COPY_DITHER_IMAGE,
            ORD_IMAGE_LIST_CREATE, ORD_IMAGE_LIST_DESTROY, ORD_IMAGE_LIST_DRAG_ENTER,
            ORD_IMAGE_LIST_DRAG_LEAVE, ORD_IMAGE_LIST_DRAG_MOVE, ORD_IMAGE_LIST_DRAG_SHOW_NOLOCK,
            ORD_IMAGE_LIST_DRAW, ORD_IMAGE_LIST_DRAW_EX, ORD_IMAGE_LIST_DRAW_INDIRECT,
            ORD_IMAGE_LIST_DUPLICATE, ORD_IMAGE_LIST_END_DRAG, ORD_IMAGE_LIST_GET_BK_COLOR,
            ORD_IMAGE_LIST_GET_DRAG_IMAGE, ORD_IMAGE_LIST_GET_ICON, ORD_IMAGE_LIST_GET_ICON_SIZE,
            ORD_IMAGE_LIST_GET_IMAGE_COUNT, ORD_IMAGE_LIST_GET_IMAGE_INFO,
            ORD_IMAGE_LIST_LOAD_IMAGE, ORD_IMAGE_LIST_MERGE, ORD_IMAGE_LIST_REMOVE,
            ORD_IMAGE_LIST_REPLACE, ORD_IMAGE_LIST_REPLACE_ICON, ORD_IMAGE_LIST_SET_BK_COLOR,
            ORD_IMAGE_LIST_SET_DRAG_CURSOR_IMAGE, ORD_IMAGE_LIST_SET_ICON_SIZE,
            ORD_IMAGE_LIST_SET_IMAGE_COUNT, ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            ORD_INITIALIZE_CRITICAL_SECTION, ORD_INPUT_DEBUG_CHAR_W,
            ORD_INTERLOCKED_COMPARE_EXCHANGE, ORD_INTERLOCKED_EXCHANGE_ADD,
            ORD_INTERLOCKED_INCREMENT, ORD_IS_CLIPBOARD_FORMAT_AVAILABLE, ORD_KERN_EXTRACT_ICONS,
            ORD_KERNEL_IO_CONTROL, ORD_KEYBD_GET_DEVICE_INFO, ORD_LEAVE_CRITICAL_SECTION,
            ORD_LOAD_CURSOR_W, ORD_LOAD_DRIVER, ORD_LOAD_FSD, ORD_LOAD_FSDEX, ORD_LOAD_IMAGE_W,
            ORD_LOAD_KERNEL_LIBRARY, ORD_LOAD_LIBRARY_EX_W, ORD_LOAD_LIBRARY_W, ORD_MBSTOWCS,
            ORD_MESSAGE_BOX_W, ORD_MOVE_FILE_W, ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            ORD_MULTI_BYTE_TO_WIDE_CHAR, ORD_NLED_GET_DEVICE_INFO, ORD_NLED_SET_DEVICE,
            ORD_OPEN_CLIPBOARD, ORD_OPEN_DEVICE_KEY, ORD_OPEN_EVENT_W, ORD_OPEN_MSG_QUEUE,
            ORD_OPEN_PROCESS, ORD_PAGE_OUT_MODULE, ORD_PEEK_MESSAGE_W, ORD_PROCESS_DETACH_ALL_DLLS,
            ORD_PURGE_COMM, ORD_QUERY_INSTRUCTION_SET, ORD_QUERY_PERFORMANCE_COUNTER,
            ORD_QUERY_PERFORMANCE_FREQUENCY, ORD_READ_MSG_QUEUE, ORD_READ_PROCESS_MEMORY,
            ORD_REGISTER_CLIPBOARD_FORMAT_W, ORD_REGISTER_DEVICE, ORD_REGISTER_POWER_RELATIONSHIP,
            ORD_REGISTER_TASK_BAR, ORD_RELEASE_MUTEX, ORD_RELEASE_POWER_RELATIONSHIP,
            ORD_RELEASE_POWER_REQUIREMENT, ORD_RELEASE_SEMAPHORE, ORD_REQUEST_DEVICE_NOTIFICATIONS,
            ORD_REQUEST_POWER_NOTIFICATIONS, ORD_RESOURCE_CREATE_LIST, ORD_RESOURCE_DESTROY_LIST,
            ORD_RESOURCE_MARK_AS_SHAREABLE, ORD_RESOURCE_RELEASE, ORD_RESOURCE_REQUEST,
            ORD_RESOURCE_REQUEST_EX, ORD_RESUME_THREAD, ORD_SELECT_OBJECT, ORD_SET_CLIPBOARD_DATA,
            ORD_SET_COMM_MASK, ORD_SET_COMM_STATE, ORD_SET_COMM_TIMEOUTS, ORD_SET_DEVICE_POWER,
            ORD_SET_LAST_ERROR, ORD_SET_POWER_REQUIREMENT, ORD_SET_SYSTEM_POWER_STATE,
            ORD_SET_THREAD_PRIORITY, ORD_SHADD_TO_RECENT_DOCS, ORD_SHCHANGE_NOTIFY_REGISTER_I,
            ORD_SHCREATE_SHORTCUT, ORD_SHCREATE_SHORTCUT_EX, ORD_SHELL_EXECUTE_EX,
            ORD_SHELL_NOTIFY_ICON, ORD_SHFILE_NOTIFY_FREE_I, ORD_SHFILE_NOTIFY_REMOVE_I,
            ORD_SHGET_FILE_INFO, ORD_SHGET_SHORTCUT_TARGET, ORD_SHGET_SPECIAL_FOLDER_PATH,
            ORD_SHNOTIFICATION_ADD_I, ORD_SHNOTIFICATION_GET_DATA_I, ORD_SHNOTIFICATION_REMOVE_I,
            ORD_SHNOTIFICATION_UPDATE_I, ORD_SLEEP, ORD_SLEEP_TILL_TICK,
            ORD_STOP_DEVICE_NOTIFICATIONS, ORD_STOP_POWER_NOTIFICATIONS, ORD_STRING_COMPRESS,
            ORD_STRING_DECOMPRESS, ORD_SUSPEND_THREAD, ORD_SYSTEM_TIME_TO_FILE_TIME,
            ORD_TERMINATE_PROCESS, ORD_THCREATE_SNAPSHOT, ORD_TLS_GET_VALUE, ORD_TLS_SET_VALUE,
            ORD_TRY_ENTER_CRITICAL_SECTION, ORD_WAIT_COMM_EVENT, ORD_WAIT_FOR_MULTIPLE_OBJECTS,
            ORD_WAIT_FOR_SINGLE_OBJECT, ORD_WCSTOMBS, ORD_WIDE_CHAR_TO_MULTI_BYTE,
            ORD_WRITE_MSG_QUEUE, ORD_WRITE_PROCESS_MEMORY,
        },
        devices::{
            CommDcb, DeviceBackend, DeviceConfig, DeviceConfigFile, DeviceDefaults, DeviceKind,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        framebuffer::{Framebuffer, PixelFormat, VirtualFramebuffer},
        gwe::{
            Message, MessagePointerPayload, PeekFlags, QS_POSTMESSAGE, QS_TIMER, Rect, SC_CLOSE,
            WM_CHAR, WM_CLOSE, WM_COMMAND, WM_DESTROYCLIPBOARD, WM_KEYDOWN, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WM_NOTIFY, WM_PAINT, WM_RENDERALLFORMATS, WM_RENDERFORMAT, WM_SYSCOMMAND,
            WM_TIMER, WM_USER, WM_WINDOWPOSCHANGED, WS_VISIBLE, WindowPos,
        },
        kernel::{
            CE_CURRENT_PROCESS_PSEUDO_HANDLE, CE_CURRENT_THREAD_PSEUDO_HANDLE, CeKernel,
            LoadedModuleMetadata, MessageQueueOptions,
        },
        memory::PROCESS_HEAP_HANDLE,
        object::MAX_SUSPEND_COUNT,
        registry::{
            ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, HKEY_LOCAL_MACHINE, RegistryValue,
        },
        resource::ResourceId,
        scheduler::SchedulerBlockedWaitKind,
        shell::{
            ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET,
            ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET,
            ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET, MessageBoxButtonLabel,
            MessageBoxButtonSlot, MessageBoxIcon, ShellNotificationCallbackArguments,
            ShellNotificationCallbackMethod, ShellSpecialFolderFallbackPolicy,
            ShellSpecialFolderSource,
        },
        thread::{
            ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE,
            ERROR_INVALID_PARAMETER, ERROR_INVALID_WINDOW_HANDLE, ERROR_NOT_OWNER,
            ERROR_NOT_SUPPORTED, ERROR_RESOURCE_NAME_NOT_FOUND, ERROR_SIGNAL_REFUSED,
        },
        timer::{INFINITE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
};

mod support;
use support::{TestGuestMemory, unique_test_root};

#[test]
fn coredll_raw_battery_power_status_uses_ce_struct_layouts() -> Result<()> {
    const STATUS_EX_SIZE: u32 = 24;
    const STATUS_EX2_SIZE: u32 = 56;
    const AC_LINE_ONLINE: u8 = 0x01;
    const BATTERY_FLAG_NO_BATTERY: u8 = 0x80;
    const BATTERY_PERCENTAGE_UNKNOWN: u8 = 0xff;
    const BATTERY_CHEMISTRY_UNKNOWN: u8 = 0xff;
    const BATTERY_LIFE_UNKNOWN: u32 = 0xffff_ffff;

    fn le_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ])
    }

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let status_ex = 0x3000_1000;
    let status_ex2 = 0x3000_1100;
    let last_change = 0x3000_1200;
    let cpu_usage = 0x3000_1240;
    let previous_cpu_usage = 0x3000_1244;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_POWER_STATUS_EX,
            [0, 1],
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
            ORD_GET_SYSTEM_POWER_STATUS_EX,
            [status_ex, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let status = memory.read_bytes(status_ex, STATUS_EX_SIZE as usize);
    assert_eq!(status[0], AC_LINE_ONLINE);
    assert_eq!(status[1], BATTERY_FLAG_NO_BATTERY);
    assert_eq!(status[2], BATTERY_PERCENTAGE_UNKNOWN);
    assert_eq!(status[3], 0);
    assert_eq!(le_u32(&status, 4), BATTERY_LIFE_UNKNOWN);
    assert_eq!(le_u32(&status, 8), BATTERY_LIFE_UNKNOWN);
    assert_eq!(status[12], 0);
    assert_eq!(status[13], BATTERY_FLAG_NO_BATTERY);
    assert_eq!(status[14], BATTERY_PERCENTAGE_UNKNOWN);
    assert_eq!(status[15], 0);
    assert_eq!(le_u32(&status, 16), BATTERY_LIFE_UNKNOWN);
    assert_eq!(le_u32(&status, 20), BATTERY_LIFE_UNKNOWN);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_POWER_STATUS_EX2,
            [0, STATUS_EX2_SIZE, 1],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_POWER_STATUS_EX2,
            [status_ex2, 32, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(32),
            ..
        }
    ));
    let partial = memory.read_bytes(status_ex2, 32);
    assert_eq!(partial[0], AC_LINE_ONLINE);
    assert_eq!(partial[1], BATTERY_FLAG_NO_BATTERY);
    assert_eq!(le_u32(&partial, 24), 0);
    assert_eq!(le_u32(&partial, 28), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_POWER_STATUS_EX2,
            [status_ex2, 64, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(STATUS_EX2_SIZE),
            ..
        }
    ));
    let full = memory.read_bytes(status_ex2, STATUS_EX2_SIZE as usize);
    assert_eq!(full[52], BATTERY_CHEMISTRY_UNKNOWN);

    match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_BATTERY_DRVR_GET_LEVELS,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        } => {}
        other => panic!("unexpected BatteryDrvrGetLevels result: {other:?}"),
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BATTERY_DRVR_SUPPORTS_CHANGE_NOTIFICATION,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BATTERY_GET_LIFE_TIME_INFO,
            [last_change, cpu_usage, previous_cpu_usage],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!((1..=9999).contains(&memory.read_u16(last_change)?));
    assert!((1..=12).contains(&memory.read_u16(last_change + 2)?));
    assert_eq!(memory.read_u32(cpu_usage)?, 0);
    assert_eq!(memory.read_u32(previous_cpu_usage)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BATTERY_NOTIFY_OF_TIME_CHANGE,
            [1, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_registry_change_notifications_are_waitable_handles() -> Result<()> {
    const EVENT_SET: u32 = 3;
    const REG_NOTIFY_CHANGE_LAST_SET: u32 = 4;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;

    let notification = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CE_FIND_FIRST_REG_CHANGE,
        [HKEY_LOCAL_MACHINE, 1, REG_NOTIFY_CHANGE_LAST_SET],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CeFindFirstRegChange did not return a handle: {other:?}"),
    };
    assert_ne!(notification, 0);
    assert_ne!(notification, 0xffff_ffff);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [notification, 0],
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
            ORD_EVENT_MODIFY,
            [notification, EVENT_SET],
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
            [notification, 0],
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
            ORD_CE_FIND_NEXT_REG_CHANGE,
            [notification],
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
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [notification, 0],
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
            ORD_CE_FIND_CLOSE_REG_CHANGE,
            [notification],
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
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [notification, 0],
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
            ORD_CE_FIND_NEXT_REG_CHANGE,
            [notification],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FIND_CLOSE_REG_CHANGE,
            [notification],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FIND_FIRST_REG_CHANGE,
            [0, 1, REG_NOTIFY_CHANGE_LAST_SET],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0xffff_ffff),
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
fn coredll_raw_device_enumerators_return_ce_multisz_lists() -> Result<()> {
    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.devices = DeviceConfigFile {
        version: 1,
        defaults: DeviceDefaults::default(),
        devices: vec![
            DeviceConfig {
                guest: "COM7:".to_owned(),
                kind: DeviceKind::Serial,
                backend: DeviceBackend::Stub,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            },
            DeviceConfig {
                guest: "ACC1:".to_owned(),
                kind: DeviceKind::IoctlDevice,
                backend: DeviceBackend::Accelerometer,
                host: None,
                remote_gps: false,
                enabled: false,
                note: None,
            },
            DeviceConfig {
                guest: "I2C1:".to_owned(),
                kind: DeviceKind::IoctlDevice,
                backend: DeviceBackend::I2cBus,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            },
        ],
    };
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let size_ptr = 0x2100;
    let list_ptr = 0x2200;
    memory.map_words(size_ptr, 1);
    memory.map_halfwords(list_ptr, 32);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_PNP_IDS,
            [0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    memory.write_word(size_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_PNP_IDS,
            [0, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(size_ptr)?, 4);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_word(size_ptr, 2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_PNP_IDS,
            [list_ptr, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_MORE_DATA),
            ..
        }
    ));
    assert_eq!(memory.read_u32(size_ptr)?, 4);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_MORE_DATA);

    memory.write_word(size_ptr, 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_PNP_IDS,
            [list_ptr, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(size_ptr)?, 4);
    assert_eq!(memory.read_u16(list_ptr)?, 0);
    assert_eq!(memory.read_u16(list_ptr + 2)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_word(size_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_DEVICES,
            [0, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let device_list_bytes = memory.read_u32(size_ptr)?;
    assert_eq!(device_list_bytes, 26);

    memory.write_word(size_ptr, device_list_bytes - 2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_DEVICES,
            [list_ptr, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_MORE_DATA),
            ..
        }
    ));
    assert_eq!(memory.read_u32(size_ptr)?, device_list_bytes);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_MORE_DATA);

    memory.write_word(size_ptr, device_list_bytes);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_DEVICES,
            [list_ptr, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(list_ptr, 16), "COM7:");
    assert_eq!(memory.read_wide_z(list_ptr + 12, 16), "I2C1:");
    assert_eq!(memory.read_u16(list_ptr + 24)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    Ok(())
}

#[test]
fn coredll_raw_get_device_keys_reports_active_and_driver_registry_paths() -> Result<()> {
    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.devices = DeviceConfigFile {
        version: 1,
        defaults: DeviceDefaults::default(),
        devices: vec![
            DeviceConfig {
                guest: "COM7:".to_owned(),
                kind: DeviceKind::Serial,
                backend: DeviceBackend::Stub,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            },
            DeviceConfig {
                guest: "ACC1:".to_owned(),
                kind: DeviceKind::IoctlDevice,
                backend: DeviceBackend::Accelerometer,
                host: None,
                remote_gps: false,
                enabled: false,
                note: None,
            },
            DeviceConfig {
                guest: "UID1:".to_owned(),
                kind: DeviceKind::IoctlDevice,
                backend: DeviceBackend::NandUuid,
                host: None,
                remote_gps: false,
                enabled: true,
                note: None,
            },
        ],
    };
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let name_ptr = 0x3100_0000;
    let missing_name_ptr = 0x3100_0100;
    let disabled_name_ptr = 0x3100_0200;
    let active_len_ptr = 0x3100_1000;
    let driver_len_ptr = 0x3100_1004;
    let active_key_ptr = 0x3100_2000;
    let driver_key_ptr = 0x3100_2100;
    memory.map_words(active_len_ptr, 2);
    memory.map_halfwords(active_key_ptr, 64);
    memory.map_halfwords(driver_key_ptr, 64);
    memory.write_wide_z(name_ptr, "com7:");
    memory.write_wide_z(missing_name_ptr, "COM8:");
    memory.write_wide_z(disabled_name_ptr, "ACC1:");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DEVICE_KEYS,
            [
                0,
                active_key_ptr,
                active_len_ptr,
                driver_key_ptr,
                driver_len_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
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
            ORD_GET_DEVICE_KEYS,
            [name_ptr, active_key_ptr, 0, driver_key_ptr, driver_len_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
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
            ORD_GET_DEVICE_KEYS,
            [
                missing_name_ptr,
                active_key_ptr,
                active_len_ptr,
                driver_key_ptr,
                driver_len_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_FILE_NOT_FOUND),
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
            ORD_GET_DEVICE_KEYS,
            [
                disabled_name_ptr,
                active_key_ptr,
                active_len_ptr,
                driver_key_ptr,
                driver_len_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_FILE_NOT_FOUND),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );

    memory.write_word(active_len_ptr, 0);
    memory.write_word(driver_len_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DEVICE_KEYS,
            [name_ptr, 0, active_len_ptr, 0, driver_len_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(active_len_ptr)?, 17);
    assert_eq!(memory.read_u32(driver_len_ptr)?, 21);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_word(active_len_ptr, 16);
    memory.write_word(driver_len_ptr, 21);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DEVICE_KEYS,
            [
                name_ptr,
                active_key_ptr,
                active_len_ptr,
                driver_key_ptr,
                driver_len_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_MORE_DATA),
            ..
        }
    ));
    assert_eq!(memory.read_u32(active_len_ptr)?, 17);
    assert_eq!(memory.read_u32(driver_len_ptr)?, 21);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_MORE_DATA);

    memory.write_word(active_len_ptr, 17);
    memory.write_word(driver_len_ptr, 21);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DEVICE_KEYS,
            [
                name_ptr,
                active_key_ptr,
                active_len_ptr,
                driver_key_ptr,
                driver_len_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(active_len_ptr)?, 17);
    assert_eq!(memory.read_u32(driver_len_ptr)?, 21);
    assert_eq!(memory.read_wide_z(active_key_ptr, 64), r"Drivers\Active\1");
    assert_eq!(
        memory.read_wide_z(driver_key_ptr, 64),
        r"Drivers\BuiltIn\COM7"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(device_key),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_DEVICE_KEY,
        [active_key_ptr],
    )
    else {
        panic!("OpenDeviceKey did not return a handle");
    };
    assert_ne!(device_key, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let device_name = kernel
        .registry
        .reg_query_value_exw(device_key, Some("Name"), None);
    assert_eq!(device_name.status, ERROR_SUCCESS);
    let device_key_value = kernel
        .registry
        .reg_query_value_exw(device_key, Some("Key"), None);
    assert_eq!(device_key_value.status, ERROR_FILE_NOT_FOUND);
    assert_eq!(kernel.registry.reg_close_key(device_key), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_DEVICE_KEY,
            [0],
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

    memory.write_wide_z(active_key_ptr, r"Drivers\Active\99");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_DEVICE_KEY,
            [active_key_ptr],
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

    kernel.registry.create_key(r"hklm\Drivers\Active\99");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_DEVICE_KEY,
            [active_key_ptr],
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

    Ok(())
}

#[test]
fn coredll_raw_device_activation_opens_configured_devices_and_active_registry() -> Result<()> {
    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.devices = DeviceConfigFile {
        version: 1,
        defaults: DeviceDefaults::default(),
        devices: vec![DeviceConfig {
            guest: "COM7:".to_owned(),
            kind: DeviceKind::Serial,
            backend: DeviceBackend::Stub,
            host: None,
            remote_gps: false,
            enabled: true,
            note: None,
        }],
    };
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let driver_key_ptr = 0x3120_0000;
    let missing_key_ptr = 0x3120_0200;
    let type_ptr = 0x3120_0400;
    let regini_ptr = 0x3120_0800;
    let value_name_ptr = 0x3120_0900;
    let value_data_ptr = 0x3120_0a00;
    memory.map_halfwords(driver_key_ptr, 64);
    memory.map_halfwords(missing_key_ptr, 64);
    memory.map_halfwords(type_ptr, 16);
    memory.map_words(regini_ptr, 4);
    memory.map_halfwords(value_name_ptr, 16);
    memory.map_words(value_data_ptr, 1);
    memory.write_wide_z(driver_key_ptr, r"Drivers\BuiltIn\COM7");
    memory.write_wide_z(missing_key_ptr, r"Drivers\BuiltIn\COM8");
    memory.write_wide_z(type_ptr, "COM");
    memory.write_wide_z(value_name_ptr, "Order");
    memory.write_word(value_data_ptr, 42);
    memory.write_word(regini_ptr, value_name_ptr);
    memory.write_word(regini_ptr + 4, value_data_ptr);
    memory.write_word(regini_ptr + 8, 4);
    memory.write_word(regini_ptr + 12, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ACTIVATE_DEVICE,
            [0, 0x1234],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ACTIVATE_DEVICE,
            [missing_key_ptr, 0x1234],
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

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(device),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_ACTIVATE_DEVICE,
        [driver_key_ptr, 0x1234],
    )
    else {
        panic!("ActivateDevice did not return a handle");
    };
    assert_ne!(device, 0);
    assert_eq!(kernel.path_for_handle(device).as_deref(), Some("COM7:"));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(
        kernel
            .registry
            .query_value(r"hklm\Drivers\Active\1", "Hnd")?
            .as_dword(),
        Some(device)
    );
    assert_eq!(
        kernel
            .registry
            .query_value(r"hklm\Drivers\Active\1", "ClientInfo")?
            .as_dword(),
        Some(0x1234)
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEACTIVATE_DEVICE,
            [device],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(kernel.path_for_handle(device).is_none());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEACTIVATE_DEVICE,
            [device],
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

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(ex_device),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_ACTIVATE_DEVICE_EX,
        [driver_key_ptr, regini_ptr, 1, 0xfeed],
    )
    else {
        panic!("ActivateDeviceEx did not return a handle");
    };
    assert_ne!(ex_device, 0);
    assert_eq!(
        kernel
            .registry
            .query_value(r"hklm\Drivers\BuiltIn\COM7", "Order")?
            .as_dword(),
        Some(42)
    );
    assert_eq!(
        kernel
            .registry
            .query_value(r"hklm\Drivers\Active\1", "ClientInfo")?
            .as_dword(),
        Some(0xfeed)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEREGISTER_DEVICE,
            [ex_device],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(registered),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTER_DEVICE,
        [type_ptr, 7, 0, 0x77],
    )
    else {
        panic!("RegisterDevice did not return a handle");
    };
    assert_ne!(registered, 0);
    assert_eq!(kernel.path_for_handle(registered).as_deref(), Some("COM7:"));
    assert_eq!(
        kernel
            .registry
            .query_value(r"hklm\Drivers\Active\1", "ClientInfo")?
            .as_dword(),
        Some(0x77)
    );

    Ok(())
}

#[test]
fn coredll_raw_power_manager_tracks_system_and_device_power_state() -> Result<()> {
    const POWER_NAME: u32 = 0x0000_0001;
    const POWER_FORCE: u32 = 0x0000_1000;
    const POWER_NOTIFY_ALL: u32 = 0xffff_ffff;
    const POWER_STATE_IDLE: u32 = 0x0010_0000;
    const POWER_STATE_OFF: u32 = 0x0002_0000;
    const D0: u32 = 0;
    const D1: u32 = 1;
    const D3: u32 = 3;
    const D5: u32 = 5;

    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.devices = DeviceConfigFile {
        version: 1,
        defaults: DeviceDefaults::default(),
        devices: vec![DeviceConfig {
            guest: "COM7:".to_owned(),
            kind: DeviceKind::Serial,
            backend: DeviceBackend::Stub,
            host: None,
            remote_gps: false,
            enabled: true,
            note: None,
        }],
    };
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let device_ptr = 0x3130_0000;
    let missing_device_ptr = 0x3130_0100;
    let system_state_ptr = 0x3130_0200;
    let pm_device_ptr = 0x3130_0300;
    let parent_ptr = 0x3130_0400;
    let child_ptr = 0x3130_0500;
    let device_state_ptr = 0x3130_1000;
    let system_flags_ptr = 0x3130_1004;
    let msg_queue_options_ptr = 0x3130_1100;
    let power_caps_ptr = 0x3130_1200;
    let system_name_ptr = 0x3130_2000;
    memory.map_words(device_state_ptr, 2);
    memory.map_words(msg_queue_options_ptr, 5);
    memory.write_word(msg_queue_options_ptr, 20);
    memory.write_word(msg_queue_options_ptr + 8, 4);
    memory.write_word(msg_queue_options_ptr + 12, 64);
    memory.write_word(msg_queue_options_ptr + 16, 1);
    memory.map_bytes(power_caps_ptr, 48);
    memory.fill(power_caps_ptr, 0, 48);
    memory.write_bytes(power_caps_ptr, &[0x1f, 0x01, 0x00, 0x00]);
    memory.map_halfwords(system_name_ptr, 32);
    memory.write_wide_z(device_ptr, "com7:");
    memory.write_wide_z(missing_device_ptr, "COM8:");
    memory.write_wide_z(system_state_ptr, "screenoff");
    memory.write_wide_z(pm_device_ptr, "BKL1:");
    memory.write_wide_z(parent_ptr, "BUS1:");
    memory.write_wide_z(child_ptr, "COM7:");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DEVICE_POWER,
            [device_ptr, POWER_NAME, device_state_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(device_state_ptr)?, D0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_DEVICE_POWER,
            [device_ptr, POWER_NAME | POWER_FORCE, D3],
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
            ORD_GET_DEVICE_POWER,
            [device_ptr, POWER_NAME, device_state_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(device_state_ptr)?, D3);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_POWER_NOTIFY,
            [device_ptr, POWER_NAME, D1],
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
            ORD_GET_DEVICE_POWER,
            [device_ptr, POWER_NAME, device_state_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(device_state_ptr)?, D1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_DEVICE_POWER,
            [missing_device_ptr, POWER_NAME, D0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_FILE_NOT_FOUND),
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
            ORD_SET_DEVICE_POWER,
            [device_ptr, POWER_NAME, D5],
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
            ORD_GET_DEVICE_POWER,
            [device_ptr, POWER_NAME, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
            ..
        }
    ));

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(requirement),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SET_POWER_REQUIREMENT,
        [device_ptr, D0, POWER_NAME, 0, 0],
    )
    else {
        panic!("SetPowerRequirement did not return a handle");
    };
    assert_ne!(requirement, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DEVICE_POWER,
            [device_ptr, POWER_NAME, device_state_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(device_state_ptr)?, D0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_POWER_REQUIREMENT,
            [requirement],
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
            ORD_RELEASE_POWER_REQUIREMENT,
            [requirement],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_HANDLE),
            ..
        }
    ));

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(pm_requirement),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SET_POWER_REQUIREMENT,
        [
            pm_device_ptr,
            D1,
            POWER_NAME,
            system_state_ptr,
            POWER_STATE_IDLE,
        ],
    )
    else {
        panic!("SetPowerRequirement for PM-only device did not return a handle");
    };
    assert_ne!(pm_requirement, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_POWER_REQUIREMENT,
            [pm_requirement],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(relationship),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTER_POWER_RELATIONSHIP,
        [parent_ptr, child_ptr, power_caps_ptr, 0x20],
    )
    else {
        panic!("RegisterPowerRelationship did not return a handle");
    };
    assert_ne!(relationship, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_POWER_RELATIONSHIP,
            [relationship],
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
            ORD_RELEASE_POWER_RELATIONSHIP,
            [relationship],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_HANDLE),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_POWER_RELATIONSHIP,
            [parent_ptr, child_ptr, 0x2000, 0],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_POWER_RELATIONSHIP,
            [0, 0, 0, 0],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_POWER_REQUIREMENT,
            [device_ptr, D5, POWER_NAME, 0, 0],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_POWER_REQUIREMENT,
            [0, D0, 0, 0, 0],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REQUEST_POWER_NOTIFICATIONS,
            [0, POWER_NOTIFY_ALL],
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

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(msg_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, msg_queue_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a handle");
    };
    assert_ne!(msg_queue, 0);
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(notification),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REQUEST_POWER_NOTIFICATIONS,
        [msg_queue, POWER_NOTIFY_ALL],
    )
    else {
        panic!("RequestPowerNotifications did not return a handle");
    };
    assert_ne!(notification, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STOP_POWER_NOTIFICATIONS,
            [notification],
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
            ORD_STOP_POWER_NOTIFICATIONS,
            [notification],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_HANDLE),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_SYSTEM_POWER_STATE,
            [0, POWER_STATE_IDLE, 0],
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
            ORD_GET_SYSTEM_POWER_STATE,
            [system_name_ptr, 32, system_flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(system_name_ptr, 32), "");
    assert_eq!(memory.read_u32(system_flags_ptr)?, POWER_STATE_IDLE);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_SYSTEM_POWER_STATE,
            [system_state_ptr, POWER_STATE_OFF, POWER_FORCE],
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
            ORD_GET_SYSTEM_POWER_STATE,
            [system_name_ptr, 32, system_flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(system_name_ptr, 32), "screenoff");
    assert_eq!(memory.read_u32(system_flags_ptr)?, POWER_STATE_OFF);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_POWER_STATE,
            [system_name_ptr, 4, system_flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_MORE_DATA),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_MORE_DATA);

    Ok(())
}

#[test]
fn coredll_raw_keybd_get_device_info_reports_ce_defaults() -> Result<()> {
    const KBDI_VKEY_TO_UNICODE_INFO_ID: u32 = 0;
    const KBDI_AUTOREPEAT_INFO_ID: u32 = 1;
    const KBDI_AUTOREPEAT_SELECTIONS_INFO_ID: u32 = 2;
    const KBDI_KEYBOARD_STATUS_ID: u32 = 3;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let info_ptr = 0x2100;
    memory.map_words(info_ptr, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KEYBD_GET_DEVICE_INFO,
            [0xfeed, info_ptr],
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
            ORD_KEYBD_GET_DEVICE_INFO,
            [KBDI_AUTOREPEAT_INFO_ID, 0],
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
            ORD_KEYBD_GET_DEVICE_INFO,
            [KBDI_VKEY_TO_UNICODE_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 0);
    assert_eq!(memory.read_u32(info_ptr + 4)?, 1);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KEYBD_GET_DEVICE_INFO,
            [KBDI_AUTOREPEAT_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 500);
    assert_eq!(memory.read_u32(info_ptr + 4)?, 20);
    assert_eq!(memory.read_u32(info_ptr + 8)?, 0xffff_ffff);
    assert_eq!(memory.read_u32(info_ptr + 12)?, 0xffff_ffff);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KEYBD_GET_DEVICE_INFO,
            [KBDI_AUTOREPEAT_SELECTIONS_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 250);
    assert_eq!(memory.read_u32(info_ptr + 4)?, 1000);
    assert_eq!(memory.read_u32(info_ptr + 8)?, 2);
    assert_eq!(memory.read_u32(info_ptr + 12)?, 30);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KEYBD_GET_DEVICE_INFO,
            [KBDI_KEYBOARD_STATUS_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 0x000f);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    Ok(())
}

#[test]
fn coredll_raw_nled_device_emulator_get_set_round_trips() -> Result<()> {
    const NLED_COUNT_INFO_ID: u32 = 0;
    const NLED_SUPPORTS_INFO_ID: u32 = 1;
    const NLED_SETTINGS_INFO_ID: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let info_ptr = 0x2100;
    memory.map_words(info_ptr, 7);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NLED_GET_DEVICE_INFO,
            [0xfeed, info_ptr],
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
            ORD_NLED_GET_DEVICE_INFO,
            [NLED_SUPPORTS_INFO_ID, 0],
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
            ORD_NLED_GET_DEVICE_INFO,
            [NLED_COUNT_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 2);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_word(info_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NLED_GET_DEVICE_INFO,
            [NLED_SUPPORTS_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 0);
    assert_eq!(memory.read_u32(info_ptr + 4)?, 100);
    assert_eq!(memory.read_u32(info_ptr + 8)?, 0);
    assert_eq!(memory.read_u32(info_ptr + 12)?, 1);
    assert_eq!(memory.read_u32(info_ptr + 16)?, 1);
    assert_eq!(memory.read_u32(info_ptr + 20)?, 0);
    assert_eq!(memory.read_u32(info_ptr + 24)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_word(info_ptr, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NLED_GET_DEVICE_INFO,
            [NLED_SUPPORTS_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 1);
    assert_eq!(memory.read_u32(info_ptr + 4)?, 0xffff_ffff);
    for offset in [8, 12, 16, 20, 24] {
        assert_eq!(memory.read_u32(info_ptr + offset)?, 0);
    }

    memory.write_word(info_ptr, 2);
    for info_id in [NLED_SUPPORTS_INFO_ID, NLED_SETTINGS_INFO_ID] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_NLED_GET_DEVICE_INFO,
                [info_id, info_ptr],
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
    }

    memory.write_word(info_ptr, 0);
    memory.write_word(info_ptr + 4, 2);
    memory.write_word(info_ptr + 8, 0);
    memory.write_word(info_ptr + 12, 300);
    memory.write_word(info_ptr + 16, 700);
    memory.write_word(info_ptr + 20, 0);
    memory.write_word(info_ptr + 24, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NLED_SET_DEVICE,
            [NLED_SETTINGS_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_word(info_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NLED_GET_DEVICE_INFO,
            [NLED_SETTINGS_INFO_ID, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr)?, 0);
    assert_eq!(memory.read_u32(info_ptr + 4)?, 2);
    assert_eq!(memory.read_u32(info_ptr + 8)?, 1000);
    assert_eq!(memory.read_u32(info_ptr + 12)?, 300);
    assert_eq!(memory.read_u32(info_ptr + 16)?, 700);
    assert_eq!(memory.read_u32(info_ptr + 20)?, 0);
    assert_eq!(memory.read_u32(info_ptr + 24)?, 0);

    memory.write_word(info_ptr, 0);
    memory.write_word(info_ptr + 4, 1);
    memory.write_word(info_ptr + 12, 0xffff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NLED_SET_DEVICE,
            [NLED_SETTINGS_INFO_ID, info_ptr],
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
            ORD_NLED_SET_DEVICE,
            [NLED_SUPPORTS_INFO_ID, info_ptr],
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
fn coredll_raw_enum_device_interfaces_exposes_advertised_interfaces() -> Result<()> {
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const BLOCK_GUID: [u8; 16] = [
        0xda, 0xed, 0xe7, 0xa4, 0x75, 0xe5, 0x52, 0x42, 0x9d, 0x6b, 0x41, 0x95, 0xd4, 0x8b, 0xb8,
        0x65,
    ];

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    kernel.advertise_device_interface(BLOCK_GUID, r"\StoreMgr\DSK5:".to_owned(), true);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let guid_ptr = 0x2100;
    let name_ptr = 0x2200;
    let size_ptr = 0x2300;
    memory.map_bytes(guid_ptr, 16);
    memory.map_halfwords(name_ptr, 64);
    memory.map_words(size_ptr, 1);

    memory.write_word(size_ptr, 4);
    let first_dispatch = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_ENUM_DEVICE_INTERFACES,
        [0, 0, guid_ptr, name_ptr, size_ptr],
    );
    assert!(
        matches!(
            first_dispatch,
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "{first_dispatch:?}"
    );
    assert_eq!(memory.read_bytes(guid_ptr, 16), BLOCK_GUID);
    assert_eq!(memory.read_u32(size_ptr)?, 32);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INSUFFICIENT_BUFFER
    );

    memory.write_word(size_ptr, 32);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_DEVICE_INTERFACES,
            [0, 0, guid_ptr, name_ptr, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(name_ptr, 64), r"\StoreMgr\DSK5:");
    assert_eq!(memory.read_u32(size_ptr)?, 32);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    memory.write_bytes(guid_ptr, &[0; 16]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_DEVICE_INTERFACES,
            [0, 0, guid_ptr, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(guid_ptr, 16), BLOCK_GUID);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_DEVICE_INTERFACES,
            [0, 1, guid_ptr, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENUM_DEVICE_INTERFACES,
            [0, 0, 0, 0, 0],
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
            ORD_ENUM_DEVICE_INTERFACES,
            [0xdead_beef, 0, guid_ptr, 0, 0],
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
fn coredll_raw_iorm_resource_manager_tracks_ce_ranges_and_exclusive_claims() -> Result<()> {
    const ERROR_BUSY: u32 = 170;
    const RREXF_REQUEST_EXCLUSIVE: u32 = 0x0001;
    const RESMGR_IRQ: u32 = 0x0001;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let mut memory = TestGuestMemory::default();
    let thread_id = 31;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESOURCE_REQUEST,
            [RESMGR_IRQ, 5, 1],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESOURCE_CREATE_LIST,
            [RESMGR_IRQ, 4, 4],
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
            ORD_RESOURCE_CREATE_LIST,
            [RESMGR_IRQ, 4, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ALREADY_EXISTS
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESOURCE_CREATE_LIST,
            [0x20, 0, 0],
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
            ORD_RESOURCE_REQUEST,
            [RESMGR_IRQ, 5, 1],
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
            ORD_RESOURCE_RELEASE,
            [RESMGR_IRQ, 5, 2],
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
            ORD_RESOURCE_REQUEST,
            [RESMGR_IRQ, 5, 2],
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
            ORD_RESOURCE_REQUEST,
            [RESMGR_IRQ, 5, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_BUSY);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESOURCE_REQUEST_EX,
            [RESMGR_IRQ, 5, 1, RREXF_REQUEST_EXCLUSIVE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_BUSY);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESOURCE_RELEASE,
            [RESMGR_IRQ, 5, 1],
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
            ORD_RESOURCE_REQUEST_EX,
            [RESMGR_IRQ, 5, 1, RREXF_REQUEST_EXCLUSIVE],
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
            ORD_RESOURCE_REQUEST,
            [RESMGR_IRQ, 5, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_BUSY);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RESOURCE_REQUEST_EX,
            [RESMGR_IRQ, 6, 1, 0x0002],
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
            ORD_RESOURCE_RELEASE,
            [RESMGR_IRQ, 5, 1],
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
            ORD_RESOURCE_RELEASE,
            [RESMGR_IRQ, 5, 1],
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
            ORD_RESOURCE_RELEASE,
            [RESMGR_IRQ, 8, 1],
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

    macro_rules! expect_iorm_bool {
        ($ordinal:expr, [$($arg:expr),* $(,)?], $expected:expr, $last_error:expr) => {{
            assert!(matches!(
                table.dispatch_raw_ordinal_with_memory(
                    &mut kernel,
                    &mut memory,
                    thread_id,
                    $ordinal,
                    [$($arg),*],
                ),
                CoredllDispatch::Returned {
                    value: CoredllValue::Bool(value),
                    ..
                } if value == $expected
            ));
            assert_eq!(kernel.threads.get_last_error(thread_id), $last_error);
        }};
    }

    expect_iorm_bool!(ORD_RESOURCE_DESTROY_LIST, [RESMGR_IRQ], false, ERROR_BUSY);
    expect_iorm_bool!(
        ORD_RESOURCE_RELEASE,
        [RESMGR_IRQ, 6, 1],
        true,
        ERROR_SUCCESS
    );
    expect_iorm_bool!(
        ORD_RESOURCE_MARK_AS_SHAREABLE,
        [RESMGR_IRQ, 5, 1, 1],
        true,
        ERROR_SUCCESS
    );
    expect_iorm_bool!(
        ORD_RESOURCE_REQUEST,
        [RESMGR_IRQ, 5, 1],
        true,
        ERROR_SUCCESS
    );
    expect_iorm_bool!(
        ORD_RESOURCE_REQUEST,
        [RESMGR_IRQ, 5, 1],
        true,
        ERROR_SUCCESS
    );
    expect_iorm_bool!(
        ORD_RESOURCE_REQUEST_EX,
        [RESMGR_IRQ, 5, 1, RREXF_REQUEST_EXCLUSIVE],
        false,
        ERROR_BUSY
    );
    expect_iorm_bool!(
        ORD_RESOURCE_MARK_AS_SHAREABLE,
        [RESMGR_IRQ, 5, 1, 0],
        false,
        ERROR_INVALID_PARAMETER
    );
    for _ in 0..2 {
        expect_iorm_bool!(
            ORD_RESOURCE_RELEASE,
            [RESMGR_IRQ, 5, 1],
            true,
            ERROR_SUCCESS
        );
    }
    expect_iorm_bool!(
        ORD_RESOURCE_MARK_AS_SHAREABLE,
        [RESMGR_IRQ, 5, 1, 0],
        true,
        ERROR_SUCCESS
    );
    expect_iorm_bool!(
        ORD_RESOURCE_REQUEST,
        [RESMGR_IRQ, 5, 1],
        true,
        ERROR_SUCCESS
    );
    expect_iorm_bool!(ORD_RESOURCE_REQUEST, [RESMGR_IRQ, 5, 1], false, ERROR_BUSY);
    expect_iorm_bool!(
        ORD_RESOURCE_RELEASE,
        [RESMGR_IRQ, 5, 1],
        true,
        ERROR_SUCCESS
    );
    expect_iorm_bool!(
        ORD_RESOURCE_MARK_AS_SHAREABLE,
        [0x30, 5, 1, 1],
        false,
        ERROR_INVALID_PARAMETER
    );
    expect_iorm_bool!(ORD_RESOURCE_DESTROY_LIST, [RESMGR_IRQ], true, ERROR_SUCCESS);
    expect_iorm_bool!(
        ORD_RESOURCE_DESTROY_LIST,
        [RESMGR_IRQ],
        false,
        ERROR_FILE_NOT_FOUND
    );
    expect_iorm_bool!(
        ORD_RESOURCE_REQUEST,
        [RESMGR_IRQ, 5, 1],
        false,
        ERROR_FILE_NOT_FOUND
    );

    Ok(())
}

#[test]
fn coredll_raw_device_notifications_track_request_handles() -> Result<()> {
    const STORAGE_MEDIA_GUID: [u8; 16] = [
        0x63, 0x40, 0xa2, 0x8c, 0xa3, 0xd9, 0x52, 0x42, 0x8a, 0x30, 0xd0, 0x3c, 0x52, 0x28, 0x80,
        0x59,
    ];

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let guid_ptr = 0x2400;
    let (message_queue, existed) = kernel.create_message_queue(
        None,
        MessageQueueOptions {
            flags: 0,
            max_messages: 8,
            max_message_bytes: 256,
            read_access: true,
        },
    )?;
    assert!(!existed);
    memory.map_bytes(guid_ptr, 16);
    memory.write_bytes(guid_ptr, &STORAGE_MEDIA_GUID);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(notification_handle),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REQUEST_DEVICE_NOTIFICATIONS,
        [guid_ptr, message_queue, 1],
    )
    else {
        panic!("RequestDeviceNotifications did not return a handle");
    };
    assert_ne!(notification_handle, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let description = kernel.handles.describe_handle(notification_handle);
    assert!(description.contains("device_notification"));
    assert!(description.contains(&format!("queue=0x{message_queue:08x}")));
    assert!(description.contains("all=true"));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STOP_DEVICE_NOTIFICATIONS,
            [notification_handle],
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
            ORD_STOP_DEVICE_NOTIFICATIONS,
            [notification_handle],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REQUEST_DEVICE_NOTIFICATIONS,
            [0, message_queue, 0],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REQUEST_DEVICE_NOTIFICATIONS,
            [guid_ptr, 0, 0],
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
fn coredll_raw_msg_queues_deliver_device_notification_devdetails() -> Result<()> {
    const ERROR_TIMEOUT: u32 = 1460;
    const STORAGE_MEDIA_GUID: [u8; 16] = [
        0x63, 0x40, 0xa2, 0x8c, 0xa3, 0xd9, 0x52, 0x42, 0x8a, 0x30, 0xd0, 0x3c, 0x52, 0x28, 0x80,
        0x59,
    ];

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let read_options_ptr = 0x2500;
    let write_options_ptr = 0x2600;
    let info_ptr = 0x2700;
    let input_ptr = 0x2800;
    let output_ptr = 0x2900;
    let bytes_read_ptr = 0x2a00;
    let flags_ptr = 0x2b00;
    let guid_ptr = 0x2c00;
    memory.map_words(read_options_ptr, 5);
    memory.write_word(read_options_ptr, 20);
    memory.write_word(read_options_ptr + 8, 0);
    memory.write_word(read_options_ptr + 12, 300);
    memory.write_word(read_options_ptr + 16, 1);
    memory.map_words(write_options_ptr, 5);
    memory.write_word(write_options_ptr, 20);
    memory.write_word(write_options_ptr + 8, 0);
    memory.write_word(write_options_ptr + 12, 300);
    memory.write_word(write_options_ptr + 16, 0);
    memory.map_words(info_ptr, 6);
    memory.map_halfwords(info_ptr + 24, 2);
    memory.write_word(info_ptr, 28);
    memory.map_bytes(input_ptr, 8);
    memory.write_bytes(input_ptr, &[0xaa, 0xbb, 0xcc, 0xdd]);
    memory.map_bytes(output_ptr, 300);
    memory.map_words(bytes_read_ptr, 1);
    memory.map_words(flags_ptr, 1);
    memory.map_bytes(guid_ptr, 16);
    memory.write_bytes(guid_ptr, &STORAGE_MEDIA_GUID);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(read_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, read_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a handle");
    };
    assert_ne!(read_queue, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(write_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_MSG_QUEUE,
        [0, read_queue, write_options_ptr],
    )
    else {
        panic!("OpenMsgQueue did not return a handle");
    };
    assert_ne!(write_queue, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [read_queue, 0],
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
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [write_queue, 0],
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
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 4, 0, 0x20],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.is_wait_ready(read_queue, thread_id), Some(true));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MSG_QUEUE_INFO,
            [read_queue, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr + 8)?, 16);
    assert_eq!(memory.read_u32(info_ptr + 12)?, 300);
    assert_eq!(memory.read_u32(info_ptr + 16)?, 1);
    assert_eq!(memory.read_u32(info_ptr + 20)?, 1);
    assert_eq!(memory.read_u16(info_ptr + 24)?, 1);
    assert_eq!(memory.read_u16(info_ptr + 26)?, 1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 300, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(bytes_read_ptr)?, 4);
    assert_eq!(memory.read_u32(flags_ptr)?, 0x20);
    assert_eq!(
        memory.read_bytes(output_ptr, 4),
        vec![0xaa, 0xbb, 0xcc, 0xdd]
    );
    assert_eq!(kernel.is_wait_ready(read_queue, thread_id), Some(false));

    kernel.advertise_device_interface(STORAGE_MEDIA_GUID, r"\StoreMgr\DSK9:".to_owned(), true);
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(notification_handle),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REQUEST_DEVICE_NOTIFICATIONS,
        [guid_ptr, read_queue, 1],
    )
    else {
        panic!("RequestDeviceNotifications did not return a handle");
    };
    assert_ne!(notification_handle, 0);
    assert_eq!(kernel.is_wait_ready(read_queue, thread_id), Some(true));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 300, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let attached = memory.read_bytes(output_ptr, memory.read_u32(bytes_read_ptr)? as usize);
    assert_eq!(&attached[0..16], &STORAGE_MEDIA_GUID);
    assert_eq!(u32::from_le_bytes(attached[20..24].try_into().unwrap()), 1);
    let name_bytes = u32::from_le_bytes(attached[24..28].try_into().unwrap()) as usize;
    let name_units: Vec<u16> = attached[28..28 + name_bytes]
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    assert_eq!(String::from_utf16_lossy(&name_units), r"\StoreMgr\DSK9:");

    kernel.advertise_device_interface(STORAGE_MEDIA_GUID, r"\StoreMgr\DSK9:".to_owned(), false);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 300, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let detached = memory.read_bytes(output_ptr, memory.read_u32(bytes_read_ptr)? as usize);
    assert_eq!(&detached[0..16], &STORAGE_MEDIA_GUID);
    assert_eq!(u32::from_le_bytes(detached[20..24].try_into().unwrap()), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STOP_DEVICE_NOTIFICATIONS,
            [notification_handle],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    kernel.advertise_device_interface(STORAGE_MEDIA_GUID, r"\StoreMgr\DSK9:".to_owned(), true);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 300, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_TIMEOUT);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_MSG_QUEUE,
            [write_queue],
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
            ORD_CLOSE_MSG_QUEUE,
            [read_queue],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_msg_queues_follow_ce_access_and_capacity_edges() -> Result<()> {
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const ERROR_PIPE_NOT_CONNECTED: u32 = 233;
    const ERROR_TIMEOUT: u32 = 1460;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let read_options_ptr = 0x3100;
    let write_options_ptr = 0x3200;
    let input_ptr = 0x3300;
    let output_ptr = 0x3400;
    let bytes_read_ptr = 0x3500;
    let flags_ptr = 0x3600;
    memory.map_words(read_options_ptr, 5);
    memory.write_word(read_options_ptr, 20);
    memory.write_word(read_options_ptr + 8, 1);
    memory.write_word(read_options_ptr + 12, 4);
    memory.write_word(read_options_ptr + 16, 1);
    memory.map_words(write_options_ptr, 5);
    memory.write_word(write_options_ptr, 20);
    memory.write_word(write_options_ptr + 8, 1);
    memory.write_word(write_options_ptr + 12, 4);
    memory.write_word(write_options_ptr + 16, 0);
    memory.map_bytes(input_ptr, 8);
    memory.write_bytes(input_ptr, &[1, 2, 3, 4, 5, 6, 7, 8]);
    memory.map_bytes(output_ptr, 8);
    memory.map_words(bytes_read_ptr, 1);
    memory.map_words(flags_ptr, 1);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(read_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, read_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a handle");
    };
    assert_ne!(read_queue, 0);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(write_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_MSG_QUEUE,
        [0, read_queue, write_options_ptr],
    )
    else {
        panic!("OpenMsgQueue did not return a handle");
    };
    assert_ne!(write_queue, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [write_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [read_queue, input_ptr, 4, 0, 0],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 0, 0, 0],
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
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 5, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INSUFFICIENT_BUFFER
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 4, 0, 0x42],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(kernel.is_wait_ready(read_queue, thread_id), Some(true));
    assert_eq!(kernel.is_wait_ready(write_queue, thread_id), Some(false));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [read_queue, 0],
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
            [write_queue, 0],
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
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr + 4, 4, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_TIMEOUT);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 2, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INSUFFICIENT_BUFFER
    );
    assert_eq!(memory.read_u32(bytes_read_ptr)?, 2);
    assert_eq!(memory.read_u32(flags_ptr)?, 0x42);
    assert_eq!(memory.read_bytes(output_ptr, 2), vec![1, 2]);
    assert_eq!(kernel.is_wait_ready(read_queue, thread_id), Some(false));
    assert_eq!(kernel.is_wait_ready(write_queue, thread_id), Some(true));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_TIMEOUT);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_MSG_QUEUE,
            [write_queue],
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
            [read_queue, 0],
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
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_PIPE_NOT_CONNECTED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_MSG_QUEUE,
            [read_queue],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_msg_queues_prioritize_ce_alert_messages() -> Result<()> {
    const ERROR_TIMEOUT: u32 = 1460;
    const MSGQUEUE_MSGALERT: u32 = 0x0000_0001;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let read_options_ptr = 0x4800;
    let write_options_ptr = 0x4900;
    let input_ptr = 0x4a00;
    let output_ptr = 0x4b00;
    let bytes_read_ptr = 0x4c00;
    let flags_ptr = 0x4d00;
    let info_ptr = 0x4e00;
    memory.map_words(read_options_ptr, 5);
    memory.write_word(read_options_ptr, 20);
    memory.write_word(read_options_ptr + 8, 1);
    memory.write_word(read_options_ptr + 12, 4);
    memory.write_word(read_options_ptr + 16, 1);
    memory.map_words(write_options_ptr, 5);
    memory.write_word(write_options_ptr, 20);
    memory.write_word(write_options_ptr + 8, 1);
    memory.write_word(write_options_ptr + 12, 4);
    memory.write_word(write_options_ptr + 16, 0);
    memory.map_bytes(input_ptr, 12);
    memory.write_bytes(
        input_ptr,
        &[
            0x10, 0x11, 0x12, 0x13, 0xa0, 0xa1, 0xa2, 0xa3, 0xb0, 0xb1, 0xb2, 0xb3,
        ],
    );
    memory.map_bytes(output_ptr, 4);
    memory.map_words(bytes_read_ptr, 1);
    memory.map_words(flags_ptr, 1);
    memory.map_words(info_ptr, 6);
    memory.map_halfwords(info_ptr + 24, 2);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(read_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, read_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a handle");
    };
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(write_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_MSG_QUEUE,
        [0, read_queue, write_options_ptr],
    )
    else {
        panic!("OpenMsgQueue did not return a handle");
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 4, 0, 0x20],
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
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr + 4, 4, 0, MSGQUEUE_MSGALERT | 0x40],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.is_wait_ready(read_queue, thread_id), Some(true));

    memory.write_word(info_ptr, 28);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MSG_QUEUE_INFO,
            [write_queue, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr + 16)?, 1);
    assert_eq!(memory.read_u32(info_ptr + 20)?, 1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr + 8, 4, 0, MSGQUEUE_MSGALERT | 0x80],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_TIMEOUT);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(bytes_read_ptr)?, 4);
    assert_eq!(memory.read_u32(flags_ptr)?, MSGQUEUE_MSGALERT | 0x40);
    assert_eq!(
        memory.read_bytes(output_ptr, 4),
        vec![0xa0, 0xa1, 0xa2, 0xa3]
    );

    memory.write_word(info_ptr, 28);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MSG_QUEUE_INFO,
            [read_queue, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr + 16)?, 1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(bytes_read_ptr)?, 4);
    assert_eq!(memory.read_u32(flags_ptr)?, 0x20);
    assert_eq!(
        memory.read_bytes(output_ptr, 4),
        vec![0x10, 0x11, 0x12, 0x13]
    );
    assert_eq!(kernel.is_wait_ready(read_queue, thread_id), Some(false));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_TIMEOUT);

    let _ = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_MSG_QUEUE,
        [write_queue],
    );
    let _ = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_MSG_QUEUE,
        [read_queue],
    );

    Ok(())
}

#[test]
fn coredll_raw_msg_queues_follow_ce_broken_end_policy() -> Result<()> {
    const ERROR_PIPE_NOT_CONNECTED: u32 = 233;
    const ERROR_TIMEOUT: u32 = 1460;
    const MSGQUEUE_ALLOW_BROKEN: u32 = 0x0000_0002;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let read_options_ptr = 0x4100;
    let write_options_ptr = 0x4200;
    let input_ptr = 0x4300;
    let output_ptr = 0x4400;
    let bytes_read_ptr = 0x4500;
    let flags_ptr = 0x4600;
    let info_ptr = 0x4700;
    memory.map_words(read_options_ptr, 5);
    memory.write_word(read_options_ptr, 20);
    memory.write_word(read_options_ptr + 8, 2);
    memory.write_word(read_options_ptr + 12, 4);
    memory.write_word(read_options_ptr + 16, 1);
    memory.map_words(write_options_ptr, 5);
    memory.write_word(write_options_ptr, 20);
    memory.write_word(write_options_ptr + 8, 2);
    memory.write_word(write_options_ptr + 12, 4);
    memory.write_word(write_options_ptr + 16, 0);
    memory.map_bytes(input_ptr, 8);
    memory.write_bytes(input_ptr, &[0x10, 0x11, 0x12, 0x13, 0x20, 0x21, 0x22, 0x23]);
    memory.map_bytes(output_ptr, 8);
    memory.map_words(bytes_read_ptr, 1);
    memory.map_words(flags_ptr, 1);
    memory.map_words(info_ptr, 6);
    memory.map_halfwords(info_ptr + 24, 2);
    memory.write_word(info_ptr, 28);

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(read_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, read_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a handle");
    };
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(write_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_MSG_QUEUE,
        [0, read_queue, write_options_ptr],
    )
    else {
        panic!("OpenMsgQueue did not return a handle");
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 4, 0, 0x31],
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
            ORD_CLOSE_MSG_QUEUE,
            [write_queue],
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
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_PIPE_NOT_CONNECTED
    );
    let _ = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_MSG_QUEUE,
        [read_queue],
    );

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(read_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, read_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a handle");
    };
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(write_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_MSG_QUEUE,
        [0, read_queue, write_options_ptr],
    )
    else {
        panic!("OpenMsgQueue did not return a handle");
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_MSG_QUEUE,
            [read_queue],
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
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 4, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_PIPE_NOT_CONNECTED
    );
    let _ = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_MSG_QUEUE,
        [write_queue],
    );

    memory.write_word(read_options_ptr + 4, MSGQUEUE_ALLOW_BROKEN);
    memory.write_word(write_options_ptr + 4, MSGQUEUE_ALLOW_BROKEN);
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(read_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, read_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a handle");
    };
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(write_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_MSG_QUEUE,
        [0, read_queue, write_options_ptr],
    )
    else {
        panic!("OpenMsgQueue did not return a handle");
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr + 4, 4, 0, 0x44],
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
            ORD_CLOSE_MSG_QUEUE,
            [write_queue],
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
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(bytes_read_ptr)?, 4);
    assert_eq!(memory.read_u32(flags_ptr)?, 0x44);
    assert_eq!(
        memory.read_bytes(output_ptr, 4),
        vec![0x20, 0x21, 0x22, 0x23]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_MSG_QUEUE,
            [read_queue, output_ptr, 4, bytes_read_ptr, 0, flags_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_TIMEOUT);
    let _ = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_MSG_QUEUE,
        [read_queue],
    );

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(write_queue),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MSG_QUEUE,
        [0, write_options_ptr],
    )
    else {
        panic!("CreateMsgQueue did not return a write handle");
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_MSG_QUEUE,
            [write_queue, input_ptr, 4, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    memory.write_word(info_ptr, 28);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MSG_QUEUE_INFO,
            [write_queue, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr + 16)?, 1);
    assert_eq!(memory.read_u16(info_ptr + 24)?, 0);
    assert_eq!(memory.read_u16(info_ptr + 26)?, 1);
    let _ = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_MSG_QUEUE,
        [write_queue],
    );

    Ok(())
}

#[test]
fn coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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

    let store_pages_ptr = 0x5120;
    let ram_pages_ptr = 0x5124;
    let page_size_ptr = 0x5128;
    memory.map_words(store_pages_ptr, 3);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_MEMORY_DIVISION,
            [store_pages_ptr, ram_pages_ptr, page_size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(page_size_ptr)?, 4096);
    assert_eq!(memory.read_u32(store_pages_ptr)?, store_size / 4096);
    assert_eq!(memory.read_u32(ram_pages_ptr)?, 16 * 1024);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_MEMORY_DIVISION,
            [0, ram_pages_ptr, page_size_ptr],
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
            ORD_GET_SYSTEM_MEMORY_DIVISION,
            [store_pages_ptr, ram_pages_ptr, 0x2fff_0000],
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
            ORD_CE_GET_THREAD_QUANTUM,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(100),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_SET_THREAD_QUANTUM,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE, 250],
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
            ORD_CE_GET_THREAD_QUANTUM,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(250),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_SET_THREAD_QUANTUM,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE, 0x8000_0000],
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
            ORD_CE_GET_THREAD_QUANTUM,
            [0xdead_beef],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
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
            ORD_CE_SET_THREAD_QUANTUM,
            [0xdead_beef, 75],
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
            ORD_CE_SET_THREAD_QUANTUM,
            [worker_thread, 75],
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
            ORD_CE_GET_THREAD_QUANTUM,
            [worker_thread],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(75),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_THREAD_QUANTUM,
            [CE_CURRENT_THREAD_PSEUDO_HANDLE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(250),
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_PROCESS,
            [0, 0, 0xdead_beef],
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
    let current_process_id = kernel.current_process_id();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_PROCESS,
            [0, 0, current_process_id],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE),
            ..
        }
    ));
    let opened_process = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_PROCESS,
        [0, 0, launch.process_id],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("expected OpenProcess handle, got {other:?}"),
    };
    assert_ne!(opened_process, 0);
    assert_ne!(opened_process, CE_CURRENT_PROCESS_PSEUDO_HANDLE);
    assert_ne!(opened_process, launch.process_handle);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_THCREATE_SNAPSHOT,
            [0x0000_0002, 0xdead_beef],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0xffff_ffff),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    let snapshot = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_THCREATE_SNAPSHOT,
        [0x0000_0002, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("expected THCreateSnapshot handle, got {other:?}"),
    };
    assert_ne!(snapshot, 0);
    assert_ne!(snapshot, 0xffff_ffff);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let snapshot_bytes = memory.read_bytes(snapshot, 60 + 2 * 564);
    let snapshot_u32 = |offset: usize| {
        u32::from_le_bytes([
            snapshot_bytes[offset],
            snapshot_bytes[offset + 1],
            snapshot_bytes[offset + 2],
            snapshot_bytes[offset + 3],
        ])
    };
    assert_eq!(snapshot_u32(0), 60);
    assert_eq!(snapshot_u32(4), 60 + 2 * 564);
    assert_eq!(snapshot_u32(16), 2);
    assert_eq!(snapshot_u32(28), 0x0000_0002);
    let read_snapshot_name = |entry_offset: usize| {
        let mut units = Vec::new();
        for index in 0..260 {
            let offset = entry_offset + 36 + index * 2;
            let unit = u16::from_le_bytes([snapshot_bytes[offset], snapshot_bytes[offset + 1]]);
            if unit == 0 {
                break;
            }
            units.push(unit);
        }
        String::from_utf16_lossy(&units)
    };
    let first_entry = 60;
    let second_entry = 60 + 564;
    assert_eq!(snapshot_u32(first_entry), 564);
    assert_eq!(snapshot_u32(first_entry + 8), 1);
    assert_eq!(read_snapshot_name(first_entry), "process.exe");
    assert_eq!(snapshot_u32(second_entry), 564);
    assert_eq!(snapshot_u32(second_entry + 8), launch.process_id);
    assert_eq!(read_snapshot_name(second_entry), "raw-child.exe");
    let process_source_ptr = 0x5080;
    let process_dest_ptr = 0x5090;
    let process_count_ptr = 0x50a0;
    memory.map_bytes(process_source_ptr, 8);
    memory.map_bytes(process_dest_ptr, 8);
    memory.map_words(process_count_ptr, 1);
    memory.write_bytes(process_source_ptr, b"abcd1234");
    memory.fill(process_dest_ptr, 0xcc, 8);
    memory.write_word(process_count_ptr, 0xffff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_PROCESS_MEMORY,
            [
                opened_process,
                process_source_ptr,
                process_dest_ptr,
                4,
                process_count_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_bytes(process_dest_ptr, 8),
        b"abcd\xcc\xcc\xcc\xcc"
    );
    assert_eq!(memory.read_u32(process_count_ptr)?, 4);
    memory.write_bytes(process_dest_ptr, b"WXYZ5678");
    memory.fill(process_source_ptr, 0, 8);
    memory.write_word(process_count_ptr, 0xffff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_PROCESS_MEMORY,
            [
                opened_process,
                process_source_ptr,
                process_dest_ptr,
                4,
                process_count_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(process_source_ptr, 8), b"WXYZ\0\0\0\0");
    assert_eq!(memory.read_u32(process_count_ptr)?, 4);
    memory.write_word(process_count_ptr, 0xffff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_PROCESS_MEMORY,
            [opened_process, 0, process_dest_ptr, 4, process_count_ptr,],
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
    assert_eq!(memory.read_u32(process_count_ptr)?, 0);
    memory.write_word(process_count_ptr, 0xffff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_PROCESS_MEMORY,
            [
                0xdead_beef,
                process_source_ptr,
                process_dest_ptr,
                4,
                process_count_ptr,
            ],
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
    assert_eq!(memory.read_u32(process_count_ptr)?, 0);
    fn read_guest_le_u32(memory: &TestGuestMemory, addr: u32) -> u32 {
        let bytes = memory.read_bytes(addr, 4);
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn read_guest_wide_z(memory: &TestGuestMemory, addr: u32, max_chars: usize) -> String {
        let bytes = memory.read_bytes(addr, max_chars * 2);
        let mut units = Vec::new();
        for chunk in bytes.chunks_exact(2) {
            let unit = u16::from_le_bytes([chunk[0], chunk[1]]);
            if unit == 0 {
                break;
            }
            units.push(unit);
        }
        String::from_utf16_lossy(&units)
    }

    const STACKSNAP_EXTENDED_INFO: u32 = 0x0000_0002;
    let call_snapshot_ptr = 0x01_9000;
    memory.map_words(call_snapshot_ptr, 1);
    memory.write_u32(call_snapshot_ptr, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALL_STACK_SNAPSHOT,
            [1, call_snapshot_ptr, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(memory.read_u32(call_snapshot_ptr)?, 0xeffe_0000 | thread_id);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    let call_snapshot_ex_ptr = call_snapshot_ptr + 0x100;
    memory.map_words(call_snapshot_ex_ptr, 7);
    for offset in (0..28).step_by(4) {
        memory.write_u32(call_snapshot_ex_ptr + offset, 0xcccc_cccc)?;
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALL_STACK_SNAPSHOT,
            [1, call_snapshot_ex_ptr, STACKSNAP_EXTENDED_INFO, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(call_snapshot_ex_ptr)?,
        0xeffe_0000 | thread_id
    );
    assert_eq!(memory.read_u32(call_snapshot_ex_ptr + 4)?, 0);
    assert_eq!(
        memory.read_u32(call_snapshot_ex_ptr + 8)?,
        kernel.current_process_id()
    );
    assert_eq!(memory.read_u32(call_snapshot_ex_ptr + 12)?, 0);
    assert_eq!(memory.read_u32(call_snapshot_ex_ptr + 16)?, 0);
    assert_eq!(memory.read_u32(call_snapshot_ex_ptr + 20)?, 0);
    assert_eq!(memory.read_u32(call_snapshot_ex_ptr + 24)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALL_STACK_SNAPSHOT,
            [1, call_snapshot_ptr, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALL_STACK_SNAPSHOT,
            [1, call_snapshot_ptr, 0x8000_0000, 0],
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

    const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
    const THSNAP_SIZE: u32 = 60;
    const PROCESSENTRY32_SIZE: u32 = 564;
    const MODULEENTRY32_SIZE: u32 = 1076;
    const THREADENTRY32_SIZE: u32 = 36;
    const PROCESSENTRY32_EXE_OFFSET: u32 = 36;
    const INVALID_TOOLHELP_SNAPSHOT: u32 = 0xffff_ffff;
    const TH32CS_SNAPHEAPLIST: u32 = 0x0000_0001;
    const TH32CS_SNAPTHREAD: u32 = 0x0000_0004;
    const TH32CS_SNAPMODULE: u32 = 0x0000_0008;
    const TH32HEAPLIST_SIZE: u32 = 24;
    const HEAPENTRY32_SIZE: u32 = 36;
    const HF32_DEFAULT: u32 = 1;
    const LF32_FIXED: u32 = 1;

    let toolhelp_module_base = 0x6200_0000;
    kernel.register_loaded_module_with_metadata(
        "toolhelp_mod.dll",
        toolhelp_module_base,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            guest_path: Some(r"\Windows\toolhelp_mod.dll".to_owned()),
            image_size: 0x12_3000,
            ref_count: 3,
            dynamic: true,
            ..LoadedModuleMetadata::default()
        },
    );
    let heap_alloc_a = kernel
        .memory
        .heap_alloc(PROCESS_HEAP_HANDLE, 0, 32)
        .expect("process heap allocation");
    let heap_alloc_b = kernel
        .memory
        .heap_alloc(PROCESS_HEAP_HANDLE, 0, 48)
        .expect("process heap allocation");

    let process_snapshot = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_THCREATE_SNAPSHOT,
        [TH32CS_SNAPHEAPLIST | TH32CS_SNAPPROCESS, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("expected THCreateSnapshot handle, got {other:?}"),
    };
    assert_ne!(process_snapshot, 0);
    assert_ne!(process_snapshot, INVALID_TOOLHELP_SNAPSHOT);
    assert_eq!(read_guest_le_u32(&memory, process_snapshot), THSNAP_SIZE);
    assert_eq!(
        read_guest_le_u32(&memory, process_snapshot + 4),
        THSNAP_SIZE + PROCESSENTRY32_SIZE * 2
    );
    assert!(read_guest_le_u32(&memory, process_snapshot + 8) >= 0x1000);
    assert_eq!(
        read_guest_le_u32(&memory, process_snapshot + 12),
        0x0040_0000
    );
    assert_eq!(read_guest_le_u32(&memory, process_snapshot + 16), 2);
    assert_eq!(
        read_guest_le_u32(&memory, process_snapshot + 28),
        TH32CS_SNAPHEAPLIST | TH32CS_SNAPPROCESS
    );
    assert_eq!(read_guest_le_u32(&memory, process_snapshot + 36), 0);
    let first_entry = process_snapshot + THSNAP_SIZE;
    let second_entry = first_entry + PROCESSENTRY32_SIZE;
    let process_ids = [
        read_guest_le_u32(&memory, first_entry + 8),
        read_guest_le_u32(&memory, second_entry + 8),
    ];
    assert!(process_ids.contains(&kernel.current_process_id()));
    assert!(process_ids.contains(&launch.process_id));
    for entry in [first_entry, second_entry] {
        assert_eq!(read_guest_le_u32(&memory, entry), PROCESSENTRY32_SIZE);
        assert_eq!(read_guest_le_u32(&memory, entry + 4), 1);
        assert_eq!(read_guest_le_u32(&memory, entry + 12), PROCESS_HEAP_HANDLE);
        assert_eq!(read_guest_le_u32(&memory, entry + 20), 1);
        assert_eq!(read_guest_le_u32(&memory, entry + 28), 3);
    }
    let launch_entry = if read_guest_le_u32(&memory, first_entry + 8) == launch.process_id {
        first_entry
    } else {
        second_entry
    };
    assert_eq!(
        read_guest_wide_z(&memory, launch_entry + PROCESSENTRY32_EXE_OFFSET, 260),
        "raw-child.exe"
    );
    let module_snapshot = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_THCREATE_SNAPSHOT,
        [TH32CS_SNAPMODULE, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("expected THCreateSnapshot module handle, got {other:?}"),
    };
    assert_ne!(module_snapshot, 0);
    assert_ne!(module_snapshot, INVALID_TOOLHELP_SNAPSHOT);
    assert_eq!(
        read_guest_le_u32(&memory, module_snapshot + 4),
        THSNAP_SIZE + MODULEENTRY32_SIZE
    );
    assert_eq!(read_guest_le_u32(&memory, module_snapshot + 16), 0);
    assert_eq!(read_guest_le_u32(&memory, module_snapshot + 20), 1);
    assert_eq!(read_guest_le_u32(&memory, module_snapshot + 24), 0);
    assert_eq!(
        read_guest_le_u32(&memory, module_snapshot + 28),
        TH32CS_SNAPMODULE
    );
    let module_entry = module_snapshot + THSNAP_SIZE;
    assert_eq!(read_guest_le_u32(&memory, module_entry), MODULEENTRY32_SIZE);
    assert_eq!(
        read_guest_le_u32(&memory, module_entry + 4),
        toolhelp_module_base
    );
    assert_eq!(
        read_guest_le_u32(&memory, module_entry + 8),
        kernel.current_process_id()
    );
    assert_eq!(read_guest_le_u32(&memory, module_entry + 12), 3);
    assert_eq!(read_guest_le_u32(&memory, module_entry + 16), 3);
    assert_eq!(
        read_guest_le_u32(&memory, module_entry + 20),
        toolhelp_module_base
    );
    assert_eq!(read_guest_le_u32(&memory, module_entry + 24), 0x12_3000);
    assert_eq!(
        read_guest_le_u32(&memory, module_entry + 28),
        toolhelp_module_base
    );
    assert_eq!(
        read_guest_wide_z(&memory, module_entry + 32, 260),
        "toolhelp_mod.dll"
    );
    assert_eq!(
        read_guest_wide_z(&memory, module_entry + 552, 260),
        r"\Windows\toolhelp_mod.dll"
    );
    assert_eq!(read_guest_le_u32(&memory, module_entry + 1072), 0);
    let thread_snapshot = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_THCREATE_SNAPSHOT,
        [TH32CS_SNAPTHREAD, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("expected THCreateSnapshot thread handle, got {other:?}"),
    };
    assert_ne!(thread_snapshot, 0);
    assert_ne!(thread_snapshot, INVALID_TOOLHELP_SNAPSHOT);
    assert_eq!(
        read_guest_le_u32(&memory, thread_snapshot + 4),
        THSNAP_SIZE + PROCESSENTRY32_SIZE * 2 + THREADENTRY32_SIZE * 2
    );
    assert_eq!(read_guest_le_u32(&memory, thread_snapshot + 16), 2);
    assert_eq!(read_guest_le_u32(&memory, thread_snapshot + 24), 2);
    assert_eq!(
        read_guest_le_u32(&memory, thread_snapshot + 28),
        TH32CS_SNAPTHREAD
    );
    let first_thread_entry = thread_snapshot + THSNAP_SIZE + PROCESSENTRY32_SIZE * 2;
    let second_thread_entry = first_thread_entry + THREADENTRY32_SIZE;
    let thread_ids = [
        read_guest_le_u32(&memory, first_thread_entry + 8),
        read_guest_le_u32(&memory, second_thread_entry + 8),
    ];
    assert!(thread_ids.contains(&thread_id));
    assert!(thread_ids.contains(&launch.thread_id));
    let current_thread_entry = if read_guest_le_u32(&memory, first_thread_entry + 8) == thread_id {
        first_thread_entry
    } else {
        second_thread_entry
    };
    let launch_thread_entry =
        if read_guest_le_u32(&memory, first_thread_entry + 8) == launch.thread_id {
            first_thread_entry
        } else {
            second_thread_entry
        };
    for entry in [current_thread_entry, launch_thread_entry] {
        assert_eq!(read_guest_le_u32(&memory, entry), THREADENTRY32_SIZE);
        assert_eq!(read_guest_le_u32(&memory, entry + 4), 1);
    }
    assert_eq!(
        read_guest_le_u32(&memory, current_thread_entry + 12),
        kernel.current_process_id()
    );
    assert_eq!(
        read_guest_le_u32(&memory, launch_thread_entry + 12),
        launch.process_id
    );
    assert_eq!(
        read_guest_le_u32(&memory, launch_thread_entry + 32),
        launch.process_id
    );
    let module_snapshot = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_THCREATE_SNAPSHOT,
        [TH32CS_SNAPMODULE, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("expected THCreateSnapshot module handle, got {other:?}"),
    };
    assert_ne!(module_snapshot, 0);
    assert_ne!(module_snapshot, INVALID_TOOLHELP_SNAPSHOT);
    let module_count = read_guest_le_u32(&memory, module_snapshot + 20);
    assert!(module_count >= 1);
    assert_eq!(read_guest_le_u32(&memory, module_snapshot + 16), 0);
    assert_eq!(read_guest_le_u32(&memory, module_snapshot + 24), 0);
    assert_eq!(
        read_guest_le_u32(&memory, module_snapshot + 28),
        TH32CS_SNAPMODULE
    );
    assert_eq!(
        read_guest_le_u32(&memory, module_snapshot + 4),
        THSNAP_SIZE + module_count * MODULEENTRY32_SIZE
    );
    let module_entry = (0..module_count)
        .map(|index| module_snapshot + THSNAP_SIZE + index * MODULEENTRY32_SIZE)
        .find(|entry| read_guest_le_u32(&memory, *entry + 20) == toolhelp_module_base)
        .expect("registered module entry");
    assert_eq!(read_guest_le_u32(&memory, module_entry), MODULEENTRY32_SIZE);
    assert_eq!(
        read_guest_le_u32(&memory, module_entry + 4),
        toolhelp_module_base
    );
    assert_eq!(
        read_guest_le_u32(&memory, module_entry + 8),
        kernel.current_process_id()
    );
    assert_eq!(read_guest_le_u32(&memory, module_entry + 12), 3);
    assert_eq!(read_guest_le_u32(&memory, module_entry + 16), 3);
    assert_eq!(read_guest_le_u32(&memory, module_entry + 24), 0x12_3000);
    assert_eq!(
        read_guest_wide_z(&memory, module_entry + 32, 260),
        "toolhelp_mod.dll"
    );
    assert_eq!(
        read_guest_wide_z(&memory, module_entry + 552, 260),
        r"\Windows\toolhelp_mod.dll"
    );
    assert_eq!(read_guest_le_u32(&memory, module_entry + 1072), 0);
    let heap_snapshot_bytes = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_HEAP_SNAPSHOT,
        [process_snapshot],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(bytes),
            ..
        } => bytes,
        other => panic!("expected GetHeapSnapshot byte count, got {other:?}"),
    };
    let heap_list = process_snapshot + THSNAP_SIZE + PROCESSENTRY32_SIZE * 2;
    let heap_entry_count = read_guest_le_u32(&memory, heap_list + 16);
    let heap_total_size = read_guest_le_u32(&memory, heap_list + 20);
    assert_eq!(read_guest_le_u32(&memory, process_snapshot + 32), 1);
    assert_eq!(
        read_guest_le_u32(&memory, process_snapshot + 4),
        THSNAP_SIZE + PROCESSENTRY32_SIZE * 2 + heap_snapshot_bytes
    );
    assert_eq!(read_guest_le_u32(&memory, heap_list), 16);
    assert_eq!(
        read_guest_le_u32(&memory, heap_list + 4),
        kernel.current_process_id()
    );
    assert_eq!(
        read_guest_le_u32(&memory, heap_list + 8),
        PROCESS_HEAP_HANDLE
    );
    assert_eq!(read_guest_le_u32(&memory, heap_list + 12), HF32_DEFAULT);
    assert!(heap_entry_count >= 2);
    assert_eq!(
        heap_snapshot_bytes,
        TH32HEAPLIST_SIZE + heap_entry_count * HEAPENTRY32_SIZE
    );
    assert_eq!(heap_total_size, heap_snapshot_bytes);
    let heap_entry_start = heap_list + TH32HEAPLIST_SIZE;
    let heap_entry_addresses = (0..heap_entry_count)
        .map(|index| {
            let entry = heap_entry_start + index * HEAPENTRY32_SIZE;
            assert_eq!(read_guest_le_u32(&memory, entry), HEAPENTRY32_SIZE);
            assert_eq!(read_guest_le_u32(&memory, entry + 16), LF32_FIXED);
            assert_eq!(read_guest_le_u32(&memory, entry + 20), 1);
            assert_eq!(
                read_guest_le_u32(&memory, entry + 28),
                kernel.current_process_id()
            );
            assert_eq!(read_guest_le_u32(&memory, entry + 32), PROCESS_HEAP_HANDLE);
            read_guest_le_u32(&memory, entry + 8)
        })
        .collect::<Vec<_>>();
    assert!(heap_entry_addresses.contains(&heap_alloc_a));
    assert!(heap_entry_addresses.contains(&heap_alloc_b));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_HEAP_SNAPSHOT,
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_THCREATE_SNAPSHOT,
            [TH32CS_SNAPPROCESS, launch.process_id],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0 && handle != INVALID_TOOLHELP_SNAPSHOT
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_THCREATE_SNAPSHOT,
            [TH32CS_SNAPPROCESS, 0xdead_beef],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(INVALID_TOOLHELP_SNAPSHOT),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    let launch_process_index = launch.process_id.saturating_sub(0x42).saturating_add(1);
    let process_exit_ptr = 0x50b0;
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
            [opened_process],
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
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let process_wait = kernel.register_blocked_waiter(
        8,
        0x108,
        vec![opened_process],
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
            [opened_process, process_exit_ptr],
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
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [opened_process, 0],
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
            [opened_process],
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
fn coredll_raw_get_call_stack_snapshot_writes_ce_frame_layouts() -> Result<()> {
    const STACKSNAP_EXTENDED_INFO: u32 = 0x0002;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 34;
    let basic_frames = 0x3a00;
    let extended_frames = 0x3b00;
    memory.map_words(basic_frames, 2);
    memory.map_words(extended_frames, 7);
    memory.write_word(basic_frames, 0xfeed_cafe);
    memory.write_word(basic_frames + 4, 0xfeed_cafe);
    for offset in (0..28).step_by(4) {
        memory.write_word(extended_frames + offset, 0xfeed_cafe);
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALL_STACK_SNAPSHOT,
            [2, basic_frames, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(basic_frames)?, 0xeffe_0000 | thread_id);
    assert_eq!(
        memory.read_u32(basic_frames + 4)?,
        0xfeed_cafe,
        "CE only writes frames it returns"
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALL_STACK_SNAPSHOT,
            [1, extended_frames, STACKSNAP_EXTENDED_INFO, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(extended_frames)?, 0xeffe_0000 | thread_id);
    assert_eq!(memory.read_u32(extended_frames + 4)?, 0);
    assert_eq!(
        memory.read_u32(extended_frames + 8)?,
        kernel.current_process_id()
    );
    assert_eq!(memory.read_u32(extended_frames + 12)?, 0);
    assert_eq!(memory.read_u32(extended_frames + 16)?, 0);
    assert_eq!(memory.read_u32(extended_frames + 20)?, 0);
    assert_eq!(memory.read_u32(extended_frames + 24)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CALL_STACK_SNAPSHOT,
            [1, extended_frames, STACKSNAP_EXTENDED_INFO, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        0,
        "valid CE calls that skip past the emulator frame are empty, not unsupported"
    );

    Ok(())
}

#[test]
fn coredll_raw_get_call_stack_snapshot_rejects_ce_invalid_parameters() -> Result<()> {
    const STACKSNAP_NEW_VM: u32 = 0x0010;
    const MAX_SKIP: u32 = 0x1_0000;
    const MAX_NUM_FRAME: u32 = 0x1_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 35;
    let frames = 0x3c00;
    memory.map_words(frames, 1);

    for args in [
        [0, frames, 0, 0],
        [1, 0, 0, 0],
        [1, frames, STACKSNAP_NEW_VM, 0],
        [1, frames, 0, MAX_SKIP + 1],
        [MAX_NUM_FRAME + 1, frames, 0, 0],
        [2, frames, 0, 0],
        [1, 0x4d00, 0, 0],
    ] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_CALL_STACK_SNAPSHOT,
                args,
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ));
        assert_eq!(
            kernel.threads.get_last_error(thread_id),
            ERROR_INVALID_PARAMETER,
            "unexpected last-error for args {args:?}"
        );
    }

    Ok(())
}

#[test]
fn coredll_raw_get_file_version_info_reads_ce_version_resource() -> Result<()> {
    const ERROR_INVALID_DATA: u32 = 13;
    const VS_FFI_SIGNATURE: u32 = 0xfeef_04bd;
    const VERSION_INFO_SIZE: u32 = 92;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("file_version_info");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(
        root.join("Docs").join("versioned.exe"),
        pe32_with_version_info_resource(VS_FFI_SIGNATURE),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("bad-version.exe"),
        pe32_with_version_info_resource(0x1234_5678),
    )
    .unwrap();
    kernel.set_file_root(&root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 164;
    let path_ptr = 0x2_4000;
    let handle_ptr = 0x2_4800;
    let info_ptr = 0x2_5000;
    memory.map_halfwords(path_ptr, 64);
    memory.map_words(handle_ptr, 1);
    memory.map_bytes(info_ptr, 128);

    memory.write_wide_z(path_ptr, r"\Docs\versioned.exe");
    memory.write_u32(handle_ptr, 0xfeed_beef)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_VERSION_INFO_SIZE_W,
            [path_ptr, handle_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(VERSION_INFO_SIZE),
            ..
        }
    ));
    assert_eq!(memory.read_u32(handle_ptr)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.fill(info_ptr, 0xcc, 128);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_VERSION_INFO_W,
            [path_ptr, 0, 48, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let copied_info = memory.read_bytes(info_ptr, 64);
    assert_eq!(
        read_test_u16(&copied_info, 0),
        48,
        "CE rewrites the copied VERHEAD length to the caller's bounded copy size"
    );
    assert_eq!(read_test_u32(&copied_info, 40), VS_FFI_SIGNATURE);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.fill(info_ptr, 0xdd, 128);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_VERSION_INFO_W,
            [path_ptr, 0, 128, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let full_info = memory.read_bytes(info_ptr, VERSION_INFO_SIZE as usize);
    assert_eq!(read_test_u16(&full_info, 0), VERSION_INFO_SIZE as u16);
    assert_eq!(read_test_u32(&full_info, 40), VS_FFI_SIGNATURE);
    assert_eq!(read_test_u32(&full_info, 44), 0x0001_0000);
    assert_eq!(read_test_u32(&full_info, 48), 0x0006_0001);
    assert_eq!(read_test_u32(&full_info, 52), 0x0000_0002);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\bad-version.exe");
    memory.write_u32(handle_ptr, 0xabcd_1234)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_VERSION_INFO_SIZE_W,
            [path_ptr, handle_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(handle_ptr)?,
        0,
        "GetFileVersionInfoSizeW clears lpdwHandle before validating the resource"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_DATA);

    memory.fill(info_ptr, 0xee, 128);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_VERSION_INFO_W,
            [path_ptr, 0, 128, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(info_ptr, 4), [0xee, 0xee, 0xee, 0xee]);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_DATA);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_open_event_w_opens_existing_named_event_only() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("module_api_file_handle");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Windows")).unwrap();
    fs::write(root.join("Windows").join("commctrl.dll"), b"module-bytes").unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 41;
    let module_name_ptr = 0x1_8000;
    let proc_w_ptr = 0x1_8040;
    let proc_a_ptr = 0x1_8080;
    let module_path_ptr = 0x1_80c0;
    let missing_module_name_ptr = 0x1_8140;
    let missing_proc_w_ptr = 0x1_8180;
    let module_info_ptr = 0x1_81c0;
    let process_path = r"\SDMMC Disk\INavi\iNavi.exe";
    let child_process_path = r"\SDMMC Disk\INavi\iSearch.exe";
    let module_base = 0x6200_0000;
    let proc_by_name = 0x6200_1234;
    let proc_by_ordinal = 0x6200_5678;
    let module_entry = 0x6200_0080;
    let module_path = r"\Windows\commctrl.dll";
    const MINFO_MODULE_INFO: u32 = 0x8;

    let mut exports_by_name = std::collections::BTreeMap::new();
    exports_by_name.insert("InitCommonControlsEx".to_owned(), proc_by_name);
    let mut exports_by_ordinal = std::collections::BTreeMap::new();
    exports_by_ordinal.insert(17, proc_by_ordinal);
    kernel.register_loaded_module_with_metadata(
        "commctrl.dll",
        module_base,
        exports_by_name,
        exports_by_ordinal,
        LoadedModuleMetadata {
            guest_path: Some(module_path.to_owned()),
            image_size: 0x10000,
            entry_point: module_entry,
            ..LoadedModuleMetadata::default()
        },
    );
    kernel.set_process_module_path(process_path);
    memory.write_wide_z(module_name_ptr, "commctrl.dll");
    memory.write_wide_z(proc_w_ptr, "InitCommonControlsEx");
    memory.write_bytes(proc_a_ptr, b"InitCommonControlsEx\0");
    memory.write_wide_z(missing_module_name_ptr, "missing.dll");
    memory.write_wide_z(missing_proc_w_ptr, "MissingExport");
    memory.map_halfwords(module_path_ptr, 64);
    memory.map_words(module_info_ptr, 3);

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
            ORD_GET_MODULE_FILE_NAME_W,
            [module_base, module_path_ptr, 64],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(copied),
            ..
        } if copied == module_path.encode_utf16().count() as u32
    ));
    assert_eq!(memory.read_wide_z(module_path_ptr, 64), module_path);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MODULE_INFORMATION,
            [
                CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                module_base,
                module_info_ptr,
                12
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(module_info_ptr)?, module_base);
    assert_eq!(memory.read_u32(module_info_ptr + 4)?, 0x10000);
    assert_eq!(memory.read_u32(module_info_ptr + 8)?, module_entry);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    memory.write_word(module_info_ptr, 0xAAAA_AAAA);
    memory.write_word(module_info_ptr + 4, 0xBBBB_BBBB);
    memory.write_word(module_info_ptr + 8, 0xCCCC_CCCC);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_MODULE_INFO,
            [
                CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                module_base,
                MINFO_MODULE_INFO,
                module_info_ptr,
                12
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(module_info_ptr)?, module_base);
    assert_eq!(memory.read_u32(module_info_ptr + 4)?, 0x10000);
    assert_eq!(memory.read_u32(module_info_ptr + 8)?, module_entry);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MODULE_INFORMATION,
            [0xDEAD_0000, module_base, module_info_ptr, 12],
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
            ORD_GET_MODULE_INFORMATION,
            [
                CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                module_base,
                module_info_ptr,
                16
            ],
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
            ORD_CE_GET_MODULE_INFO,
            [
                CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                module_base,
                MINFO_MODULE_INFO + 1,
                module_info_ptr,
                12
            ],
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
    let module_file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CE_OPEN_FILE_HANDLE,
        [module_base],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("unexpected CeOpenFileHandle dispatch: {other:?}"),
    };
    assert_ne!(module_file, u32::MAX);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(kernel.read_file(module_file, 12)?, b"module-bytes");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_OPEN_FILE_HANDLE,
            [0xDEAD_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0xffff_ffff),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    let process_name_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_PROC_NAME,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("unexpected GetProcName dispatch: {other:?}"),
    };
    assert_ne!(process_name_ptr, 0);
    assert_eq!(memory.read_wide_z(process_name_ptr, 260), process_path);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_NAME,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == process_name_ptr
    ));
    kernel.set_process_module_path(child_process_path);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_NAME,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr != 0 && ptr != process_name_ptr && memory.read_wide_z(ptr, 260) == child_process_path
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MODULE_FILE_NAME_W,
            [0xDEAD_0000, module_path_ptr, 64],
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
            ORD_GET_PROC_ADDRESS_IN_PROCESS,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, module_name_ptr, proc_w_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == proc_by_name
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_IN_PROCESS,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, module_name_ptr, 17],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == proc_by_ordinal
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_IN_PROCESS,
            [
                CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                missing_module_name_ptr,
                proc_w_ptr
            ],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_IN_PROCESS,
            [
                CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                module_name_ptr,
                missing_proc_w_ptr
            ],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_IN_PROCESS,
            [0xDEAD_0000, module_name_ptr, proc_w_ptr],
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
    let stats = kernel.runtime_loader_stats();
    assert_eq!(stats.export_lookup_count, 6);
    assert_eq!(stats.export_lookup_miss_count, 1);

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

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_loadlibrary_refcounts_dynamic_modules_and_ex_flags_reuse_loaded_modules()
-> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 42;
    let module_name_ptr = 0x1_9000;
    let resource_name_ptr = 0x1_9080;
    let proc_name_ptr = 0x1_90c0;
    let deferred_name_ptr = 0x1_9100;
    let driver_name_ptr = 0x1_9180;
    let driver_proc_name_ptr = 0x1_91c0;
    let late_resource_name_ptr = 0x1_9200;
    let noresolve_name_ptr = 0x1_9280;
    let missing_name_ptr = 0x1_9040;
    let module_base = 0x6300_0000;
    let resource_base = 0x6310_0000;
    let deferred_base = 0x6320_0000;
    let driver_base = 0x6328_0000;
    let late_resource_base = 0x6330_0000;
    let noresolve_base = 0x6340_0000;

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

    kernel.register_loaded_module_with_metadata(
        "resource.dll",
        resource_base,
        std::collections::BTreeMap::from([("visibleexport".to_owned(), resource_base + 0x1234)]),
        std::collections::BTreeMap::from([(7, resource_base + 0x5678)]),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\resource.dll".to_owned()),
            image_size: 0x10000,
            load_flags: 0x0000_0003,
            ..LoadedModuleMetadata::default()
        },
    );
    memory.write_wide_z(resource_name_ptr, "resource.dll");
    memory.write_bytes(proc_name_ptr, b"VisibleExport\0");

    kernel.register_loaded_module_with_metadata(
        "late_resource.dll",
        late_resource_base,
        std::collections::BTreeMap::from([(
            "visibleexport".to_owned(),
            late_resource_base + 0x1234,
        )]),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\late_resource.dll".to_owned()),
            image_size: 0x10000,
            ..LoadedModuleMetadata::default()
        },
    );
    memory.write_wide_z(late_resource_name_ptr, "late_resource.dll");

    kernel.register_loaded_module_with_metadata(
        "noresolve_request.dll",
        noresolve_base,
        std::collections::BTreeMap::from([("visibleexport".to_owned(), noresolve_base + 0x1234)]),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\noresolve_request.dll".to_owned()),
            image_size: 0x10000,
            ..LoadedModuleMetadata::default()
        },
    );
    memory.write_wide_z(noresolve_name_ptr, "noresolve_request.dll");

    kernel.register_loaded_module_with_metadata(
        "deferred.dll",
        deferred_base,
        std::collections::BTreeMap::from([("visibleexport".to_owned(), deferred_base + 0x1234)]),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\deferred.dll".to_owned()),
            image_size: 0x10000,
            load_flags: 0x0000_0001,
            ..LoadedModuleMetadata::default()
        },
    );
    memory.write_wide_z(deferred_name_ptr, "deferred.dll");

    kernel.register_loaded_module_with_metadata(
        "storage_driver.dll",
        driver_base,
        std::collections::BTreeMap::from([("Init".to_owned(), driver_base + 0x1110)]),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\storage_driver.dll".to_owned()),
            image_size: 0x10000,
            ..LoadedModuleMetadata::default()
        },
    );
    memory.write_wide_z(driver_name_ptr, "storage_driver.dll");
    memory.write_bytes(driver_proc_name_ptr, b"Init\0");
    memory.write_wide_z(missing_name_ptr, "missing.dll");

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
    assert_eq!(
        kernel
            .loaded_module_by_handle(module_base)
            .unwrap()
            .ref_count,
        3
    );

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
    assert_eq!(
        kernel
            .loaded_module_by_handle(module_base)
            .unwrap()
            .ref_count,
        2
    );

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
    assert_eq!(
        kernel
            .loaded_module_by_handle(module_base)
            .unwrap()
            .ref_count,
        3
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_EX_W,
            [resource_name_ptr, 0, 0x0000_0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == resource_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_EX_W,
            [late_resource_name_ptr, 0, 0x0000_0002],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == late_resource_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        kernel
            .loaded_module_by_handle(late_resource_base)
            .unwrap()
            .load_flags
            & 0x0000_0003,
        0x0000_0003
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_A,
            [late_resource_base, proc_name_ptr],
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
            [noresolve_name_ptr, 0, 0x0000_0001],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == noresolve_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        kernel
            .loaded_module_by_handle(noresolve_base)
            .unwrap()
            .load_flags
            & 0x0000_0001,
        0x0000_0001
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_A,
            [noresolve_base, proc_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == noresolve_base + 0x1234
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_W,
            [noresolve_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == noresolve_base
    ));
    assert_eq!(
        kernel
            .loaded_module_by_handle(noresolve_base)
            .unwrap()
            .load_flags
            & 0x0000_0001,
        0
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_EX_W,
            [module_name_ptr, 0x1234, 0],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_LIBRARY_W,
            [deferred_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == deferred_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let deferred = kernel.loaded_module_by_handle(deferred_base).unwrap();
    assert_eq!(deferred.ref_count, 2);
    assert_eq!(deferred.load_flags & 0x0000_0001, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_DRIVER,
            [driver_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == driver_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let driver = kernel.loaded_module_by_handle(driver_base).unwrap();
    assert_eq!(driver.ref_count, 2);
    assert_eq!(driver.load_flags & (0x0001 << 16), 0x0001 << 16);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_A,
            [driver_base, driver_proc_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == driver_base + 0x1110
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_A,
            [resource_base, proc_name_ptr],
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
            ORD_LOAD_DRIVER,
            [missing_name_ptr],
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
            ORD_LOAD_DRIVER,
            [0],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_W,
            [resource_base, 7],
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
            [module_name_ptr, 0, 0x0000_0008],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == module_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        kernel
            .loaded_module_by_handle(module_base)
            .unwrap()
            .ref_count,
        4
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
fn coredll_raw_load_fsd_validates_device_driver_and_exports() -> Result<()> {
    const LOADFSD_ASYNCH: u32 = 0x0000;
    const LOADFSD_SYNCH: u32 = 0x0001;
    const ERROR_PROC_NOT_FOUND: u32 = 127;
    const LLIB_NO_PAGING: u32 = 0x0001;
    const LOAD_DRIVER_FLAGS: u32 = LLIB_NO_PAGING << 16;

    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.devices = DeviceConfigFile {
        version: 1,
        defaults: DeviceDefaults::default(),
        devices: vec![DeviceConfig {
            guest: "COM9:".to_owned(),
            kind: DeviceKind::Serial,
            backend: DeviceBackend::Stub,
            host: None,
            remote_gps: false,
            enabled: true,
            note: None,
        }],
    };
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 44;
    let type_ptr = 0x1_9400;
    let fsd_name_ptr = 0x1_9480;
    let bad_fsd_name_ptr = 0x1_9500;
    let missing_name_ptr = 0x1_9580;
    let fsd_base = 0x6340_0000;
    let bad_fsd_base = 0x6350_0000;

    memory.write_wide_z(type_ptr, "COM");
    memory.write_wide_z(fsd_name_ptr, "fatfsd.dll");
    memory.write_wide_z(bad_fsd_name_ptr, "badfsd.dll");
    memory.write_wide_z(missing_name_ptr, "missingfsd.dll");
    kernel.register_loaded_module_with_metadata(
        "fatfsd.dll",
        fsd_base,
        std::collections::BTreeMap::from([
            ("FSD_MountDisk".to_owned(), fsd_base + 0x1000),
            ("FSD_UnmountDisk".to_owned(), fsd_base + 0x1100),
        ]),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\fatfsd.dll".to_owned()),
            image_size: 0x18000,
            ..LoadedModuleMetadata::default()
        },
    );
    kernel.register_loaded_module_with_metadata(
        "badfsd.dll",
        bad_fsd_base,
        std::collections::BTreeMap::from([("FSD_MountDisk".to_owned(), bad_fsd_base + 0x1000)]),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\badfsd.dll".to_owned()),
            image_size: 0x10000,
            ..LoadedModuleMetadata::default()
        },
    );

    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(device),
        ..
    } = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTER_DEVICE,
        [type_ptr, 9, 0, 0],
    )
    else {
        panic!("RegisterDevice did not return a handle");
    };
    assert_ne!(device, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_FSD,
            [device, fsd_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let fsd = kernel.loaded_module_by_handle(fsd_base).unwrap();
    assert_eq!(fsd.ref_count, 2);
    assert_eq!(fsd.load_flags & LOAD_DRIVER_FLAGS, LOAD_DRIVER_FLAGS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_FSDEX,
            [device, fsd_name_ptr, LOADFSD_SYNCH],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(
        kernel.loaded_module_by_handle(fsd_base).unwrap().ref_count,
        3
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_FSDEX,
            [device, fsd_name_ptr, 0x2],
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
            ORD_LOAD_FSDEX,
            [0xffff_1234, fsd_name_ptr, LOADFSD_ASYNCH],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_FSD,
            [device, bad_fsd_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_PROC_NOT_FOUND
    );
    assert_eq!(
        kernel
            .loaded_module_by_handle(bad_fsd_base)
            .unwrap()
            .ref_count,
        1
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_FSD,
            [device, missing_name_ptr],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_FSD,
            [device, 0x7fff_0000],
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
fn coredll_raw_load_kernel_library_applies_ce_flags_and_keeps_exports_visible() -> Result<()> {
    const LOAD_LIBRARY_IN_KERNEL: u32 = 0x0000_8000;
    const MF_NO_THREAD_CALLS: u32 = 0x0000_0400;
    const LLIB_NO_PAGING: u32 = 0x0001;
    const LOAD_KERNEL_LIBRARY_FLAGS: u32 =
        LOAD_LIBRARY_IN_KERNEL | MF_NO_THREAD_CALLS | (LLIB_NO_PAGING << 16);

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 43;
    let module_name_ptr = 0x1_9200;
    let proc_name_ptr = 0x1_9280;
    let missing_name_ptr = 0x1_9300;
    let module_base = 0x6330_0000;
    let io_control = module_base + 0x2440;

    kernel.register_loaded_module_with_metadata(
        "kernel_driver.dll",
        module_base,
        std::collections::BTreeMap::from([("IOControl".to_owned(), io_control)]),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            guest_path: Some(r"\Windows\kernel_driver.dll".to_owned()),
            image_size: 0x14000,
            ..LoadedModuleMetadata::default()
        },
    );
    memory.write_wide_z(module_name_ptr, "kernel_driver.dll");
    memory.write_bytes(proc_name_ptr, b"IOControl\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_KERNEL_LIBRARY,
            [module_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == module_base
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let loaded = kernel.loaded_module_by_handle(module_base).unwrap();
    assert_eq!(loaded.ref_count, 2);
    assert_eq!(
        loaded.load_flags & LOAD_KERNEL_LIBRARY_FLAGS,
        LOAD_KERNEL_LIBRARY_FLAGS
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PROC_ADDRESS_A,
            [module_base, proc_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(address),
            ..
        } if address == io_control
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(missing_name_ptr, "missing_kernel_driver.dll");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_KERNEL_LIBRARY,
            [missing_name_ptr],
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
            ORD_LOAD_KERNEL_LIBRARY,
            [0x7fff_0000],
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
fn coredll_raw_query_instruction_set_reports_ce_mipsii_compatibility() -> Result<()> {
    const PROCESSOR_QUERY_INSTRUCTION: u32 = 0;
    const PROCESSOR_ARM_V4_INSTRUCTION: u32 = 0x0501_0000;
    const PROCESSOR_ARM_V4I_INSTRUCTION: u32 = 0x0502_0000;
    const PROCESSOR_ARM_V4TFP_INSTRUCTION: u32 = 0x0503_0001;
    const PROCESSOR_MIPS_MIPS16_INSTRUCTION: u32 = 0x0101_0000;
    const PROCESSOR_MIPS_MIPSII_INSTRUCTION: u32 = 0x0102_0000;
    const PROCESSOR_MIPS_MIPSIIFP_INSTRUCTION: u32 = 0x0102_0001;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 44;
    let out_ptr = 0x1_9400;
    memory.map_words(out_ptr, 1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_QUERY_INSTRUCTION_SET,
            [PROCESSOR_QUERY_INSTRUCTION, out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(out_ptr)?, PROCESSOR_MIPS_MIPSII_INSTRUCTION);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_QUERY_INSTRUCTION_SET,
            [PROCESSOR_MIPS_MIPSII_INSTRUCTION, out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(out_ptr)?, PROCESSOR_MIPS_MIPSII_INSTRUCTION);

    for unsupported in [
        PROCESSOR_ARM_V4_INSTRUCTION,
        PROCESSOR_ARM_V4I_INSTRUCTION,
        PROCESSOR_ARM_V4TFP_INSTRUCTION,
        PROCESSOR_MIPS_MIPS16_INSTRUCTION,
        PROCESSOR_MIPS_MIPSIIFP_INSTRUCTION,
    ] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_QUERY_INSTRUCTION_SET,
                [unsupported, out_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ));
        assert_eq!(kernel.threads.get_last_error(thread_id), 0);
        assert_eq!(memory.read_u32(out_ptr)?, PROCESSOR_MIPS_MIPSII_INSTRUCTION);
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_QUERY_INSTRUCTION_SET,
            [PROCESSOR_QUERY_INSTRUCTION, 0],
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
fn coredll_raw_page_out_module_validates_ce_process_module_edges() -> Result<()> {
    const PAGE_OUT_PROCESS_ONLY: u32 = 0;
    const PAGE_OUT_DLL_USED_ONLY_BY_THISPROC: u32 = 1;
    const PAGE_OUT_ALL_DEPENDENT_DLL: u32 = 2;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let mut memory = TestGuestMemory::default();
    let thread_id = 77;
    let module_base = 0x6600_0000;

    kernel.register_loaded_module_with_metadata(
        "pageable.dll",
        module_base,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            ref_count: 2,
            ..LoadedModuleMetadata::default()
        },
    );
    let launch = kernel.queue_process_launch(Some("pageout-child.exe".to_owned()), None);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PAGE_OUT_MODULE,
            [
                CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                module_base,
                PAGE_OUT_PROCESS_ONLY,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(kernel.is_loaded_module_handle(module_base));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PAGE_OUT_MODULE,
            [module_base, PAGE_OUT_PROCESS_ONLY],
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
            ORD_PAGE_OUT_MODULE,
            [launch.process_handle, 0, PAGE_OUT_ALL_DEPENDENT_DLL],
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
            ORD_PAGE_OUT_MODULE,
            [
                launch.process_handle,
                module_base,
                PAGE_OUT_DLL_USED_ONLY_BY_THISPROC,
            ],
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
            ORD_PAGE_OUT_MODULE,
            [0xdead_beef, module_base, 0x99],
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
            ORD_PAGE_OUT_MODULE,
            [CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0, PAGE_OUT_PROCESS_ONLY],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PAGE_OUT_MODULE,
            [launch.process_handle, 0x1234_0000, PAGE_OUT_PROCESS_ONLY],
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PAGE_OUT_MODULE,
            [0x1234_0000, PAGE_OUT_PROCESS_ONLY],
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
fn coredll_raw_disable_thread_library_calls_validates_module_handles() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    assert!(
        kernel
            .loaded_module_by_handle(module_base)
            .unwrap()
            .thread_library_calls_disabled
    );

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
fn coredll_raw_string_compress_decompress_matches_ce_raw_packet_edges() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 42;
    let input_ptr = 0x3000_1000;
    let packet_ptr = 0x3000_2000;
    let output_ptr = 0x3000_3000;

    memory.write_bytes(input_ptr, b"A\0B\0C\0D\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 8, packet_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(6),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u16(packet_ptr)?, 0x8004);
    assert_eq!(memory.read_bytes(packet_ptr + 2, 4), b"ABCD");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_DECOMPRESS,
            [packet_ptr, 6, output_ptr, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(8),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_bytes(output_ptr, 8), b"A\0B\0C\0D\0");

    memory.write_bytes(input_ptr, b"\0A\0B\0C\0D");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 8, packet_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(6),
            ..
        }
    ));
    assert_eq!(memory.read_u16(packet_ptr)?, 0x4000);
    assert_eq!(memory.read_bytes(packet_ptr + 2, 4), b"ABCD");

    memory.write_bytes(output_ptr, &[0xff; 8]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_DECOMPRESS,
            [packet_ptr, 6, output_ptr, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(8),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(output_ptr, 8), b"\0A\0B\0C\0D");

    memory.write_bytes(input_ptr, b"A\0B\0C\0D");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 7, packet_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(6),
            ..
        }
    ));
    assert_eq!(memory.read_u16(packet_ptr)?, 0x8004);
    assert_eq!(memory.read_bytes(packet_ptr + 2, 4), b"ABCD");

    memory.write_bytes(output_ptr, &[0xff; 8]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_DECOMPRESS,
            [packet_ptr, 6, output_ptr, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(8),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(output_ptr, 8), b"A\0B\0C\0D\0");

    memory.write_bytes(input_ptr, b"A\0B\0C\0D\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 8, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(6),
            ..
        }
    ));

    memory.write_bytes(input_ptr, &[0; 8]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 8, packet_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u16(packet_ptr)?, 0);

    memory.write_bytes(output_ptr, &[0xff; 8]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_DECOMPRESS,
            [packet_ptr, 2, output_ptr, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(output_ptr, 8), [0; 8]);

    memory.write_bytes(input_ptr, b"ABCDEFGH");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 8, packet_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_bytes(input_ptr, b"AAAAAAAAAAAAAAAA");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 16, packet_ptr, 32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(10),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let compressed_header = memory.read_u16(packet_ptr)?;
    assert_eq!(compressed_header & 0xc000, 0);
    assert_eq!(compressed_header & 0x3fff, 4);
    assert_eq!(
        memory.read_bytes(packet_ptr + 2, 4),
        [0xce, 0x53, 0x87, b'A']
    );
    assert_eq!(
        memory.read_bytes(packet_ptr + 6, 4),
        [0xce, 0x53, 0x87, b'A']
    );

    memory.write_bytes(output_ptr, &[0xff; 16]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_DECOMPRESS,
            [packet_ptr, 10, output_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(16),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_bytes(output_ptr, 16), b"AAAAAAAAAAAAAAAA");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_COMPRESS,
            [input_ptr, 16, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(10),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_bytes(packet_ptr, &[0x04, 0x00, 1, 2, 3, 4]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_DECOMPRESS,
            [packet_ptr, 6, output_ptr, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    memory.write_bytes(packet_ptr, &[0x02, 0x80, b'A', b'B', 1, 2]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_DECOMPRESS,
            [packet_ptr, 6, output_ptr, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
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
fn coredll_raw_process_detach_all_dlls_drains_imported_module_refs() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 42;

    kernel.register_loaded_module_with_metadata(
        "normal.dll",
        0x6500_0000,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            ref_count: 2,
            ..LoadedModuleMetadata::default()
        },
    );
    kernel.register_loaded_module_with_metadata(
        "disabled.dll",
        0x6510_0000,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            thread_library_calls_disabled: true,
            ..LoadedModuleMetadata::default()
        },
    );
    kernel.register_loaded_module_with_metadata(
        "noresolve.dll",
        0x6520_0000,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            load_flags: 0x0000_0001,
            ..LoadedModuleMetadata::default()
        },
    );
    kernel.register_loaded_module_with_metadata(
        "datafile.dll",
        0x6530_0000,
        std::collections::BTreeMap::new(),
        std::collections::BTreeMap::new(),
        LoadedModuleMetadata {
            dynamic: true,
            load_flags: 0x0000_0003,
            ..LoadedModuleMetadata::default()
        },
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PROCESS_DETACH_ALL_DLLS,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(kernel.loaded_module_handle("normal.dll"), None);
    assert_eq!(kernel.loaded_module_handle("disabled.dll"), None);
    assert_eq!(
        kernel.loaded_module_handle("noresolve.dll"),
        Some(0x6520_0000)
    );
    assert_eq!(
        kernel.loaded_module_handle("datafile.dll"),
        Some(0x6530_0000)
    );
    Ok(())
}

#[test]
fn shell_execute_ex_resolves_registry_association_and_queues_process() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    const FILE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
    const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x0000_0100;
    const SFGAO_LINK: u32 = 0x0001_0000;
    const SFGAO_READONLY: u32 = 0x0004_0000;
    const SFGAO_FOLDER: u32 = 0x2000_0000;
    const SFGAO_FILESYSTEM: u32 = 0x4000_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shget_file_info");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(root.join("Docs").join("morning.nav"), b"route").unwrap();
    fs::write(root.join("Docs").join("readonly.txt"), b"locked").unwrap();
    let mut readonly_permissions = fs::metadata(root.join("Docs").join("readonly.txt"))
        .unwrap()
        .permissions();
    readonly_permissions.set_readonly(true);
    fs::set_permissions(root.join("Docs").join("readonly.txt"), readonly_permissions).unwrap();
    fs::write(root.join("Docs").join("viewer.exe"), b"MZ").unwrap();
    fs::write(
        root.join("Docs").join("two-icons.exe"),
        pe32_with_group_icon_count(2),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("multi-size-icons.exe"),
        pe32_with_multi_size_icon_group(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("named-group-icon.exe"),
        pe32_with_string_named_icon_group(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("sparse-group-icon.exe"),
        pe32_with_sparse_group_icon_id(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("bad-group-icon.exe"),
        pe32_with_malformed_icon_group_type(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("missing-icon-resource.exe"),
        pe32_with_missing_icon_resource(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("missing-small-icon-resource.exe"),
        pe32_with_missing_small_icon_resource(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("missing-and-mask-icon.exe"),
        pe32_with_missing_and_mask_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("eight-bpp-icon.exe"),
        pe32_with_8bpp_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("one-bpp-icon.exe"),
        pe32_with_1bpp_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("core-header-icon.exe"),
        pe32_with_core_header_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("two-bpp-icon.exe"),
        pe32_with_2bpp_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("four-bpp-icon.exe"),
        pe32_with_4bpp_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("twentyfour-bpp-icon.exe"),
        pe32_with_24bpp_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("thirtytwo-bpp-icon.exe"),
        pe32_with_32bpp_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("masked-twentyfour-bpp-icon.exe"),
        pe32_with_24bpp_masked_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("sixteen-bpp-rgb-icon.exe"),
        pe32_with_16bpp_rgb_icon(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("bitfields-icon.exe"),
        pe32_with_16bpp_bitfields_icon(),
    )
    .unwrap();
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
        SFGAO_FILESYSTEM
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
    memory.write_u32(large_icon_ptr + 4, 0xdead_beef)?;
    memory.write_u32(small_icon_ptr + 4, 0xfeed_face)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0, large_icon_ptr, small_icon_ptr, 2],
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
    assert_eq!(
        memory.read_u32(large_icon_ptr + 4)?,
        0xdead_beef,
        "non-PE ExtractIconExW fallback should only fill the index-zero slot"
    );
    assert_eq!(
        memory.read_u32(small_icon_ptr + 4)?,
        0xfeed_face,
        "non-PE ExtractIconExW fallback should leave later slots untouched"
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0, 0, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        0,
        "non-PE ExtractIconExW with no output arrays is a no-op, not a synthetic extraction"
    );
    memory.write_u32(large_icon_ptr, 0xface_cafe)?;
    memory.write_u32(small_icon_ptr, 0xc001_d00d)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 1, large_icon_ptr, small_icon_ptr, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u32(large_icon_ptr)?,
        0xface_cafe,
        "non-PE ExtractIconExW index-one miss should not mutate large output"
    );
    assert_eq!(
        memory.read_u32(small_icon_ptr)?,
        0xc001_d00d,
        "non-PE ExtractIconExW index-one miss should not mutate small output"
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

    memory.write_wide_z(path_ptr, r"\Docs\morning.nav");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0xdead_beef)?;
    memory.write_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET, 0xfeed_face)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [path_ptr, 0, info_ptr, SHFILEINFO_SIZE_W, SHGFI_ICON],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x000b_f100),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle(expected_nav_icon)
    );
    assert_eq!(
        memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?,
        expected_nav_icon,
        "CE fills iIcon for SHGFI_ICON even without SHGFI_SYSICONINDEX"
    );
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET)?,
        0xfeed_face,
        "CE does not clear dwAttributes when SHGFI_ATTRIBUTES is absent"
    );

    memory.write_wide_z(path_ptr, r"\Docs\two-icons.exe");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0xffff_ffff, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    memory.write_u32(large_icon_ptr, 0)?;
    memory.write_u32(large_icon_ptr + 4, 0)?;
    memory.write_u32(small_icon_ptr, 0)?;
    memory.write_u32(small_icon_ptr + 4, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 0, large_icon_ptr, small_icon_ptr, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    let first_large_icon = memory.read_u32(large_icon_ptr)?;
    let second_large_icon = memory.read_u32(large_icon_ptr + 4)?;
    assert_ne!(first_large_icon, 0);
    assert_ne!(second_large_icon, 0);
    assert_ne!(first_large_icon, second_large_icon);
    assert_eq!(memory.read_u32(small_icon_ptr)?, first_large_icon);
    assert_eq!(memory.read_u32(small_icon_ptr + 4)?, second_large_icon);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_u32(large_icon_ptr, 0)?;
    memory.write_u32(large_icon_ptr + 4, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EXTRACT_ICON_EX_W,
            [path_ptr, 1, large_icon_ptr, 0, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_ne!(memory.read_u32(large_icon_ptr)?, 0);
    assert_eq!(memory.read_u32(large_icon_ptr + 4)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\multi-size-icons.exe");
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
    let large_icon = memory.read_u32(large_icon_ptr)?;
    let small_icon = memory.read_u32(small_icon_ptr)?;
    assert_ne!(large_icon, 0);
    assert_ne!(small_icon, 0);
    assert_ne!(large_icon, small_icon);
    let large_color_bitmap = kernel
        .resources
        .icon(large_icon)
        .expect("large PE icon")
        .color_bitmap;
    let small_color_bitmap = kernel
        .resources
        .icon(small_icon)
        .expect("small PE icon")
        .color_bitmap;
    let large_bitmap = kernel
        .resources
        .bitmap(large_color_bitmap)
        .expect("large PE icon bitmap");
    let small_bitmap = kernel
        .resources
        .bitmap(small_color_bitmap)
        .expect("small PE icon bitmap");
    assert_eq!((large_bitmap.width, large_bitmap.height), (32, 32));
    assert_eq!((small_bitmap.width, small_bitmap.height), (16, 16));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\eight-bpp-icon.exe");
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
    let eight_bpp_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(eight_bpp_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, eight_bpp_icon);
    let eight_bpp_color_bitmap = kernel
        .resources
        .icon(eight_bpp_icon)
        .expect("8bpp PE icon")
        .color_bitmap;
    let eight_bpp_bitmap = kernel
        .resources
        .bitmap(eight_bpp_color_bitmap)
        .expect("8bpp PE icon bitmap");
    assert_eq!(eight_bpp_bitmap.bits_pixel, 8);
    assert_eq!(eight_bpp_bitmap.color_table.len(), 256);
    assert_eq!(
        eight_bpp_bitmap.color_table[7],
        [0x07, 0x47, 0x87, 0x00],
        "8bpp RT_ICON color table should be preserved on the extracted bitmap"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

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

    memory.write_wide_z(path_ptr, r"\Docs\one-bpp-icon.exe");
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
    let one_bpp_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(one_bpp_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, one_bpp_icon);
    let one_bpp_color_bitmap = kernel
        .resources
        .icon(one_bpp_icon)
        .expect("1bpp PE icon")
        .color_bitmap;
    let one_bpp_bitmap = kernel
        .resources
        .bitmap(one_bpp_color_bitmap)
        .expect("1bpp PE icon bitmap");
    assert_eq!(one_bpp_bitmap.bits_pixel, 1);
    assert_eq!(one_bpp_bitmap.color_table.len(), 2);
    assert_eq!(
        one_bpp_bitmap.color_table[0],
        [0x00, 0x00, 0xff, 0x00],
        "1bpp RT_ICON palette entry zero should be preserved"
    );
    assert_eq!(
        one_bpp_bitmap.color_table[1],
        [0x00, 0xff, 0x00, 0x00],
        "1bpp RT_ICON palette entry one should be preserved"
    );
    let mut one_bpp_framebuffer = VirtualFramebuffer::new(4, 2, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut one_bpp_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 1, 0, one_bpp_icon, 2, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let one_bpp_left = PixelFormat::Rgb565.bytes_per_pixel();
    let one_bpp_right = one_bpp_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &one_bpp_framebuffer.pixels()[one_bpp_left..one_bpp_left + 2],
        &[0xe0, 0x07],
        "1bpp RT_ICON draw should decode the high-bit palette index"
    );
    assert_eq!(
        &one_bpp_framebuffer.pixels()[one_bpp_right..one_bpp_right + 2],
        &[0x00, 0xf8],
        "1bpp RT_ICON draw should decode the following zero palette bit"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\core-header-icon.exe");
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
    let core_header_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(core_header_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, core_header_icon);
    let core_header_color_bitmap = kernel
        .resources
        .icon(core_header_icon)
        .expect("BITMAPCOREHEADER PE icon")
        .color_bitmap;
    let core_header_bitmap = kernel
        .resources
        .bitmap(core_header_color_bitmap)
        .expect("BITMAPCOREHEADER PE icon bitmap");
    assert_eq!(core_header_bitmap.bits_pixel, 1);
    assert_eq!(
        core_header_bitmap.color_table,
        vec![[0x00, 0x00, 0xff, 0], [0x00, 0xff, 0x00, 0]],
        "BITMAPCOREHEADER RT_ICON RGBTRIPLE table should be preserved"
    );
    let mut core_header_framebuffer = VirtualFramebuffer::new(4, 2, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut core_header_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 1, 0, core_header_icon, 2, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let core_header_left = PixelFormat::Rgb565.bytes_per_pixel();
    let core_header_right = core_header_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &core_header_framebuffer.pixels()[core_header_left..core_header_left + 2],
        &[0xe0, 0x07],
        "BITMAPCOREHEADER RT_ICON draw should decode the high-bit palette index"
    );
    assert_eq!(
        &core_header_framebuffer.pixels()[core_header_right..core_header_right + 2],
        &[0x00, 0xf8],
        "BITMAPCOREHEADER RT_ICON draw should decode the following zero palette bit"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\two-bpp-icon.exe");
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
    let two_bpp_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(two_bpp_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, two_bpp_icon);
    let two_bpp_color_bitmap = kernel
        .resources
        .icon(two_bpp_icon)
        .expect("2bpp PE icon")
        .color_bitmap;
    let two_bpp_bitmap = kernel
        .resources
        .bitmap(two_bpp_color_bitmap)
        .expect("2bpp PE icon bitmap");
    assert_eq!(two_bpp_bitmap.bits_pixel, 2);
    assert_eq!(two_bpp_bitmap.color_table.len(), 4);
    assert_eq!(
        two_bpp_bitmap.color_table[2],
        [0x00, 0x00, 0xff, 0x00],
        "2bpp RT_ICON palette entry two should be preserved"
    );
    let mut two_bpp_framebuffer = VirtualFramebuffer::new(4, 3, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut two_bpp_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 1, 1, two_bpp_icon, 2, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let two_bpp_left = two_bpp_framebuffer.stride() + PixelFormat::Rgb565.bytes_per_pixel();
    let two_bpp_right = two_bpp_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &two_bpp_framebuffer.pixels()[two_bpp_left..two_bpp_left + 2],
        &[0x00, 0xf8],
        "2bpp RT_ICON draw should decode the high two-bit palette index"
    );
    assert_eq!(
        &two_bpp_framebuffer.pixels()[two_bpp_right..two_bpp_right + 2],
        &[0xe0, 0x07],
        "2bpp RT_ICON draw should decode the following two-bit palette index"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\four-bpp-icon.exe");
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
    let four_bpp_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(four_bpp_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, four_bpp_icon);
    let four_bpp_color_bitmap = kernel
        .resources
        .icon(four_bpp_icon)
        .expect("4bpp PE icon")
        .color_bitmap;
    let four_bpp_bitmap = kernel
        .resources
        .bitmap(four_bpp_color_bitmap)
        .expect("4bpp PE icon bitmap");
    assert_eq!(four_bpp_bitmap.bits_pixel, 4);
    assert_eq!(four_bpp_bitmap.color_table.len(), 16);
    assert_eq!(
        four_bpp_bitmap.color_table[9],
        [0x09, 0x39, 0x99, 0x00],
        "4bpp RT_ICON color table should be preserved on the extracted bitmap"
    );
    let mut four_bpp_framebuffer = VirtualFramebuffer::new(4, 4, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut four_bpp_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 1, 1, four_bpp_icon, 2, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let four_bpp_left = 1 * four_bpp_framebuffer.stride() + PixelFormat::Rgb565.bytes_per_pixel();
    let four_bpp_right = four_bpp_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &four_bpp_framebuffer.pixels()[four_bpp_left..four_bpp_left + 2],
        &[0xc1, 0x99],
        "4bpp RT_ICON draw should decode the high-nibble palette index"
    );
    assert_eq!(
        &four_bpp_framebuffer.pixels()[four_bpp_right..four_bpp_right + 2],
        &[0xa0, 0x91],
        "4bpp RT_ICON draw should decode the low-nibble palette index"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\twentyfour-bpp-icon.exe");
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
    let twentyfour_bpp_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(twentyfour_bpp_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, twentyfour_bpp_icon);
    let twentyfour_bpp_icon_obj = kernel
        .resources
        .icon(twentyfour_bpp_icon)
        .expect("24bpp PE icon");
    let twentyfour_bpp_bitmap = kernel
        .resources
        .bitmap(twentyfour_bpp_icon_obj.color_bitmap)
        .expect("24bpp PE icon bitmap");
    assert_eq!(twentyfour_bpp_bitmap.bits_pixel, 24);
    assert_eq!(
        (twentyfour_bpp_bitmap.width, twentyfour_bpp_bitmap.height),
        (2, 1)
    );
    assert_ne!(
        twentyfour_bpp_icon_obj.mask_bitmap, 0,
        "24bpp RT_ICON with trailing AND-mask bytes should create a mask bitmap"
    );
    let mut twentyfour_bpp_framebuffer = VirtualFramebuffer::new(4, 2, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut twentyfour_bpp_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 1, 0, twentyfour_bpp_icon, 2, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let twentyfour_left = PixelFormat::Rgb565.bytes_per_pixel();
    let twentyfour_right = twentyfour_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &twentyfour_bpp_framebuffer.pixels()[twentyfour_left..twentyfour_left + 2],
        &[0x00, 0xf8],
        "24bpp RT_ICON draw should decode the first padded BGR source pixel"
    );
    assert_eq!(
        &twentyfour_bpp_framebuffer.pixels()[twentyfour_right..twentyfour_right + 2],
        &[0xe0, 0x07],
        "24bpp RT_ICON draw should decode the second padded BGR source pixel"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\thirtytwo-bpp-icon.exe");
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
    let thirtytwo_bpp_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(thirtytwo_bpp_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, thirtytwo_bpp_icon);
    let thirtytwo_bpp_icon_obj = kernel
        .resources
        .icon(thirtytwo_bpp_icon)
        .expect("32bpp PE icon");
    let thirtytwo_bpp_bitmap = kernel
        .resources
        .bitmap(thirtytwo_bpp_icon_obj.color_bitmap)
        .expect("32bpp PE icon bitmap");
    assert_eq!(thirtytwo_bpp_bitmap.bits_pixel, 32);
    assert_eq!(
        (thirtytwo_bpp_bitmap.width, thirtytwo_bpp_bitmap.height),
        (2, 1)
    );
    assert_ne!(
        thirtytwo_bpp_icon_obj.mask_bitmap, 0,
        "32bpp RT_ICON with trailing AND-mask bytes should create a mask bitmap"
    );
    let mut thirtytwo_bpp_framebuffer = VirtualFramebuffer::new(4, 2, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut thirtytwo_bpp_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 1, 0, thirtytwo_bpp_icon, 2, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        &thirtytwo_bpp_framebuffer.pixels()[twentyfour_left..twentyfour_left + 2],
        &[0x82, 0xc9],
        "32bpp RT_ICON draw should decode the first BGRA source pixel"
    );
    assert_eq!(
        &thirtytwo_bpp_framebuffer.pixels()[twentyfour_right..twentyfour_right + 2],
        &[0x82, 0xc9],
        "32bpp RT_ICON draw should decode the following BGRA source pixel"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\masked-twentyfour-bpp-icon.exe");
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
    let masked_twentyfour_bpp_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(masked_twentyfour_bpp_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, masked_twentyfour_bpp_icon);
    let masked_twentyfour_bpp_icon_obj = kernel
        .resources
        .icon(masked_twentyfour_bpp_icon)
        .expect("masked 24bpp PE icon");
    assert_ne!(
        masked_twentyfour_bpp_icon_obj.mask_bitmap, 0,
        "masked 24bpp RT_ICON should create a real AND-mask bitmap"
    );
    let masked_twentyfour_mask = kernel
        .resources
        .bitmap(masked_twentyfour_bpp_icon_obj.mask_bitmap)
        .expect("masked 24bpp PE icon mask bitmap");
    assert_eq!(masked_twentyfour_mask.bits_pixel, 1);
    assert_eq!(memory.read_u8(masked_twentyfour_mask.bits_ptr)?, 0x40);
    let mut masked_twentyfour_bpp_framebuffer = VirtualFramebuffer::new(4, 2, PixelFormat::Rgb565)?;
    for pixel in masked_twentyfour_bpp_framebuffer
        .pixels_mut()
        .chunks_mut(PixelFormat::Rgb565.bytes_per_pixel())
    {
        pixel.copy_from_slice(&[0x1f, 0x00]);
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut masked_twentyfour_bpp_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 1, 0, masked_twentyfour_bpp_icon, 2, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        &masked_twentyfour_bpp_framebuffer.pixels()[twentyfour_left..twentyfour_left + 2],
        &[0x00, 0xf8],
        "24bpp RT_ICON normal draw should paint an unmasked color pixel"
    );
    assert_eq!(
        &masked_twentyfour_bpp_framebuffer.pixels()[twentyfour_right..twentyfour_right + 2],
        &[0x1f, 0x00],
        "24bpp RT_ICON normal draw should leave the destination behind an AND-mask pixel"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\sixteen-bpp-rgb-icon.exe");
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
    let sixteen_bpp_rgb_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(sixteen_bpp_rgb_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, sixteen_bpp_rgb_icon);
    let sixteen_bpp_rgb_color_bitmap = kernel
        .resources
        .icon(sixteen_bpp_rgb_icon)
        .expect("16bpp BI_RGB PE icon")
        .color_bitmap;
    let sixteen_bpp_rgb_bitmap = kernel
        .resources
        .bitmap(sixteen_bpp_rgb_color_bitmap)
        .expect("16bpp BI_RGB PE icon bitmap");
    assert_eq!(sixteen_bpp_rgb_bitmap.bits_pixel, 16);
    assert_eq!(
        sixteen_bpp_rgb_bitmap.rgb_masks,
        Some([0x0000_7c00, 0x0000_03e0, 0x0000_001f]),
        "CE treats 16bpp BI_RGB DIB pixels as RGB555"
    );
    let mut sixteen_bpp_rgb_framebuffer = VirtualFramebuffer::new(2, 2, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut sixteen_bpp_rgb_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 0, 0, sixteen_bpp_rgb_icon, 1, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let sixteen_bpp_rgb_pixel = u16::from_le_bytes(
        sixteen_bpp_rgb_framebuffer.pixels()[0..PixelFormat::Rgb565.bytes_per_pixel()]
            .try_into()
            .unwrap(),
    );
    assert_eq!(
        sixteen_bpp_rgb_pixel, 0xf800,
        "RGB555 BI_RGB red should render as full red in the RGB565 framebuffer"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\bitfields-icon.exe");
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
    let bitfields_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(bitfields_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, bitfields_icon);
    let bitfields_color_bitmap = kernel
        .resources
        .icon(bitfields_icon)
        .expect("16bpp bitfields PE icon")
        .color_bitmap;
    let bitfields_bitmap = kernel
        .resources
        .bitmap(bitfields_color_bitmap)
        .expect("16bpp bitfields PE icon bitmap");
    assert_eq!(bitfields_bitmap.bits_pixel, 16);
    let bitfields_color_bits_ptr = bitfields_bitmap.bits_ptr;
    assert_eq!(
        bitfields_bitmap.rgb_masks,
        Some([0x0000_7c00, 0x0000_03e0, 0x0000_001f]),
        "BI_BITFIELDS RT_ICON masks should survive extraction"
    );
    let bitfields_mask_bitmap = kernel
        .resources
        .icon(bitfields_icon)
        .expect("16bpp bitfields PE icon")
        .mask_bitmap;
    assert_ne!(
        bitfields_mask_bitmap, 0,
        "BI_BITFIELDS RT_ICON with a trailing AND mask should create a mask bitmap"
    );
    let bitfields_mask_bits_ptr = kernel
        .resources
        .bitmap(bitfields_mask_bitmap)
        .expect("16bpp bitfields PE icon mask bitmap")
        .bits_ptr;
    let mut bitfields_framebuffer = VirtualFramebuffer::new(2, 2, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut bitfields_framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 0, 0, bitfields_icon, 1, 1, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let bitfields_pixel = u16::from_le_bytes(
        bitfields_framebuffer.pixels()[0..PixelFormat::Rgb565.bytes_per_pixel()]
            .try_into()
            .unwrap(),
    );
    assert_eq!(
        bitfields_pixel, 0xf800,
        "RGB555 bitfield red should render as full red in the RGB565 framebuffer"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [bitfields_icon],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.icon(bitfields_icon).is_none());
    assert!(kernel.resources.bitmap(bitfields_color_bitmap).is_none());
    assert!(kernel.resources.bitmap(bitfields_mask_bitmap).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, bitfields_color_bits_ptr)
            .is_none(),
        "DestroyIcon should free PE-extracted icon color bitmap storage"
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, bitfields_mask_bits_ptr)
            .is_none(),
        "DestroyIcon should free PE-extracted icon mask bitmap storage"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\named-group-icon.exe");
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
    let named_large_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(named_large_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, named_large_icon);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\sparse-group-icon.exe");
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
            [path_ptr, 7, large_icon_ptr, small_icon_ptr, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let sparse_large_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(
        sparse_large_icon, 0,
        "integer RT_GROUP_ICON resource IDs should be usable as ExtractIconExW indexes"
    );
    assert_eq!(memory.read_u32(small_icon_ptr)?, sparse_large_icon);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    for malformed_icon_path in [
        r"\Docs\bad-group-icon.exe",
        r"\Docs\missing-icon-resource.exe",
        r"\Docs\missing-small-icon-resource.exe",
    ] {
        memory.write_wide_z(path_ptr, malformed_icon_path);
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
        memory.write_u32(large_icon_ptr, 0xdead_beef)?;
        memory.write_u32(small_icon_ptr, 0xfeed_face)?;
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_EXTRACT_ICON_EX_W,
                [path_ptr, 0, large_icon_ptr, small_icon_ptr, 1],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ));
        assert_eq!(
            kernel.threads.get_last_error(thread_id),
            ERROR_RESOURCE_NAME_NOT_FOUND
        );
        assert_eq!(memory.read_u32(large_icon_ptr)?, 0xdead_beef);
        assert_eq!(memory.read_u32(small_icon_ptr)?, 0xfeed_face);
    }

    memory.write_wide_z(path_ptr, r"\Docs\missing-and-mask-icon.exe");
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
    let missing_mask_icon = memory.read_u32(large_icon_ptr)?;
    assert_ne!(missing_mask_icon, 0);
    assert_eq!(memory.read_u32(small_icon_ptr)?, missing_mask_icon);
    let missing_mask_icon_obj = kernel
        .resources
        .icon(missing_mask_icon)
        .expect("missing-AND-mask PE icon");
    assert_ne!(missing_mask_icon_obj.color_bitmap, 0);
    assert_eq!(
        missing_mask_icon_obj.mask_bitmap, 0,
        "RT_ICON without trailing AND-mask bytes should still extract the color bitmap"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\Docs\viewer.lnk");
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
    assert_eq!(memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?, 2);
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        shell_pseudo_icon_handle_with_overlay(2, 1)
    );
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET)?,
        SFGAO_FILESYSTEM | SFGAO_LINK
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
        SFGAO_FILESYSTEM | SFGAO_FOLDER
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

    memory.write_wide_z(path_ptr, r"\Docs\readonly.txt");
    memory.write_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET, 0)?;
    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [
            path_ptr,
            FILE_ATTRIBUTE_ARCHIVE | FILE_ATTRIBUTE_READONLY,
            info_ptr,
            SHFILEINFO_SIZE_W,
            SHGFI_ATTRIBUTES,
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
        SFGAO_FILESYSTEM | SFGAO_READONLY
    );

    memory.write_wide_z(path_ptr, r"\Docs\morning.nav");
    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0x1234_5678)?;
    memory.write_u32(info_ptr + SHFILEINFO_IICON_OFFSET, 0x9abc_def0)?;
    memory.write_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET, 0x0bad_f00d)?;
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
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?,
        0x1234_5678,
        "CE leaves hIcon untouched when SHGFI_ICON is absent"
    );
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_IICON_OFFSET)?,
        0x9abc_def0,
        "CE leaves iIcon untouched when no icon/index flag is requested"
    );
    assert_eq!(
        memory.read_u32(info_ptr + SHFILEINFO_ATTRIBUTES_OFFSET)?,
        0x0bad_f00d,
        "CE leaves dwAttributes untouched when SHGFI_ATTRIBUTES is absent"
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_DISPLAY_NAME_OFFSET, 260),
        "morning.nav"
    );
    assert_eq!(
        memory.read_wide_z(info_ptr + SHFILEINFO_TYPE_NAME_OFFSET, 80),
        "Route Plan"
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

#[test]
fn coredll_raw_load_cursor_w_resolves_resource_cursor_payload_like_ce() -> Result<()> {
    const RT_CURSOR: u32 = 1;
    const RT_GROUP_CURSOR: u32 = 12;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;
    let module = 0x0040_0000;

    let cursor_dib = icon_dib_32bpp(16, 16, 0x40);
    let cursor_payload = rt_cursor_resource(4, 5, &cursor_dib);
    let group = rt_group_cursor_resource(&[(16, 16, 4, 5, cursor_payload.len() as u32, 201)]);
    let group_ptr = 0x2_8000;
    let cursor_ptr = 0x2_8100;
    let icon_info_ptr = 0x2_9000;
    memory.map_bytes(group_ptr, group.len() as u32);
    memory.map_bytes(cursor_ptr, cursor_payload.len() as u32);
    memory.map_words(icon_info_ptr, 5);
    memory.write_bytes(group_ptr, &group);
    memory.write_bytes(cursor_ptr, &cursor_payload);
    let group_handle = kernel.resources.register(
        module,
        ResourceId::Integer(88),
        ResourceId::Integer(RT_GROUP_CURSOR as u16),
        group_ptr,
        group.len() as u32,
    );
    kernel.resources.register(
        module,
        ResourceId::Integer(201),
        ResourceId::Integer(RT_CURSOR as u16),
        cursor_ptr,
        cursor_payload.len() as u32,
    );

    let cursor = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_CURSOR_W,
        [module, 88],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(cursor),
            ..
        } => cursor,
        other => panic!("LoadCursorW(resource cursor) returned unexpected result: {other:?}"),
    };
    assert_ne!(cursor, 0);
    assert_ne!(
        cursor, group_handle,
        "CE LoadCursorW resolves RT_GROUP_CURSOR to an RT_CURSOR payload"
    );
    let cursor_object = kernel
        .resources
        .icon(cursor)
        .expect("resource cursor should be an icon-pool object");
    assert!(!cursor_object.is_icon);
    assert_eq!((cursor_object.x_hotspot, cursor_object.y_hotspot), (4, 5));
    let color_bitmap = kernel
        .resources
        .bitmap(cursor_object.color_bitmap)
        .expect("resource cursor color bitmap");
    assert_eq!((color_bitmap.width, color_bitmap.height), (16, 16));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ICON_INFO,
            [cursor, icon_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(icon_info_ptr)?, 0);
    assert_eq!(memory.read_u32(icon_info_ptr + 4)?, 4);
    assert_eq!(memory.read_u32(icon_info_ptr + 8)?, 5);
    let mask_bitmap = memory.read_u32(icon_info_ptr + 12)?;
    let color_bitmap = memory.read_u32(icon_info_ptr + 16)?;
    assert_ne!(mask_bitmap, 0);
    assert_ne!(color_bitmap, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_CURSOR,
            [cursor],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.icon(cursor).is_none());
    assert!(kernel.resources.bitmap(mask_bitmap).is_none());
    assert!(kernel.resources.bitmap(color_bitmap).is_none());
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_load_image_w_selects_requested_icon_resource_size_like_ce() -> Result<()> {
    const IMAGE_ICON: u32 = 1;
    const RT_ICON: u32 = 3;
    const RT_GROUP_ICON: u32 = 14;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;
    let module = 0x0040_0000;

    let icon_16 = icon_dib_32bpp(16, 16, 0x30);
    let icon_32 = icon_dib_32bpp(32, 32, 0x80);
    let group = rt_group_icon_resource(&[
        (16, 16, 32, icon_16.len() as u32, 101),
        (32, 32, 32, icon_32.len() as u32, 102),
    ]);
    let group_ptr = 0x2_0000;
    let icon_16_ptr = 0x2_1000;
    let icon_32_ptr = 0x2_3000;
    memory.map_bytes(group_ptr, group.len() as u32);
    memory.map_bytes(icon_16_ptr, icon_16.len() as u32);
    memory.map_bytes(icon_32_ptr, icon_32.len() as u32);
    memory.write_bytes(group_ptr, &group);
    memory.write_bytes(icon_16_ptr, &icon_16);
    memory.write_bytes(icon_32_ptr, &icon_32);
    let group_handle = kernel.resources.register(
        module,
        ResourceId::Integer(77),
        ResourceId::Integer(RT_GROUP_ICON as u16),
        group_ptr,
        group.len() as u32,
    );
    kernel.resources.register(
        module,
        ResourceId::Integer(101),
        ResourceId::Integer(RT_ICON as u16),
        icon_16_ptr,
        icon_16.len() as u32,
    );
    kernel.resources.register(
        module,
        ResourceId::Integer(102),
        ResourceId::Integer(RT_ICON as u16),
        icon_32_ptr,
        icon_32.len() as u32,
    );

    let small_icon = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [module, 77, IMAGE_ICON, 16, 16, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(icon),
            ..
        } => icon,
        other => panic!("LoadImageW(16x16 IMAGE_ICON) returned unexpected result: {other:?}"),
    };
    assert_ne!(small_icon, 0);
    assert_ne!(
        small_icon, group_handle,
        "CE LoadImageW resolves an RT_GROUP_ICON entry instead of returning the group handle"
    );
    let small_bitmap = kernel
        .resources
        .icon(small_icon)
        .and_then(|icon| kernel.resources.bitmap(icon.color_bitmap))
        .expect("16x16 LoadImageW icon should be bitmap-backed");
    assert_eq!((small_bitmap.width, small_bitmap.height), (16, 16));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    let large_icon = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [module, 77, IMAGE_ICON, 32, 32, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(icon),
            ..
        } => icon,
        other => panic!("LoadImageW(32x32 IMAGE_ICON) returned unexpected result: {other:?}"),
    };
    assert_ne!(large_icon, 0);
    assert_ne!(large_icon, small_icon);
    let large_bitmap = kernel
        .resources
        .icon(large_icon)
        .and_then(|icon| kernel.resources.bitmap(icon.color_bitmap))
        .expect("32x32 LoadImageW icon should be bitmap-backed");
    assert_eq!((large_bitmap.width, large_bitmap.height), (32, 32));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    let missing_group = rt_group_icon_resource(&[(16, 16, 32, icon_16.len() as u32, 999)]);
    let missing_group_ptr = 0x2_7000;
    memory.map_bytes(missing_group_ptr, missing_group.len() as u32);
    memory.write_bytes(missing_group_ptr, &missing_group);
    kernel.resources.register(
        module,
        ResourceId::Integer(78),
        ResourceId::Integer(RT_GROUP_ICON as u16),
        missing_group_ptr,
        missing_group.len() as u32,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_IMAGE_W,
            [module, 78, IMAGE_ICON, 16, 16, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_RESOURCE_NAME_NOT_FOUND
    );

    Ok(())
}

#[test]
fn coredll_raw_kern_extract_icons_copies_group_rt_icon_payloads() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("kern_extract_icons");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Docs")).unwrap();
    fs::write(
        root.join("Docs").join("multi-size-icons.exe"),
        pe32_with_multi_size_icon_group(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("sparse-group-icon.exe"),
        pe32_with_sparse_multi_size_icon_group_id(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("missing-first-kern-icon.exe"),
        pe32_with_missing_small_icon_resource(),
    )
    .unwrap();
    fs::write(
        root.join("Docs").join("missing-second-kern-icon.exe"),
        pe32_with_missing_second_icon_resource(),
    )
    .unwrap();
    let sparse_bytes = fs::read(root.join("Docs").join("sparse-group-icon.exe")).unwrap();
    let sparse_pe =
        wince_emulation_v3::pe::PeImage::parse_bytes("sparse-group-icon.exe", &sparse_bytes)?;
    let sparse_group_ids = sparse_pe
        .resource_data_entries()?
        .into_iter()
        .filter(|resource| resource.kind == 14)
        .map(|resource| resource.name)
        .collect::<Vec<_>>();
    assert_eq!(sparse_group_ids, vec![7]);
    kernel.set_file_root(&root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let path_ptr = 0x2_4000;
    let large_out = 0x2_5000;
    let small_out = 0x2_5004;
    memory.map_halfwords(path_ptr, 64);
    memory.map_words(large_out, 2);
    memory.write_wide_z(path_ptr, r"\Docs\multi-size-icons.exe");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 1, large_out, small_out, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let large_ptr = memory.read_u32(large_out)?;
    let small_ptr = memory.read_u32(small_out)?;
    assert_ne!(large_ptr, 0);
    assert_ne!(small_ptr, 0);
    assert_ne!(large_ptr, small_ptr);

    let large_header = memory.read_bytes(large_ptr, 16);
    let small_header = memory.read_bytes(small_ptr, 16);
    assert_eq!(
        u32::from_le_bytes(large_header[0..4].try_into().unwrap()),
        40
    );
    assert_eq!(
        i32::from_le_bytes(large_header[4..8].try_into().unwrap()),
        32
    );
    assert_eq!(
        u32::from_le_bytes(large_header[8..12].try_into().unwrap()),
        64,
        "RT_ICON payload stores XOR and AND mask heights stacked"
    );
    assert_eq!(
        i32::from_le_bytes(small_header[4..8].try_into().unwrap()),
        16
    );
    assert_eq!(
        u32::from_le_bytes(small_header[8..12].try_into().unwrap()),
        32
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_RESOURCE_NAME_NOT_FOUND
    );

    memory.write_wide_z(path_ptr, r"\Docs\sparse-group-icon.exe");
    memory.write_word(large_out, 0xdead_beef);
    memory.write_word(small_out, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 0, large_out, small_out, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_RESOURCE_NAME_NOT_FOUND
    );
    assert_eq!(memory.read_u32(large_out)?, 0xdead_beef);
    assert_eq!(memory.read_u32(small_out)?, 0xfeed_face);

    let sparse_result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_KERN_EXTRACT_ICONS,
        [path_ptr, 7, large_out, small_out, 0],
    );
    assert!(
        matches!(
            sparse_result,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(2),
                ..
            }
        ),
        "unexpected sparse KernExtractIcons result: {sparse_result:?}, last_error={}",
        kernel.threads.get_last_error(thread_id)
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_ne!(memory.read_u32(large_out)?, 0xdead_beef);
    assert_ne!(memory.read_u32(small_out)?, 0xfeed_face);

    memory.write_wide_z(path_ptr, r"\Docs\missing-first-kern-icon.exe");
    memory.write_u32(small_out, 0xfeed_face)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 1, 0, small_out, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_RESOURCE_NAME_NOT_FOUND
    );
    assert_eq!(
        memory.read_u32(small_out)?,
        0,
        "small-only KernExtractIcons should fail when CE's selected small RT_ICON is missing"
    );

    memory.write_u32(large_out, 0xdead_beef)?;
    memory.write_u32(small_out, 0xfeed_face)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 1, large_out, small_out, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_RESOURCE_NAME_NOT_FOUND,
        "CE leaves the resource error visible when one requested icon fails"
    );
    assert_ne!(
        memory.read_u32(large_out)?,
        0xdead_beef,
        "large output should still extract CE's selected 32x32 RT_ICON"
    );
    assert_eq!(
        memory.read_u32(small_out)?,
        0,
        "CE assigns NULL to the failed requested small icon pointer"
    );

    memory.write_wide_z(path_ptr, r"\Docs\missing-second-kern-icon.exe");
    memory.write_u32(large_out, 0xdead_beef)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 1, large_out, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_RESOURCE_NAME_NOT_FOUND
    );
    assert_eq!(
        memory.read_u32(large_out)?,
        0,
        "large-only KernExtractIcons should fail when CE's selected large RT_ICON is missing"
    );

    memory.write_u32(small_out, 0xfeed_face)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 1, 0, small_out, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let small_only_ptr = memory.read_u32(small_out)?;
    assert_ne!(
        small_only_ptr, 0xfeed_face,
        "small-only KernExtractIcons should not require CE's selected large RT_ICON"
    );
    let small_only_header = memory.read_bytes(small_only_ptr, 16);
    assert_eq!(
        i32::from_le_bytes(small_only_header[4..8].try_into().unwrap()),
        16
    );

    memory.write_wide_z(path_ptr, r"\Docs\multi-size-icons.exe");
    memory.write_word(large_out, 0xdead_beef);
    memory.write_word(small_out, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KERN_EXTRACT_ICONS,
            [path_ptr, 99, large_out, small_out, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_RESOURCE_NAME_NOT_FOUND
    );
    assert_eq!(memory.read_u32(large_out)?, 0xdead_beef);
    assert_eq!(memory.read_u32(small_out)?, 0xfeed_face);

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
    let overlay = if (1..=4).contains(&overlay) {
        overlay
    } else {
        0
    };
    0x000b_8000 | stock | ((overlay & 0x0f) << 24)
}

fn pe32_with_group_icon_count(group_count: u16) -> Vec<u8> {
    let groups: Vec<_> = (0..group_count)
        .map(|index| TestIconGroup {
            images: vec![TestIconImage {
                width: 1,
                height: 1,
                channel: 0x30u8.saturating_add((index as u8).saturating_mul(0x40)),
            }],
        })
        .collect();
    pe32_with_icon_groups(&groups)
}

fn pe32_with_multi_size_icon_group() -> Vec<u8> {
    pe32_with_icon_groups(&[TestIconGroup {
        images: vec![
            TestIconImage {
                width: 16,
                height: 16,
                channel: 0x30,
            },
            TestIconImage {
                width: 32,
                height: 32,
                channel: 0x80,
            },
        ],
    }])
}

fn pe32_with_string_named_icon_group() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let rsrc = 0x200usize;
    let group_type_dir = rsrc + 0x60;
    let group_name_string_offset = 0x180usize;
    put_test_u16(&mut bytes, group_type_dir + 12, 1);
    put_test_u16(&mut bytes, group_type_dir + 14, 0);
    put_test_u32(
        &mut bytes,
        group_type_dir + 16,
        0x8000_0000 | group_name_string_offset as u32,
    );
    put_test_resource_name_string(&mut bytes, rsrc + group_name_string_offset, "APPICON");
    bytes
}

fn pe32_with_sparse_group_icon_id() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let rsrc = 0x200usize;
    let group_type_dir = rsrc + 0x60;
    put_test_u32(&mut bytes, group_type_dir + 16, 7);
    bytes
}

fn pe32_with_sparse_multi_size_icon_group_id() -> Vec<u8> {
    let mut bytes = pe32_with_multi_size_icon_group();
    let rsrc = 0x200usize;
    let group_type_dir = rsrc + 0x60;
    put_test_u32(&mut bytes, group_type_dir + 16, 7);
    bytes
}

fn rt_group_icon_resource(entries: &[(u8, u8, u16, u32, u16)]) -> Vec<u8> {
    let mut bytes = vec![0; 6 + entries.len() * 14];
    put_test_u16(&mut bytes, 0, 0);
    put_test_u16(&mut bytes, 2, 1);
    put_test_u16(&mut bytes, 4, entries.len() as u16);
    for (index, (width, height, bit_count, bytes_in_resource, id)) in
        entries.iter().copied().enumerate()
    {
        let entry = 6 + index * 14;
        bytes[entry] = width;
        bytes[entry + 1] = height;
        bytes[entry + 2] = 0;
        bytes[entry + 3] = 0;
        put_test_u16(&mut bytes, entry + 4, 1);
        put_test_u16(&mut bytes, entry + 6, bit_count);
        put_test_u32(&mut bytes, entry + 8, bytes_in_resource);
        put_test_u16(&mut bytes, entry + 12, id);
    }
    bytes
}

fn rt_group_cursor_resource(entries: &[(u8, u8, u16, u16, u32, u16)]) -> Vec<u8> {
    let mut bytes = vec![0; 6 + entries.len() * 14];
    put_test_u16(&mut bytes, 0, 0);
    put_test_u16(&mut bytes, 2, 2);
    put_test_u16(&mut bytes, 4, entries.len() as u16);
    for (index, (width, height, hotspot_x, hotspot_y, bytes_in_resource, id)) in
        entries.iter().copied().enumerate()
    {
        let entry = 6 + index * 14;
        bytes[entry] = width;
        bytes[entry + 1] = height;
        bytes[entry + 2] = 0;
        bytes[entry + 3] = 0;
        put_test_u16(&mut bytes, entry + 4, hotspot_x);
        put_test_u16(&mut bytes, entry + 6, hotspot_y);
        put_test_u32(&mut bytes, entry + 8, bytes_in_resource);
        put_test_u16(&mut bytes, entry + 12, id);
    }
    bytes
}

fn rt_cursor_resource(hotspot_x: u16, hotspot_y: u16, dib: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(4 + dib.len());
    bytes.extend_from_slice(&hotspot_x.to_le_bytes());
    bytes.extend_from_slice(&hotspot_y.to_le_bytes());
    bytes.extend_from_slice(dib);
    bytes
}

fn pe32_with_malformed_icon_group_type() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 2, 2);
    bytes
}

fn pe32_with_missing_icon_resource() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 18, 999);
    bytes
}

fn pe32_with_missing_small_icon_resource() -> Vec<u8> {
    let mut bytes = pe32_with_multi_size_icon_group();
    put_test_u16(&mut bytes, 0x200 + 0x500 + 18, 999);
    bytes
}

fn pe32_with_missing_second_icon_resource() -> Vec<u8> {
    let mut bytes = pe32_with_multi_size_icon_group();
    put_test_u16(&mut bytes, 0x200 + 0x500 + 32, 999);
    bytes
}

fn pe32_with_version_info_resource(signature: u32) -> Vec<u8> {
    let resource_size = 0x1000u32;
    let mut bytes = vec![0u8; 0x200 + resource_size as usize];
    put_test_bytes(&mut bytes, 0, b"MZ");
    put_test_u32(&mut bytes, 0x3c, 0x80);
    put_test_bytes(&mut bytes, 0x80, b"PE\0\0");

    let coff = 0x84;
    put_test_u16(&mut bytes, coff, 0x0166);
    put_test_u16(&mut bytes, coff + 2, 1);
    put_test_u16(&mut bytes, coff + 16, 0xe0);
    put_test_u16(&mut bytes, coff + 18, 0x0102);

    let optional = 0x98;
    put_test_u16(&mut bytes, optional, 0x010b);
    put_test_u32(&mut bytes, optional + 8, 0x200);
    put_test_u32(&mut bytes, optional + 24, 0x1000);
    put_test_u32(&mut bytes, optional + 28, 0x0040_0000);
    put_test_u32(&mut bytes, optional + 32, 0x1000);
    put_test_u32(&mut bytes, optional + 36, 0x200);
    put_test_u32(&mut bytes, optional + 56, 0x1000 + resource_size);
    put_test_u32(&mut bytes, optional + 60, 0x200);
    put_test_u16(&mut bytes, optional + 68, 2);
    put_test_u32(&mut bytes, optional + 72, 0x100000);
    put_test_u32(&mut bytes, optional + 76, 0x1000);
    put_test_u32(&mut bytes, optional + 80, 0x100000);
    put_test_u32(&mut bytes, optional + 84, 0x1000);
    put_test_u32(&mut bytes, optional + 92, 16);
    put_test_u32(&mut bytes, optional + 112, 0x1000);
    put_test_u32(&mut bytes, optional + 116, resource_size);

    let section = 0x178;
    put_test_bytes(&mut bytes, section, b".rsrc\0\0\0");
    put_test_u32(&mut bytes, section + 8, resource_size);
    put_test_u32(&mut bytes, section + 12, 0x1000);
    put_test_u32(&mut bytes, section + 16, resource_size);
    put_test_u32(&mut bytes, section + 20, 0x200);
    put_test_u32(&mut bytes, section + 36, 0x4000_0040);

    let rsrc = 0x200;
    let type_dir = rsrc + 0x20;
    let name_dir = rsrc + 0x40;
    let data_entry = rsrc + 0x60;
    let version_data_offset = 0x100usize;
    let version_info = version_info_resource_data(signature);
    put_test_resource_directory(&mut bytes, rsrc, 1);
    put_test_u32(&mut bytes, rsrc + 16, 16);
    put_test_u32(&mut bytes, rsrc + 20, 0x8000_0020);
    put_test_resource_directory(&mut bytes, type_dir, 1);
    put_test_u32(&mut bytes, type_dir + 16, 1);
    put_test_u32(&mut bytes, type_dir + 20, 0x8000_0040);
    put_test_resource_directory(&mut bytes, name_dir, 1);
    put_test_u32(&mut bytes, name_dir + 16, 0x0409);
    put_test_u32(&mut bytes, name_dir + 20, 0x60);
    put_test_u32(&mut bytes, data_entry, 0x1000 + version_data_offset as u32);
    put_test_u32(&mut bytes, data_entry + 4, version_info.len() as u32);
    put_test_bytes(&mut bytes, rsrc + version_data_offset, &version_info);
    bytes
}

fn version_info_resource_data(signature: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 92];
    put_test_u16(&mut bytes, 0, 92);
    put_test_u16(&mut bytes, 2, 52);
    put_test_u16(&mut bytes, 4, 0);
    for (index, unit) in "VS_VERSION_INFO"
        .encode_utf16()
        .chain(std::iter::once(0))
        .enumerate()
    {
        put_test_u16(&mut bytes, 6 + index * 2, unit);
    }
    put_test_u32(&mut bytes, 40, signature);
    put_test_u32(&mut bytes, 44, 0x0001_0000);
    put_test_u32(&mut bytes, 48, 0x0006_0001);
    put_test_u32(&mut bytes, 52, 0x0000_0002);
    bytes
}

fn pe32_with_missing_and_mask_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let truncated_icon_size = 44u32; // BITMAPINFOHEADER + one 32bpp XOR pixel, no AND mask.
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, truncated_icon_size);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, truncated_icon_size);
    bytes
}

fn pe32_with_8bpp_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_8bpp(1, 1, 7);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 8);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_1bpp_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_1bpp(2, 1, 0x80);
    put_test_u8(&mut bytes, 0x200 + 0x500 + 6, 2);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 1);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_core_header_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_core_1bpp(2, 1, 0x80);
    put_test_u8(&mut bytes, 0x200 + 0x500 + 6, 2);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 1);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_2bpp_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_2bpp(2, 1, 2, 1);
    put_test_u8(&mut bytes, 0x200 + 0x500 + 6, 2);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 2);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_4bpp_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_4bpp(2, 1, 9, 4);
    put_test_u8(&mut bytes, 0x200 + 0x500 + 6, 2);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 4);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_24bpp_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_24bpp(2, 1, &[[0x00, 0x00, 0xff], [0x00, 0xff, 0x00]], 0x00);
    put_test_u8(&mut bytes, 0x200 + 0x500 + 6, 2);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 24);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_32bpp_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_32bpp(2, 1, 0x10);
    put_test_u8(&mut bytes, 0x200 + 0x500 + 6, 2);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 32);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_24bpp_masked_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_24bpp(2, 1, &[[0x00, 0x00, 0xff], [0x00, 0xff, 0x00]], 0x40);
    put_test_u8(&mut bytes, 0x200 + 0x500 + 6, 2);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 24);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_16bpp_rgb_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_16bpp_rgb(1, 1, 0x7c00);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 16);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

fn pe32_with_16bpp_bitfields_icon() -> Vec<u8> {
    let mut bytes = pe32_with_group_icon_count(1);
    let icon_dib = icon_dib_16bpp_bitfields(1, 1, 0x7c00);
    put_test_u32(&mut bytes, 0x200 + 0x2a0 + 4, icon_dib.len() as u32);
    put_test_u16(&mut bytes, 0x200 + 0x500 + 12, 16);
    put_test_u32(&mut bytes, 0x200 + 0x500 + 14, icon_dib.len() as u32);
    put_test_bytes(&mut bytes, 0x200 + 0x700, &icon_dib);
    bytes
}

struct TestIconGroup {
    images: Vec<TestIconImage>,
}

#[derive(Clone, Copy)]
struct TestIconImage {
    width: u8,
    height: u8,
    channel: u8,
}

fn pe32_with_icon_groups(groups: &[TestIconGroup]) -> Vec<u8> {
    let resource_size = 0x4000u32;
    let mut bytes = vec![0u8; 0x200 + resource_size as usize];
    put_test_bytes(&mut bytes, 0, b"MZ");
    put_test_u32(&mut bytes, 0x3c, 0x80);
    put_test_bytes(&mut bytes, 0x80, b"PE\0\0");

    let coff = 0x84;
    put_test_u16(&mut bytes, coff, 0x0166);
    put_test_u16(&mut bytes, coff + 2, 1);
    put_test_u16(&mut bytes, coff + 16, 0xe0);
    put_test_u16(&mut bytes, coff + 18, 0x0102);

    let optional = 0x98;
    put_test_u16(&mut bytes, optional, 0x010b);
    put_test_u32(&mut bytes, optional + 8, 0x200);
    put_test_u32(&mut bytes, optional + 24, 0x1000);
    put_test_u32(&mut bytes, optional + 28, 0x0040_0000);
    put_test_u32(&mut bytes, optional + 32, 0x1000);
    put_test_u32(&mut bytes, optional + 36, 0x200);
    put_test_u32(&mut bytes, optional + 56, 0x1000 + resource_size);
    put_test_u32(&mut bytes, optional + 60, 0x200);
    put_test_u16(&mut bytes, optional + 68, 2);
    put_test_u32(&mut bytes, optional + 72, 0x100000);
    put_test_u32(&mut bytes, optional + 76, 0x1000);
    put_test_u32(&mut bytes, optional + 80, 0x100000);
    put_test_u32(&mut bytes, optional + 84, 0x1000);
    put_test_u32(&mut bytes, optional + 92, 16);
    put_test_u32(&mut bytes, optional + 112, 0x1000);
    put_test_u32(&mut bytes, optional + 116, resource_size);

    let section = 0x178;
    put_test_bytes(&mut bytes, section, b".rsrc\0\0\0");
    put_test_u32(&mut bytes, section + 8, resource_size);
    put_test_u32(&mut bytes, section + 12, 0x1000);
    put_test_u32(&mut bytes, section + 16, resource_size);
    put_test_u32(&mut bytes, section + 20, 0x200);
    put_test_u32(&mut bytes, section + 36, 0x4000_0040);

    let rsrc = 0x200;
    put_test_resource_directory(&mut bytes, rsrc, 2);
    put_test_u32(&mut bytes, rsrc + 16, 3);
    put_test_u32(&mut bytes, rsrc + 20, 0x8000_0020);
    put_test_u32(&mut bytes, rsrc + 24, 14);
    put_test_u32(&mut bytes, rsrc + 28, 0x8000_0060);

    let icon_type_dir = rsrc + 0x20;
    let group_type_dir = rsrc + 0x60;
    let icon_name_dir_base = 0x0a0usize;
    let group_name_dir_base = 0x1a0usize;
    let icon_data_entry_base = 0x2a0usize;
    let group_data_entry_base = 0x3a0usize;
    let group_data_base = 0x500usize;
    let icon_data_base = 0x700usize;
    let icon_count: usize = groups.iter().map(|group| group.images.len()).sum();

    put_test_resource_directory(&mut bytes, icon_type_dir, icon_count as u16);
    put_test_resource_directory(&mut bytes, group_type_dir, groups.len() as u16);
    let mut icon_index = 0usize;
    let mut group_icon_entries: Vec<Vec<(u16, TestIconImage, usize)>> = Vec::new();
    for group in groups {
        let mut current_group_entries = Vec::new();
        for image in &group.images {
            let icon_resource_id = 100 + icon_index as u32;
            let icon_name_dir_offset = icon_name_dir_base + icon_index * 0x20;
            put_test_u32(
                &mut bytes,
                icon_type_dir + 16 + icon_index * 8,
                icon_resource_id,
            );
            put_test_u32(
                &mut bytes,
                icon_type_dir + 20 + icon_index * 8,
                0x8000_0000 | icon_name_dir_offset as u32,
            );
            let icon_name_dir = rsrc + icon_name_dir_offset;
            put_test_resource_directory(&mut bytes, icon_name_dir, 1);
            let icon_data_entry_offset = icon_data_entry_base + icon_index * 16;
            put_test_u32(&mut bytes, icon_name_dir + 16, 0x0409);
            put_test_u32(
                &mut bytes,
                icon_name_dir + 20,
                icon_data_entry_offset as u32,
            );

            let icon_dib = icon_dib_32bpp(image.width as i32, image.height as i32, image.channel);
            let icon_data_offset = icon_data_base + icon_index * 0x500;
            let icon_data_entry = rsrc + icon_data_entry_offset;
            put_test_u32(
                &mut bytes,
                icon_data_entry,
                0x1000 + icon_data_offset as u32,
            );
            put_test_u32(&mut bytes, icon_data_entry + 4, icon_dib.len() as u32);
            put_test_bytes(&mut bytes, rsrc + icon_data_offset, &icon_dib);
            current_group_entries.push((icon_resource_id as u16, *image, icon_dib.len()));
            icon_index += 1;
        }
        group_icon_entries.push(current_group_entries);
    }

    for (index, group_entries) in group_icon_entries.iter().enumerate() {
        let group_name_dir_offset = group_name_dir_base + index * 0x20;
        put_test_u32(
            &mut bytes,
            group_type_dir + 16 + index * 8,
            (index + 1) as u32,
        );
        put_test_u32(
            &mut bytes,
            group_type_dir + 20 + index * 8,
            0x8000_0000 | group_name_dir_offset as u32,
        );
        let group_name_dir = rsrc + group_name_dir_offset;
        put_test_resource_directory(&mut bytes, group_name_dir, 1);
        let group_data_entry_offset = group_data_entry_base + index * 16;
        put_test_u32(&mut bytes, group_name_dir + 16, 0x0409);
        put_test_u32(
            &mut bytes,
            group_name_dir + 20,
            group_data_entry_offset as u32,
        );

        let group_data_offset = group_data_base + index * 0x20;
        let group_data_entry = rsrc + group_data_entry_offset;
        put_test_u32(
            &mut bytes,
            group_data_entry,
            0x1000 + group_data_offset as u32,
        );
        put_test_u32(
            &mut bytes,
            group_data_entry + 4,
            (6 + group_entries.len() * 14) as u32,
        );
        let group_data = rsrc + group_data_offset;
        put_test_u16(&mut bytes, group_data, 0);
        put_test_u16(&mut bytes, group_data + 2, 1);
        put_test_u16(&mut bytes, group_data + 4, group_entries.len() as u16);
        for (entry_index, (icon_resource_id, image, icon_dib_len)) in
            group_entries.iter().copied().enumerate()
        {
            let entry = group_data + 6 + entry_index * 14;
            bytes[entry] = image.width;
            bytes[entry + 1] = image.height;
            bytes[entry + 2] = 0;
            bytes[entry + 3] = 0;
            put_test_u16(&mut bytes, entry + 4, 1);
            put_test_u16(&mut bytes, entry + 6, 32);
            put_test_u32(&mut bytes, entry + 8, icon_dib_len as u32);
            put_test_u16(&mut bytes, entry + 12, icon_resource_id);
        }
    }

    bytes
}

fn icon_dib_32bpp(width: i32, height: i32, channel: u8) -> Vec<u8> {
    let xor_stride = (width as usize) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&32u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    for _ in 0..(width as usize * height as usize) {
        bytes.extend_from_slice(&[channel, channel.saturating_add(0x20), 0xcc, 0xff]);
    }
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn icon_dib_8bpp(width: i32, height: i32, color_index: u8) -> Vec<u8> {
    let xor_stride = (((width as usize * 8) + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    for index in 0..=u8::MAX {
        bytes.extend_from_slice(&[index, index.wrapping_add(0x40), index.wrapping_add(0x80), 0]);
    }
    bytes.push(color_index);
    bytes.resize(bytes.len() + xor_size.saturating_sub(1), 0);
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn icon_dib_1bpp(width: i32, height: i32, row_bits: u8) -> Vec<u8> {
    let xor_stride = ((width as usize + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0x00, 0x00, 0xff, 0x00]);
    bytes.extend_from_slice(&[0x00, 0xff, 0x00, 0x00]);
    bytes.push(row_bits);
    bytes.resize(bytes.len() + xor_size.saturating_sub(1), 0);
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn icon_dib_core_1bpp(width: u16, height: u16, row_bits: u8) -> Vec<u8> {
    let xor_stride = ((width as usize + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&12u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&[0x00, 0x00, 0xff]);
    bytes.extend_from_slice(&[0x00, 0xff, 0x00]);
    bytes.push(row_bits);
    bytes.resize(bytes.len() + xor_size.saturating_sub(1), 0);
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn icon_dib_2bpp(width: i32, height: i32, first_index: u8, second_index: u8) -> Vec<u8> {
    let xor_stride = (((width as usize * 2) + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    bytes.extend_from_slice(&[0x00, 0xff, 0x00, 0x00]);
    bytes.extend_from_slice(&[0x00, 0x00, 0xff, 0x00]);
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0x00]);
    bytes.push(((first_index & 0x03) << 6) | ((second_index & 0x03) << 4));
    bytes.resize(bytes.len() + xor_size.saturating_sub(1), 0);
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn icon_dib_4bpp(width: i32, height: i32, first_index: u8, second_index: u8) -> Vec<u8> {
    let xor_stride = (((width as usize * 4) + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    for index in 0..16u8 {
        bytes.extend_from_slice(&[index, index.wrapping_add(0x30), index.wrapping_add(0x90), 0]);
    }
    bytes.push(((first_index & 0x0f) << 4) | (second_index & 0x0f));
    bytes.resize(bytes.len() + xor_size.saturating_sub(1), 0);
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn icon_dib_24bpp(
    width: i32,
    height: i32,
    pixels_bgr: &[[u8; 3]],
    and_mask_first_byte: u8,
) -> Vec<u8> {
    let xor_stride = (((width as usize * 24) + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    for row in 0..height as usize {
        let row_start = row * width as usize;
        let row_end = row_start + width as usize;
        for pixel in &pixels_bgr[row_start..row_end] {
            bytes.extend_from_slice(pixel);
        }
        let row_bytes = (width as usize) * 3;
        bytes.resize(bytes.len() + xor_stride.saturating_sub(row_bytes), 0);
    }
    bytes.push(and_mask_first_byte);
    bytes.resize(bytes.len() + and_size.saturating_sub(1), 0);
    bytes
}

fn icon_dib_16bpp_rgb(width: i32, height: i32, pixel: u16) -> Vec<u8> {
    let xor_stride = (((width as usize * 16) + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&pixel.to_le_bytes());
    bytes.resize(bytes.len() + xor_size.saturating_sub(2), 0);
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn icon_dib_16bpp_bitfields(width: i32, height: i32, pixel: u16) -> Vec<u8> {
    let xor_stride = (((width as usize * 16) + 31) / 32) * 4;
    let xor_size = xor_stride * height as usize;
    let and_stride = ((width as usize + 31) / 32) * 4;
    let and_size = and_stride * height as usize;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&(height * 2).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&(xor_size as u32).to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0x0000_7c00u32.to_le_bytes());
    bytes.extend_from_slice(&0x0000_03e0u32.to_le_bytes());
    bytes.extend_from_slice(&0x0000_001fu32.to_le_bytes());
    bytes.extend_from_slice(&pixel.to_le_bytes());
    bytes.resize(bytes.len() + xor_size.saturating_sub(2), 0);
    bytes.resize(bytes.len() + and_size, 0);
    bytes
}

fn put_test_resource_directory(bytes: &mut [u8], offset: usize, id_entries: u16) {
    put_test_u16(bytes, offset + 14, id_entries);
}

fn put_test_resource_name_string(bytes: &mut [u8], offset: usize, value: &str) {
    let units: Vec<u16> = value.encode_utf16().collect();
    put_test_u16(bytes, offset, units.len() as u16);
    for (index, unit) in units.into_iter().enumerate() {
        put_test_u16(bytes, offset + 2 + index * 2, unit);
    }
}

fn put_test_bytes(bytes: &mut [u8], offset: usize, value: &[u8]) {
    bytes[offset..offset + value.len()].copy_from_slice(value);
}

fn put_test_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_test_u8(bytes: &mut [u8], offset: usize, value: u8) {
    bytes[offset] = value;
}

fn put_test_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_test_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_test_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
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

fn bmp_2x1_white_white_24bpp() -> Vec<u8> {
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
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00]);
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

fn bmp_2x2_solid_24bpp(bgr: [u8; 3]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&70u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54u32.to_le_bytes());
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    for _ in 0..2 {
        bytes.extend_from_slice(&bgr);
        bytes.extend_from_slice(&bgr);
        bytes.extend_from_slice(&[0x00, 0x00]);
    }
    bytes
}

fn bmp_2x2_diagonal_mask_24bpp() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&70u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&54u32.to_le_bytes());
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&2i32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&24u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00]);
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x00, 0x00]);
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
    const SHFILEINFO_HICON_OFFSET: u32 = 0;
    const SHFILEINFO_IICON_OFFSET: u32 = 4;
    const SHGFI_ICON: u32 = 0x0000_0100;
    const SHGFI_SYSICONINDEX: u32 = 0x0000_4000;
    const SHGFI_SMALLICON: u32 = 0x0000_0001;
    const SHGFI_USEFILEATTRIBUTES: u32 = 0x0000_0010;
    const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;
    const SHELL_SYSTEM_IMAGE_LIST_HANDLE: u32 = 0x000b_f000;
    const SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE: u32 = 0x000b_f100;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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

    let ret = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHGET_FILE_INFO,
        [path_ptr, 0, info_ptr, SHFILEINFO_SIZE_W, SHGFI_SYSICONINDEX],
    );
    assert!(
        matches!(
            ret,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE),
                ..
            }
        ),
        "CE SHGetFileInfo should return the large system image list without SHGFI_SMALLICON"
    );
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
            value: CoredllValue::U32(SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE),
            ..
        }
    ));
    let icon_index = memory.read_i32(info_ptr + SHFILEINFO_IICON_OFFSET)?;

    memory.write_u32(info_ptr + SHFILEINFO_HICON_OFFSET, 0)?;
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
            SHGFI_USEFILEATTRIBUTES | SHGFI_ICON,
        ],
    );
    assert!(matches!(
        ret,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE),
            ..
        }
    ));
    assert_ne!(memory.read_u32(info_ptr + SHFILEINFO_HICON_OFFSET)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_IMAGE_COUNT,
            [SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE],
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
            ORD_IMAGE_LIST_GET_ICON_SIZE,
            [SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE, size_ptr, size_ptr + 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(size_ptr)?, 32);
    assert_eq!(memory.read_i32(size_ptr + 4)?, 32);
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
            ORD_IMAGE_LIST_GET_ICON,
            [SHELL_SYSTEM_IMAGE_LIST_HANDLE, icon_index as u32, 0x0500],
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
            ORD_IMAGE_LIST_GET_IMAGE_INFO,
            [
                SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE,
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
    assert_eq!(memory.read_i32(image_info_ptr + 24)?, icon_index * 32 + 32);
    assert_eq!(memory.read_i32(image_info_ptr + 28)?, 32);
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
    let pixel = |framebuffer: &VirtualFramebuffer, x: usize, y: usize| {
        let offset = y * framebuffer.stride() + x * PixelFormat::Rgb565.bytes_per_pixel();
        [
            framebuffer.pixels()[offset],
            framebuffer.pixels()[offset + 1],
        ]
    };
    assert_eq!(pixel(&framebuffer, 0, 0), [0, 0]);
    assert_ne!(pixel(&framebuffer, 5, 9), [0, 0]);
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
                20,
                8,
                16,
                16,
                0,
                0,
                0x0500,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        pixel(&framebuffer, 32, 20),
        pixel(&framebuffer, 21, 9),
        "invalid CE overlay slots above four should not paint the synthetic overlay marker"
    );

    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 24, 24);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [
                SHELL_SYSTEM_IMAGE_LIST_HANDLE,
                icon_index as u32,
                mem_dc,
                2,
                2,
                16,
                16,
                0,
                0,
                0x0100,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 1, 1),
        0,
        "system pseudo-icon selected-DIB draw should leave pixels outside the target untouched"
    );
    let selected_dib_body = rgb565_at(&memory, bits_ptr, stride, 3, 3);
    let selected_dib_overlay = rgb565_at(&memory, bits_ptr, stride, 14, 14);
    assert_ne!(
        selected_dib_body, 0,
        "system pseudo-icon selected-DIB draw should paint the synthetic body"
    );
    assert_ne!(
        selected_dib_overlay, 0,
        "system pseudo-icon selected-DIB draw should paint the synthetic overlay marker"
    );
    assert_ne!(
        selected_dib_overlay, selected_dib_body,
        "system pseudo-icon selected-DIB overlay should be visibly distinct from the body"
    );
    for handle in [
        SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE,
        SHELL_SYSTEM_IMAGE_LIST_HANDLE,
    ] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_IMAGE_LIST_DESTROY,
                [handle],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
        assert_eq!(
            kernel.threads.get_last_error(thread_id),
            0,
            "destroying an SHGetFileInfo system image list should be a successful shell teardown"
        );
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON_SIZE,
            [SHELL_SYSTEM_LARGE_IMAGE_LIST_HANDLE, size_ptr, size_ptr + 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(size_ptr)?, 32);
    assert_eq!(memory.read_i32(size_ptr + 4)?, 32);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    Ok(())
}

#[test]
fn image_list_ordinals_track_created_lists_and_icons() -> Result<()> {
    const IMAGE_BITMAP: u32 = 0;
    const LR_LOADFROMFILE: u32 = 0x0000_0010;
    const CLR_NONE: u32 = 0xffff_ffff;
    const CLR_DEFAULT: u32 = 0xff00_0000;
    const ILC_MASK: u32 = 0x0001;
    const ILC_COLOR4: u32 = 0x0004;
    const ILC_COLOR8: u32 = 0x0008;
    const ILC_COLORDDB: u32 = 0x00fe;
    const ILC_COLOR24: u32 = 0x0018;
    const ILC_COLOR32: u32 = 0x0020;
    const ILC_SHARED: u32 = 0x0100;
    const ILC_LARGESMALL: u32 = 0x0200;
    const ILC_UNIQUE: u32 = 0x0400;
    const ILC_PALETTE: u32 = 0x0800;
    const ILC_MIRROR: u32 = 0x2000;
    const ILC_VIRTUAL: u32 = 0x8000;
    const ILC_WIN95: u32 = ILC_MASK | ILC_COLORDDB | ILC_SHARED | ILC_PALETTE;
    const ILD_MASK: u32 = 0x0010;
    const ILD_IMAGE: u32 = 0x0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    fs::write(
        root.join("Images").join("white-mask.bmp"),
        bmp_2x1_white_white_24bpp(),
    )
    .unwrap();
    fs::write(root.join("Images").join("red.bmp"), bmp_2x1_red_red_24bpp()).unwrap();
    fs::write(
        root.join("Images").join("red2.bmp"),
        bmp_2x2_solid_24bpp([0x00, 0x00, 0xff]),
    )
    .unwrap();
    fs::write(
        root.join("Images").join("green2.bmp"),
        bmp_2x2_solid_24bpp([0x00, 0xff, 0x00]),
    )
    .unwrap();
    fs::write(
        root.join("Images").join("diagonal-mask.bmp"),
        bmp_2x2_diagonal_mask_24bpp(),
    )
    .unwrap();
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

    for invalid_flags in [ILC_LARGESMALL, ILC_UNIQUE, 0x1000, 0x0001_0000] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_IMAGE_LIST_CREATE,
                [16, 16, invalid_flags, 1, 1],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(0),
                ..
            }
        ));
        assert_eq!(
            kernel.threads.get_last_error(thread_id),
            ERROR_INVALID_PARAMETER,
            "ImageList_Create should reject non-CE ILC flags {invalid_flags:#x}"
        );
    }

    let valid_private_flags =
        ILC_MASK | ILC_COLOR32 | ILC_SHARED | ILC_PALETTE | ILC_MIRROR | ILC_VIRTUAL;
    let valid_private_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, valid_private_flags, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE-valid private flags: {other:?}"),
    };
    assert_ne!(valid_private_list, 0);
    assert_eq!(
        kernel
            .resources
            .image_list(valid_private_list)
            .unwrap()
            .flags,
        valid_private_flags
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DESTROY,
            [valid_private_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let win95_compat_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, 0xffff_ffff, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE -1 compatibility flags: {other:?}"),
    };
    assert_ne!(win95_compat_list, 0);
    assert_eq!(
        kernel
            .resources
            .image_list(win95_compat_list)
            .unwrap()
            .flags,
        ILC_WIN95
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DESTROY,
            [win95_compat_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let default_color_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, 0, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE default color flags: {other:?}"),
    };
    assert_ne!(default_color_list, 0);
    assert_eq!(
        kernel
            .resources
            .image_list(default_color_list)
            .unwrap()
            .flags,
        ILC_COLORDDB
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DESTROY,
            [default_color_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let first_palette_bits = 0x2_c000;
    let second_palette_bits = 0x2_c100;
    memory.map_bytes(first_palette_bits, 4);
    memory.write_bytes(first_palette_bits, &[0, 0, 0, 0]);
    memory.map_bytes(second_palette_bits, 4);
    memory.write_bytes(second_palette_bits, &[0, 0, 0, 0]);
    let first_palette_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 8, first_palette_bits);
    let first_palette = vec![[0x00, 0x00, 0xff, 0], [0x00, 0xff, 0x00, 0]];
    kernel
        .resources
        .bitmap_mut(first_palette_bitmap)
        .unwrap()
        .color_table = first_palette.clone();
    let second_palette_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 8, second_palette_bits);
    kernel
        .resources
        .bitmap_mut(second_palette_bitmap)
        .unwrap()
        .color_table = vec![[0xff, 0x00, 0x00, 0]];

    let palette_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLOR8, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE 8bpp palette flags: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette_list, first_palette_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .resources
            .image_list(palette_list)
            .unwrap()
            .colors_set
    );
    assert_eq!(
        kernel
            .resources
            .image_list(palette_list)
            .unwrap()
            .color_table,
        first_palette
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette_list, second_palette_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(palette_list)
            .unwrap()
            .color_table,
        first_palette,
        "CE latches the first indexed source palette for non-DDB image lists"
    );
    let (palette_dc, palette_dst_bits, palette_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette_list, 1, palette_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, palette_dst_bits, palette_dst_stride, 0, 0),
        0xf800,
        "later indexed images in a non-DDB image list draw through the latched first palette"
    );

    let palette_ddb_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLORDDB, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE DDB flags: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette_ddb_list, first_palette_bitmap, 0],
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
            [palette_ddb_list, second_palette_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert!(
        !kernel
            .resources
            .image_list(palette_ddb_list)
            .unwrap()
            .colors_set
    );
    assert!(
        kernel
            .resources
            .image_list(palette_ddb_list)
            .unwrap()
            .color_table
            .is_empty()
    );
    let (ddb_palette_dc, ddb_palette_dst_bits, ddb_palette_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette_ddb_list, 1, ddb_palette_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, ddb_palette_dst_bits, ddb_palette_dst_stride, 0, 0),
        0x001f,
        "DDB image lists keep using each bitmap's own indexed color table"
    );

    let first_palette4_bits = 0x2_c200;
    let second_palette4_bits = 0x2_c300;
    memory.map_bytes(first_palette4_bits, 4);
    memory.write_bytes(first_palette4_bits, &[0x00, 0, 0, 0]);
    memory.map_bytes(second_palette4_bits, 4);
    memory.write_bytes(second_palette4_bits, &[0x10, 0, 0, 0]);
    let first_palette4_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 4, first_palette4_bits);
    let first_palette4 = vec![[0x00, 0x00, 0xff, 0], [0x00, 0xff, 0x00, 0]];
    kernel
        .resources
        .bitmap_mut(first_palette4_bitmap)
        .unwrap()
        .color_table = first_palette4.clone();
    let second_palette4_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 4, second_palette4_bits);
    kernel
        .resources
        .bitmap_mut(second_palette4_bitmap)
        .unwrap()
        .color_table = vec![[0x00, 0x00, 0x00, 0], [0xff, 0x00, 0x00, 0]];

    let palette4_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLOR4, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE 4bpp palette flags: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette4_list, first_palette4_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(palette4_list)
            .unwrap()
            .color_table,
        first_palette4
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette4_list, second_palette4_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let (palette4_dc, palette4_dst_bits, palette4_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette4_list, 1, palette4_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, palette4_dst_bits, palette4_dst_stride, 0, 0),
        0x07e0,
        "4bpp later indexed images in a non-DDB image list draw through the latched first palette"
    );

    let palette4_ddb_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLORDDB, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE DDB flags: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette4_ddb_list, first_palette4_bitmap, 0],
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
            [palette4_ddb_list, second_palette4_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let (palette4_ddb_dc, palette4_ddb_dst_bits, palette4_ddb_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette4_ddb_list, 1, palette4_ddb_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(
            &memory,
            palette4_ddb_dst_bits,
            palette4_ddb_dst_stride,
            0,
            0
        ),
        0x001f,
        "4bpp DDB image lists keep using each bitmap's own indexed color table"
    );

    let first_palette2_bits = 0x2_c400;
    let second_palette2_bits = 0x2_c500;
    memory.map_bytes(first_palette2_bits, 4);
    memory.write_bytes(first_palette2_bits, &[0x00, 0, 0, 0]);
    memory.map_bytes(second_palette2_bits, 4);
    memory.write_bytes(second_palette2_bits, &[0x80, 0, 0, 0]);
    let first_palette2_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 2, first_palette2_bits);
    let first_palette2 = vec![
        [0x00, 0x00, 0xff, 0],
        [0x00, 0x00, 0x00, 0],
        [0x00, 0xff, 0x00, 0],
    ];
    kernel
        .resources
        .bitmap_mut(first_palette2_bitmap)
        .unwrap()
        .color_table = first_palette2.clone();
    let second_palette2_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 2, second_palette2_bits);
    let second_palette2 = vec![
        [0x00, 0x00, 0x00, 0],
        [0x00, 0x00, 0x00, 0],
        [0xff, 0x00, 0x00, 0],
    ];
    kernel
        .resources
        .bitmap_mut(second_palette2_bitmap)
        .unwrap()
        .color_table = second_palette2;

    let palette2_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLOR4, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => {
            panic!("ImageList_Create rejected CE 4bpp palette flags for 2bpp sources: {other:?}")
        }
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette2_list, first_palette2_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(palette2_list)
            .unwrap()
            .color_table,
        first_palette2
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette2_list, second_palette2_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let (palette2_dc, palette2_dst_bits, palette2_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette2_list, 1, palette2_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, palette2_dst_bits, palette2_dst_stride, 0, 0),
        0x07e0,
        "2bpp later indexed images in a non-DDB image list draw through the latched first palette"
    );

    let palette2_ddb_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLORDDB, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE DDB flags for 2bpp sources: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette2_ddb_list, first_palette2_bitmap, 0],
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
            [palette2_ddb_list, second_palette2_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let (palette2_ddb_dc, palette2_ddb_dst_bits, palette2_ddb_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette2_ddb_list, 1, palette2_ddb_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(
            &memory,
            palette2_ddb_dst_bits,
            palette2_ddb_dst_stride,
            0,
            0
        ),
        0x001f,
        "2bpp DDB image lists keep using each bitmap's own indexed color table"
    );

    let first_palette1_bits = 0x2_c600;
    let second_palette1_bits = 0x2_c700;
    memory.map_bytes(first_palette1_bits, 4);
    memory.write_bytes(first_palette1_bits, &[0x00, 0, 0, 0]);
    memory.map_bytes(second_palette1_bits, 4);
    memory.write_bytes(second_palette1_bits, &[0x80, 0, 0, 0]);
    let first_palette1_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 1, first_palette1_bits);
    let first_palette1 = vec![[0x00, 0x00, 0xff, 0], [0x00, 0xff, 0x00, 0]];
    kernel
        .resources
        .bitmap_mut(first_palette1_bitmap)
        .unwrap()
        .color_table = first_palette1.clone();
    let second_palette1_bitmap = kernel
        .resources
        .create_bitmap(1, 1, 1, 1, second_palette1_bits);
    kernel
        .resources
        .bitmap_mut(second_palette1_bitmap)
        .unwrap()
        .color_table = vec![[0x00, 0x00, 0x00, 0], [0xff, 0x00, 0x00, 0]];

    let palette1_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLOR4, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => {
            panic!("ImageList_Create rejected CE 4bpp palette flags for 1bpp sources: {other:?}")
        }
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette1_list, first_palette1_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(palette1_list)
            .unwrap()
            .color_table,
        first_palette1
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette1_list, second_palette1_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let (palette1_dc, palette1_dst_bits, palette1_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette1_list, 1, palette1_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, palette1_dst_bits, palette1_dst_stride, 0, 0),
        0x07e0,
        "1bpp later indexed images in a non-DDB image list draw through the latched first palette"
    );

    let palette1_ddb_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLORDDB, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE DDB flags for 1bpp sources: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [palette1_ddb_list, first_palette1_bitmap, 0],
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
            [palette1_ddb_list, second_palette1_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let (palette1_ddb_dc, palette1_ddb_dst_bits, palette1_ddb_dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [palette1_ddb_list, 1, palette1_ddb_dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(
            &memory,
            palette1_ddb_dst_bits,
            palette1_ddb_dst_stride,
            0,
            0
        ),
        0x001f,
        "1bpp DDB image lists keep using each bitmap's own indexed color table"
    );

    let image_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [24, 20, ILC_MASK, 1, 2],
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
    let add_bitmap_24 = kernel.resources.create_bitmap(24, 20, 1, 24, 0);
    let add_mask_24 = kernel.resources.create_bitmap(24, 20, 1, 1, 0);
    let undersized_bitmap = kernel.resources.create_bitmap(8, 20, 1, 24, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [image_list, undersized_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0xffff_ffff),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    assert_eq!(kernel.resources.image_list_count(image_list), Some(0));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [image_list, add_bitmap_24, add_mask_24],
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
            [image_list, 0xffff_fffe, 0x000b_8123],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0xffff_ffff),
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
    memory.write_word(size_ptr, 0x7fff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_ICON_SIZE,
            [image_list, size_ptr, 0],
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
    assert_eq!(
        memory.read_i32(size_ptr)?,
        0x7fff_ffff,
        "CE ImageList_GetIconSize validates both output pointers before writing cx"
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_ICON_SIZE,
            [image_list, 24, 20],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_ICON_SIZE,
            [image_list, 0, 0xffff_fffb],
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
            ORD_IMAGE_LIST_GET_ICON_SIZE,
            [image_list, size_ptr, size_ptr + 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(size_ptr)?, 0);
    assert_eq!(memory.read_i32(size_ptr + 4)?, -5);
    assert_eq!(
        kernel
            .resources
            .image_list(image_list)
            .unwrap()
            .images
            .len(),
        0,
        "CE ImageList_SetIconSize stores changed zero/negative dimensions and clears images"
    );
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
            ORD_IMAGE_LIST_GET_IMAGE_COUNT,
            [image_list],
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
            ORD_IMAGE_LIST_GET_ICON,
            [image_list, 1, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    let add_bitmap_32 = kernel.resources.create_bitmap(32, 18, 1, 24, 0);
    let add_mask_32 = kernel.resources.create_bitmap(32, 18, 1, 1, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [image_list, add_bitmap_32, add_mask_32],
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

    memory.write_word(image_info_ptr, 0xCAFE_BABE);
    memory.write_word(image_info_ptr + 4, 0xFACE_CAFE);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_IMAGE_INFO,
            [0xDEAD_0001, 0, image_info_ptr],
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
    assert_eq!(memory.read_u32(image_info_ptr)?, 0xCAFE_BABE);
    assert_eq!(memory.read_u32(image_info_ptr + 4)?, 0xFACE_CAFE);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_IMAGE_INFO,
            [image_list, 99, image_info_ptr],
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
    assert_eq!(memory.read_u32(image_info_ptr)?, 0xCAFE_BABE);
    assert_eq!(memory.read_u32(image_info_ptr + 4)?, 0xFACE_CAFE);

    let no_mask_overlay_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [16, 16, 0, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(no-mask overlay) did not return a handle: {other:?}"),
    };
    let add_bitmap_16 = kernel.resources.create_bitmap(16, 16, 1, 24, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [no_mask_overlay_list, add_bitmap_16, 0],
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
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            [no_mask_overlay_list, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(
        !kernel
            .resources
            .image_list(no_mask_overlay_list)
            .unwrap()
            .overlays
            .contains_key(&1)
    );

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
            [image_list, 1, 5],
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
            .get(&2)
            .map(|overlay| overlay.image_index),
        Some(1)
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

    memory.write_wide_z(bitmap_path_ptr, r"\Images\masked.bmp");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
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

    let loaded = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [
            0,
            bitmap_path_ptr,
            2,
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
    assert_eq!(loaded_list.width, 2);
    assert_eq!(loaded_list.height, 1);
    assert_eq!(loaded_list.flags, ILC_MASK | ILC_COLOR24);
    assert_eq!(loaded_list.images.len(), 1);
    assert_ne!(loaded_list.images[0].bitmap, 0);
    assert_ne!(loaded_list.images[0].mask, 0);
    assert_eq!(loaded_list.images[0].transparent_color, Some(0x00ff_00ff));
    let loaded_mask = kernel.resources.bitmap(loaded_list.images[0].mask).unwrap();
    assert_eq!(loaded_mask.bits_pixel, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            [loaded, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
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
            [loaded, 0, hdc, 6, 7, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(!framebuffer.dirty_rects().is_empty());
    let pixel_offset = 7 * framebuffer.stride() + 7 * PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[pixel_offset..pixel_offset + 2],
        &[0xe0, 0x07],
        "ImageList_DrawEx should blit the loaded bitmap's non-transparent pixel, not just the pseudo-icon placeholder"
    );
    let default_loaded = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [
            0,
            bitmap_path_ptr,
            2,
            1,
            CLR_DEFAULT,
            IMAGE_BITMAP,
            LR_LOADFROMFILE,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_LoadImage(CLR_DEFAULT) did not return a handle: {other:?}"),
    };
    assert_eq!(
        kernel.resources.image_list(default_loaded).unwrap().images[0].transparent_color,
        Some(0x00ff_00ff),
        "CE ImageList_LoadImage forwards CLR_DEFAULT through AddMasked upper-left sampling"
    );
    let default_loaded_mask = kernel.resources.image_list(default_loaded).unwrap().images[0].mask;
    assert_ne!(
        default_loaded_mask, 0,
        "CE ImageList_LoadImage(CLR_DEFAULT) creates a real AddMasked mono mask"
    );
    let default_loaded_mask_bitmap = kernel.resources.bitmap(default_loaded_mask).unwrap();
    assert_eq!(default_loaded_mask_bitmap.bits_pixel, 1);
    assert_eq!(
        memory.read_u8(default_loaded_mask_bitmap.bits_ptr)?,
        0x80,
        "CE ImageList_LoadImage(CLR_DEFAULT) mask marks only the sampled-color pixel white"
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [default_loaded, 0, hdc, 2, 4, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let default_loaded_left = 4 * framebuffer.stride() + 2 * PixelFormat::Rgb565.bytes_per_pixel();
    let default_loaded_right = default_loaded_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[default_loaded_left..default_loaded_left + 2],
        &[0, 0],
        "ImageList_LoadImage(CLR_DEFAULT) should mask the sampled upper-left color"
    );
    assert_eq!(
        &framebuffer.pixels()[default_loaded_right..default_loaded_right + 2],
        &[0xe0, 0x07],
        "ImageList_LoadImage(CLR_DEFAULT) should still draw non-sampled colors"
    );

    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 12, 12);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [loaded, 0, mem_dc, 4, 5, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 5, 5),
        0x07e0,
        "ImageList_DrawEx should blit bitmap-backed entries into selected memory DIBs"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 3, 5),
        0,
        "ImageList_DrawEx should leave memory DIB pixels outside the target rect untouched"
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_BK_COLOR,
            [loaded, 0x00ff_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(_),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW,
            [loaded, 0, mem_dc, 6, 5, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 6, 5),
        0x001f,
        "CE ImageList_Draw defaults rgbBk to the image-list background color"
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW,
            [loaded, 0, mem_dc, 8, 5, 0x0001],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 8, 5),
        0,
        "explicit ILD_TRANSPARENT should still leave the transparent source pixel untouched"
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
    assert_eq!(resource_loaded_list.flags, ILC_COLOR24);
    assert_ne!(resource_loaded_list.images[0].bitmap, 0);
    assert_eq!(resource_loaded_list.images[0].mask, 0);
    assert_eq!(resource_loaded_list.images[0].transparent_color, None);
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
    let unmasked_icon = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_GET_ICON,
        [resource_loaded, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_GetIcon(unmasked) did not return an icon: {other:?}"),
    };
    let unmasked_icon_info_ptr = 0x2030_3900;
    memory.map_words(unmasked_icon_info_ptr, 5);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ICON_INFO,
            [unmasked_icon, unmasked_icon_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let unmasked_icon_mask = memory.read_u32(unmasked_icon_info_ptr + 12)?;
    assert_ne!(unmasked_icon_mask, 0);
    let unmasked_icon_mask_bitmap = kernel
        .resources
        .bitmap(unmasked_icon_mask)
        .expect("unmasked ImageList_GetIcon should create a mask bitmap");
    assert_eq!(
        memory.read_u8(unmasked_icon_mask_bitmap.bits_ptr)? & 0x80,
        0x80,
        "CE ImageList_GetIcon initializes unmasked icon masks to white"
    );
    let unmasked_icon_color = memory.read_u32(unmasked_icon_info_ptr + 16)?;
    assert_ne!(unmasked_icon_color, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DESTROY,
            [resource_loaded],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.image_list(resource_loaded).is_none());
    assert!(
        kernel.resources.bitmap(unmasked_icon_mask).is_some(),
        "CE ImageList_GetIcon returns an icon whose mask survives source list destruction"
    );
    assert!(
        kernel.resources.bitmap(unmasked_icon_color).is_some(),
        "CE ImageList_GetIcon returns an icon whose color bitmap survives source list destruction"
    );
    for offset in (0..20).step_by(4) {
        memory.write_u32(unmasked_icon_info_ptr + offset, 0)?;
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ICON_INFO,
            [unmasked_icon, unmasked_icon_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(unmasked_icon_info_ptr + 12)?,
        unmasked_icon_mask
    );
    assert_eq!(
        memory.read_u32(unmasked_icon_info_ptr + 16)?,
        unmasked_icon_color
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [unmasked_icon],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.icon(unmasked_icon).is_none());
    assert!(kernel.resources.bitmap(unmasked_icon_mask).is_none());
    assert!(kernel.resources.bitmap(unmasked_icon_color).is_none());

    memory.write_wide_z(bitmap_path_ptr, r"\Images\masked.bmp");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_LOAD_IMAGE,
            [
                0,
                bitmap_path_ptr,
                0xffff_ffff,
                1,
                0xffff_ffff,
                IMAGE_BITMAP,
                LR_LOADFROMFILE,
            ],
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

    memory.write_wide_z(bitmap_path_ptr, r"\Images\masked.bmp");
    let zero_cx_loaded = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [
            0,
            bitmap_path_ptr,
            0,
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
        other => {
            panic!("ImageList_LoadImage(cx=0 strip bitmap) did not return a handle: {other:?}")
        }
    };
    let zero_cx_loaded_list = kernel.resources.image_list(zero_cx_loaded).unwrap();
    assert_eq!(
        zero_cx_loaded_list.width, 1,
        "CE ImageList_LoadImage treats cx=0 as the bitmap height"
    );
    assert_eq!(zero_cx_loaded_list.height, 1);
    assert_eq!(zero_cx_loaded_list.images.len(), 2);
    assert_eq!(zero_cx_loaded_list.images[0].source_x, 0);
    assert_eq!(zero_cx_loaded_list.images[1].source_x, 1);

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
    let wide_masked_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [4, 1, 0, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(wide masked) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD_MASKED,
            [wide_masked_list, masked_bitmap, 0x00ff_00ff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0xffff_ffff),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    assert_eq!(kernel.resources.image_list_count(wide_masked_list), Some(0));
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
    let default_masked_list = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("ImageList_Create(default masked) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD_MASKED,
            [default_masked_list, masked_bitmap, 0xff00_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .image_list(default_masked_list)
            .unwrap()
            .images[0]
            .transparent_color,
        Some(0x00ff_00ff),
        "CE ImageList_AddMasked resolves CLR_DEFAULT from the source bitmap's upper-left pixel"
    );
    let default_mask_handle = kernel
        .resources
        .image_list(default_masked_list)
        .unwrap()
        .images[0]
        .mask;
    assert_ne!(
        default_mask_handle, 0,
        "CE ImageList_AddMasked creates a real mono mask bitmap"
    );
    let default_mask_bitmap = kernel.resources.bitmap(default_mask_handle).unwrap();
    assert_eq!(default_mask_bitmap.bits_pixel, 1);
    assert_eq!(
        memory.read_u8(default_mask_bitmap.bits_ptr)?,
        0x80,
        "CE ImageList_AddMasked mask marks only the sampled-color pixel white"
    );
    let default_masked_left = 4 * PixelFormat::Rgb565.bytes_per_pixel();
    let default_masked_right = default_masked_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [default_masked_list, 0, hdc, 4, 0, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        &framebuffer.pixels()[default_masked_left..default_masked_left + 2],
        &[0, 0],
        "ImageList_AddMasked(CLR_DEFAULT) should mask the sampled upper-left color"
    );
    assert_eq!(
        &framebuffer.pixels()[default_masked_right..default_masked_right + 2],
        &[0xe0, 0x07],
        "ImageList_AddMasked(CLR_DEFAULT) should still draw non-sampled colors"
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
    let mask_image = kernel
        .resources
        .image_list(mask_handle_list)
        .unwrap()
        .images[0]
        .clone();
    assert_ne!(mask_image.bitmap, masked_bitmap);
    assert_ne!(mask_image.mask, mask_bitmap);
    assert!(kernel.resources.bitmap(mask_image.bitmap).is_some());
    assert!(kernel.resources.bitmap(mask_image.mask).is_some());
    assert_eq!(mask_image.transparent_color, None);
    let extracted_icon = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_GET_ICON,
        [mask_handle_list, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_GetIcon(mask handle) did not return a handle: {other:?}"),
    };
    assert_ne!(extracted_icon, 0);
    assert_ne!(
        extracted_icon & 0x00ff_ffff,
        0x000b_8000 | (mask_image.bitmap & 0x0000_ffff),
        "CE ImageList_GetIcon creates a real icon instead of returning a bitmap pseudo-handle"
    );
    let extracted_icon_obj = kernel
        .resources
        .icon(extracted_icon)
        .expect("bitmap-backed ImageList_GetIcon should create a real icon")
        .clone();
    assert_ne!(extracted_icon_obj.color_bitmap, 0);
    assert_ne!(extracted_icon_obj.mask_bitmap, 0);
    let extracted_color = kernel
        .resources
        .bitmap(extracted_icon_obj.color_bitmap)
        .expect("extracted icon color bitmap");
    let extracted_mask = kernel
        .resources
        .bitmap(extracted_icon_obj.mask_bitmap)
        .expect("extracted icon mask bitmap");
    assert_eq!((extracted_color.width, extracted_color.height), (2, 1));
    assert_eq!((extracted_mask.width, extracted_mask.height), (2, 1));
    let extracted_color_bits = memory.read_bytes(
        extracted_color.bits_ptr,
        extracted_color.width_bytes as usize,
    );
    assert_eq!(
        &extracted_color_bits[4..8],
        &[0x00, 0xff, 0x00, 0xff],
        "ImageList_GetIcon color pass should render the non-masked source pixel"
    );
    let extracted_mask_byte = memory.read_u8(extracted_mask.bits_ptr)?;
    assert_eq!(
        extracted_mask_byte & 0xc0,
        0x80,
        "ImageList_GetIcon mask pass should preserve CE white/black mask pixels"
    );
    let extracted_color_handle = extracted_icon_obj.color_bitmap;
    let extracted_mask_handle = extracted_icon_obj.mask_bitmap;
    let extracted_color_bits_ptr = extracted_color.bits_ptr;
    let extracted_mask_bits_ptr = extracted_mask.bits_ptr;
    assert!(extracted_icon_obj.owns_color_bitmap);
    assert!(extracted_icon_obj.owns_mask_bitmap);
    let extracted_icon_info_ptr = 0x2030_3a00;
    memory.map_words(extracted_icon_info_ptr, 5);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ICON_INFO,
            [extracted_icon, extracted_icon_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(extracted_icon_info_ptr)?,
        1,
        "ImageList_GetIcon should return an icon, not a cursor"
    );
    assert_eq!(
        memory.read_u32(extracted_icon_info_ptr + 12)?,
        extracted_mask_handle,
        "GetIconInfo should expose ImageList_GetIcon's rendered mask bitmap"
    );
    assert_eq!(
        memory.read_u32(extracted_icon_info_ptr + 16)?,
        extracted_color_handle,
        "GetIconInfo should expose ImageList_GetIcon's rendered color bitmap"
    );
    assert_ne!(
        memory.read_u32(extracted_icon_info_ptr + 12)?,
        mask_image.mask,
        "GetIconInfo should not leak the source image-list mask handle"
    );
    assert_ne!(
        memory.read_u32(extracted_icon_info_ptr + 16)?,
        mask_image.bitmap,
        "GetIconInfo should not leak the source image-list color handle"
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [extracted_icon],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.icon(extracted_icon).is_none());
    assert!(kernel.resources.bitmap(extracted_color_handle).is_none());
    assert!(kernel.resources.bitmap(extracted_mask_handle).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, extracted_color_bits_ptr)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, extracted_mask_bits_ptr)
            .is_none()
    );
    let mask_handle_duplicate = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_DUPLICATE,
        [mask_handle_list],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Duplicate(mask handle) did not return a handle: {other:?}"),
    };
    assert_ne!(mask_handle_duplicate, 0);
    assert_ne!(mask_handle_duplicate, mask_handle_list);
    let original_mask_image = kernel
        .resources
        .image_list(mask_handle_list)
        .unwrap()
        .images[0]
        .clone();
    let duplicate_mask_image = kernel
        .resources
        .image_list(mask_handle_duplicate)
        .unwrap()
        .images[0]
        .clone();
    assert_ne!(
        duplicate_mask_image.bitmap, original_mask_image.bitmap,
        "CE ImageList_Duplicate copies image bitmap storage instead of aliasing the source list"
    );
    assert_ne!(
        duplicate_mask_image.mask, original_mask_image.mask,
        "CE ImageList_Duplicate copies mask bitmap storage instead of aliasing the source list"
    );
    assert_ne!(
        kernel
            .resources
            .bitmap(duplicate_mask_image.bitmap)
            .unwrap()
            .bits_ptr,
        kernel
            .resources
            .bitmap(original_mask_image.bitmap)
            .unwrap()
            .bits_ptr
    );
    assert_ne!(
        kernel
            .resources
            .bitmap(duplicate_mask_image.mask)
            .unwrap()
            .bits_ptr,
        kernel
            .resources
            .bitmap(original_mask_image.mask)
            .unwrap()
            .bits_ptr
    );
    let duplicate_bitmap_bits = kernel
        .resources
        .bitmap(duplicate_mask_image.bitmap)
        .unwrap()
        .bits_ptr;
    let duplicate_mask_bits = kernel
        .resources
        .bitmap(duplicate_mask_image.mask)
        .unwrap()
        .bits_ptr;
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, duplicate_bitmap_bits)
            .is_some()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, duplicate_mask_bits)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DESTROY,
            [mask_handle_duplicate],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.image_list(mask_handle_duplicate).is_none());
    assert!(
        kernel
            .resources
            .bitmap(duplicate_mask_image.bitmap)
            .is_none()
    );
    assert!(kernel.resources.bitmap(duplicate_mask_image.mask).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, duplicate_bitmap_bits)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, duplicate_mask_bits)
            .is_none()
    );
    assert!(
        kernel
            .resources
            .bitmap(original_mask_image.bitmap)
            .is_some()
    );
    assert!(kernel.resources.bitmap(original_mask_image.mask).is_some());
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
    let bk_reset_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(bk reset) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [bk_reset_list, masked_bitmap, mask_bitmap],
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
            ORD_IMAGE_LIST_SET_BK_COLOR,
            [bk_reset_list, 0x0000_00ff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(CLR_NONE),
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
            [
                bk_reset_list,
                0,
                hdc,
                8,
                5,
                2,
                1,
                CLR_NONE,
                CLR_NONE,
                ILD_IMAGE
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let reset_left = 5 * framebuffer.stride() + 8 * PixelFormat::Rgb565.bytes_per_pixel();
    let reset_right = reset_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[reset_left..reset_left + 2],
        &[0x00, 0xf8],
        "ImageList_SetBkColor should rewrite mask-on backing pixels before ILD_IMAGE draws"
    );
    assert_eq!(
        &framebuffer.pixels()[reset_right..reset_right + 2],
        &[0xe0, 0x07],
        "ImageList_SetBkColor should leave mask-off source pixels unchanged"
    );
    let original_bitmap_bits = kernel
        .resources
        .bitmap(original_mask_image.bitmap)
        .unwrap()
        .bits_ptr;
    let original_mask_bits = kernel
        .resources
        .bitmap(original_mask_image.mask)
        .unwrap()
        .bits_ptr;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_REMOVE,
            [mask_handle_list, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.resources.image_list_count(mask_handle_list), Some(0));
    assert!(
        kernel
            .resources
            .bitmap(original_mask_image.bitmap)
            .is_none()
    );
    assert!(kernel.resources.bitmap(original_mask_image.mask).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, original_bitmap_bits)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, original_mask_bits)
            .is_none()
    );
    let replace_cleanup_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(replace cleanup) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [replace_cleanup_list, masked_bitmap, mask_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let replaced_image = kernel
        .resources
        .image_list(replace_cleanup_list)
        .unwrap()
        .images[0]
        .clone();
    let replaced_bitmap_bits = kernel
        .resources
        .bitmap(replaced_image.bitmap)
        .unwrap()
        .bits_ptr;
    let replaced_mask_bits = kernel
        .resources
        .bitmap(replaced_image.mask)
        .unwrap()
        .bits_ptr;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_REPLACE,
            [replace_cleanup_list, 0, masked_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let replacement_image = kernel
        .resources
        .image_list(replace_cleanup_list)
        .unwrap()
        .images[0]
        .clone();
    assert_ne!(replacement_image.bitmap, replaced_image.bitmap);
    assert_eq!(replacement_image.mask, 0);
    assert!(kernel.resources.bitmap(replaced_image.bitmap).is_none());
    assert!(kernel.resources.bitmap(replaced_image.mask).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, replaced_bitmap_bits)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, replaced_mask_bits)
            .is_none()
    );
    let replace_icon_cleanup_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => {
            panic!("ImageList_Create(replace icon cleanup) did not return a handle: {other:?}")
        }
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [replace_icon_cleanup_list, masked_bitmap, mask_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let icon_replaced_image = kernel
        .resources
        .image_list(replace_icon_cleanup_list)
        .unwrap()
        .images[0]
        .clone();
    let icon_replaced_bitmap_bits = kernel
        .resources
        .bitmap(icon_replaced_image.bitmap)
        .unwrap()
        .bits_ptr;
    let icon_replaced_mask_bits = kernel
        .resources
        .bitmap(icon_replaced_image.mask)
        .unwrap()
        .bits_ptr;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_REPLACE_ICON,
            [replace_icon_cleanup_list, 0, 0x000b_8123],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let icon_replacement = &kernel
        .resources
        .image_list(replace_icon_cleanup_list)
        .unwrap()
        .images[0];
    assert_eq!(icon_replacement.bitmap, 0);
    assert_eq!(icon_replacement.mask, 0);
    assert_eq!(icon_replacement.icon, 0x000b_8123);
    assert!(
        kernel
            .resources
            .bitmap(icon_replaced_image.bitmap)
            .is_none()
    );
    assert!(kernel.resources.bitmap(icon_replaced_image.mask).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, icon_replaced_bitmap_bits)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, icon_replaced_mask_bits)
            .is_none()
    );
    let dither_mask_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_MASK, 2, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(dither mask) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [dither_mask_list, masked_bitmap, mask_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.resources.image_list_count(dither_mask_list), Some(2));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY_DITHER_IMAGE,
            [dither_mask_list, 0, 0, 0, dither_mask_list, 1, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let (dither_mask_dc, dither_mask_bits, dither_mask_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 1, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [
                dither_mask_list,
                0,
                dither_mask_dc,
                0,
                0,
                1,
                1,
                0,
                0,
                0x0010
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, dither_mask_bits, dither_mask_stride, 0, 0),
        0xffff,
        "CopyDitherImage should OR a 50% CE dither pattern into copied masks before SRCAND"
    );
    let dither_image = kernel
        .resources
        .image_list(dither_mask_list)
        .unwrap()
        .images[0]
        .clone();
    let dither_bitmap_bits = kernel
        .resources
        .bitmap(dither_image.bitmap)
        .unwrap()
        .bits_ptr;
    let dither_mask_bits_ptr = kernel.resources.bitmap(dither_image.mask).unwrap().bits_ptr;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_IMAGE_COUNT,
            [dither_mask_list, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.bitmap(dither_image.bitmap).is_some());
    assert!(kernel.resources.bitmap(dither_image.mask).is_some());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, dither_bitmap_bits)
            .is_some()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, dither_mask_bits_ptr)
            .is_some()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_IMAGE_COUNT,
            [dither_mask_list, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.bitmap(dither_image.bitmap).is_none());
    assert!(kernel.resources.bitmap(dither_image.mask).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, dither_bitmap_bits)
            .is_none()
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, dither_mask_bits_ptr)
            .is_none()
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
        [2, 1, ILC_MASK, 2, 1],
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
            ORD_DELETE_OBJECT,
            [red_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(
        kernel.resources.bitmap(red_bitmap).is_none(),
        "caller-owned source bitmap should be deletable after ImageList_Add"
    );
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [overlay_list, 0, hdc, 12, 7, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let lifetime_pixel = 7 * framebuffer.stride() + 12 * PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[lifetime_pixel..lifetime_pixel + 2],
        &[0x00, 0xf8],
        "ImageList_Add should copy source pixels so later DeleteObject does not invalidate drawing"
    );
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
    let overlay_record = kernel
        .resources
        .image_list(overlay_list)
        .unwrap()
        .overlays
        .get(&1)
        .copied()
        .expect("overlay metadata");
    assert_eq!(overlay_record.image_index, 1);
    assert_eq!(overlay_record.x, 1);
    assert_eq!(overlay_record.y, 0);
    assert_eq!(overlay_record.width, 1);
    assert_eq!(overlay_record.height, 1);
    assert_eq!(overlay_record.flags, 0x0020);
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
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [overlay_list, 0, hdc, 6, 8, 2, 1, 0, 0, 0x0110],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let overlay_mask_right = 8 * framebuffer.stride() + 7 * PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[overlay_mask_right..overlay_mask_right + 2],
        &[0x00, 0x00],
        "ImageList overlay draw with ILD_MASK should draw the overlay mask instead of skipping the overlay"
    );

    memory.write_wide_z(bitmap_path_ptr, r"\Images\white-mask.bmp");
    let white_mask_bitmap = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("LoadImageW(white mask bitmap) did not return a bitmap: {other:?}"),
    };
    assert_ne!(white_mask_bitmap, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [overlay_list, masked_bitmap, white_mask_bitmap],
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
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            [overlay_list, 2, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let white_overlay_record = kernel
        .resources
        .image_list(overlay_list)
        .unwrap()
        .overlays
        .get(&2)
        .copied()
        .expect("white overlay metadata");
    assert_eq!(white_overlay_record.image_index, 2);
    assert_eq!(white_overlay_record.x, 0);
    assert_eq!(white_overlay_record.y, 0);
    assert_eq!(white_overlay_record.width, 0);
    assert_eq!(white_overlay_record.height, 0);
    assert_eq!(white_overlay_record.flags, 0);
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [overlay_list, 0, hdc, 10, 8, 2, 1, 0, 0, 0x0200],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let white_overlay_left = 8 * framebuffer.stride() + 10 * PixelFormat::Rgb565.bytes_per_pixel();
    let white_overlay_right = white_overlay_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[white_overlay_left..white_overlay_left + 2],
        &[0x00, 0xf8],
        "all-white overlay masks should leave the base image's left pixel intact"
    );
    assert_eq!(
        &framebuffer.pixels()[white_overlay_right..white_overlay_right + 2],
        &[0x00, 0xf8],
        "all-white overlay masks should leave the base image's right pixel intact"
    );

    memory.write_wide_z(bitmap_path_ptr, r"\Images\red2.bmp");
    let diagonal_base_bitmap = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("LoadImageW(red2 bitmap) did not return a bitmap: {other:?}"),
    };
    assert_ne!(diagonal_base_bitmap, 0);
    memory.write_wide_z(bitmap_path_ptr, r"\Images\green2.bmp");
    let diagonal_overlay_bitmap = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("LoadImageW(green2 bitmap) did not return a bitmap: {other:?}"),
    };
    assert_ne!(diagonal_overlay_bitmap, 0);
    memory.write_wide_z(bitmap_path_ptr, r"\Images\diagonal-mask.bmp");
    let diagonal_mask_bitmap = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("LoadImageW(diagonal mask bitmap) did not return a bitmap: {other:?}"),
    };
    assert_ne!(diagonal_mask_bitmap, 0);
    let diagonal_overlay_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 2, ILC_MASK, 2, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(diagonal overlay) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [diagonal_overlay_list, diagonal_base_bitmap, 0],
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
            [
                diagonal_overlay_list,
                diagonal_overlay_bitmap,
                diagonal_mask_bitmap
            ],
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
            [diagonal_overlay_list, 1, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let diagonal_overlay_record = kernel
        .resources
        .image_list(diagonal_overlay_list)
        .unwrap()
        .overlays
        .get(&1)
        .copied()
        .expect("diagonal overlay metadata");
    assert_eq!(diagonal_overlay_record.image_index, 1);
    assert_eq!(diagonal_overlay_record.x, 0);
    assert_eq!(diagonal_overlay_record.y, 0);
    assert_eq!(diagonal_overlay_record.width, 2);
    assert_eq!(diagonal_overlay_record.height, 2);
    assert_eq!(
        diagonal_overlay_record.flags, 0,
        "non-rectangular overlay masks should not be promoted to ILD_IMAGE"
    );
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [diagonal_overlay_list, 0, hdc, 14, 8, 2, 2, 0, 0, 0x0100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let diagonal_top_left = 8 * framebuffer.stride() + 14 * PixelFormat::Rgb565.bytes_per_pixel();
    let diagonal_top_right = diagonal_top_left + PixelFormat::Rgb565.bytes_per_pixel();
    let diagonal_bottom_left =
        9 * framebuffer.stride() + 14 * PixelFormat::Rgb565.bytes_per_pixel();
    let diagonal_bottom_right = diagonal_bottom_left + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[diagonal_top_left..diagonal_top_left + 2],
        &[0xe0, 0x07],
        "diagonal overlay mask should paint the top-left black-mask pixel"
    );
    assert_eq!(
        &framebuffer.pixels()[diagonal_top_right..diagonal_top_right + 2],
        &[0x00, 0xf8],
        "diagonal overlay mask should preserve the top-right white-mask pixel"
    );
    assert_eq!(
        &framebuffer.pixels()[diagonal_bottom_left..diagonal_bottom_left + 2],
        &[0x00, 0xf8],
        "diagonal overlay mask should preserve the bottom-left white-mask pixel"
    );
    assert_eq!(
        &framebuffer.pixels()[diagonal_bottom_right..diagonal_bottom_right + 2],
        &[0xe0, 0x07],
        "diagonal overlay mask should paint the bottom-right black-mask pixel"
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
    assert_eq!(merged_list.width, 32);
    assert_eq!(merged_list.height, 18);
    assert_eq!(merged_list.flags, ILC_MASK | ILC_COLORDDB);
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
    assert_eq!(image_list_after_dither.images[0].bitmap, 0x000a_3333);
    assert_eq!(image_list_after_dither.images[0].icon, 0);
    let dither_copy = image_list_after_dither.last_dither_copy.unwrap();
    assert_eq!(dither_copy.dst_image_list, image_list);
    assert_eq!(dither_copy.dst_index, 0);
    assert_eq!(dither_copy.x, 6);
    assert_eq!(dither_copy.y, 9);
    assert_eq!(dither_copy.src_image_list, duplicate);
    assert_eq!(dither_copy.src_index, 1);
    assert_eq!(dither_copy.flags, 0x0200);

    memory.write_wide_z(bitmap_path_ptr, r"\Images\red.bmp");
    let dither_dst = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [
            0,
            bitmap_path_ptr,
            2,
            1,
            CLR_NONE,
            IMAGE_BITMAP,
            LR_LOADFROMFILE,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_LoadImage(dither dst) did not return a handle: {other:?}"),
    };
    memory.write_wide_z(bitmap_path_ptr, r"\Images\masked.bmp");
    let dither_src = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_LOAD_IMAGE,
        [
            0,
            bitmap_path_ptr,
            2,
            1,
            CLR_NONE,
            IMAGE_BITMAP,
            LR_LOADFROMFILE,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_LoadImage(dither src) did not return a handle: {other:?}"),
    };
    let dither_dst_bitmap = kernel.resources.image_list(dither_dst).unwrap().images[0].bitmap;
    assert_ne!(dither_dst_bitmap, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY_DITHER_IMAGE,
            [dither_dst, 0, 0, 0, dither_src, 0, 0x02ff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let dither_dst_after = kernel.resources.image_list(dither_dst).unwrap();
    assert_eq!(dither_dst_after.images[0].bitmap, dither_dst_bitmap);
    assert_eq!(
        dither_dst_after.last_dither_copy.unwrap().flags,
        0x0200,
        "CE masks CopyDitherImage fStyle down to ILD_OVERLAYMASK"
    );
    let (dither_mem_dc, dither_bits, dither_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 4, 2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [dither_dst, 0, dither_mem_dc, 0, 0, 2, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, dither_bits, dither_stride, 0, 0),
        0xf81f,
        "CopyDitherImage should copy source pixels into the destination bitmap"
    );
    assert_eq!(
        rgb565_at(&memory, dither_bits, dither_stride, 1, 0),
        0x07e0,
        "CopyDitherImage should mutate the destination bitmap instead of replacing metadata only"
    );

    let dither_offset_dst_bits = 0x2_d000;
    let dither_offset_dst_mask_bits = 0x2_d100;
    let dither_offset_src_bits = 0x2_d200;
    let dither_offset_src_mask_bits = 0x2_d300;
    memory.map_bytes(dither_offset_dst_bits, 24);
    memory.map_bytes(dither_offset_dst_mask_bits, 12);
    memory.map_bytes(dither_offset_src_bits, 8);
    memory.map_bytes(dither_offset_src_mask_bits, 8);
    for row in 0..3 {
        for col in 0..3 {
            memory.write_bytes(
                dither_offset_dst_bits + row * 8 + col * 2,
                &0xf800u16.to_le_bytes(),
            );
        }
        memory.write_bytes(dither_offset_dst_mask_bits + row * 4, &[0xff, 0, 0, 0]);
    }
    memory.write_bytes(dither_offset_src_bits, &0x07e0u16.to_le_bytes());
    memory.write_bytes(dither_offset_src_bits + 2, &0x001fu16.to_le_bytes());
    memory.write_bytes(dither_offset_src_bits + 4, &0xf81fu16.to_le_bytes());
    memory.write_bytes(dither_offset_src_bits + 6, &0xffffu16.to_le_bytes());
    memory.write_bytes(dither_offset_src_mask_bits, &[0, 0, 0, 0, 0, 0, 0, 0]);
    let dither_offset_dst_bitmap =
        kernel
            .resources
            .create_bitmap(3, -3, 1, 16, dither_offset_dst_bits);
    let dither_offset_dst_mask =
        kernel
            .resources
            .create_bitmap(3, -3, 1, 1, dither_offset_dst_mask_bits);
    let dither_offset_src_bitmap =
        kernel
            .resources
            .create_bitmap(2, -2, 1, 16, dither_offset_src_bits);
    let dither_offset_src_mask =
        kernel
            .resources
            .create_bitmap(2, -2, 1, 1, dither_offset_src_mask_bits);
    kernel
        .resources
        .bitmap_mut(dither_offset_dst_mask)
        .unwrap()
        .color_table = vec![[0x00, 0x00, 0x00, 0], [0xff, 0xff, 0xff, 0]];
    kernel
        .resources
        .bitmap_mut(dither_offset_src_mask)
        .unwrap()
        .color_table = vec![[0x00, 0x00, 0x00, 0], [0xff, 0xff, 0xff, 0]];
    let dither_offset_dst = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [3, 3, ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(dither offset dst) failed: {other:?}"),
    };
    let dither_offset_src = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 2, ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(dither offset src) failed: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [
                dither_offset_dst,
                dither_offset_dst_bitmap,
                dither_offset_dst_mask
            ],
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
            [
                dither_offset_src,
                dither_offset_src_bitmap,
                dither_offset_src_mask
            ],
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
            ORD_IMAGE_LIST_COPY_DITHER_IMAGE,
            [dither_offset_dst, 0, 1, 1, dither_offset_src, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let (dither_offset_image_dc, dither_offset_image_bits, dither_offset_image_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 3, 3);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [
                dither_offset_dst,
                0,
                dither_offset_image_dc,
                0,
                0,
                3,
                3,
                0,
                0,
                ILD_IMAGE,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_image_bits,
            dither_offset_image_stride,
            0,
            0
        ),
        0xf800,
        "CopyDitherImage should leave pixels outside the offset copy rectangle unchanged"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_image_bits,
            dither_offset_image_stride,
            1,
            1
        ),
        0x07e0,
        "CopyDitherImage should place source pixels at the CE destination offset"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_image_bits,
            dither_offset_image_stride,
            2,
            1
        ),
        0x001f,
        "CopyDitherImage should copy the source top row at the destination offset"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_image_bits,
            dither_offset_image_stride,
            1,
            2
        ),
        0xf81f,
        "CopyDitherImage should copy the source bottom row at the destination offset"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_image_bits,
            dither_offset_image_stride,
            2,
            2
        ),
        0xffff,
        "CopyDitherImage should preserve all copied source pixels inside the offset rectangle"
    );

    let (dither_offset_mask_dc, dither_offset_mask_bits, dither_offset_mask_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 3, 3);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [
                dither_offset_dst,
                0,
                dither_offset_mask_dc,
                0,
                0,
                3,
                3,
                0,
                0,
                ILD_MASK,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_mask_bits,
            dither_offset_mask_stride,
            0,
            0
        ),
        0xffff,
        "CopyDitherImage should leave destination mask pixels outside the offset copy unchanged"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_mask_bits,
            dither_offset_mask_stride,
            1,
            1
        ),
        0xffff,
        "CopyDitherImage should OR the first copied mask pixel with CE's 50% dither pattern"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_mask_bits,
            dither_offset_mask_stride,
            2,
            1
        ),
        0x0000,
        "CopyDitherImage should keep the alternating dither mask pixel black"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_mask_bits,
            dither_offset_mask_stride,
            1,
            2
        ),
        0x0000,
        "CopyDitherImage should keep the opposite alternating dither mask pixel black"
    );
    assert_eq!(
        rgb565_at(
            &memory,
            dither_offset_mask_bits,
            dither_offset_mask_stride,
            2,
            2
        ),
        0xffff,
        "CopyDitherImage should produce the second white checkerboard mask pixel"
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [image_list, 0, duplicate, 1, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
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
            value: CoredllValue::Handle(handle),
            ..
        } if handle == (0x000b_8000 | (0x000a_3333 & 0xffff))
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAG_MOVE,
            [6, 7],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(kernel.resources.image_list_drag().is_none());
    memory.write_word(drag_point_ptr, 0x7fff_ffff);
    memory.write_word(drag_point_ptr + 4, 0x7fff_ffff);
    memory.write_word(drag_hotspot_ptr, 0x7fff_ffff);
    memory.write_word(drag_hotspot_ptr + 4, 0x7fff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_DRAG_IMAGE,
            [drag_point_ptr, drag_hotspot_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_i32(drag_point_ptr)?, 0);
    assert_eq!(memory.read_i32(drag_point_ptr + 4)?, 0);
    assert_eq!(memory.read_i32(drag_hotspot_ptr)?, 0);
    assert_eq!(memory.read_i32(drag_hotspot_ptr + 4)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_DRAG_CURSOR_IMAGE,
            [image_list, 99, 4, 5],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(kernel.resources.image_list_drag().is_none());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAG_ENTER,
            [0x0007_0000, 12, 13],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(kernel.resources.image_list_drag().is_none());
    memory.write_word(drag_point_ptr, 0x7fff_ffff);
    memory.write_word(drag_point_ptr + 4, 0x7fff_ffff);
    memory.write_word(drag_hotspot_ptr, 0x7fff_ffff);
    memory.write_word(drag_hotspot_ptr + 4, 0x7fff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_GET_DRAG_IMAGE,
            [drag_point_ptr, drag_hotspot_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_i32(drag_point_ptr)?, 12);
    assert_eq!(memory.read_i32(drag_point_ptr + 4)?, 13);
    assert_eq!(memory.read_i32(drag_hotspot_ptr)?, 0);
    assert_eq!(memory.read_i32(drag_hotspot_ptr + 4)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAG_LEAVE,
            [0x0007_1234],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
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
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(kernel.resources.image_list_drag().is_none());
    let drag_point_before_begin = kernel.resources.image_list_drag_position();
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
    let drag_image_list = drag.image_list;
    assert_ne!(drag_image_list, image_list);
    assert_eq!(
        kernel.resources.image_list_count(drag_image_list),
        Some(1),
        "CE BeginDrag exposes a one-image internal drag list"
    );
    assert_eq!(drag.index, 0);
    assert_eq!(drag.hotspot_x, 2);
    assert_eq!(drag.hotspot_y, 3);
    assert!(!drag.visible);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_BEGIN_DRAG,
            [duplicate, 1, 8, 9],
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
    let drag = kernel.resources.image_list_drag().unwrap();
    assert_eq!(
        drag.image_list, drag_image_list,
        "CE BeginDrag ignores a second active drag instead of replacing the current drag image"
    );
    assert_eq!(drag.index, 0);
    assert_eq!(drag.hotspot_x, 2);
    assert_eq!(drag.hotspot_y, 3);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAG_MOVE,
            [6, 7],
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
        } if handle == drag_image_list
    ));
    assert_eq!(memory.read_i32(drag_point_ptr)?, drag_point_before_begin.0);
    assert_eq!(
        memory.read_i32(drag_point_ptr + 4)?,
        drag_point_before_begin.1
    );
    assert_eq!(memory.read_i32(drag_hotspot_ptr)?, 2);
    assert_eq!(memory.read_i32(drag_hotspot_ptr + 4)?, 3);
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
            ORD_IMAGE_LIST_DRAG_MOVE,
            [30, 31],
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
    let merged_drag_image_list = kernel.resources.image_list_drag().unwrap().image_list;
    assert_ne!(merged_drag_image_list, drag_image_list);
    assert_ne!(merged_drag_image_list, duplicate);
    assert_eq!(
        kernel.resources.image_list_count(merged_drag_image_list),
        Some(2),
        "CE SetDragCursorImage merges the dither drag image and cursor image into a transient drag list"
    );
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
        } if handle == merged_drag_image_list
    ));
    assert_eq!(memory.read_i32(drag_point_ptr)?, 20);
    assert_eq!(memory.read_i32(drag_point_ptr + 4)?, 21);
    assert_eq!(memory.read_i32(drag_hotspot_ptr)?, 2);
    assert_eq!(memory.read_i32(drag_hotspot_ptr + 4)?, 3);
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
    assert!(kernel.resources.image_list(drag_image_list).is_none());
    assert!(
        kernel
            .resources
            .image_list(merged_drag_image_list)
            .is_none()
    );

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
            ORD_IMAGE_LIST_SET_IMAGE_COUNT,
            [image_list, u32::MAX],
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
            [image_list, 0xffff_fffe],
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
fn image_list_copy_honors_ce_move_swap_flags() -> Result<()> {
    const ILCF_MOVE: u32 = 0x0000_0000;
    const ILCF_SWAP: u32 = 0x0000_0001;
    const ERROR_INVALID_PARAMETER: u32 = 87;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;

    let list_a = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [16, 16, 0, 2, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(list_a) failed: {other:?}"),
    };
    let list_b = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [16, 16, 0, 2, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create(list_b) failed: {other:?}"),
    };

    for (list, icon_a, icon_b) in [
        (list_a, 0x000b_8101, 0x000b_8102),
        (list_b, 0x000b_8201, 0x000b_8202),
    ] {
        for icon in [icon_a, icon_b] {
            assert!(matches!(
                table.dispatch_raw_ordinal_with_memory(
                    &mut kernel,
                    &mut memory,
                    thread_id,
                    ORD_IMAGE_LIST_REPLACE_ICON,
                    [list, 0xffff_ffff, icon],
                ),
                CoredllDispatch::Returned {
                    value: CoredllValue::U32(_),
                    ..
                }
            ));
        }
    }
    {
        let list = kernel.resources.image_list_mut(list_a).unwrap();
        list.images[0].source_x = 11;
        list.images[0].source_y = 12;
        list.images[1].source_x = 21;
        list.images[1].source_y = 22;
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [list_a, 0, list_b, 1, 0x0000_0002],
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
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[0].icon,
        0x000b_8101
    );
    assert_eq!(
        kernel.resources.image_list(list_b).unwrap().images[1].icon,
        0x000b_8202
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [list_a, 0, list_b, 1, ILCF_SWAP],
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
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[0].icon,
        0x000b_8101
    );
    assert_eq!(
        kernel.resources.image_list(list_b).unwrap().images[1].icon,
        0x000b_8202
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [list_a, 0, list_a, 1, ILCF_SWAP],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[0].icon,
        0x000b_8102
    );
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[1].icon,
        0x000b_8101
    );
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[0].source_x,
        11,
        "CE ImageList_Copy swaps payload pixels/icons without swapping destination slot rectangles"
    );
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[0].source_y,
        12
    );
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[1].source_x,
        21
    );
    assert_eq!(
        kernel.resources.image_list(list_a).unwrap().images[1].source_y,
        22
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [list_a, 0, list_a, 1, ILCF_MOVE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let list_a_after_self_move = kernel.resources.image_list(list_a).unwrap();
    assert_eq!(
        list_a_after_self_move.images.len(),
        2,
        "ImageList_Copy should copy within the list without removing the source image"
    );
    assert_eq!(list_a_after_self_move.images[0].icon, 0x000b_8101);
    assert_eq!(list_a_after_self_move.images[1].icon, 0x000b_8101);
    assert_eq!(list_a_after_self_move.images[0].source_x, 11);
    assert_eq!(list_a_after_self_move.images[0].source_y, 12);
    assert_eq!(
        list_a_after_self_move.images[1].source_x, 21,
        "CE ImageList_Copy move preserves the destination slot rectangle"
    );
    assert_eq!(list_a_after_self_move.images[1].source_y, 22);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [list_a, 0, list_a, 4, ILCF_MOVE],
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
fn image_list_copy_preserves_indexed_pixel_indices() -> Result<()> {
    const ILC_COLOR8: u32 = 0x0008;
    const ILCF_MOVE: u32 = 0x0000_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 45;

    let dst_bits = 0x2_d000;
    let src_bits = 0x2_d100;
    memory.map_bytes(dst_bits, 4);
    memory.write_bytes(dst_bits, &[0, 0, 0, 0]);
    memory.map_bytes(src_bits, 4);
    memory.write_bytes(src_bits, &[1, 0, 0, 0]);

    let dst_bitmap = kernel.resources.create_bitmap(1, 1, 1, 8, dst_bits);
    let first_palette = vec![[0x00, 0x00, 0xff, 0], [0x00, 0xff, 0x00, 0]];
    kernel.resources.bitmap_mut(dst_bitmap).unwrap().color_table = first_palette;
    let src_bitmap = kernel.resources.create_bitmap(1, 1, 1, 8, src_bits);
    kernel.resources.bitmap_mut(src_bitmap).unwrap().color_table =
        vec![[0x00, 0x00, 0x00, 0], [0xff, 0x00, 0x00, 0]];

    let image_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [1, 1, ILC_COLOR8, 2, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("ImageList_Create rejected CE 8bpp palette flags: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [image_list, dst_bitmap, 0],
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
            [image_list, src_bitmap, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));

    let stored_dst_bitmap = kernel.resources.image_list(image_list).unwrap().images[0].bitmap;
    let stored_dst_bits = kernel
        .resources
        .bitmap(stored_dst_bitmap)
        .expect("image-list copy stores owned bitmaps")
        .bits_ptr;
    assert_eq!(memory.read_u8(stored_dst_bits)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_COPY,
            [image_list, 0, image_list, 1, ILCF_MOVE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u8(stored_dst_bits)?,
        1,
        "CE ImageList_Copy BitBlts indexed slots as raw indices instead of palette-converted RGB"
    );

    let (dc, draw_bits, draw_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 1, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_EX,
            [image_list, 0, dc, 0, 0, 1, 1, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        rgb565_at(&memory, draw_bits, draw_stride, 0, 0),
        0x07e0,
        "the copied raw index should render through the list's first latched palette"
    );

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
    const ILD_TRANSPARENT: u32 = 0x0001;
    const ILD_BLEND25: u32 = 0x0002;
    const ILD_BLEND50: u32 = 0x0004;
    const ILD_BLEND75: u32 = 0x0008;
    const ILD_SELECTED: u32 = ILD_BLEND50;
    const ILD_FOCUS: u32 = ILD_BLEND25;
    const ILD_MASK: u32 = 0x0010;
    const ILD_IMAGE: u32 = 0x0020;
    const ILD_ROP: u32 = 0x0040;
    const SRCINVERT: u32 = 0x0066_0046;
    const CLR_NONE: u32 = 0xffff_ffff;
    const CLR_DEFAULT: u32 = 0xff00_0000;
    const ERROR_INVALID_PARAMETER: u32 = 87;
    const IMLDP_WORDS: u32 = 14; // 56-byte struct / 4 = 14 words
    const PARAMS_PTR: u32 = 0x3_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("image_list_draw_indirect");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    // 2x1 BMP: pixel 0 = magenta (0xFF00FF), pixel 1 = green (0x00FF00).
    fs::write(root.join("mg.bmp"), bmp_2x1_magenta_green_24bpp()).unwrap();
    fs::write(root.join("mask.bmp"), bmp_2x1_white_black_24bpp()).unwrap();
    fs::write(root.join("red.bmp"), bmp_2x1_red_red_24bpp()).unwrap();
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

    for invalid_size in [40, 60] {
        memory.write_word(PARAMS_PTR, invalid_size); // cbSize
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_IMAGE_LIST_DRAW_INDIRECT,
                [PARAMS_PTR],
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
        assert!(
            kernel
                .resources
                .image_list(il)
                .and_then(|list| list.last_draw)
                .is_none(),
            "ImageList_DrawIndirect should reject cbSize={invalid_size} before recording a draw"
        );
    }

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
        memory.read_u32(PARAMS_PTR + 48)?,
        ILD_TRANSPARENT,
        "CE mutates fStyle in IMAGELISTDRAWPARAMS when rgbBk=CLR_NONE forces transparent drawing"
    );
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

    // CE DrawIndirect uses dwRop for ILD_IMAGE when ILD_ROP is set.
    memory
        .write_u16(bits_ptr + 3 * 2, 0xf800)
        .expect("destination pixel should be writable");
    memory.write_word(PARAMS_PTR + 16, 3); // x=3
    memory.write_word(PARAMS_PTR + 20, 0); // y=0
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 32, 1); // xBitmap=1 -> green source pixel
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk
    memory.write_word(PARAMS_PTR + 44, CLR_DEFAULT); // rgbFg
    memory.write_word(PARAMS_PTR + 48, ILD_IMAGE | ILD_ROP); // fStyle
    memory.write_word(PARAMS_PTR + 52, SRCINVERT); // dwRop
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
        0xffe0,
        "ILD_ROP should apply dwRop instead of copying the source image"
    );
    let timing = kernel.display_perf_timing_bytes();
    assert_eq!(
        u32::from_le_bytes(timing[0..4].try_into().unwrap()),
        SRCINVERT,
        "CE ImageList_DrawIndirect ILD_IMAGE|ILD_ROP routes through StretchBlt_I with dwRop"
    );
    assert_eq!(u32::from_le_bytes(timing[4..8].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(timing[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(timing[12..16].try_into().unwrap()), 0);

    // CE's private ILD_BLENDMASK includes ILD_BLEND75 (0x0008). The CE blend
    // path treats non-BLEND50 styles as the 25% alpha branch for image-list DIBs.
    memory.write_word(PARAMS_PTR + 16, 2); // x=2
    memory.write_word(PARAMS_PTR + 20, 1); // y=1
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 32, 1); // xBitmap=1 -> green source pixel
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk
    memory.write_word(PARAMS_PTR + 44, 0x0000_00ff); // rgbFg=red COLORREF
    memory.write_word(PARAMS_PTR + 48, ILD_BLEND75); // fStyle
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
        rgb565_at(&memory, bits_ptr, stride, 2, 1),
        0x3de0,
        "ILD_BLEND75 should enter CE's blend path instead of drawing unchanged green"
    );

    // CE commctrl.h aliases ILD_SELECTED to ILD_BLEND50 and ILD_FOCUS to
    // ILD_BLEND25; both should flow through the same image-list blend branch.
    memory.write_word(PARAMS_PTR + 16, 0); // x=0
    memory.write_word(PARAMS_PTR + 20, 1); // y=1
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 32, 1); // xBitmap=1 -> green source pixel
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk
    memory.write_word(PARAMS_PTR + 44, 0x0000_00ff); // rgbFg=red COLORREF
    memory.write_word(PARAMS_PTR + 48, ILD_SELECTED); // fStyle=ILD_BLEND50 alias
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
        rgb565_at(&memory, bits_ptr, stride, 0, 1),
        0x7be0,
        "ILD_SELECTED should behave as CE's ILD_BLEND50 alias"
    );
    memory.write_word(PARAMS_PTR + 16, 1); // x=1
    memory.write_word(PARAMS_PTR + 48, ILD_FOCUS); // fStyle=ILD_BLEND25 alias
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
        rgb565_at(&memory, bits_ptr, stride, 1, 1),
        0x3de0,
        "ILD_FOCUS should behave as CE's ILD_BLEND25 alias"
    );

    // rgbFg=CLR_NONE follows CE's destination-blend branch for 16-bit image lists:
    // the existing destination pixel is blended with the source image at 50%.
    memory
        .write_u16(bits_ptr + stride + 3 * 2, 0xf800)
        .expect("destination pixel should be writable");
    memory.write_word(PARAMS_PTR + 16, 3); // x=3
    memory.write_word(PARAMS_PTR + 20, 1); // y=1
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 32, 1); // xBitmap=1 -> green source pixel
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk
    memory.write_word(PARAMS_PTR + 44, CLR_NONE); // rgbFg=destination blend
    memory.write_word(PARAMS_PTR + 48, 0x0004); // fStyle=ILD_BLEND50
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
        rgb565_at(&memory, bits_ptr, stride, 3, 1),
        0x7be0,
        "rgbFg=CLR_NONE should blend source green with the existing red destination pixel"
    );

    // CE does not apply that destination blend to ILD_MASK draws. The private
    // code ORs the mask with a 50% mono dither, sets ILD_TRANSPARENT, and then
    // draws the mask with SRCAND.
    let il_mask = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, ILC_COLOR16 | ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("ImageList_Create(mask blend) failed: {other:?}"),
    };
    memory.write_wide_z(path_ptr, r"\mask.bmp");
    let mask_handle = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("LoadImage mask failed: {other:?}"),
    };
    memory.write_wide_z(path_ptr, r"\mg.bmp");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [il_mask, bmp_handle, mask_handle],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    memory
        .write_u16(bits_ptr + stride, 0xffff)
        .expect("left mask destination should be writable");
    memory
        .write_u16(bits_ptr + stride + 2, 0xffff)
        .expect("right mask destination should be writable");
    memory.write_word(PARAMS_PTR + 4, il_mask); // himl=il_mask
    memory.write_word(PARAMS_PTR + 8, 0); // i=0
    memory.write_word(PARAMS_PTR + 16, 0); // x=0
    memory.write_word(PARAMS_PTR + 20, 1); // y=1
    memory.write_word(PARAMS_PTR + 24, 2); // cx=2
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 32, 0); // xBitmap=0
    memory.write_word(PARAMS_PTR + 36, 0); // yBitmap=0
    memory.write_word(PARAMS_PTR + 40, 0); // rgbBk=black, not CLR_NONE
    memory.write_word(PARAMS_PTR + 44, CLR_NONE); // rgbFg=destination/dither blend
    memory.write_word(PARAMS_PTR + 48, ILD_MASK | ILD_BLEND50); // fStyle
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
    assert_eq!(
        memory.read_u32(PARAMS_PTR + 48)?,
        ILD_MASK | ILD_BLEND50 | ILD_TRANSPARENT,
        "CE writes ILD_TRANSPARENT back after rgbFg=CLR_NONE blend-mask setup"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 0, 1),
        0xffff,
        "white mask pixels should survive the CE SRCAND mask pass"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 1, 1),
        0,
        "mask-only blend should dither/SRCAND instead of tinting the black mask pixel with the destination"
    );

    // With cx/cy set to zero, CE defaults to the remaining source rectangle after
    // xBitmap/yBitmap are applied. xBitmap=1 on a 2x1 image therefore draws one
    // pixel, not the full two-pixel image width.
    memory.write_word(PARAMS_PTR + 16, 0); // x=0
    memory.write_word(PARAMS_PTR + 20, 1); // y=1
    memory.write_word(PARAMS_PTR + 24, 0); // cx=0 -> image width - xBitmap
    memory.write_word(PARAMS_PTR + 28, 0); // cy=0 -> image height
    memory.write_word(PARAMS_PTR + 32, 1); // xBitmap=1
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk
    memory.write_word(PARAMS_PTR + 44, CLR_DEFAULT); // rgbFg
    memory.write_word(PARAMS_PTR + 48, 0); // fStyle
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
    let draw = kernel.resources.image_list(il).unwrap().last_draw.unwrap();
    assert_eq!(draw.width, 1);
    assert_eq!(draw.height, 1);
    assert_eq!(
        memory.read_i32(PARAMS_PTR + 24)?,
        1,
        "CE writes the defaulted cx back into IMAGELISTDRAWPARAMS"
    );
    assert_eq!(
        memory.read_i32(PARAMS_PTR + 28)?,
        1,
        "CE writes the defaulted cy back into IMAGELISTDRAWPARAMS"
    );
    assert_eq!(
        memory.read_u32(PARAMS_PTR + 48)?,
        ILD_TRANSPARENT,
        "CE writes the normalized transparent fStyle bit back into IMAGELISTDRAWPARAMS"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 0, 1),
        green_rgb565,
        "cx=0 with xBitmap=1 should default to the remaining one-pixel source width"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 1, 1),
        0,
        "cx=0 with xBitmap=1 should not draw the skipped source column"
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_BK_COLOR,
            [il2, 0x00ff_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(CLR_NONE),
            ..
        }
    ));

    // Draw with rgb_bk=CLR_NONE (ILD_TRANSPARENT forced): transparent pixel stays 0.
    memory.write_word(PARAMS_PTR + 4, il2); // himl=il2
    memory.write_word(PARAMS_PTR + 16, 2); // x=2
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 32, 0); // xBitmap=0
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk=CLR_NONE
    memory.write_word(PARAMS_PTR + 48, 0); // fStyle
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

    // Draw with rgb_bk=CLR_DEFAULT: CE resolves it to the image-list bk color.
    memory.write_word(PARAMS_PTR + 16, 0); // x=0
    memory.write_word(PARAMS_PTR + 20, 1); // y=1
    memory.write_word(PARAMS_PTR + 24, 1); // cx=1
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 40, CLR_DEFAULT); // rgbBk=image-list background
    memory.write_word(PARAMS_PTR + 48, 0); // fStyle
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
    let draw = kernel.resources.image_list(il2).unwrap().last_draw.unwrap();
    assert_eq!(draw.rgb_bk, 0x00ff_0000);
    assert_eq!(
        memory.read_u32(PARAMS_PTR + 40)?,
        0x00ff_0000,
        "CE writes the resolved image-list background color back into IMAGELISTDRAWPARAMS"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 0, 1),
        0x001f,
        "rgb_bk=CLR_DEFAULT should fill transparent pixels with the image-list bk color"
    );

    // Draw with rgb_bk=green (0x0000FF00): transparent area should be filled with green.
    memory.write_word(PARAMS_PTR + 16, 3); // x=3
    memory.write_word(PARAMS_PTR + 20, 0); // y=0
    memory.write_word(PARAMS_PTR + 40, 0x0000_ff00); // rgbBk=green
    memory.write_word(PARAMS_PTR + 48, 0); // fStyle
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

    // CE's overlay pass mutates the caller's IMAGELISTDRAWPARAMS again after
    // the base draw: i/x/y/cx/cy/fStyle become the overlay draw values.
    let overlay_list = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMAGE_LIST_CREATE,
        [2, 1, ILC_COLOR16 | ILC_MASK, 1, 1],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("ImageList_Create(overlay params) failed: {other:?}"),
    };
    memory.write_wide_z(path_ptr, r"\red.bmp");
    let overlay_bitmap = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("LoadImage overlay bitmap failed: {other:?}"),
    };
    memory.write_wide_z(path_ptr, r"\mask.bmp");
    let overlay_mask = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("LoadImage overlay mask failed: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_ADD,
            [overlay_list, overlay_bitmap, overlay_mask],
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
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            [overlay_list, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let overlay_record = kernel
        .resources
        .image_list(overlay_list)
        .unwrap()
        .overlays
        .get(&1)
        .copied()
        .expect("overlay bounds should be recorded");
    assert_eq!(overlay_record.x, 1);
    assert_eq!(overlay_record.width, 1);
    assert_eq!(overlay_record.flags, 0x0020);
    assert_eq!(
        kernel
            .resources
            .set_image_list_overlay_bounds(overlay_list, 1, 7, 8, 9, 10, 0),
        Some(true)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_SET_OVERLAY_IMAGE,
            [overlay_list, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let unchanged_overlay_record = kernel
        .resources
        .image_list(overlay_list)
        .unwrap()
        .overlays
        .get(&1)
        .copied()
        .expect("unchanged overlay bounds should be preserved");
    assert_eq!(unchanged_overlay_record.x, 7);
    assert_eq!(unchanged_overlay_record.y, 8);
    assert_eq!(unchanged_overlay_record.width, 9);
    assert_eq!(unchanged_overlay_record.height, 10);
    assert_eq!(unchanged_overlay_record.flags, 0);
    assert_eq!(
        kernel
            .resources
            .set_image_list_overlay_bounds(overlay_list, 1, 1, 0, 1, 1, 0x0020),
        Some(true)
    );

    memory.write_word(PARAMS_PTR, 56); // cbSize
    memory.write_word(PARAMS_PTR + 4, overlay_list); // himl
    memory.write_word(PARAMS_PTR + 8, 0); // i=0
    memory.write_word(PARAMS_PTR + 12, mem_dc); // hdcDst
    memory.write_word(PARAMS_PTR + 16, 0); // x=0
    memory.write_word(PARAMS_PTR + 20, 0); // y=0
    memory.write_word(PARAMS_PTR + 24, 2); // cx=2
    memory.write_word(PARAMS_PTR + 28, 1); // cy=1
    memory.write_word(PARAMS_PTR + 32, 0); // xBitmap=0
    memory.write_word(PARAMS_PTR + 36, 0); // yBitmap=0
    memory.write_word(PARAMS_PTR + 40, CLR_NONE); // rgbBk
    memory.write_word(PARAMS_PTR + 44, CLR_DEFAULT); // rgbFg
    memory.write_word(PARAMS_PTR + 48, 0x0100); // fStyle=overlay slot 1
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
    assert_eq!(
        memory.read_i32(PARAMS_PTR + 8)?,
        0,
        "CE rewrites i to the registered overlay image index"
    );
    assert_eq!(
        memory.read_i32(PARAMS_PTR + 16)?,
        1,
        "CE offsets x by the overlay mask bounds"
    );
    assert_eq!(
        memory.read_i32(PARAMS_PTR + 20)?,
        0,
        "CE offsets y by the overlay mask bounds"
    );
    assert_eq!(
        memory.read_i32(PARAMS_PTR + 24)?,
        1,
        "CE rewrites cx to the overlay mask width"
    );
    assert_eq!(
        memory.read_i32(PARAMS_PTR + 28)?,
        1,
        "CE rewrites cy to the overlay mask height"
    );
    assert_eq!(
        memory.read_u32(PARAMS_PTR + 48)?,
        ILD_TRANSPARENT | 0x0020,
        "CE strips the overlay mask and leaves the overlay pass fStyle"
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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

    memory.write_word(data + 12, NIF_MESSAGE | NIF_ICON | NIF_TIP);
    memory.write_word(data + 16, WM_USER + 99);
    memory.write_word(data + 20, 0x000b_8fff);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Duplicate should fail");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, data],
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
    let icon = kernel.shell.notify_icon(hwnd, 77).expect("notify icon");
    assert_eq!(icon.callback_message, WM_USER + 88);
    assert_eq!(icon.icon, 0x000b_8001);
    assert_eq!(icon.tip, "Route ready");

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

    memory.write_word(data + 12, NIF_ICON);
    memory.write_word(data + 20, 0);
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
    assert_eq!(icon.icon, 0x000b_8001);
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
fn shell_notify_icon_deletes_existing_record_after_owner_window_is_stale() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIM_MODIFY: u32 = 1;
    const NIM_DELETE: u32 = 2;
    const NIF_ICON: u32 = 0x0000_0002;
    const NIF_TIP: u32 = 0x0000_0004;
    const HHTBF_DESTROYICON: u32 = 0x1000_0000;
    const NID_SIZE: u32 = 160;
    const NID_TIP_OFFSET: u32 = 24;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHELL_NOTIFY_STALE", "", None, 0, 0, 0);
    let data = 0x2_f000;
    memory.map_words(data, NID_SIZE / 4);
    memory.map_halfwords(data + NID_TIP_OFFSET, 64);
    memory.write_word(data, NID_SIZE);
    memory.write_word(data + 4, hwnd);
    memory.write_word(data + 8, 91);
    memory.write_word(data + 12, NIF_ICON | NIF_TIP | HHTBF_DESTROYICON);
    memory.write_word(data + 20, 0x000c_9001);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Stale delete");

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
    assert!(kernel.shell.notify_icon(hwnd, 91).is_some());
    assert!(kernel.gwe.destroy_window(hwnd, kernel.timers.tick_count()));

    memory.write_word(data + 12, NIF_TIP);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Dead modify");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_MODIFY, data],
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
    let icon = kernel
        .shell
        .notify_icon(hwnd, 91)
        .expect("stale-owner notify icon remains after failed modify");
    assert_eq!(icon.tip, "Stale delete");

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
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert!(kernel.shell.notify_icon(hwnd, 91).is_none());
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000c_9001]
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
fn shell_notify_icon_posts_registered_taskbar_message_with_copied_data() -> Result<()> {
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
    const WM_HANDLESHELLNOTIFYICON: u32 = WM_USER + 0x0bad;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let app_hwnd = kernel.create_window_ex_w(thread_id, "SHELL_NOTIFY_APP", "", None, 0, 0, 0);
    let taskbar_hwnd =
        kernel.create_window_ex_w(thread_id, "HHTaskBar", "", None, 0, WS_VISIBLE, 0);
    let data = 0x2_d000;
    let msg_ptr = 0x2_e000;
    memory.map_words(data, NID_SIZE / 4);
    memory.map_halfwords(data + NID_TIP_OFFSET, 64);
    memory.map_words(msg_ptr, 7);
    memory.write_word(data, NID_SIZE);
    memory.write_word(data + 4, app_hwnd);
    memory.write_word(data + 8, 88);
    memory.write_word(data + 12, NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_STATE);
    memory.write_word(data + 16, WM_USER + 123);
    memory.write_word(data + 20, 0x000b_8123);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Taskbar copy");
    memory.write_word(data + NID_STATE_OFFSET, 0x4);
    memory.write_word(data + NID_STATE_MASK_OFFSET, 0xff);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_TASK_BAR,
            [taskbar_hwnd],
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
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, data],
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
                msg_ptr,
                taskbar_hwnd,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, taskbar_hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_HANDLESHELLNOTIFYICON);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, NIM_ADD);
    let copied_nid = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(copied_nid, 0);
    assert_eq!(
        kernel.gwe.message_pointer_payload(copied_nid),
        Some(MessagePointerPayload::NotifyIcon(
            wince_emulation_v3::ce::gwe::NotifyIconMessage {
                hwnd: app_hwnd,
                id: 88,
                flags: NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_STATE,
                callback_message: WM_USER + 123,
                icon: 0x000b_8123,
                tip: "Taskbar copy".to_owned(),
                state: 0x4,
                state_mask: 0xff,
            }
        ))
    );
    assert_eq!(memory.read_u32(copied_nid)?, 152);
    assert_eq!(memory.read_u32(copied_nid + 4)?, app_hwnd);
    assert_eq!(memory.read_u32(copied_nid + 8)?, 88);
    assert_eq!(
        memory.read_u32(copied_nid + 12)?,
        NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_STATE
    );
    assert_eq!(memory.read_u32(copied_nid + 16)?, WM_USER + 123);
    assert_eq!(memory.read_u32(copied_nid + 20)?, 0x000b_8123);
    assert_eq!(
        memory.read_wide_z(copied_nid + NID_TIP_OFFSET, 64),
        "Taskbar copy"
    );
    assert_eq!(memory.read_u32(copied_nid + NID_STATE_OFFSET)?, 0x4);
    assert_eq!(memory.read_u32(copied_nid + NID_STATE_MASK_OFFSET)?, 0xff);
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, copied_nid)
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
    assert!(kernel.gwe.message_pointer_payload(copied_nid).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, copied_nid)
            .is_none()
    );

    memory.write_wide_z(data + NID_TIP_OFFSET, "Duplicate should not post");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, data],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [
                msg_ptr,
                taskbar_hwnd,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    memory.write_word(data + 8, 89);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_MODIFY, data],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [
                msg_ptr,
                taskbar_hwnd,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    memory.write_word(data + 8, 88);
    memory.write_word(data + 12, NIF_TIP);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Taskbar modify");
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [
                msg_ptr,
                taskbar_hwnd,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, taskbar_hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_HANDLESHELLNOTIFYICON);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, NIM_MODIFY);
    let copied_modify = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(copied_modify, 0);
    assert_eq!(
        memory.read_wide_z(copied_modify + NID_TIP_OFFSET, 64),
        "Taskbar modify"
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
    assert!(kernel.gwe.message_pointer_payload(copied_modify).is_none());

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
    assert!(kernel.shell.notify_icon(app_hwnd, 88).is_none());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [
                msg_ptr,
                taskbar_hwnd,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, taskbar_hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_HANDLESHELLNOTIFYICON);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, NIM_DELETE);
    let copied_delete = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(copied_delete, 0);
    assert_eq!(
        memory.read_wide_z(copied_delete + NID_TIP_OFFSET, 64),
        "Taskbar modify"
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
    assert!(kernel.gwe.message_pointer_payload(copied_delete).is_none());

    memory.write_word(data + 12, NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_STATE);
    memory.write_word(data + 16, WM_USER + 123);
    memory.write_word(data + 20, 0x000b_8123);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Taskbar copy again");
    memory.write_word(data + NID_STATE_OFFSET, 0x4);
    memory.write_word(data + NID_STATE_MASK_OFFSET, 0xff);
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [
                msg_ptr,
                taskbar_hwnd,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let copied_readd = memory.read_u32(msg_ptr + 12)?;
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
    assert!(kernel.gwe.message_pointer_payload(copied_readd).is_none());

    assert!(kernel.destroy_window(taskbar_hwnd));
    memory.write_word(data + 12, NIF_TIP);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Stale taskbar HWND");
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
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let icon = kernel
        .shell
        .notify_icon(app_hwnd, 88)
        .expect("notify icon after stale taskbar update");
    assert_eq!(icon.tip, "Stale taskbar HWND");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [
                msg_ptr,
                0,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_TASK_BAR,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    memory.write_word(data + 12, NIF_TIP);
    memory.write_wide_z(data + NID_TIP_OFFSET, "No taskbar post");
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [
                msg_ptr,
                taskbar_hwnd,
                WM_HANDLESHELLNOTIFYICON,
                WM_HANDLESHELLNOTIFYICON,
                1,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn dispatch_message_releases_only_matching_private_pointer_payload_type() -> Result<()> {
    const WM_HANDLESHELLNOTIFYICON: u32 = WM_USER + 0x0bad;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let hwnd = kernel.create_window_ex_w(thread_id, "PRIVATE_PAYLOAD", "", None, 0, 0, 0);
    let msg_ptr = 0x2_f000;
    memory.map_words(msg_ptr, 7);
    let payload_ptr = kernel
        .memory
        .heap_alloc(PROCESS_HEAP_HANDLE, 0, 28)
        .expect("private payload allocation");
    let payload = MessagePointerPayload::WindowPos(WindowPos {
        hwnd,
        insert_after: 0,
        x: 1,
        y: 2,
        width: 3,
        height: 4,
        flags: 5,
    });
    assert!(
        kernel
            .gwe
            .insert_message_pointer_payload(payload_ptr, payload.clone())
    );
    memory.write_word(msg_ptr, hwnd);
    memory.write_word(msg_ptr + 4, WM_HANDLESHELLNOTIFYICON);
    memory.write_word(msg_ptr + 8, 0);
    memory.write_word(msg_ptr + 12, payload_ptr);
    memory.write_word(msg_ptr + 16, 0);

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
    assert_eq!(
        kernel.gwe.message_pointer_payload(payload_ptr),
        Some(payload)
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, payload_ptr)
            .is_some()
    );

    memory.write_word(msg_ptr + 4, WM_WINDOWPOSCHANGED);
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
    assert!(kernel.gwe.message_pointer_payload(payload_ptr).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, payload_ptr)
            .is_none()
    );

    Ok(())
}

#[test]
fn shell_notify_icon_add_respects_member_flags() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIF_TIP: u32 = 0x0000_0004;
    const NIF_STATE: u32 = 0x0000_0008;
    const NID_SIZE: u32 = 160;
    const NID_TIP_OFFSET: u32 = 24;
    const NID_STATE_OFFSET: u32 = 152;
    const NID_STATE_MASK_OFFSET: u32 = 156;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHELL_NOTIFY_FLAGS", "", None, 0, 0, 0);
    let data = 0x2_a000;
    memory.map_words(data, NID_SIZE / 4);
    memory.map_halfwords(data + NID_TIP_OFFSET, 64);
    memory.write_word(data, NID_SIZE);
    memory.write_word(data + 4, hwnd);
    memory.write_word(data + 8, 12);
    memory.write_word(data + 12, NIF_TIP | NIF_STATE);
    memory.write_word(data + 16, WM_USER + 99);
    memory.write_word(data + 20, 0x000b_8aaa);
    memory.write_wide_z(data + NID_TIP_OFFSET, "Visible tip");
    memory.write_word(data + NID_STATE_OFFSET, 0b1010);
    memory.write_word(data + NID_STATE_MASK_OFFSET, 0b0110);

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
    let icon = kernel.shell.notify_icon(hwnd, 12).expect("notify icon");
    assert_eq!(icon.callback_message, 0);
    assert_eq!(icon.icon, 0);
    assert_eq!(icon.tip, "Visible tip");
    assert_eq!(icon.state, 0b0010);
    assert!(!kernel.post_shell_notify_icon_callback(hwnd, 12, WM_LBUTTONDOWN));

    Ok(())
}

#[test]
fn shell_notify_icon_uses_trap_size_to_bound_optional_state() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIM_MODIFY: u32 = 1;
    const NIF_TIP: u32 = 0x0000_0004;
    const NIF_STATE: u32 = 0x0000_0008;
    const CE_NID_SIZE: u32 = 152;
    const EXTENDED_NID_SIZE: u32 = 160;
    const NID_TIP_OFFSET: u32 = 24;
    const NID_STATE_OFFSET: u32 = 152;
    const NID_STATE_MASK_OFFSET: u32 = 156;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHELL_NOTIFY_TRAP_SIZE", "", None, 0, 0, 0);
    let data = 0x2_a800;
    memory.map_words(data, EXTENDED_NID_SIZE / 4);
    memory.map_halfwords(data + NID_TIP_OFFSET, 64);
    memory.write_word(data, EXTENDED_NID_SIZE);
    memory.write_word(data + 4, hwnd);
    memory.write_word(data + 8, 13);
    memory.write_word(data + 12, NIF_TIP | NIF_STATE);
    memory.write_wide_z(data + NID_TIP_OFFSET, "CE-sized trap");
    memory.write_word(data + NID_STATE_OFFSET, 0xffff_ffff);
    memory.write_word(data + NID_STATE_MASK_OFFSET, 0xffff_ffff);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, data, CE_NID_SIZE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let icon = kernel.shell.notify_icon(hwnd, 13).expect("notify icon");
    assert_eq!(icon.tip, "CE-sized trap");
    assert_eq!(
        icon.state, 0,
        "CE shell trap size should prevent reading optional state bytes"
    );

    memory.write_word(data + 12, NIF_STATE);
    memory.write_word(data + NID_STATE_OFFSET, 0x2);
    memory.write_word(data + NID_STATE_MASK_OFFSET, 0x2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_MODIFY, data, CE_NID_SIZE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let icon = kernel.shell.notify_icon(hwnd, 13).expect("notify icon");
    assert_eq!(
        icon.state, 0,
        "CE-sized modify should leave extended state untouched"
    );

    Ok(())
}

#[test]
fn shell_notify_icon_requires_ce_fixed_notifyicondata_size() -> Result<()> {
    const NIM_ADD: u32 = 0;
    const NIF_TIP: u32 = 0x0000_0004;
    const HEADER_ONLY_NID_SIZE: u32 = 24;
    const CE_NID_SIZE: u32 = 152;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHELL_NOTIFY_SHORT", "", None, 0, 0, 0);
    let data = 0x2_b000;
    memory.map_words(data, HEADER_ONLY_NID_SIZE / 4);
    memory.write_word(data, HEADER_ONLY_NID_SIZE);
    memory.write_word(data + 4, hwnd);
    memory.write_word(data + 8, 23);
    memory.write_word(data + 12, NIF_TIP);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, data],
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
    assert!(kernel.shell.notify_icon(hwnd, 23).is_none());

    let bad_tip_data = 0x2_c000;
    memory.map_words(bad_tip_data, HEADER_ONLY_NID_SIZE / 4);
    memory.write_word(bad_tip_data, CE_NID_SIZE);
    memory.write_word(bad_tip_data + 4, hwnd);
    memory.write_word(bad_tip_data + 8, 24);
    memory.write_word(bad_tip_data + 12, NIF_TIP);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHELL_NOTIFY_ICON,
            [NIM_ADD, bad_tip_data],
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
    assert!(kernel.shell.notify_icon(hwnd, 24).is_none());

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
    let config = RuntimeConfig::load_default()?;
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
    let out_title_zero_cb = 0x3004_0000;
    let msg_ptr = 0x3002_fc00;
    let clsid = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x44, 0x33, 0x22, 0x11, 0xaa, 0xbb, 0xcc,
        0xdd,
    ];
    kernel.com.register_class_guid(clsid, 0x000c_0f00);
    memory.map_words(data, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(title, 64);
    memory.map_bytes(html, 128);
    memory.map_words(out, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(out_title, 64);
    memory.map_bytes(out_html, 128);
    memory.map_words(html_len, 1);
    memory.map_bytes(out_title_zero_cb, 520);
    memory.map_words(msg_ptr, 7);
    memory.write_word(data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(data + 4, 301);
    memory.write_word(data + 8, SHNP_INFORM);
    memory.write_word(data + 12, 0);
    memory.write_word(data + 16, 0x000b_9001);
    memory.write_word(data + 20, SHNUM_TITLE | SHNUM_HTML);
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
    memory.write_word(data + 40, 0);
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
    assert_eq!(record.callback_ptr, 0);
    assert_eq!(
        kernel
            .shell
            .notification_priority_keys(SHNP_INFORM)
            .collect::<Vec<_>>(),
        vec![(clsid, 301)]
    );
    assert_eq!(
        kernel
            .shell
            .notification_priority_keys(SHNP_ICONIC)
            .collect::<Vec<_>>(),
        Vec::<([u8; 16], u32)>::new()
    );
    assert!(kernel.post_shell_notification_callback(clsid, 301, SHNN_SHOW, 0, 0));
    let callbacks = kernel
        .shell
        .notification_callbacks()
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        callbacks.is_empty(),
        "CE taskbar PopUp sends SHNN_SHOW to the sink window without invoking the COM callback"
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
    assert_eq!(callbacks.len(), 1);
    assert_eq!(
        callbacks[0].method,
        ShellNotificationCallbackMethod::OnLinkSelected {
            link: "cmd:route".to_owned()
        }
    );
    assert_eq!(callbacks[0].callback_ptr, 0x000c_0f00);
    assert_eq!(
        callbacks[0].vtable_offset,
        ISHELL_NOTIFICATION_CALLBACK_ON_LINK_SELECTED_VTABLE_OFFSET
    );
    assert_eq!(
        callbacks[0].arguments,
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
    assert_eq!(callbacks.len(), 2);
    assert_eq!(
        callbacks[1].method,
        ShellNotificationCallbackMethod::OnDismiss { timed_out: true }
    );
    assert_eq!(callbacks[1].callback_ptr, 0x000c_0f00);
    assert_eq!(
        callbacks[1].vtable_offset,
        ISHELL_NOTIFICATION_CALLBACK_ON_DISMISS_VTABLE_OFFSET
    );
    assert_eq!(
        callbacks[1].arguments,
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
    assert_eq!(callbacks.len(), 3);
    assert_eq!(
        callbacks[2].method,
        ShellNotificationCallbackMethod::OnCommandSelected { command_id: 0x1234 }
    );
    assert_eq!(callbacks[2].callback_ptr, 0x000c_0f00);
    assert_eq!(
        callbacks[2].vtable_offset,
        ISHELL_NOTIFICATION_CALLBACK_ON_COMMAND_SELECTED_VTABLE_OFFSET
    );
    assert_eq!(
        callbacks[2].arguments,
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
    assert_eq!(memory.read_u32(html_len)?, 17);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_GET_DATA_I,
            [data + 24, 16, 301, 0, 0, out_title_zero_cb, 0, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(out_title_zero_cb, 64), "Route alert");

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
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.hwnd_sink, hwnd);
    assert_eq!(record.title, "Ignored sink");
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
    assert_eq!(record.title, "Ignored sink");
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
    assert_eq!(record.flags, SHNUM_TITLE | SHNUM_HTML);
    assert_eq!(record.hwnd_sink, hwnd);
    assert_eq!(record.lparam, 0xCAFE_BABE);
    assert_eq!(record.title, "Route changed");
    assert_eq!(record.html, "<i>Later</i>");
    assert_eq!(
        kernel
            .shell
            .notification_priority_keys(SHNP_INFORM)
            .collect::<Vec<_>>(),
        Vec::<([u8; 16], u32)>::new()
    );
    assert_eq!(
        kernel
            .shell
            .notification_priority_keys(SHNP_ICONIC)
            .collect::<Vec<_>>(),
        vec![(clsid, 301)]
    );
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_9001]
    );

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
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_9001]
    );

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

    memory.write_word(data + 12, 9);
    memory.write_word(data + 16, 0x000b_9003);
    memory.write_word(data + 20, SHNUM_TITLE | SHNUM_HTML);
    memory.write_wide_z(title, "Duplicate iconic");
    memory.write_wide_z(html, "<p>Duplicate</p>");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_ADD_I,
            [data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_DATA),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 301).expect("notification");
    assert_eq!(record.priority, SHNP_ICONIC);
    assert_eq!(record.duration_cs, 0);
    assert_eq!(record.icon, 0x000b_9002);
    assert_eq!(record.title, "Route changed");
    assert_eq!(record.html, "<i>Later</i>");
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_9001],
        "duplicate iconic add should not replace or destroy the existing CE tray icon"
    );

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
    assert_eq!(memory.read_u32(out + 20)?, SHNUM_TITLE | SHNUM_HTML);
    assert_eq!(memory.read_u32(out + 40)?, hwnd);
    assert_eq!(memory.read_u32(out + 52)?, 0xCAFE_BABE);

    memory.write_wide_z(out_html, "stale html");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_GET_DATA_I,
            [data + 24, 16, 301, 0, 0, 0, 0, out_html, 128, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(
        memory.read_wide_z(out_html, 32),
        "",
        "CE GetNotificationData ignores cbHTML without pdwHTMLLength and returns an empty HTML buffer"
    );

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
    assert_eq!(kernel.shell.notification_callbacks().count(), 0);
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_9001, 0x000b_9002]
    );
    assert_eq!(
        kernel
            .shell
            .notification_priority_keys(SHNP_ICONIC)
            .collect::<Vec<_>>(),
        Vec::<([u8; 16], u32)>::new()
    );
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
fn shnotification_i_unregistered_callback_clsid_falls_back_to_sink_window() -> Result<()> {
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_INFORM: u32 = 0x1b1;
    const SHNN_LINKSEL: u32 = 0xffff_fc18;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHN_UNREGISTERED_COM", "", None, 0, 0, 0);
    let data = 0x3006_0000;
    let title = 0x3006_1000;
    let html = 0x3006_2000;
    let msg_ptr = 0x3006_3000;
    let clsid = [
        0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
        0xb0,
    ];
    memory.map_words(data, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(title, 64);
    memory.map_bytes(html, 64);
    memory.map_words(msg_ptr, 7);
    memory.write_word(data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(data + 4, 902);
    memory.write_word(data + 8, SHNP_INFORM);
    memory.write_word(data + 12, 0);
    memory.write_word(data + 16, 0);
    memory.write_word(data + 20, 0x10);
    memory.write_bytes(data + 24, &clsid);
    memory.write_word(data + 40, hwnd);
    memory.write_word(data + 52, 0x0bad_f00d);
    memory.write_wide_z(title, "Window fallback");
    memory.write_wide_z(html, "<a href=\"cmd:open\">Open</a>");

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

    assert!(kernel.post_shell_notification_link_callback(clsid, 902, "cmd:open"));
    assert_eq!(kernel.shell.notification_callbacks().count(), 0);
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
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 902);
    let nmshn_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(nmshn_ptr, 0);
    assert_eq!(memory.read_u32(nmshn_ptr + 8)?, SHNN_LINKSEL);
    assert_eq!(memory.read_u32(nmshn_ptr + 12)?, 0x0bad_f00d);
    let link_ptr = memory.read_u32(nmshn_ptr + 20)?;
    assert_eq!(memory.read_wide_z(link_ptr, 32), "cmd:open");

    Ok(())
}

#[test]
fn shnotification_i_add_uses_marshalled_html_pointer_presence() -> Result<()> {
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_INFORM: u32 = 0x1b1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHN_HTML", "", None, 0, 0, 0);
    let data = 0x3006_0000;
    let title = 0x3006_1000;
    let empty_html = 0x3006_2000;
    let clsid = [
        0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
        0x30,
    ];
    memory.map_words(data, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(title, 64);
    memory.map_bytes(empty_html, 4);
    memory.write_word(data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(data + 4, 901);
    memory.write_word(data + 8, SHNP_INFORM);
    memory.write_word(data + 12, 0);
    memory.write_word(data + 20, 0x10);
    memory.write_bytes(data + 24, &clsid);
    memory.write_word(data + 40, hwnd);
    memory.write_wide_z(title, "Title only");
    memory.write_wide_z(empty_html, "");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_ADD_I,
            [data, SHNOTIFICATIONDATA_SIZE, title, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_INVALID_PARAMETER),
            ..
        }
    ));
    assert!(kernel.shell.notification(clsid, 901).is_none());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_ADD_I,
            [data, SHNOTIFICATIONDATA_SIZE, 0, empty_html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 901).expect("notification");
    assert_eq!(record.title, "");
    assert_eq!(record.html, "");
    assert_eq!(record.duration_cs, 5);

    Ok(())
}

#[test]
fn shnotification_i_add_captures_content_by_grf_flags_like_ce() -> Result<()> {
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_ICONIC: u32 = 0x1b2;
    const SHNUM_HTML: u32 = 0x0008;
    const SHNUM_TITLE: u32 = 0x0010;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHN_FLAGS", "", None, 0, 0, 0);
    let data = 0x3006_8000;
    let title = 0x3006_9000;
    let html = 0x3006_a000;
    let html_len = 0x3006_b000;
    let clsid = [
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
        0x40,
    ];
    memory.map_words(data, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(title, 64);
    memory.map_bytes(html, 128);
    memory.map_words(html_len, 1);
    memory.write_word(data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(data + 4, 904);
    memory.write_word(data + 8, SHNP_ICONIC);
    memory.write_word(data + 12, 0);
    memory.write_word(data + 20, SHNUM_TITLE);
    memory.write_bytes(data + 24, &clsid);
    memory.write_word(data + 40, hwnd);
    memory.write_wide_z(title, "Flagged title");
    memory.write_wide_z(html, "<b>Pointer present</b>");

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
    let record = kernel.shell.notification(clsid, 904).expect("notification");
    assert_eq!(record.title, "Flagged title");
    assert_eq!(
        record.html, "",
        "CE TaskbarBubble only captures HTML on add when SHNUM_HTML is set"
    );
    memory.write_word(html_len, 0xffff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_GET_DATA_I,
            [data + 24, 16, 904, 0, 0, 0, 0, 0, 0, html_len],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(html_len)?,
        0,
        "CE reports no HTML allocation when SHNUM_HTML was not captured"
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_HTML, data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 904).expect("notification");
    assert_eq!(record.title, "Flagged title");
    assert_eq!(record.html, "<b>Pointer present</b>");
    memory.write_word(html_len, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_GET_DATA_I,
            [data + 24, 16, 904, 0, 0, 0, 0, 0, 0, html_len],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(html_len)?, 23);

    memory.write_wide_z(html, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_HTML, data, SHNOTIFICATIONDATA_SIZE, title, html],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    memory.write_word(html_len, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_GET_DATA_I,
            [data + 24, 16, 904, 0, 0, 0, 0, 0, 0, html_len],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(html_len)?,
        1,
        "CE distinguishes captured empty HTML from no HTML allocation"
    );

    Ok(())
}

#[test]
fn shnotification_i_clears_overlong_taskbar_titles_like_ce() -> Result<()> {
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_ICONIC: u32 = 0x1b2;
    const SHNUM_TITLE: u32 = 0x0010;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 47;
    let hwnd = kernel.create_window_ex_w(thread_id, "SHN_LONG_TITLE", "", None, 0, 0, 0);
    let data = 0x3007_0000;
    let title = 0x3007_1000;
    let clsid = [
        0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
        0xd0,
    ];
    memory.map_words(data, SHNOTIFICATIONDATA_SIZE / 4);
    memory.map_bytes(title, 600);
    memory.write_word(data, SHNOTIFICATIONDATA_SIZE);
    memory.write_word(data + 4, 903);
    memory.write_word(data + 8, SHNP_ICONIC);
    memory.write_word(data + 12, 0);
    memory.write_word(data + 20, SHNUM_TITLE);
    memory.write_bytes(data + 24, &clsid);
    memory.write_word(data + 40, hwnd);

    let overlong_title = "T".repeat(260);
    memory.write_wide_z(title, &overlong_title);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_ADD_I,
            [data, SHNOTIFICATIONDATA_SIZE, title, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 903).expect("notification");
    assert_eq!(record.title, "");

    memory.write_wide_z(title, "Short title");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_TITLE, data, SHNOTIFICATIONDATA_SIZE, title, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 903).expect("notification");
    assert_eq!(record.title, "Short title");

    memory.write_wide_z(title, &overlong_title);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHNOTIFICATION_UPDATE_I,
            [SHNUM_TITLE, data, SHNOTIFICATIONDATA_SIZE, title, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let record = kernel.shell.notification(clsid, 903).expect("notification");
    assert_eq!(record.title, "");

    Ok(())
}

#[test]
fn shnotification_i_posts_timeout_dismiss_and_removes_expired_record() -> Result<()> {
    const SHNOTIFICATIONDATA_SIZE: u32 = 56;
    const SHNP_ICONIC: u32 = 0x1b2;
    const SHNN_DISMISS: u32 = 0xffff_fc17;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    memory.write_word(data + 8, SHNP_ICONIC);
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
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SLEEP,
            [998]
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_NOTIFY, WM_NOTIFY, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_SLEEP, [1]),
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
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_9001]
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

    Ok(())
}

#[test]
fn sh_change_notify_i_tracks_register_remove_and_free_state() -> Result<()> {
    const ERROR_INVALID_WINDOW_HANDLE: u32 = 1400;
    const SHCNE_CREATE: u32 = 0x0000_0002;
    const SHCNE_DELETE: u32 = 0x0000_0004;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    const SHNP_ICONIC: u32 = 0x1b2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    kernel.com.register_class_guid(clsid, 0x000c_0f04);
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
    memory.write_word(notify_data + 8, SHNP_ICONIC);
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
    assert!(kernel.post_shell_notification_dismiss_callback(clsid, 402, false));
    assert_eq!(kernel.shell.notification_callbacks().count(), 1);

    assert!(kernel.destroy_window(hwnd));

    assert!(kernel.shell.notify_icon(hwnd, 11).is_none());
    assert_eq!(
        kernel
            .shell
            .destroyed_notify_icons()
            .copied()
            .collect::<Vec<_>>(),
        vec![0x000b_8002, 0x000b_9003]
    );
    assert!(kernel.shell.notification(clsid, 402).is_none());
    assert_eq!(kernel.shell.notification_callbacks().count(), 0);
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
    memory.write_word(notify_data + 8, SHNP_ICONIC);
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
        vec![0x000b_8002, 0x000b_9003, 0x000b_8003, 0x000b_9004]
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
    const MB_SETFOREGROUND: u32 = 0x0001_0000;
    const MB_TOPMOST: u32 = 0x0004_0000;
    const MB_RTLREADING: u32 = 0x0010_0000;
    const MB_UNDEFINED_ICON_NIBBLE: u32 = 0x0000_0050;
    const MB_DESKTOP_ONLY_TASKMODAL: u32 = 0x0000_2000;
    const WS_EX_TOPMOST: u32 = 0x0000_0008;
    const IDCANCEL: u32 = 2;
    const IDYES: u32 = 6;
    const IDNO: u32 = 7;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    assert!(record.dialog_was_active);
    assert_eq!(record.dialog_ex_style & WS_EX_TOPMOST, 0);
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

    let supported_high_flags = MB_CANCEL | MB_SETFOREGROUND | MB_TOPMOST | MB_RTLREADING;
    let supported_high = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, supported_high_flags],
    );
    assert!(matches!(
        supported_high,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDCANCEL),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("supported-high-flags record");
    assert_eq!(record.style, supported_high_flags);
    assert_eq!(record.result, IDCANCEL);
    assert!(record.dialog_was_active);
    assert_eq!(record.dialog_ex_style & WS_EX_TOPMOST, WS_EX_TOPMOST);

    let record_count = kernel.shell.message_boxes().count();
    let unsupported_desktop_flag = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_CANCEL | MB_DESKTOP_ONLY_TASKMODAL],
    );
    assert!(matches!(
        unsupported_desktop_flag,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    assert_eq!(kernel.shell.message_boxes().count(), record_count);

    let unsupported_icon_flag = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [hwnd, text, caption, MB_CANCEL | MB_UNDEFINED_ICON_NIBBLE],
    );
    assert!(matches!(
        unsupported_icon_flag,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    assert_eq!(kernel.shell.message_boxes().count(), record_count);

    Ok(())
}

#[test]
fn message_box_w_restores_previous_owner_focus_after_default_teardown() -> Result<()> {
    const MB_OK: u32 = 0x0000_0000;
    const IDOK: u32 = 1;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 49;
    let owner = kernel.create_window_ex_w(thread_id, "MSGBOX_RESTORE_OWNER", "", None, 0, 0, 0);
    let owner_child = kernel.create_window_ex_w(
        thread_id,
        "Button",
        "Owner child",
        Some(owner),
        77,
        WS_CHILD | WS_VISIBLE,
        0,
    );
    let text = 0x3004_3000;
    let caption = 0x3004_4000;
    memory.write_wide_z(text, "Restore focus");
    memory.write_wide_z(caption, "iNavi");

    assert_eq!(kernel.set_focus(Some(owner_child)), None);
    assert_eq!(kernel.gwe.get_focus(), Some(owner_child));
    assert!(kernel.gwe.active_window_is_within(owner));

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MESSAGE_BOX_W,
        [owner, text, caption, MB_OK],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(IDOK),
            ..
        }
    ));
    let record = kernel
        .shell
        .last_message_box()
        .expect("focus-restore message box record");
    assert_eq!(record.owner_hwnd, owner);
    assert_eq!(record.result, IDOK);
    assert_eq!(record.owner_was_enabled, Some(true));
    assert!(record.dialog_was_active);
    assert!(!kernel.gwe.is_window(record.dialog_hwnd));
    assert_eq!(kernel.gwe.get_focus(), Some(owner_child));
    assert!(kernel.gwe.active_window_is_within(owner));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
fn coredll_multibyte_to_wide_char_handles_ascii_acp() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let input = 0x1000;
    let output = 0x2000;
    memory.write_bytes(input, b"Car\\2\\223001\0");
    memory.map_halfwords(output, 16);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MULTI_BYTE_TO_WIDE_CHAR,
            [0, 0, input, u32::MAX, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(13),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MULTI_BYTE_TO_WIDE_CHAR,
            [0, 0, input, u32::MAX, output, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(13),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(output, 16), "Car\\2\\223001");
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_multibyte_to_wide_char_explicit_ascii_len_omits_nul() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let input = 0x1000;
    let output = 0x2000;
    memory.write_bytes(input, b"ABC\0");
    memory.map_halfwords(output, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MULTI_BYTE_TO_WIDE_CHAR,
            [0, 0, input, 3, output, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(3),
            ..
        }
    ));
    assert_eq!(memory.read_u16(output)?, u16::from(b'A'));
    assert_eq!(memory.read_u16(output + 2)?, u16::from(b'B'));
    assert_eq!(memory.read_u16(output + 4)?, u16::from(b'C'));
    assert_eq!(memory.read_u16(output + 6)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_wide_char_to_multi_byte_uses_korean_acp() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
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

    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
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
    let config = RuntimeConfig::load_default()?;
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
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let remote_target = kernel.remote_gps_target();
    let com = kernel.create_file_w(&remote_target, GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
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
    const MS_CTS_ON: u32 = 0x0010;
    const MS_DSR_ON: u32 = 0x0020;
    const MS_RLSD_ON: u32 = 0x0080;
    const PURGE_TXCLEAR: u32 = 0x0004;
    const PURGE_RXCLEAR: u32 = 0x0008;
    const COMMPROP_SIZE: u32 = 64;

    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.devices = DeviceConfigFile {
        version: 1,
        defaults: DeviceDefaults::default(),
        devices: vec![DeviceConfig {
            guest: "COM7:".to_owned(),
            kind: DeviceKind::Serial,
            backend: DeviceBackend::Stub,
            host: None,
            remote_gps: false,
            enabled: true,
            note: None,
        }],
    };
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let remote_target = kernel.remote_gps_target();
    let com = kernel.create_file_w(&remote_target, GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
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

    let comm_prop_ptr = 0x3100_1100;
    memory.map_bytes(comm_prop_ptr, COMMPROP_SIZE);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_COMM_PROPERTIES,
            [com, 0]
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
            ORD_GET_COMM_PROPERTIES,
            [0xdead_beef, comm_prop_ptr]
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_COMM_PROPERTIES,
            [com, comm_prop_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let comm_prop = memory.read_bytes(comm_prop_ptr, COMMPROP_SIZE as usize);
    let prop_u16 = |offset: usize| u16::from_le_bytes([comm_prop[offset], comm_prop[offset + 1]]);
    let prop_u32 = |offset: usize| {
        u32::from_le_bytes([
            comm_prop[offset],
            comm_prop[offset + 1],
            comm_prop[offset + 2],
            comm_prop[offset + 3],
        ])
    };
    assert_eq!(prop_u16(0), 0xffff);
    assert_eq!(prop_u16(2), 0xffff);
    assert_eq!(prop_u32(4), 0x0000_0001);
    assert_eq!(prop_u32(12), 16);
    assert_eq!(prop_u32(16), 16);
    assert_eq!(prop_u32(20), 0x0002_0000);
    assert_eq!(prop_u32(24), 0x0000_0001);
    assert_eq!(prop_u32(28), 0x0000_01ff);
    assert_eq!(prop_u32(32), 0x0000_007f);
    assert_eq!(prop_u32(36), 0x1007_fffb);
    assert_eq!(prop_u16(40), 0x000f);
    assert_eq!(prop_u16(42), 0x1f05);
    assert_eq!(prop_u32(44), 0);
    assert_eq!(prop_u32(48), 0);
    assert_eq!(prop_u32(52), 0);
    assert_eq!(prop_u32(56), 0);
    assert_eq!(prop_u16(60), 0);

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

    let modem_status_ptr = 0x3100_5000;
    memory.map_words(modem_status_ptr, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_COMM_MODEM_STATUS,
            [com, modem_status_ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(modem_status_ptr)?,
        MS_CTS_ON | MS_DSR_ON | MS_RLSD_ON
    );

    let response = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "nmea",
        "sentences": ["$GPRMC,raw*00"]
    }));
    assert_eq!(response["accepted"], 1);
    assert!(kernel.serial_read_ready(com));
    assert!(kernel.serial_comm_event_ready(com));
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
    let config = RuntimeConfig::load_default()?;
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
