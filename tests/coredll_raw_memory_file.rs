use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        cemath::CeMathValue,
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_ACOS, ORD_AFS_CLOSE_ALL_FILE_HANDLES, ORD_AFS_CREATE_DIRECTORY_W,
            ORD_AFS_CREATE_FILE_W, ORD_AFS_DELETE_FILE_W, ORD_AFS_FIND_FIRST_CHANGE_NOTIFICATION_W,
            ORD_AFS_FIND_FIRST_FILE_W, ORD_AFS_FS_IO_CONTROL_W, ORD_AFS_GET_DISK_FREE_SPACE,
            ORD_AFS_GET_FILE_ATTRIBUTES_W, ORD_AFS_GET_FILE_SECURITY_W, ORD_AFS_MOVE_FILE_W,
            ORD_AFS_NOTIFY_MOUNTED_FS, ORD_AFS_PRESTO_CHANGO_FILE_NAME,
            ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION, ORD_AFS_REMOVE_DIRECTORY_W,
            ORD_AFS_SET_FILE_ATTRIBUTES_W, ORD_AFS_SET_FILE_SECURITY_W, ORD_AFS_UNMOUNT, ORD_ASIN,
            ORD_ATAN, ORD_ATAN2, ORD_ATOF, ORD_ATOI, ORD_CE_FS_IO_CONTROL_W,
            ORD_CE_GET_FILE_NOTIFICATION_INFO, ORD_CE_GET_VOLUME_INFO_W,
            ORD_CE_REGISTER_FILE_SYSTEM_NOTIFICATION, ORD_CEIL, ORD_CHAR_LOWER_BUFF_W,
            ORD_CHAR_LOWER_W, ORD_CHAR_UPPER_BUFF_W, ORD_CHAR_UPPER_W, ORD_CLEAR_COMM_BREAK,
            ORD_CLOSE_HANDLE, ORD_COPY_FILE_W, ORD_COS, ORD_COSH, ORD_CREATE_DIRECTORY_W,
            ORD_CREATE_FILE_MAPPING_W, ORD_CREATE_FILE_W, ORD_CREATE_PARTITION,
            ORD_CREATE_PARTITION_EX, ORD_D_TO_ULL, ORD_DELETE_AND_RENAME_FILE, ORD_DELETE_FILE_W,
            ORD_DELETE_PARTITION, ORD_DEREGISTER_AFS, ORD_DEREGISTER_AFSNAME,
            ORD_DEVICE_IO_CONTROL, ORD_DISMOUNT_PARTITION, ORD_DISMOUNT_STORE, ORD_DPA_CLONE,
            ORD_DPA_CREATE, ORD_DPA_DESTROY, ORD_DPA_DESTROY_CALLBACK, ORD_DPA_ENUM_CALLBACK,
            ORD_DPA_GET_PTR, ORD_DPA_GROW, ORD_DPA_INSERT_PTR, ORD_DPADD, ORD_DPCMP, ORD_DPDIV,
            ORD_DPMUL, ORD_DPSUB, ORD_DPTOFP, ORD_DPTOLI, ORD_DPTOUL, ORD_DSA_CLONE,
            ORD_DSA_CREATE, ORD_DSA_DESTROY, ORD_DSA_DESTROY_CALLBACK, ORD_DSA_ENUM_CALLBACK,
            ORD_DSA_GET_ITEM_PTR, ORD_DSA_GROW, ORD_DSA_INSERT_ITEM, ORD_DSA_SET_RANGE,
            ORD_DUPLICATE_HANDLE, ORD_EQD, ORD_EQS, ORD_ESCAPE_COMM_FUNCTION, ORD_EXP, ORD_F_TO_LL,
            ORD_FABS, ORD_FCLOSE, ORD_FEOF, ORD_FERROR, ORD_FFLUSH, ORD_FGETS, ORD_FIND_CLOSE,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION, ORD_FIND_CLOSE_PARTITION, ORD_FIND_CLOSE_STORE,
            ORD_FIND_FIRST_CHANGE_NOTIFICATION_W, ORD_FIND_FIRST_FILE_W, ORD_FIND_FIRST_PARTITION,
            ORD_FIND_FIRST_STORE, ORD_FIND_NEXT_CHANGE_NOTIFICATION, ORD_FIND_NEXT_FILE_W,
            ORD_FIND_NEXT_PARTITION, ORD_FIND_NEXT_STORE, ORD_FLOOR, ORD_FLUSH_FILE_BUFFERS,
            ORD_FLUSH_INSTRUCTION_CACHE, ORD_FLUSH_VIEW_OF_FILE, ORD_FLUSH_VIEW_OF_FILE_MAYBE,
            ORD_FMOD, ORD_FMODF, ORD_FOPEN, ORD_FORMAT_PARTITION, ORD_FORMAT_PARTITION_EX,
            ORD_FORMAT_STORE, ORD_FPADD, ORD_FPCMP, ORD_FPDIV, ORD_FPMUL, ORD_FPSUB, ORD_FPTODP,
            ORD_FPTOLI, ORD_FPTOUL, ORD_FREAD, ORD_FREE, ORD_FSEEK, ORD_FTELL, ORD_FWRITE, ORD_GES,
            ORD_GET_COMM_MODEM_STATUS, ORD_GET_DISK_FREE_SPACE_EX_W, ORD_GET_FILE_ATTRIBUTES_EX_W,
            ORD_GET_FILE_ATTRIBUTES_W, ORD_GET_FILE_SECURITY_W, ORD_GET_FILE_SIZE,
            ORD_GET_MODULE_FILE_NAME_W, ORD_GET_OBJECT_W, ORD_GET_PARTITION_INFO,
            ORD_GET_PROCESS_HEAP, ORD_GET_STORE_INFO, ORD_GET_VERSION_EX, ORD_GTD, ORD_GTS,
            ORD_HEAP_ALLOC, ORD_HEAP_CREATE, ORD_HEAP_DESTROY, ORD_HEAP_FREE, ORD_HEAP_RE_ALLOC,
            ORD_HEAP_SIZE, ORD_HEAP_VALIDATE, ORD_HYPOT, ORD_INTERLOCKED_DECREMENT,
            ORD_INTERLOCKED_EXCHANGE, ORD_INTERLOCKED_TEST_EXCHANGE, ORD_IS_BAD_READ_PTR,
            ORD_IS_BAD_WRITE_PTR, ORD_IS_SYSTEM_FILE, ORD_IS_VALID_LOCALE, ORD_ISWCTYPE, ORD_LED,
            ORD_LES, ORD_LITODP, ORD_LITOFP, ORD_LL_DIV, ORD_LL_LSHIFT, ORD_LL_MUL, ORD_LL_REM,
            ORD_LL_RSHIFT, ORD_LOAD_IMAGE_W, ORD_LOCAL_ALLOC, ORD_LOCAL_ALLOC_IN_PROCESS,
            ORD_LOCAL_FREE, ORD_LOCAL_FREE_IN_PROCESS, ORD_LOCAL_RE_ALLOC, ORD_LOCAL_SIZE,
            ORD_LOCAL_SIZE_IN_PROCESS, ORD_LOCK_FILE_EX, ORD_LOG, ORD_LOG10, ORD_LTS, ORD_MALLOC,
            ORD_MAP_VIEW_OF_FILE, ORD_MEMCMP, ORD_MEMCPY, ORD_MEMMOVE, ORD_MEMSET,
            ORD_MOUNT_PARTITION, ORD_MOVE_FILE_W, ORD_MSIZE, ORD_MULTI_BYTE_TO_WIDE_CHAR, ORD_NED,
            ORD_NES, ORD_OPEN_PARTITION, ORD_OPEN_STORE, ORD_OPERATOR_DELETE,
            ORD_OPERATOR_DELETE_ARRAY, ORD_OPERATOR_DELETE_ARRAY_NOTHROW, ORD_OPERATOR_NEW,
            ORD_OPERATOR_NEW_ARRAY, ORD_OPERATOR_NEW_ARRAY_NOTHROW, ORD_POW, ORD_PRINTF, ORD_RAND,
            ORD_READ_FILE, ORD_REALLOC, ORD_REG_CLOSE_KEY, ORD_REG_CREATE_KEY_EX_W,
            ORD_REG_DELETE_KEY_W, ORD_REG_DELETE_VALUE_W, ORD_REG_ENUM_KEY_EX_W,
            ORD_REG_ENUM_VALUE_W, ORD_REG_OPEN_KEY_EX_W, ORD_REG_QUERY_INFO_KEY_W,
            ORD_REG_QUERY_VALUE_EX_W, ORD_REG_SET_VALUE_EX_W, ORD_REGISTER_AFSEX,
            ORD_REGISTER_AFSNAME, ORD_REGISTRY_DELETE_VALUE, ORD_REGISTRY_GET_DWORD,
            ORD_REGISTRY_GET_STRING, ORD_REGISTRY_SET_DWORD, ORD_REGISTRY_SET_STRING,
            ORD_REGISTRY_TEST_EXCHANGE_DWORD, ORD_REMOTE_HEAP_ALLOC, ORD_REMOTE_HEAP_FREE,
            ORD_REMOTE_HEAP_RE_ALLOC, ORD_REMOTE_HEAP_SIZE, ORD_REMOTE_LOCAL_ALLOC,
            ORD_REMOTE_LOCAL_FREE, ORD_REMOTE_LOCAL_RE_ALLOC, ORD_REMOTE_LOCAL_SIZE,
            ORD_REMOVE_DIRECTORY_W, ORD_RENAME_PARTITION, ORD_SECURITY_GEN_COOKIE,
            ORD_SECURITY_GEN_COOKIE2, ORD_SET_COMM_BREAK, ORD_SET_FILE_ATTRIBUTES_W,
            ORD_SET_FILE_POINTER, ORD_SET_FILE_SECURITY_W, ORD_SET_FILE_TIME,
            ORD_SET_PARTITION_ATTRIBUTES, ORD_SETUP_COMM, ORD_SHLOAD_DIBITMAP, ORD_SIN, ORD_SINH,
            ORD_SNPRINTF, ORD_SNWPRINTF, ORD_SPRINTF, ORD_SQRT, ORD_SRAND, ORD_STRCAT, ORD_STRCPY,
            ORD_STRING_CB_CAT_W, ORD_STRING_CCH_CAT_W, ORD_STRING_CCH_LENGTH_W, ORD_STRTOK,
            ORD_STRTOUL, ORD_STRUPR, ORD_SWPRINTF, ORD_TAN, ORD_TANH, ORD_TLS_CALL, ORD_TOLOWER,
            ORD_TOUPPER, ORD_ULL_DIV, ORD_ULL_REM, ORD_ULL_RSHIFT, ORD_ULTODP, ORD_ULTOFP,
            ORD_UNLOCK_FILE_EX, ORD_UNMAP_VIEW_OF_FILE, ORD_VIRTUAL_ALLOC, ORD_VIRTUAL_FREE,
            ORD_VSNPRINTF, ORD_VSNWPRINTF, ORD_VSPRINTF, ORD_VSWPRINTF, ORD_WAIT_FOR_SINGLE_OBJECT,
            ORD_WCSCHR, ORD_WCSCPY, ORD_WCSDUP, ORD_WCSICMP, ORD_WCSLEN, ORD_WCSNCMP, ORD_WCSNCPY,
            ORD_WCSNICMP, ORD_WCSPBRK, ORD_WCSRCHR, ORD_WCSSTR, ORD_WCSTOUL, ORD_WFOPEN,
            ORD_WIDE_CHAR_TO_MULTI_BYTE, ORD_WRITE_FILE, ORD_WSPRINTF_W, ORD_WTOL, ORD_WVSPRINTF_W,
        },
        file::{
            CREATE_ALWAYS, FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_HIDDEN,
            FILE_ATTRIBUTE_SYSTEM, GENERIC_READ, GENERIC_WRITE, HostFileSystem, OPEN_ALWAYS,
            OPEN_EXISTING,
        },
        kernel::CeKernel,
        memory::{
            HEAP_NO_SERIALIZE, HEAP_REALLOC_IN_PLACE_ONLY, HEAP_ZERO_MEMORY, LMEM_ZEROINIT,
            MEM_COMMIT, MEM_RELEASE,
        },
        registry::{
            ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, HKEY_CURRENT_USER,
            HKEY_LOCAL_MACHINE, REG_DWORD,
        },
        thread::{
            ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE,
            ERROR_INVALID_PARAMETER, ERROR_LOCK_VIOLATION, ERROR_NO_MORE_FILES,
            ERROR_NOT_SAME_DEVICE, ERROR_NOT_SUPPORTED,
        },
        timer::{WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::{MountConfig, RuntimeConfig},
};

mod support;
use support::{TestGuestMemory, unique_test_root};

const ERROR_PATH_NOT_FOUND: u32 = 3;

#[test]
fn coredll_raw_string_conversion_ordinals_round_trip_ascii() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let narrow = 0x1_0000;
    let wide = 0x1_0100;
    let round_trip = 0x1_0200;
    let lower = 0x1_0300;
    let upper = 0x1_0340;
    memory.map_bytes(narrow, 32);
    memory.map_halfwords(wide, 64);
    memory.map_bytes(round_trip, 64);
    memory.write_bytes(narrow, b"api-storm\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MULTI_BYTE_TO_WIDE_CHAR,
            [0, 0, narrow, u32::MAX, wide, 64],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(10),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(wide, 64), "api-storm");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WIDE_CHAR_TO_MULTI_BYTE,
            [0, 0, wide, u32::MAX, round_trip, 64, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(10),
            ..
        }
    ));
    assert_eq!(&memory.read_bytes(round_trip, 10), b"api-storm\0");

    memory.write_wide_z(lower, "abc");
    memory.write_wide_z(upper, "XYZ");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CHAR_UPPER_W,
            [lower],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == lower
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CHAR_LOWER_W,
            [upper],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == upper
    ));
    assert_eq!(memory.read_wide_z(lower, 8), "ABC");
    assert_eq!(memory.read_wide_z(upper, 8), "xyz");
    Ok(())
}

#[test]
fn coredll_raw_is_valid_locale_accepts_korean_and_defaults() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_VALID_LOCALE,
            [0x0412, 1],
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
            ORD_IS_VALID_LOCALE,
            [0x0400, 2],
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
            ORD_IS_VALID_LOCALE,
            [0x1234, 1],
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
            ORD_IS_VALID_LOCALE,
            [0x0412, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_realloc_allocates_copies_and_frees_process_heap_blocks() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let process_heap = kernel.memory.get_process_heap();

    let ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REALLOC,
        [0, 16],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("realloc(NULL, size) did not allocate: {other:?}"),
    };
    assert_ne!(ptr, 0);
    assert_eq!(kernel.memory.heap_size(process_heap, 0, ptr), Some(16));

    memory.write_bytes(ptr, b"ce-realloc-proof");
    let grown = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REALLOC,
        [ptr, 64],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("realloc(ptr, larger) did not return a pointer: {other:?}"),
    };
    assert_ne!(grown, 0);
    assert_ne!(grown, ptr);
    assert_eq!(kernel.memory.heap_size(process_heap, 0, ptr), None);
    assert_eq!(kernel.memory.heap_size(process_heap, 0, grown), Some(64));
    assert_eq!(&memory.read_bytes(grown, 16), b"ce-realloc-proof");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REALLOC,
            [grown, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.memory.heap_size(process_heap, 0, grown), None);
    Ok(())
}

#[test]
fn coredll_raw_msize_returns_process_heap_block_size() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MALLOC,
        [37],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("malloc did not return a pointer: {other:?}"),
    };
    assert_ne!(ptr, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSIZE,
            [ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(37),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSIZE,
            [ptr + 1],
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

    Ok(())
}

#[test]
fn coredll_raw_strcat_appends_narrow_string_and_returns_dest() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_1000;
    let src = 0x1_1100;

    memory.map_bytes(dest, 32);
    memory.write_bytes(dest, b"Auth");
    memory.write_bytes(dest + 4, &[0]);
    memory.write_bytes(src, b"Library.dll\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRCAT,
            [dest, src],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == dest
    ));
    assert_eq!(&memory.read_bytes(dest, 16), b"AuthLibrary.dll\0");
    Ok(())
}

#[test]
fn coredll_raw_strupr_uppercases_narrow_string_in_place() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let string = 0x1_1200;

    memory.map_bytes(string, 32);
    memory.write_bytes(string, b"Seoul-Station_123\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRUPR,
            [string],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == string
    ));
    assert_eq!(&memory.read_bytes(string, 18), b"SEOUL-STATION_123\0");
    Ok(())
}

#[test]
fn coredll_raw_string_cb_cat_w_appends_with_byte_capacity() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let src = 0x1_0100;
    memory.map_halfwords(dest, 24);
    memory.map_halfwords(src, 16);
    memory.write_wide_z(dest, r"\Windows");
    memory.write_wide_z(src, r"\Desktop");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_CB_CAT_W,
            [dest, 18 * 2, src],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 24), r"\Windows\Desktop");

    memory.write_wide_z(dest, "abc");
    memory.write_wide_z(src, "defghi");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_CB_CAT_W,
            [dest, 6 * 2, src],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x8007_007a),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 16), "abcde");
    Ok(())
}

#[test]
fn coredll_raw_wsprintf_w_formats_mfc_class_names() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    memory.map_halfwords(dest, 128);
    memory.map_halfwords(format, 64);
    memory.write_wide_z(format, "Afx:%p:%x");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WSPRINTF_W,
            [dest, format, 0x0001_0000, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(14),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 128), "Afx:00010000:0");
    Ok(())
}

#[test]
fn coredll_raw_swprintf_formats_zero_padded_integer_widths() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    memory.map_halfwords(dest, 128);
    memory.map_halfwords(format, 64);
    memory.write_wide_z(format, r"Sky\%03d:%04X:%5u");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SWPRINTF,
            [dest, format, 1, 0x2a, 7],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(18),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 128), r"Sky\001:002A:    7");
    Ok(())
}

#[test]
fn coredll_raw_vswprintf_reads_guest_va_list() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let text = 0x1_0300;
    let va_list = 0x1_0400;
    memory.map_halfwords(dest, 128);
    memory.map_halfwords(format, 64);
    memory.map_halfwords(text, 32);
    memory.map_words(va_list, 4);
    memory.write_wide_z(format, "Afx:%p:%x:%s");
    memory.write_wide_z(text, "class");
    memory.write_word(va_list, 0x0001_0000);
    memory.write_word(va_list + 4, 0x2a);
    memory.write_word(va_list + 8, text);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VSWPRINTF,
            [dest, format, va_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(21),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 128), "Afx:00010000:2a:class");

    let narrow_text = 0x1_0600;
    memory.map_bytes(narrow_text, 32);
    memory.write_wide_z(format, "Afx:%hs");
    memory.write_bytes(narrow_text, b"narrow\0");
    memory.write_word(va_list, narrow_text);
    memory.write_wide_z(dest, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VSWPRINTF,
            [dest, format, va_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(10),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 128), "Afx:narrow");

    let wide_text = 0x1_0500;
    memory.map_halfwords(wide_text, 32);
    memory.write_wide_z(format, "Afx:%ls");
    memory.write_wide_z(wide_text, "wide");
    memory.write_word(va_list, wide_text);
    memory.write_wide_z(dest, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VSWPRINTF,
            [dest, format, va_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(8),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 128), "Afx:wide");
    Ok(())
}

#[test]
fn coredll_raw_swprintf_uses_wide_default_for_percent_s() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let text = 0x1_0300;
    memory.map_halfwords(dest, 128);
    memory.map_halfwords(format, 64);
    memory.map_halfwords(text, 128);
    memory.write_wide_z(format, r"%s\res");
    memory.write_wide_z(text, r"\SDMMC Disk\INavi");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SWPRINTF,
            [dest, format, text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(21),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 128), r"\SDMMC Disk\INavi\res");
    Ok(())
}

#[test]
fn coredll_raw_wvsprintf_w_uses_wide_default_for_percent_s() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let text = 0x1_0300;
    let va_list = 0x1_0400;
    memory.map_halfwords(dest, 128);
    memory.map_halfwords(format, 64);
    memory.map_halfwords(text, 32);
    memory.map_words(va_list, 4);
    memory.write_wide_z(format, "Afx:%p:%x:%s");
    memory.write_wide_z(text, "class");
    memory.write_word(va_list, 0x0001_0000);
    memory.write_word(va_list + 4, 0x2a);
    memory.write_word(va_list + 8, text);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WVSPRINTF_W,
            [dest, format, va_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(21),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 128), "Afx:00010000:2a:class");
    Ok(())
}

#[test]
fn coredll_raw_sprintf_uses_narrow_default_for_percent_s() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let narrow_text = 0x1_0300;
    let wide_text = 0x1_0400;
    memory.map_bytes(dest, 128);
    memory.map_bytes(format, 64);
    memory.map_bytes(narrow_text, 64);
    memory.map_halfwords(wide_text, 64);
    memory.write_bytes(format, b"%s\\%ls:%x\0");
    memory.write_bytes(narrow_text, b"FontResHigh.utf\0");
    memory.write_wide_z(wide_text, "wide");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SPRINTF,
            [dest, format, narrow_text, wide_text, 0x2a],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(23),
            ..
        }
    ));
    assert_eq!(
        String::from_utf8(memory.read_bytes(dest, 24).to_vec()).unwrap(),
        "FontResHigh.utf\\wide:2a\0"
    );
    Ok(())
}

#[test]
fn coredll_raw_sprintf_encodes_wide_korean_as_cp949() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let wide_text = 0x1_0300;
    let expected = [
        0xbe, 0xe7, 0xc3, 0xb5, 0xb1, 0xb8, b' ', 0xb8, 0xf1, 0xb5, 0xbf,
    ];
    memory.map_bytes(dest, 128);
    memory.map_bytes(format, 16);
    memory.map_halfwords(wide_text, 32);
    memory.write_bytes(format, b"%ls\0");
    memory.write_wide_z(wide_text, "양천구 목동");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SPRINTF,
            [dest, format, wide_text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(11),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(dest, 12), [&expected[..], &[0]].concat());
    Ok(())
}

#[test]
fn coredll_raw_sprintf_decodes_narrow_korean_as_cp949() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let narrow_text = 0x1_0300;
    memory.map_bytes(dest, 128);
    memory.map_bytes(format, 16);
    memory.map_bytes(narrow_text, 16);
    memory.write_bytes(format, b"[%s]\0");
    memory.write_bytes(narrow_text, &[0xbf, 0xee, 0xc0, 0xfc, 0]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SPRINTF,
            [dest, format, narrow_text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(6),
            ..
        }
    ));
    assert_eq!(
        memory.read_bytes(dest, 7),
        &[b'[', 0xbf, 0xee, 0xc0, 0xfc, b']', 0]
    );
    Ok(())
}

#[test]
fn coredll_raw_wide_char_to_multi_byte_cp_acp_encodes_ascii_path() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let wide_text = 0x1_0000;
    let dest = 0x1_0200;
    let text = "\\\\SDMMC Disk\\INavi\\happyway_win.exe";
    memory.map_halfwords(wide_text, 128);
    memory.map_bytes(dest, 128);
    memory.write_wide_z(wide_text, text);

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WIDE_CHAR_TO_MULTI_BYTE,
        [0, 0, wide_text, -1i32 as u32, dest, 128, 0, 0],
    );
    assert!(
        matches!(
            result,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(36),
                ..
            }
        ),
        "result={result:?} last_error={}",
        kernel.threads.get_last_error(thread_id)
    );
    assert_eq!(
        memory.read_bytes(dest, 36),
        [text.as_bytes(), &[0]].concat()
    );
    Ok(())
}

#[test]
fn coredll_raw_wide_char_to_multi_byte_positive_len_stops_at_nul() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let wide_text = 0x1_0000;
    let dest = 0x1_0800;
    let text = "\\\\SDMMC Disk\\INavi\\happyway_win.exe";
    memory.map_halfwords(wide_text, 600);
    memory.map_bytes(dest, 256);
    memory.write_wide_z(wide_text, text);
    let trailing_start = text.encode_utf16().count() as u32 + 1;
    for index in trailing_start..513 {
        memory.write_halfword(wide_text + index * 2, b'X' as u16);
    }

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WIDE_CHAR_TO_MULTI_BYTE,
        [0, 0, wide_text, 513, dest, 256, 0, 0],
    );
    assert!(
        matches!(
            result,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(36),
                ..
            }
        ),
        "result={result:?} last_error={}",
        kernel.threads.get_last_error(thread_id)
    );
    assert_eq!(
        memory.read_bytes(dest, 36),
        [text.as_bytes(), &[0]].concat()
    );
    Ok(())
}

#[test]
fn coredll_raw_snprintf_formats_with_count_limit() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    memory.map_bytes(dest, 8);
    memory.map_bytes(format, 32);
    memory.write_bytes(dest, b"XXXXXXXX");
    memory.write_bytes(format, b"%03u\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SNPRINTF,
            [dest, 4, format, 7],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(3),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(dest, 5), b"007\0X");
    Ok(())
}

#[test]
fn coredll_raw_snprintf_reports_truncation_without_forced_terminator() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    memory.map_bytes(dest, 8);
    memory.map_bytes(format, 32);
    memory.write_bytes(dest, b"XXXXXXXX");
    memory.write_bytes(format, b"%s:%u\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SNPRINTF,
            [dest, 4, format, 0, 123],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(dest, 6), b"(nulXX");
    Ok(())
}

#[test]
fn coredll_raw_vsnprintf_reads_guest_va_list() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let text = 0x1_0300;
    let va_list = 0x1_0400;
    memory.map_bytes(dest, 32);
    memory.map_bytes(format, 32);
    memory.map_bytes(text, 32);
    memory.map_words(va_list, 4);
    memory.write_bytes(format, b"%s-%x\0");
    memory.write_bytes(text, b"ce\0");
    memory.write_word(va_list, text);
    memory.write_word(va_list + 4, 0x2a);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VSNPRINTF,
            [dest, 16, format, va_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(5),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(dest, 6), b"ce-2a\0");
    Ok(())
}

#[test]
fn coredll_raw_vsprintf_reads_guest_va_list() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let text = 0x1_0300;
    let va_list = 0x1_0400;
    memory.map_bytes(dest, 32);
    memory.map_bytes(format, 32);
    memory.map_bytes(text, 32);
    memory.map_words(va_list, 4);
    memory.write_bytes(format, b"%s:%03u\0");
    memory.write_bytes(text, b"ce\0");
    memory.write_word(va_list, text);
    memory.write_word(va_list + 4, 7);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VSPRINTF,
            [dest, format, va_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(6),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(dest, 7), b"ce:007\0");
    Ok(())
}

#[test]
fn coredll_raw_snwprintf_formats_wide_with_count_limit() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let text = 0x1_0300;
    memory.map_halfwords(dest, 8);
    memory.map_halfwords(format, 32);
    memory.map_halfwords(text, 16);
    memory.write_wide_z(dest, "XXXXXXX");
    memory.write_wide_z(format, "%s:%02x");
    memory.write_wide_z(text, "CE");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SNWPRINTF,
            [dest, 6, format, text, 7],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(5),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 8), "CE:07");
    Ok(())
}

#[test]
fn coredll_raw_vsnwprintf_reads_guest_va_list() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let format = 0x1_0200;
    let text = 0x1_0300;
    let va_list = 0x1_0400;
    memory.map_halfwords(dest, 16);
    memory.map_halfwords(format, 32);
    memory.map_halfwords(text, 16);
    memory.map_words(va_list, 4);
    memory.write_wide_z(format, "%s-%x");
    memory.write_wide_z(text, "CE");
    memory.write_word(va_list, text);
    memory.write_word(va_list + 4, 0x2a);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VSNWPRINTF,
            [dest, 16, format, va_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(5),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 16), "CE-2a");
    Ok(())
}

#[test]
fn coredll_raw_toupper_tolower_preserve_c_int_semantics() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TOUPPER,
            [b'a' as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value == b'A' as u32
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TOLOWER,
            [b'Z' as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value == b'z' as u32
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TOUPPER,
            [u32::MAX],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_stdio_reads_host_backed_files() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("raw_stdio");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    fs::write(sdmmc_root.join("font.bin"), b"abcdefg").unwrap();
    fs::write(sdmmc_root.join("lines.txt"), b"one\ntwo").unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path = 0x1_0000;
    let mode = 0x1_0100;
    let buffer = 0x1_0200;
    let wide_path = 0x1_0300;
    let wide_mode = 0x1_0400;
    memory.map_bytes(path, 64);
    memory.map_bytes(mode, 8);
    memory.map_bytes(buffer, 16);
    memory.map_halfwords(wide_path, 64);
    memory.map_halfwords(wide_mode, 8);
    memory.write_bytes(path, b"\\SDMMC Disk\\font.bin\0");
    memory.write_bytes(mode, b"rb\0");

    let stream = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FOPEN,
        [path, mode],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("fopen did not return a stream: {other:?}"),
    };
    assert_ne!(stream, 0);
    memory.write_wide_z(wide_path, "\\SDMMC Disk\\font.bin");
    memory.write_wide_z(wide_mode, "rb");
    let wide_stream = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WFOPEN,
        [wide_path, wide_mode],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("_wfopen did not return a stream: {other:?}"),
    };
    assert_ne!(wide_stream, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FCLOSE,
            [wide_stream],
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
            ORD_FREAD,
            [buffer, 2, 3, stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(3),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(buffer, 6), b"abcdef");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FTELL,
            [stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(6),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FSEEK,
            [stream, 0, 0],
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
            ORD_FREAD,
            [buffer, 1, 7, stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(7),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(buffer, 7), b"abcdefg");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FEOF,
            [stream],
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
            ORD_FREAD,
            [buffer, 1, 1, stream],
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
            ORD_FEOF,
            [stream],
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
            ORD_FSEEK,
            [stream, (-3_i32) as u32, 2],
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
            ORD_FEOF,
            [stream],
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
            ORD_FREAD,
            [buffer, 1, 8, stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(3),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(buffer, 3), b"efg");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FEOF,
            [stream],
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
            ORD_FCLOSE,
            [stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    memory.write_bytes(path, b"\\SDMMC Disk\\lines.txt\0");
    let stream = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FOPEN,
        [path, mode],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("second fopen did not return a stream: {other:?}"),
    };
    assert_ne!(stream, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FGETS,
            [buffer, 6, stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(value),
            ..
        } if value == buffer
    ));
    assert_eq!(memory.read_bytes(buffer, 5), b"one\n\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FREAD,
            [buffer, 1, 8, stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(3),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(buffer, 3), b"two");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FCLOSE,
            [stream],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_rand_uses_seeded_crt_sequence() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_SRAND, [1]),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_RAND, []),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(41),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_RAND, []),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(18467),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_string_cch_cat_w_appends_with_character_capacity() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let src = 0x1_0100;
    memory.map_halfwords(dest, 24);
    memory.map_halfwords(src, 16);
    memory.write_wide_z(dest, r"\Windows");
    memory.write_wide_z(src, r"\Desktop");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_CCH_CAT_W,
            [dest, 18, src],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 24), r"\Windows\Desktop");

    memory.write_wide_z(dest, "abc");
    memory.write_wide_z(src, "defghi");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_CCH_CAT_W,
            [dest, 6, src],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x8007_007a),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(dest, 16), "abcde");
    Ok(())
}

#[test]
fn coredll_raw_string_cch_length_w_counts_bounded_wide_chars() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let src = 0x1_0000;
    let out_len = 0x1_0100;
    memory.map_halfwords(src, 64);
    memory.map_words(out_len, 1);
    memory.write_wide_z(src, "{01234567-89ab-cdef-0123-456789abcdef}");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_CCH_LENGTH_W,
            [src, 260, out_len],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(out_len)?, 38);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRING_CCH_LENGTH_W,
            [src, 3, out_len],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x8007_0057),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_wcscpy_copies_wide_string_and_returns_dest() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let src = 0x1_0100;
    memory.map_halfwords(dest, 32);
    memory.map_halfwords(src, 32);
    memory.write_wide_z(src, r"\Windows\Desktop");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSCPY,
            [dest, src],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == dest
    ));
    assert_eq!(memory.read_wide_z(dest, 32), r"\Windows\Desktop");
    Ok(())
}

#[test]
fn coredll_raw_wcschr_finds_wide_character() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let text = 0x1_0000;
    memory.map_halfwords(text, 32);
    memory.write_wide_z(text, r"\Windows\Desktop");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSCHR,
            [text.wrapping_add(2), b'\\' as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == text.wrapping_add(16)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSCHR,
            [text, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == text.wrapping_add(32)
    ));
    Ok(())
}

#[test]
fn coredll_raw_wcslen_counts_wide_chars_before_null() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let text = 0x1_0000;
    memory.map_halfwords(text, 32);
    memory.write_wide_z(text, r"\Windows\Desktop");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSLEN,
            [text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(16),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_security_gen_cookie_ordinals_return_usable_cookie() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    for ordinal in [ORD_SECURITY_GEN_COOKIE, ORD_SECURITY_GEN_COOKIE2] {
        match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ordinal,
            [],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::U32(cookie),
                ..
            } => {
                assert_ne!(cookie, 0);
                assert_ne!(cookie, 0xbb40_e64e);
            }
            other => panic!("unexpected security cookie dispatch for {ordinal}: {other:?}"),
        }
    }
    Ok(())
}

#[test]
fn coredll_raw_wcsncpy_copies_and_pads_wide_strings() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let src = 0x1_0100;
    memory.map_halfwords(dest, 8);
    memory.map_halfwords(src, 8);
    memory.write_wide_z(src, "WCE");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSNCPY,
            [dest, src, 12],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == dest
    ));
    assert_eq!(memory.read_wide_z(dest, 8), "WCE");
    assert_eq!(memory.read_u16(dest + 6)?, 0);
    assert_eq!(memory.read_u16(dest + 8)?, 0);
    assert_eq!(memory.read_u16(dest + 10)?, 0);
    Ok(())
}

#[test]
fn coredll_raw_wcsncpy_uses_ce_byte_counts_for_path_prefixes() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let src = 0x1_0100;
    memory.map_halfwords(dest, 64);
    memory.map_halfwords(src, 64);
    for index in 0..64 {
        memory.write_halfword(dest + index * 2, b'X' as u16);
    }
    memory.write_halfword(dest + 34, 0);
    memory.write_wide_z(src, r"\SDMMC Disk\INavi\iNavi.exe");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSNCPY,
            [dest, src, 34],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == dest
    ));
    assert_eq!(memory.read_wide_z(dest, 64), r"\SDMMC Disk\INavi");
    assert_eq!(memory.read_u16(dest + 32)?, b'i' as u16);
    assert_eq!(memory.read_u16(dest + 34)?, 0);
    assert_eq!(memory.read_u16(dest + 36)?, b'X' as u16);
    Ok(())
}

#[test]
fn coredll_raw_wcsncpy_accepts_pointer_backed_wide_source() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dest = 0x1_0000;
    let src_obj = 0x1_0100;
    let src = 0x1_0200;
    memory.map_halfwords(dest, 64);
    memory.map_words(src_obj, 1);
    memory.map_halfwords(src, 64);
    memory.write_word(src_obj, src);
    memory.write_wide_z(src, r"\SDMMC Disk\INavi\iNavi.exe");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSNCPY,
            [dest, src_obj, 34],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == dest
    ));
    assert_eq!(memory.read_wide_z(dest, 64), r"\SDMMC Disk\INavi");
    Ok(())
}

#[test]
fn coredll_raw_wcsrchr_keeps_match_before_unmapped_tail() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let string = 0x1_0000;
    memory.map_halfwords(string, 1);
    memory.write_halfword(string, b'\\' as u16);

    let slash = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WCSRCHR,
        [string, '\\' as u32],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("wcsrchr did not return a pointer: {other:?}"),
    };
    assert_eq!(slash, string);
    Ok(())
}

#[test]
fn coredll_raw_wcsstr_finds_wide_substrings() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let haystack = 0x1_0000;
    let needle = 0x1_0100;
    let empty = 0x1_0200;
    let absent = 0x1_0300;
    memory.map_halfwords(haystack, 32);
    memory.map_halfwords(needle, 16);
    memory.map_halfwords(empty, 1);
    memory.map_halfwords(absent, 16);
    memory.write_wide_z(haystack, r"\SDMMC Disk\INavi\res");
    memory.write_wide_z(needle, "INavi");
    memory.write_halfword(empty, 0);
    memory.write_wide_z(absent, "Windows");

    let found = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WCSSTR,
        [haystack, needle],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("wcsstr did not return a pointer: {other:?}"),
    };
    assert_eq!(memory.read_wide_z(found, 16), "INavi\\res");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSSTR,
            [haystack, empty],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == haystack
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSSTR,
            [haystack, absent],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_wcspbrk_finds_first_accepted_wide_char() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let string = 0x1_0000;
    let accept = 0x1_0100;
    let absent = 0x1_0200;
    memory.map_halfwords(string, 32);
    memory.map_halfwords(accept, 8);
    memory.map_halfwords(absent, 8);
    memory.write_wide_z(string, r"\SDMMC Disk\INavi\res");
    memory.write_wide_z(accept, ":/\\");
    memory.write_wide_z(absent, "XYZ");

    let found = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WCSPBRK,
        [string, accept],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("wcspbrk did not return a pointer: {other:?}"),
    };
    assert_eq!(found, string);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSPBRK,
            [string.wrapping_add(2), accept],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } if ptr == string.wrapping_add(22)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSPBRK,
            [string, absent],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_wcsstr_keeps_match_before_unmapped_tail() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let haystack = 0x1_0000;
    let needle = 0x1_0100;
    memory.map_halfwords(haystack, 4);
    memory.map_halfwords(needle, 4);
    memory.write_wide_z(haystack, "abc");
    memory.write_wide_z(needle, "abc");

    let found = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WCSSTR,
        [haystack, needle],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("wcsstr did not return a pointer: {other:?}"),
    };
    assert_eq!(found, haystack);
    Ok(())
}

#[test]
fn coredll_raw_strtoul_parses_base_prefixes_and_endptr() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let input = 0x1_0000;
    let endptr = 0x1_0100;
    memory.map_bytes(input, 16);
    memory.map_words(endptr, 1);
    memory.write_bytes(input, b"  0x2aZ\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRTOUL,
            [input, endptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(42),
            ..
        }
    ));
    assert_eq!(memory.read_u32(endptr)?, input + 6);
    Ok(())
}

#[test]
fn coredll_raw_strtoul_honors_explicit_base_and_negative_wrap() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let hex = 0x1_0000;
    let negative = 0x1_0100;
    memory.map_bytes(hex, 8);
    memory.map_bytes(negative, 8);
    memory.write_bytes(hex, b"ff00;\0");
    memory.write_bytes(negative, b"-1,\0");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRTOUL,
            [hex, 0, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0xff00),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRTOUL,
            [negative, 0, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_wcstoul_parses_wide_decimal_and_endptr() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let input = 0x1_0000;
    let endptr = 0x1_0100;
    memory.map_bytes(input, 32);
    memory.map_words(endptr, 1);
    memory.write_wide_z(input, "  103x");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSTOUL,
            [input, endptr, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(103),
            ..
        }
    ));
    assert_eq!(memory.read_u32(endptr)?, input + 10);
    Ok(())
}

#[test]
fn coredll_raw_wcstoul_honors_base_prefixes_and_negative_wrap() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let hex = 0x1_0000;
    let negative = 0x1_0100;
    memory.map_bytes(hex, 32);
    memory.map_bytes(negative, 16);
    memory.write_wide_z(hex, "0x2a;");
    memory.write_wide_z(negative, "-1,");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSTOUL,
            [hex, 0, 0],
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
            ORD_WCSTOUL,
            [negative, 0, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_registry_ordinals_create_query_enum_and_delete_values() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let key_name = 0x2_0000;
    let value_name = 0x2_0100;
    let key_out = 0x2_0200;
    let disposition_out = 0x2_0204;
    let value_data = 0x2_0300;
    let query_type = 0x2_0400;
    let query_data = 0x2_0500;
    let query_data_len = 0x2_0600;
    let enum_name = 0x2_0700;
    let enum_name_len = 0x2_0800;
    let enum_type = 0x2_0804;
    let enum_data = 0x2_0900;
    let enum_data_len = 0x2_0a00;

    memory.write_wide_z(key_name, "Software\\RawRegistry");
    memory.write_wide_z(value_name, "Number");
    memory.map_words(key_out, 2);
    memory.map_bytes(value_data, 4);
    memory.write_bytes(value_data, &0x5566_7788u32.to_le_bytes());
    memory.map_words(query_type, 1);
    memory.map_bytes(query_data, 4);
    memory.map_words(query_data_len, 1);
    memory.write_word(query_data_len, 4);
    memory.map_halfwords(enum_name, 32);
    memory.map_words(enum_name_len, 1);
    memory.write_word(enum_name_len, 32);
    memory.map_words(enum_type, 1);
    memory.map_bytes(enum_data, 16);
    memory.map_words(enum_data_len, 1);
    memory.write_word(enum_data_len, 16);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REG_CREATE_KEY_EX_W,
            [
                HKEY_CURRENT_USER,
                key_name,
                0,
                0,
                0,
                0,
                0,
                key_out,
                disposition_out
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let hkey = memory.read_u32(key_out)?;
    assert_ne!(hkey, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REG_SET_VALUE_EX_W,
            [hkey, value_name, 0, REG_DWORD, value_data, 4],
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
            ORD_REG_QUERY_VALUE_EX_W,
            [hkey, value_name, 0, query_type, query_data, query_data_len],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_u32(query_type)?, REG_DWORD);
    assert_eq!(memory.read_u32(query_data_len)?, 4);
    assert_eq!(
        u32::from_le_bytes(memory.read_bytes(query_data, 4).try_into().unwrap()),
        0x5566_7788
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REG_ENUM_VALUE_W,
            [
                hkey,
                0,
                enum_name,
                enum_name_len,
                0,
                enum_type,
                enum_data,
                enum_data_len
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(enum_name, 32), "number");
    assert_eq!(memory.read_u32(enum_type)?, REG_DWORD);
    assert_eq!(memory.read_u32(enum_data_len)?, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REG_DELETE_VALUE_W,
            [hkey, value_name],
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
            ORD_REG_CLOSE_KEY,
            [hkey],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_memory_and_file_ordinals_use_virtual_ce_heap_and_guest_buffers() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("raw_memory_file");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    fs::create_dir_all(root.join("Documents")).unwrap();
    fs::write(sdmmc_root.join("mapinfo.bin"), b"mounted").unwrap();
    fs::write(sdmmc_root.join("z-next.bin"), b"next").unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let process_heap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_PROCESS_HEAP,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(heap),
            ..
        } => heap,
        other => panic!("GetProcessHeap did not return a heap: {other:?}"),
    };

    let local = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOCAL_ALLOC,
        [LMEM_ZEROINIT, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("LocalAlloc did not return a pointer: {other:?}"),
    };
    assert_eq!(kernel.memory.local_size(local), Some(1));
    assert!(kernel.memory.allocation(local).unwrap().zeroed);

    let local = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOCAL_RE_ALLOC,
        [local, 24, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("LocalReAlloc did not resize pointer: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOCAL_SIZE,
            [local],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(24),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOCAL_FREE,
            [local],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));

    let heap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_HEAP_CREATE,
        [HEAP_NO_SERIALIZE, 0x1000, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(heap),
            ..
        } => heap,
        other => panic!("HeapCreate did not return a heap: {other:?}"),
    };
    let heap_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_HEAP_ALLOC,
        [heap, HEAP_ZERO_MEMORY, 32],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("HeapAlloc did not return a pointer: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_BAD_READ_PTR,
            [heap_ptr, 32],
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
            ORD_IS_BAD_WRITE_PTR,
            [heap_ptr, 32],
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
            ORD_IS_BAD_READ_PTR,
            [heap_ptr, 33],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_ne!(heap, process_heap);
    let malloc_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MALLOC,
        [16],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("SDK malloc ordinal did not return a pointer: {other:?}"),
    };
    let new_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPERATOR_NEW,
        [8],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("SDK operator new ordinal did not return a pointer: {other:?}"),
    };
    assert_eq!(
        kernel.memory.heap_size(process_heap, 0, malloc_ptr),
        Some(16)
    );
    assert_eq!(kernel.memory.heap_size(process_heap, 0, new_ptr), Some(8));
    let new_array_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPERATOR_NEW_ARRAY,
        [24],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("SDK operator new[] ordinal did not return a pointer: {other:?}"),
    };
    assert_ne!(new_array_ptr, 0);
    assert_eq!(
        kernel.memory.heap_size(process_heap, 0, new_array_ptr),
        Some(24)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPERATOR_DELETE_ARRAY,
            [new_array_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .memory
            .heap_size(process_heap, 0, new_array_ptr)
            .is_none()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPERATOR_DELETE,
            [new_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(kernel.memory.heap_size(process_heap, 0, new_ptr).is_none());

    memory.map_bytes(0x5f00, 32);
    memory.write_wide_z(0x5f00, " -123abc");
    assert_eq!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WTOL,
            [0x5f00],
        ),
        CoredllDispatch::Returned {
            export: table.resolve_ordinal(ORD_WTOL).unwrap().clone(),
            value: CoredllValue::U32((-123_i32) as u32),
        }
    );

    memory.map_bytes(0x6000, 8);
    memory.map_bytes(0x6010, 8);
    memory.map_bytes(0x6020, 8);
    memory.map_bytes(0x6030, 16);
    memory.map_bytes(0x6040, 16);
    memory.map_bytes(0x6050, 16);
    memory.map_bytes(0x6060, 32);
    memory.map_bytes(0x6080, 8);
    memory.write_bytes(0x6000, b"ABCDEFGH");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MEMCPY,
            [0x6010, 0x6000, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x6010),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(0x6010, 8), b"ABCDEFGH");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MEMMOVE,
            [0x6012, 0x6010, 6],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x6012),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(0x6010, 8), b"ABABCDEF");
    let chunk_base = 0x7_0000;
    let chunk_len = 0x1_1000u32;
    memory.map_bytes(chunk_base, chunk_len + 8);
    let pattern: Vec<u8> = (0..chunk_len).map(|index| (index & 0xff) as u8).collect();
    memory.write_bytes(chunk_base, &pattern);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MEMMOVE,
            [chunk_base + 8, chunk_base, chunk_len],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(value),
            ..
        } if value == chunk_base + 8
    ));
    assert_eq!(
        memory.read_bytes(chunk_base + 8, chunk_len as usize),
        pattern
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MEMSET,
            [0x6010, 0x2a, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x6010),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(0x6010, 8), b"****CDEF");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MEMCMP,
            [0x6010, 0x6010, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    memory.write_bytes(0x6020, b"****CDFG");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MEMCMP,
            [0x6010, 0x6020, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if (value as i32) < 0
    ));
    memory.write_bytes(0x6030, b"MapPrefix\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRCPY,
            [0x6040, 0x6030],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x6040),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(0x6040, 10), b"MapPrefix\0");
    memory.write_bytes(0x6050, b" \t-1234px\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_ATOI, [0x6050]),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value as i32 == -1234
    ));
    memory.write_bytes(0x6060, b" alpha,beta;gamma\0");
    memory.write_bytes(0x6080, b" ,;\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRTOK,
            [0x6060, 0x6080],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x6061),
            ..
        }
    ));
    assert_eq!(&memory.read_bytes(0x6061, 6), b"alpha\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRTOK,
            [0, 0x6080],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x6067),
            ..
        }
    ));
    assert_eq!(&memory.read_bytes(0x6067, 5), b"beta\0");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRTOK,
            [0, 0x6080],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x606c),
            ..
        }
    ));

    kernel.set_process_module_base(0x0001_0000);
    kernel.set_process_module_path("\\Program Files\\INavi\\INavi.exe");
    memory.map_halfwords(0x6100, 260);
    let copied = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_MODULE_FILE_NAME_W,
        [0, 0x6100, 260],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(copied),
            ..
        } => copied,
        other => panic!("GetModuleFileNameW did not copy module path: {other:?}"),
    };
    assert_eq!(
        copied,
        "\\Program Files\\INavi\\INavi.exe".encode_utf16().count() as u32
    );
    assert_eq!(
        memory.read_wide_z(0x6100, 260),
        "\\Program Files\\INavi\\INavi.exe"
    );
    memory.map_halfwords(0x6400, 260);
    let copied_from_hinstance = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_MODULE_FILE_NAME_W,
        [0x0001_0000, 0x6400, 260],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(copied),
            ..
        } => copied,
        other => panic!("GetModuleFileNameW(hInstance) did not copy module path: {other:?}"),
    };
    assert_eq!(copied_from_hinstance, copied);
    assert_eq!(
        memory.read_wide_z(0x6400, 260),
        "\\Program Files\\INavi\\INavi.exe"
    );
    let slash = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WCSRCHR,
        [0x6100, '\\' as u32],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("SDK wcsrchr ordinal did not return a pointer: {other:?}"),
    };
    assert_eq!(memory.read_wide_z(slash + 2, 32), "INavi.exe");
    memory.map_halfwords(0x3000_0000, 0x1000);
    let dup = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_WCSDUP,
        [slash + 2],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("SDK _wcsdup ordinal did not return a pointer: {other:?}"),
    };
    assert_eq!(memory.read_wide_z(dup, 32), "INavi.exe");
    assert!(kernel.memory.heap_size(process_heap, 0, dup).is_some());
    let afx = 0x6800;
    let afx_lower = 0x6840;
    let solution = 0x6880;
    let wce = 0x68c0;
    memory.write_wide_z(afx, "Afx");
    memory.write_wide_z(afx_lower, "afxWindow");
    memory.write_wide_z(solution, "Solution_iNavi");
    memory.write_wide_z(wce, "WCE_Solution_iNavi");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSNICMP,
            [afx_lower, afx, 3],
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
            ORD_WCSNICMP,
            [solution, afx, 3],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSNCMP,
            [solution, solution, 16],
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
            ORD_WCSNCMP,
            [afx_lower, afx, 3],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSNCMP,
            [afx_lower, afx, 0],
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
            ORD_WCSICMP,
            [solution, solution],
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
            ORD_WCSICMP,
            [afx_lower, afx],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WCSNICMP,
            [wce, wce + 0x08, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FREE,
            [dup],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(kernel.memory.heap_size(process_heap, 0, dup).is_none());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_HEAP_SIZE,
            [heap, 0, heap_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(32),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_HEAP_FREE,
            [heap, 0, heap_ptr],
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
            ORD_HEAP_DESTROY,
            [heap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let virtual_base = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_VIRTUAL_ALLOC,
        [0, 0x1234, MEM_COMMIT, 4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("VirtualAlloc did not return a base: {other:?}"),
    };
    assert!(kernel.memory.virtual_allocation(virtual_base).is_some());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_BAD_WRITE_PTR,
            [virtual_base, 0x1234],
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
            ORD_FLUSH_INSTRUCTION_CACHE,
            [0xffff_fffe, virtual_base, 0x1234],
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
            ORD_VIRTUAL_FREE,
            [virtual_base, 0, MEM_RELEASE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let com_path_ptr = 0x1_0800;
    let ioctl_in = 0x1_0900;
    let ioctl_out = 0x1_0a00;
    let ioctl_returned = 0x1_0b00;
    memory.write_wide_z(com_path_ptr, "COM1:");
    memory.write_bytes(ioctl_in, &[1, 2, 3, 4]);
    memory.map_bytes(ioctl_out, 8);
    memory.map_words(ioctl_returned, 1);
    memory.write_word(ioctl_returned, 0xffff_ffff);
    let com = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            com_path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_EXISTING,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW(COM1:) did not return a device handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                com,
                0x1234_5678,
                ioctl_in,
                4,
                ioctl_out,
                8,
                ioctl_returned,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(ioctl_returned)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [com],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let missing_path_ptr = 0x1_0c00;
    memory.write_wide_z(missing_path_ptr, "\\ResidentFlash\\missing-file.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_FILE_W,
            [missing_path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(u32::MAX),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );

    let empty_path_ptr = 0x1_0d00;
    memory.write_wide_z(empty_path_ptr, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_FILE_W,
            [empty_path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(u32::MAX),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_PATH_NOT_FOUND
    );

    let path_ptr = 0x1_1000;
    let write_buffer = 0x1_2000;
    let read_buffer = 0x1_3000;
    let count_ptr = 0x1_4000;
    memory.write_wide_z(path_ptr, "\\ResidentFlash\\raw-file.bin");
    memory.write_bytes(write_buffer, b"raw-file");
    memory.map_bytes(read_buffer, 16);
    memory.map_words(count_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            CREATE_ALWAYS,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_FILE,
            [file, write_buffer, 8, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 8);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_SIZE,
            [file, count_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(8),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_POINTER,
            [file, 0, 0, 0],
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
            ORD_FLUSH_FILE_BUFFERS,
            [file],
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
            ORD_CLOSE_HANDLE,
            [file],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not reopen file: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_FILE,
            [file, read_buffer, 8, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 8);
    assert_eq!(memory.read_bytes(read_buffer, 8), b"raw-file");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_POINTER,
            [file, 0xffff_fffc, 0, 1],
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
            ORD_READ_FILE,
            [file, read_buffer, 4, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 4);
    assert_eq!(memory.read_bytes(read_buffer, 4), b"file");

    let find_pattern_ptr = 0x1_5000;
    let find_data_ptr = 0x1_6000;
    memory.map_words(find_data_ptr, 11);
    memory.map_halfwords(find_data_ptr + 40, 260);

    memory.write_wide_z(find_pattern_ptr, "\\");
    let root_find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_FILE_W,
        [find_pattern_ptr, find_data_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstFileW did not return a root find handle: {other:?}"),
    };
    assert_ne!(root_find, u32::MAX);
    assert_eq!(memory.read_u32(find_data_ptr)?, 0x110);
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "SDMMC Disk");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_FILE_W,
            [root_find, find_data_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(find_data_ptr)?, 0x10);
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "Documents");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE,
            [root_find],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_wide_z(find_pattern_ptr, "\\SDMMC Disk");
    let sdmmc_attrs = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_FILE_ATTRIBUTES_W,
        [find_pattern_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(attributes),
            ..
        } => attributes,
        other => panic!("GetFileAttributesW did not return attributes: {other:?}"),
    };
    assert_eq!(sdmmc_attrs, 0x110);

    let free_to_caller_ptr = 0x1_7200;
    let total_ptr = 0x1_7210;
    let total_free_ptr = 0x1_7220;
    memory.map_words(free_to_caller_ptr, 2);
    memory.map_words(total_ptr, 2);
    memory.map_words(total_free_ptr, 2);
    let read_u64 = |memory: &TestGuestMemory, ptr: u32| -> Result<u64> {
        Ok(u64::from(memory.read_u32(ptr)?)
            | (u64::from(memory.read_u32(ptr.wrapping_add(4))?) << 32))
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DISK_FREE_SPACE_EX_W,
            [
                find_pattern_ptr,
                free_to_caller_ptr,
                total_ptr,
                total_free_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        read_u64(&memory, free_to_caller_ptr)?,
        4096_u64 * 1024 * 1024
    );
    assert_eq!(read_u64(&memory, total_ptr)?, 8192_u64 * 1024 * 1024);
    assert_eq!(read_u64(&memory, total_free_ptr)?, 4096_u64 * 1024 * 1024);

    let object_store = kernel.files.object_store();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DISK_FREE_SPACE_EX_W,
            [0, free_to_caller_ptr, total_ptr, total_free_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        read_u64(&memory, free_to_caller_ptr)?,
        object_store.free_bytes
    );
    assert_eq!(read_u64(&memory, total_ptr)?, object_store.total_bytes);
    assert_eq!(read_u64(&memory, total_free_ptr)?, object_store.free_bytes);

    let sectors_per_cluster_ptr = 0x1_7230;
    let bytes_per_sector_ptr = 0x1_7240;
    let free_clusters_ptr = 0x1_7250;
    let clusters_ptr = 0x1_7260;
    memory.map_words(sectors_per_cluster_ptr, 1);
    memory.map_words(bytes_per_sector_ptr, 1);
    memory.map_words(free_clusters_ptr, 1);
    memory.map_words(clusters_ptr, 1);
    memory.write_wide_z(find_pattern_ptr, "\\SDMMC Disk");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_GET_DISK_FREE_SPACE,
            [
                0,
                find_pattern_ptr,
                sectors_per_cluster_ptr,
                bytes_per_sector_ptr,
                free_clusters_ptr,
                clusters_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(sectors_per_cluster_ptr)?, 8);
    assert_eq!(memory.read_u32(bytes_per_sector_ptr)?, 512);
    assert_eq!(memory.read_u32(free_clusters_ptr)?, 1_048_576);
    assert_eq!(memory.read_u32(clusters_ptr)?, 2_097_152);

    const FSCTL_GET_VOLUME_INFO: u32 = 0x0009_0080;
    const CE_VOLUME_INFO_SIZE: u32 = 144;
    const CE_VOLUME_ATTRIBUTE_REMOVABLE: u32 = 0x0000_0004;
    const CE_VOLUME_FLAG_STORE: u32 = 0x0000_0020;
    const CE_VOLUME_FLAG_RAMFS: u32 = 0x0000_0040;

    let info_level_ptr = 0x1_7270;
    let bytes_returned_ptr = 0x1_7280;
    let volume_info_ptr = 0x3000_2000;
    memory.map_words(info_level_ptr, 1);
    memory.map_words(bytes_returned_ptr, 1);
    memory.write_word(info_level_ptr, 0);

    let read_le_u32 = |bytes: &[u8], offset: usize| -> u32 {
        u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("volume info DWORD"),
        )
    };
    let read_fixed_wide = |bytes: &[u8], offset: usize, chars: usize| -> String {
        let units = bytes[offset..offset + chars * 2]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|unit| *unit != 0)
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    };
    let assert_sdmmc_volume_info = |memory: &TestGuestMemory| {
        let volume = memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&volume, 0), CE_VOLUME_INFO_SIZE);
        assert_eq!(
            read_le_u32(&volume, 4) & CE_VOLUME_ATTRIBUTE_REMOVABLE,
            CE_VOLUME_ATTRIBUTE_REMOVABLE
        );
        assert_eq!(
            read_le_u32(&volume, 8) & CE_VOLUME_FLAG_STORE,
            CE_VOLUME_FLAG_STORE
        );
        assert_eq!(read_le_u32(&volume, 12), 4096);
        assert_eq!(read_fixed_wide(&volume, 16, 32), "SDMMC Disk");
        assert_eq!(read_fixed_wide(&volume, 80, 32), "SDMMC Disk");
    };
    let assert_object_store_volume_info = |memory: &TestGuestMemory| {
        let volume = memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&volume, 0), CE_VOLUME_INFO_SIZE);
        assert_eq!(
            read_le_u32(&volume, 8) & CE_VOLUME_FLAG_RAMFS,
            CE_VOLUME_FLAG_RAMFS
        );
        assert_eq!(read_le_u32(&volume, 12), 4096);
        assert_eq!(read_fixed_wide(&volume, 16, 32), "ObjectStore");
        assert_eq!(read_fixed_wide(&volume, 80, 32), "ObjectStore");
    };

    memory.write_wide_z(find_pattern_ptr, "\\SDMMC Disk");
    match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CE_FS_IO_CONTROL_W,
        [
            find_pattern_ptr,
            FSCTL_GET_VOLUME_INFO,
            info_level_ptr,
            4,
            volume_info_ptr,
            CE_VOLUME_INFO_SIZE,
            bytes_returned_ptr,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        } => {}
        other => panic!(
            "CeFsIoControlW(FSCTL_GET_VOLUME_INFO) returned {other:?}, last_error={}",
            kernel.threads.get_last_error(thread_id)
        ),
    }
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, CE_VOLUME_INFO_SIZE);
    assert_sdmmc_volume_info(&memory);

    memory.write_wide_z(find_pattern_ptr, "\\SDMMC Disk\\mapinfo.bin");
    let sdmmc_file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [find_pattern_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not open mounted file: {other:?}"),
    };
    memory.write_bytes(volume_info_ptr, &[0x7b; CE_VOLUME_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0xfeed_beef);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                sdmmc_file,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                4,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, CE_VOLUME_INFO_SIZE);
    assert_sdmmc_volume_info(&memory);

    memory.write_bytes(volume_info_ptr, &[0x7c; CE_VOLUME_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0x1234_abcd);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                sdmmc_file,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                0,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
                0,
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0x1234_abcd);
    assert_eq!(
        memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize),
        vec![0x7c; CE_VOLUME_INFO_SIZE as usize]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [sdmmc_file],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_wide_z(find_pattern_ptr, "\\SDMMC Disk");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_VOLUME_INFO_W,
            [find_pattern_ptr, 0, volume_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_sdmmc_volume_info(&memory);

    memory.write_wide_z(find_pattern_ptr, "\\");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_VOLUME_INFO_W,
            [find_pattern_ptr, 0, volume_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_object_store_volume_info(&memory);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_FS_IO_CONTROL_W,
            [
                0,
                0,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                4,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, CE_VOLUME_INFO_SIZE);
    assert_object_store_volume_info(&memory);

    let owner_process = kernel.current_process_id();
    let sdmmc_volume = kernel.create_volume_handle_for_guest_root("\\SDMMC Disk")?;
    memory.write_bytes(volume_info_ptr, &[0xa5; CE_VOLUME_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_FS_IO_CONTROL_W,
            [
                sdmmc_volume,
                0,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                4,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, CE_VOLUME_INFO_SIZE);
    assert_sdmmc_volume_info(&memory);

    kernel.set_current_process_id(owner_process + 1);
    memory.write_bytes(volume_info_ptr, &[0x5a; CE_VOLUME_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0x1234_5678);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id + 1,
            ORD_AFS_FS_IO_CONTROL_W,
            [
                sdmmc_volume,
                0,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                4,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id + 1),
        ERROR_ACCESS_DENIED
    );
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0x1234_5678);
    assert_eq!(
        memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize),
        vec![0x5a; CE_VOLUME_INFO_SIZE as usize]
    );
    kernel.set_current_process_id(owner_process);

    memory.write_wide_z(find_pattern_ptr, "\\SDMMC Disk");
    let exact_sdmmc_find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_FILE_W,
        [find_pattern_ptr, find_data_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstFileW did not return an exact mount handle: {other:?}"),
    };
    assert_ne!(exact_sdmmc_find, u32::MAX);
    assert_eq!(memory.read_u32(find_data_ptr)?, 0x110);
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "SDMMC Disk");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE,
            [exact_sdmmc_find],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_wide_z(find_pattern_ptr, "\\SDMMC Disk\\*.bin");
    let find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_FILE_W,
        [find_pattern_ptr, find_data_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstFileW did not return a find handle: {other:?}"),
    };
    assert_ne!(find, u32::MAX);
    assert_eq!(memory.read_u32(find_data_ptr)?, 0x20);
    assert_eq!(memory.read_u32(find_data_ptr + 32)?, 7);
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "mapinfo.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_FILE_W,
            [find, find_data_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(find_data_ptr)?, 0x20);
    assert_eq!(memory.read_u32(find_data_ptr + 32)?, 4);
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "z-next.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_FILE_W,
            [find, find_data_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_FILES
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE,
            [find],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_store_manager_enumerates_mounted_stores() -> Result<()> {
    const STORE_INFO_SIZE: u32 = 232;
    const PARTITION_INFO_SIZE: u32 = 296;
    const CE_VOLUME_INFO_SIZE: u32 = 144;
    const FSD_DISK_INFO_SIZE: u32 = 24;
    const STORAGE_DEVICE_INFO_SIZE: u32 = 80;
    const DISK_IOCTL_INITIALIZED: u32 = 4;
    const DISK_IOCTL_GETNAME: u32 = 9;
    const FSCTL_REFRESH_VOLUME: u32 = 0x0009_007c;
    const FSCTL_GET_VOLUME_INFO: u32 = 0x0009_0080;
    const IOCTL_DISK_DEVICE_INFO: u32 = 0x0007_1800;
    const IOCTL_DISK_GETINFO: u32 = 0x0007_1c00;
    const IOCTL_DISK_INITIALIZED: u32 = 0x0007_1c10;
    const IOCTL_DISK_GETNAME: u32 = 0x0007_1c20;
    const IOCTL_DISK_GET_STORAGEID: u32 = 0x0007_1c24;
    const IOCTL_DISK_STANDBY_NOW: u32 = 0x0007_1c1c;
    const IOCTL_DISK_DELETE_CLUSTER: u32 = 0x0007_1c40;
    const IOCTL_DISK_FLUSH_CACHE: u32 = 0x0007_1c54;
    const STORE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
    const STORE_ATTRIBUTE_REMOVABLE: u32 = 0x0000_0002;
    const STORE_ATTRIBUTE_AUTOMOUNT: u32 = 0x0000_0020;
    const PARTITION_ATTRIBUTE_MOUNTED: u32 = 0x0000_0010;
    const CE_VOLUME_ATTRIBUTE_REMOVABLE: u32 = 0x0000_0004;
    const CE_VOLUME_FLAG_STORE: u32 = 0x0000_0020;
    const STORAGE_DEVICE_CLASS_BLOCK: u32 = 0x0000_0001;
    const STORAGE_DEVICE_TYPE_FLASH: u32 = 1 << 1;
    const STORAGE_DEVICE_TYPE_REMOVABLE_DRIVE: u32 = 1 << 30;
    const STORAGE_DEVICE_FLAG_READWRITE: u32 = 1 << 0;
    const STORAGE_DEVICE_FLAG_READONLY: u32 = 1 << 1;
    const ERROR_GEN_FAILURE: u32 = 31;
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const ERROR_BAD_ARGUMENTS: u32 = 160;
    const INVALID_HANDLE_VALUE: u32 = 0xffff_ffff;

    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.storage.mounts.clear();
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("store_manager");
    let sd_root = root.join("sd");
    let flash_root = root.join("flash");
    fs::create_dir_all(&sd_root).unwrap();
    fs::create_dir_all(&flash_root).unwrap();
    kernel.set_file_root(&root);
    kernel.files.mount(MountConfig {
        name: Some("SDMMC Disk".to_owned()),
        device_name: Some("DSK1:".to_owned()),
        bus_name: None,
        guest_root: "\\SDMMC Disk".to_owned(),
        host_root: Some(sd_root),
        total_mbytes: 128,
        free_mbytes: 64,
        writable: true,
        removable: true,
        system: false,
        hidden: false,
        interface_classes: Vec::new(),
        registry_roots: vec![r"Drivers\BuiltIn\SDMemory".to_owned()],
        registry_subkey: Some("SDMemory".to_owned()),
    });
    kernel.files.mount(MountConfig {
        name: Some("ResidentFlash".to_owned()),
        device_name: Some("DSK2:".to_owned()),
        bus_name: None,
        guest_root: "\\ResidentFlash".to_owned(),
        host_root: Some(flash_root),
        total_mbytes: 32,
        free_mbytes: 8,
        writable: false,
        removable: false,
        system: false,
        hidden: false,
        interface_classes: Vec::new(),
        registry_roots: vec![r"Drivers\BuiltIn\FlashDisk".to_owned()],
        registry_subkey: None,
    });

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let name_ptr = 0x1_0000;
    let info_level_ptr = 0x1_0100;
    let bytes_returned_ptr = 0x1_0120;
    let info_ptr = 0x3020_0000;
    let partition_info_ptr = 0x3021_0000;
    let volume_info_ptr = 0x3022_0000;
    let disk_info_ptr = 0x3023_0000;
    let storage_device_info_ptr = 0x3024_0000;
    let disk_name_ptr = 0x3025_0000;
    let storage_id_ptr = 0x3026_0000;
    memory.map_halfwords(name_ptr, 32);
    memory.map_words(info_level_ptr, 1);
    memory.map_words(bytes_returned_ptr, 1);
    memory.map_bytes(info_ptr, STORE_INFO_SIZE);
    memory.map_bytes(partition_info_ptr, PARTITION_INFO_SIZE);
    memory.map_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE);
    memory.map_bytes(disk_info_ptr, FSD_DISK_INFO_SIZE);
    memory.map_bytes(storage_device_info_ptr, STORAGE_DEVICE_INFO_SIZE);
    memory.map_bytes(disk_name_ptr, 64);
    memory.map_bytes(storage_id_ptr, 16);
    memory.write_word(info_level_ptr, 0);

    let read_le_u32 = |bytes: &[u8], offset: usize| -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("DWORD"))
    };
    let read_le_u64 = |bytes: &[u8], offset: usize| -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("QWORD"))
    };
    let read_fixed_wide = |bytes: &[u8], offset: usize, chars: usize| -> String {
        let units = bytes[offset..offset + chars * 2]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|unit| *unit != 0)
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    };
    let assert_sd_store = |memory: &TestGuestMemory| {
        let info = memory.read_bytes(info_ptr, STORE_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&info, 0), STORE_INFO_SIZE);
        assert_eq!(read_fixed_wide(&info, 4, 8), "DSK1:");
        assert_eq!(read_fixed_wide(&info, 20, 32), "SDMMC Disk");
        assert_eq!(read_le_u32(&info, 84), STORAGE_DEVICE_CLASS_BLOCK);
        assert_eq!(
            read_le_u32(&info, 88),
            STORAGE_DEVICE_TYPE_FLASH | STORAGE_DEVICE_TYPE_REMOVABLE_DRIVE
        );
        assert_eq!(read_le_u32(&info, 92), STORAGE_DEVICE_INFO_SIZE);
        assert_eq!(read_fixed_wide(&info, 96, 32), "SDMemory");
        assert_eq!(read_le_u32(&info, 160), STORAGE_DEVICE_CLASS_BLOCK);
        assert_eq!(read_le_u32(&info, 168), STORAGE_DEVICE_FLAG_READWRITE);
        assert_eq!(read_le_u32(&info, 172), STORAGE_DEVICE_FLAG_READWRITE);
        assert_eq!(read_le_u64(&info, 176), 128 * 1024 * 1024 / 512);
        assert_eq!(read_le_u32(&info, 184), 512);
        assert_eq!(read_le_u64(&info, 188), 64 * 1024 * 1024 / 512);
        assert_eq!(read_le_u64(&info, 196), 64 * 1024 * 1024 / 512);
        assert_eq!(
            read_le_u32(&info, 220),
            STORE_ATTRIBUTE_REMOVABLE | STORE_ATTRIBUTE_AUTOMOUNT
        );
        assert_eq!(read_le_u32(&info, 224), 1);
        assert_eq!(read_le_u32(&info, 228), 1);
    };
    let assert_sd_partition = |memory: &TestGuestMemory| {
        let info = memory.read_bytes(partition_info_ptr, PARTITION_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&info, 0), PARTITION_INFO_SIZE);
        assert_eq!(read_fixed_wide(&info, 4, 32), "SDMMC Disk");
        assert_eq!(read_fixed_wide(&info, 68, 32), "FATFS");
        assert_eq!(read_fixed_wide(&info, 132, 64), "SDMMC Disk");
        assert_eq!(read_le_u64(&info, 260), 128 * 1024 * 1024 / 512);
        assert_eq!(read_le_u64(&info, 268), 0);
        assert_eq!(read_le_u64(&info, 276), 0);
        assert_eq!(read_le_u32(&info, 284), PARTITION_ATTRIBUTE_MOUNTED);
        assert_eq!(info[288], 0x04);
    };
    let assert_sd_volume = |memory: &TestGuestMemory| {
        let info = memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&info, 0), CE_VOLUME_INFO_SIZE);
        assert_eq!(
            read_le_u32(&info, 4) & CE_VOLUME_ATTRIBUTE_REMOVABLE,
            CE_VOLUME_ATTRIBUTE_REMOVABLE
        );
        assert_eq!(
            read_le_u32(&info, 8) & CE_VOLUME_FLAG_STORE,
            CE_VOLUME_FLAG_STORE
        );
        assert_eq!(read_le_u32(&info, 12), 4096);
        assert_eq!(read_fixed_wide(&info, 16, 32), "SDMMC Disk");
        assert_eq!(read_fixed_wide(&info, 80, 32), "SDMMC Disk");
    };
    let assert_sd_disk_info = |memory: &TestGuestMemory| {
        let info = memory.read_bytes(disk_info_ptr, FSD_DISK_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&info, 0), 128 * 1024 * 1024 / 512);
        assert_eq!(read_le_u32(&info, 4), 512);
        assert_eq!(read_le_u32(&info, 8), 1);
        assert_eq!(read_le_u32(&info, 12), 1);
        assert_eq!(read_le_u32(&info, 16), 128 * 1024 * 1024 / 512);
        assert_eq!(read_le_u32(&info, 20), 0);
    };
    let assert_sd_storage_device_info = |memory: &TestGuestMemory| {
        let info = memory.read_bytes(storage_device_info_ptr, STORAGE_DEVICE_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&info, 0), STORAGE_DEVICE_INFO_SIZE);
        assert_eq!(read_fixed_wide(&info, 4, 32), "SDMemory");
        assert_eq!(read_le_u32(&info, 68), STORAGE_DEVICE_CLASS_BLOCK);
        assert_eq!(
            read_le_u32(&info, 72),
            STORAGE_DEVICE_TYPE_FLASH | STORAGE_DEVICE_TYPE_REMOVABLE_DRIVE
        );
        assert_eq!(read_le_u32(&info, 76), STORAGE_DEVICE_FLAG_READWRITE);
    };
    let assert_synthetic_storage_id = |memory: &TestGuestMemory| {
        let info = memory.read_bytes(storage_id_ptr, 16);
        assert_eq!(read_le_u32(&info, 0), 16);
        assert_eq!(read_le_u32(&info, 4), 0x03);
        assert_eq!(read_le_u32(&info, 8), 0);
        assert_eq!(read_le_u32(&info, 12), 0);
    };
    let assert_flash_store = |memory: &TestGuestMemory| {
        let info = memory.read_bytes(info_ptr, STORE_INFO_SIZE as usize);
        assert_eq!(read_le_u32(&info, 0), STORE_INFO_SIZE);
        assert_eq!(read_fixed_wide(&info, 4, 8), "DSK2:");
        assert_eq!(read_fixed_wide(&info, 20, 32), "ResidentFlash");
        assert_eq!(read_le_u32(&info, 88), STORAGE_DEVICE_TYPE_FLASH);
        assert_eq!(read_fixed_wide(&info, 96, 32), "FlashDisk");
        assert_eq!(read_le_u32(&info, 172), STORAGE_DEVICE_FLAG_READONLY);
        assert_eq!(read_le_u64(&info, 176), 32 * 1024 * 1024 / 512);
        assert_eq!(read_le_u64(&info, 188), 8 * 1024 * 1024 / 512);
        assert_eq!(
            read_le_u32(&info, 220),
            STORE_ATTRIBUTE_READONLY | STORE_ATTRIBUTE_AUTOMOUNT
        );
    };

    memory.write_wide_z(name_ptr, "DSK1:");
    let store_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_STORE,
        [name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("OpenStore returned {other:?}"),
    };
    assert_ne!(store_handle, INVALID_HANDLE_VALUE);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_STORE_INFO,
            [store_handle, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_sd_store(&memory);

    memory.write_bytes(disk_info_ptr, &[0xa1; FSD_DISK_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_GETINFO,
                disk_info_ptr,
                FSD_DISK_INFO_SIZE,
                disk_info_ptr,
                FSD_DISK_INFO_SIZE,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, FSD_DISK_INFO_SIZE);
    assert_sd_disk_info(&memory);

    memory.write_bytes(
        storage_device_info_ptr,
        &[0xa2; STORAGE_DEVICE_INFO_SIZE as usize],
    );
    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_DEVICE_INFO,
                storage_device_info_ptr,
                STORAGE_DEVICE_INFO_SIZE,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u32(bytes_returned_ptr)?,
        STORAGE_DEVICE_INFO_SIZE
    );
    assert_sd_storage_device_info(&memory);

    memory.write_bytes(
        storage_device_info_ptr,
        &[0xa3; STORAGE_DEVICE_INFO_SIZE as usize],
    );
    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_DEVICE_INFO,
                storage_device_info_ptr,
                12,
                0,
                0,
                bytes_returned_ptr,
                0,
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0xfeed_face);
    assert_eq!(
        memory.read_bytes(storage_device_info_ptr, STORAGE_DEVICE_INFO_SIZE as usize),
        vec![0xa3; STORAGE_DEVICE_INFO_SIZE as usize]
    );

    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_FLUSH_CACHE,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_INITIALIZED,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_STANDBY_NOW,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_bytes(storage_id_ptr, &[0xa4; 16]);
    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_GET_STORAGEID,
                0,
                0,
                storage_id_ptr,
                16,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 16);
    assert_synthetic_storage_id(&memory);

    memory.write_bytes(disk_name_ptr, &[0xab; 64]);
    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_GETNAME,
                0,
                0,
                disk_name_ptr,
                64,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 12);
    assert_eq!(
        read_fixed_wide(&memory.read_bytes(disk_name_ptr, 64), 0, 8),
        "DSK1:"
    );

    memory.write_bytes(disk_name_ptr, &[0xac; 64]);
    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                DISK_IOCTL_GETNAME,
                disk_name_ptr,
                8,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0xfeed_face);
    assert_eq!(memory.read_bytes(disk_name_ptr, 64), vec![0xac; 64]);

    memory.write_bytes(storage_id_ptr, &[0xa5; 16]);
    memory.write_word(bytes_returned_ptr, 0xfeed_face);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                store_handle,
                IOCTL_DISK_GET_STORAGEID,
                0,
                0,
                storage_id_ptr,
                12,
                bytes_returned_ptr,
                0,
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0xfeed_face);
    assert_eq!(memory.read_bytes(storage_id_ptr, 16), vec![0xa5; 16]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FORMAT_STORE,
            [store_handle],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISMOUNT_STORE,
            [store_handle],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);

    memory.write_wide_z(name_ptr, "NewPart");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_PARTITION,
            [store_handle, name_ptr, 0x04, 0, 16, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_PARTITION_EX,
            [store_handle, name_ptr, 0x04, 0, 16, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);

    memory.write_wide_z(name_ptr, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_PARTITION,
            [store_handle, name_ptr, 0x04, 0, 16, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_BAD_ARGUMENTS
    );

    memory.write_wide_z(name_ptr, "MissingPartition");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_PARTITION,
            [store_handle, name_ptr],
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

    memory.write_wide_z(name_ptr, "SDMMC Disk");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_PARTITION,
            [store_handle, name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);

    memory.write_wide_z(name_ptr, "SDMMC Disk");
    let partition_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_PARTITION,
        [store_handle, name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("OpenPartition returned {other:?}"),
    };
    assert_ne!(partition_handle, INVALID_HANDLE_VALUE);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PARTITION_INFO,
            [partition_handle, partition_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_sd_partition(&memory);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOUNT_PARTITION,
            [partition_handle],
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
            ORD_DISMOUNT_PARTITION,
            [partition_handle],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    memory.write_bytes(partition_info_ptr, &[0x6d; PARTITION_INFO_SIZE as usize]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PARTITION_INFO,
            [partition_handle, partition_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_sd_partition(&memory);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_PARTITION_ATTRIBUTES,
            [partition_handle, PARTITION_ATTRIBUTE_MOUNTED],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);

    memory.write_wide_z(name_ptr, "Renamed");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RENAME_PARTITION,
            [partition_handle, name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);
    memory.write_wide_z(name_ptr, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RENAME_PARTITION,
            [partition_handle, name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_BAD_ARGUMENTS
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FORMAT_PARTITION,
            [partition_handle, 0x04, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FORMAT_PARTITION_EX,
            [partition_handle, 0x04, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_GEN_FAILURE);

    memory.write_bytes(volume_info_ptr, &[0x7b; CE_VOLUME_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0xfeed_beef);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                4,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, CE_VOLUME_INFO_SIZE);
    assert_sd_volume(&memory);

    memory.write_bytes(volume_info_ptr, &[0x7c; CE_VOLUME_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0x1234_abcd);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                0,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
                0,
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0x1234_abcd);
    assert_eq!(
        memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize),
        vec![0x7c; CE_VOLUME_INFO_SIZE as usize]
    );

    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                FSCTL_REFRESH_VOLUME,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_bytes(disk_info_ptr, &[0xa6; FSD_DISK_INFO_SIZE as usize]);
    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                IOCTL_DISK_GETINFO,
                0,
                0,
                disk_info_ptr,
                FSD_DISK_INFO_SIZE,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, FSD_DISK_INFO_SIZE);
    assert_sd_disk_info(&memory);

    memory.write_bytes(
        storage_device_info_ptr,
        &[0xa7; STORAGE_DEVICE_INFO_SIZE as usize],
    );
    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                IOCTL_DISK_DEVICE_INFO,
                storage_device_info_ptr,
                STORAGE_DEVICE_INFO_SIZE,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u32(bytes_returned_ptr)?,
        STORAGE_DEVICE_INFO_SIZE
    );
    assert_sd_storage_device_info(&memory);

    memory.write_bytes(storage_id_ptr, &[0xaa; 16]);
    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                IOCTL_DISK_GET_STORAGEID,
                0,
                0,
                storage_id_ptr,
                16,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 16);
    assert_synthetic_storage_id(&memory);

    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                IOCTL_DISK_FLUSH_CACHE,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                DISK_IOCTL_INITIALIZED,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                IOCTL_DISK_DELETE_CLUSTER,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_bytes(disk_name_ptr, &[0xa8; 64]);
    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                IOCTL_DISK_GETNAME,
                0,
                0,
                disk_name_ptr,
                64,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 22);
    assert_eq!(
        read_fixed_wide(&memory.read_bytes(disk_name_ptr, 64), 0, 32),
        "SDMMC Disk"
    );

    memory.write_bytes(disk_name_ptr, &[0xa9; 64]);
    memory.write_word(bytes_returned_ptr, 0xfeed_babe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                partition_handle,
                IOCTL_DISK_GETNAME,
                0,
                0,
                disk_name_ptr,
                8,
                bytes_returned_ptr,
                0,
            ],
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0xfeed_babe);
    assert_eq!(memory.read_bytes(disk_name_ptr, 64), vec![0xa9; 64]);

    let partition_search_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_PARTITION,
        [store_handle, partition_info_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstPartition returned {other:?}"),
    };
    assert_ne!(partition_search_handle, INVALID_HANDLE_VALUE);
    assert_sd_partition(&memory);

    memory.write_bytes(partition_info_ptr, &[0xa5; PARTITION_INFO_SIZE as usize]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_PARTITION,
            [partition_search_handle, partition_info_ptr],
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
    assert_eq!(
        memory.read_bytes(partition_info_ptr, PARTITION_INFO_SIZE as usize),
        vec![0xa5; PARTITION_INFO_SIZE as usize]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_PARTITION,
            [partition_search_handle],
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
            ORD_FIND_CLOSE_PARTITION,
            [partition_search_handle],
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
    memory.write_wide_z(name_ptr, "MissingPartition");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPEN_PARTITION,
            [store_handle, name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(INVALID_HANDLE_VALUE),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );

    memory.write_wide_z(name_ptr, r"\DSK1:");
    let slash_store_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPEN_STORE,
        [name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("OpenStore with leading slash returned {other:?}"),
    };
    assert_ne!(slash_store_handle, INVALID_HANDLE_VALUE);

    let search_handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_STORE,
        [info_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstStore returned {other:?}"),
    };
    assert_ne!(search_handle, INVALID_HANDLE_VALUE);
    assert_sd_store(&memory);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_STORE,
            [search_handle, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_flash_store(&memory);

    memory.write_bytes(info_ptr, &[0xa5; STORE_INFO_SIZE as usize]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_STORE,
            [search_handle, info_ptr],
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
    assert_eq!(
        memory.read_bytes(info_ptr, STORE_INFO_SIZE as usize),
        vec![0xa5; STORE_INFO_SIZE as usize]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_STORE,
            [search_handle],
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
            ORD_FIND_CLOSE_STORE,
            [search_handle],
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
    for handle in [store_handle, slash_store_handle, partition_handle] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CLOSE_HANDLE,
                [handle],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    Ok(())
}

#[test]
fn coredll_raw_lock_file_ex_validates_file_handle_and_overlapped() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("lock_file_ex_validation");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let find_pattern_ptr = 0x1_0100;
    let find_data_ptr = 0x1_0200;
    let overlapped_ptr = 0x1_1000;
    let overlapped2_ptr = 0x1_1040;
    memory.write_wide_z(path_ptr, "\\ResidentFlash\\locked.bin");
    memory.write_wide_z(find_pattern_ptr, "\\ResidentFlash\\*.bin");
    memory.map_words(find_data_ptr, 11);
    memory.map_halfwords(find_data_ptr + 40, 260);
    memory.map_bytes(overlapped_ptr, 20);
    memory.map_bytes(overlapped2_ptr, 20);
    memory.write_word(overlapped_ptr + 8, 3);
    memory.write_bytes(overlapped_ptr + 8, &3u32.to_le_bytes());
    memory.write_word(overlapped2_ptr + 8, 5);
    memory.write_bytes(overlapped2_ptr + 8, &5u32.to_le_bytes());

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            CREATE_ALWAYS,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    let peer_file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_EXISTING,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("second CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(peer_file, u32::MAX);

    match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOCK_FILE_EX,
        [file, 0, 0, 7, 0, overlapped_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        } => {}
        other => panic!(
            "LockFileEx valid file returned {other:?}, last_error={}",
            kernel.threads.get_last_error(thread_id)
        ),
    }
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(memory.read_u32(overlapped_ptr + 8)?, 3);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOCK_FILE_EX,
            [peer_file, 0, 0, 2, 0, overlapped2_ptr],
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
            ORD_LOCK_FILE_EX,
            [peer_file, 0x0000_0003, 0, 2, 0, overlapped2_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_LOCK_VIOLATION
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_UNLOCK_FILE_EX,
            [peer_file, 0, 7, 0, overlapped_ptr],
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
            ORD_UNLOCK_FILE_EX,
            [file, 0, 7, 0, overlapped_ptr],
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
            ORD_UNLOCK_FILE_EX,
            [peer_file, 0, 2, 0, overlapped2_ptr],
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
            ORD_UNLOCK_FILE_EX,
            [peer_file, 0, 2, 0, overlapped2_ptr],
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
            ORD_LOCK_FILE_EX,
            [file, 0, 0, 1, 0, 0],
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
            ORD_LOCK_FILE_EX,
            [file, 0, 0, 0, 0, overlapped_ptr],
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

    memory.write_word(overlapped2_ptr + 8, u32::MAX);
    memory.write_word(overlapped2_ptr + 12, u32::MAX);
    memory.write_bytes(overlapped2_ptr + 8, &u32::MAX.to_le_bytes());
    memory.write_bytes(overlapped2_ptr + 12, &u32::MAX.to_le_bytes());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOCK_FILE_EX,
            [file, 0, 0, 1, 0, overlapped2_ptr],
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
            ORD_UNLOCK_FILE_EX,
            [file, 0, 1, 0, overlapped2_ptr],
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
            ORD_LOCK_FILE_EX,
            [file, 0, 0, 2, 0, overlapped2_ptr],
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

    memory.write_word(overlapped2_ptr + 8, 100);
    memory.write_word(overlapped2_ptr + 12, 0);
    memory.write_bytes(overlapped2_ptr + 8, &100u32.to_le_bytes());
    memory.write_bytes(overlapped2_ptr + 12, &0u32.to_le_bytes());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOCK_FILE_EX,
            [file, 0x0000_0002, 0, 4, 0, overlapped2_ptr],
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
            ORD_CLOSE_HANDLE,
            [file],
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
            ORD_LOCK_FILE_EX,
            [peer_file, 0x0000_0002, 0, 4, 0, overlapped2_ptr],
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
            ORD_UNLOCK_FILE_EX,
            [peer_file, 0, 4, 0, overlapped2_ptr],
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
            ORD_LOCK_FILE_EX,
            [file, 0, 0, 7, 0, 0x2_0000],
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

    let find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_FILE_W,
        [find_pattern_ptr, find_data_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstFileW did not return a find handle: {other:?}"),
    };
    assert_ne!(find, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOCK_FILE_EX,
            [find, 0, 0, 7, 0, overlapped_ptr],
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
            ORD_CLOSE_HANDLE,
            [peer_file],
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
            ORD_FIND_CLOSE,
            [find],
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
fn coredll_raw_system_and_hidden_mounts_follow_fsdmgr_attributes() -> Result<()> {
    const FSCTL_GET_VOLUME_INFO: u32 = 0x0009_0080;
    const CE_VOLUME_INFO_SIZE: u32 = 144;
    const CE_VOLUME_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
    const CE_VOLUME_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
    const CE_VOLUME_ATTRIBUTE_REMOVABLE: u32 = 0x0000_0004;
    const CE_VOLUME_ATTRIBUTE_SYSTEM: u32 = 0x0000_0008;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("system_hidden_mount_attributes");
    let object_root = root.join("object");
    let system_root = root.join("system_volume");
    let hidden_root = root.join("hidden_volume");
    let readonly_root = root.join("readonly_volume");
    fs::create_dir_all(&object_root).unwrap();
    fs::create_dir_all(&system_root).unwrap();
    fs::create_dir_all(&hidden_root).unwrap();
    fs::create_dir_all(&readonly_root).unwrap();
    fs::write(system_root.join("system.bin"), b"system").unwrap();
    fs::write(hidden_root.join("secret.bin"), b"hidden").unwrap();
    kernel.set_file_root(&object_root);
    kernel.files.mount(MountConfig {
        name: Some("ResidentFlash".to_owned()),
        device_name: None,
        bus_name: None,
        guest_root: "\\ResidentFlash".to_owned(),
        host_root: Some(system_root.clone()),
        total_mbytes: 64,
        free_mbytes: 32,
        writable: true,
        removable: false,
        system: true,
        hidden: false,
        interface_classes: Vec::new(),
        registry_roots: Vec::new(),
        registry_subkey: None,
    });
    kernel.files.mount(MountConfig {
        name: Some("HiddenStore".to_owned()),
        device_name: None,
        bus_name: None,
        guest_root: "\\HiddenStore".to_owned(),
        host_root: Some(hidden_root.clone()),
        total_mbytes: 16,
        free_mbytes: 8,
        writable: true,
        removable: false,
        system: false,
        hidden: true,
        interface_classes: Vec::new(),
        registry_roots: Vec::new(),
        registry_subkey: None,
    });

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let find_data_ptr = 0x1_1000;
    let info_level_ptr = 0x1_2000;
    let bytes_returned_ptr = 0x1_2004;
    let volume_info_ptr = 0x3020_0000;
    memory.map_halfwords(path_ptr, 128);
    memory.map_words(find_data_ptr, 11);
    memory.map_halfwords(find_data_ptr + 40, 260);
    memory.map_words(info_level_ptr, 2);
    memory.map_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE);
    memory.write_word(info_level_ptr, 0);

    memory.write_wide_z(path_ptr, r"\ResidentFlash\system.bin");
    let system_file_attrs = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_FILE_ATTRIBUTES_W,
        [path_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(attributes),
            ..
        } => attributes,
        other => panic!("GetFileAttributesW did not return attributes: {other:?}"),
    };
    assert_eq!(
        system_file_attrs,
        FILE_ATTRIBUTE_ARCHIVE | FILE_ATTRIBUTE_SYSTEM
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_SYSTEM_FILE,
            [path_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\HiddenStore\secret.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_SYSTEM_FILE,
            [path_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_wide_z(path_ptr, r"\ResidentFlash\missing.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_SYSTEM_FILE,
            [path_ptr],
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
            ORD_IS_SYSTEM_FILE,
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

    memory.write_wide_z(path_ptr, r"\ResidentFlash\*.bin");
    let system_find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_FILE_W,
        [path_ptr, find_data_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstFileW did not return a system-volume find handle: {other:?}"),
    };
    assert_ne!(system_find, u32::MAX);
    assert_eq!(
        memory.read_u32(find_data_ptr)?,
        FILE_ATTRIBUTE_ARCHIVE | FILE_ATTRIBUTE_SYSTEM
    );
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "system.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE,
            [system_find],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_wide_z(path_ptr, r"\");
    let root_find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_FILE_W,
        [path_ptr, find_data_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstFileW did not return a root find handle: {other:?}"),
    };
    assert_ne!(root_find, u32::MAX);
    assert_eq!(
        memory.read_u32(find_data_ptr)?,
        FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_SYSTEM
    );
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "ResidentFlash");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_FILE_W,
            [root_find, find_data_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_FILES
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE,
            [root_find],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_wide_z(path_ptr, r"\HiddenStore");
    let hidden_find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_FILE_W,
        [path_ptr, find_data_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstFileW did not return a hidden mount handle: {other:?}"),
    };
    assert_ne!(hidden_find, u32::MAX);
    assert_eq!(
        memory.read_u32(find_data_ptr)?,
        FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_HIDDEN
    );
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "HiddenStore");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE,
            [hidden_find],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let read_le_u32 = |bytes: &[u8], offset: usize| -> u32 {
        u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("volume info DWORD"),
        )
    };
    memory.write_wide_z(path_ptr, r"\ResidentFlash\system.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_VOLUME_INFO_W,
            [path_ptr, 0, volume_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let volume = memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize);
    assert_eq!(read_le_u32(&volume, 0), CE_VOLUME_INFO_SIZE);
    assert_eq!(
        read_le_u32(&volume, 4) & CE_VOLUME_ATTRIBUTE_SYSTEM,
        CE_VOLUME_ATTRIBUTE_SYSTEM
    );

    memory.write_wide_z(path_ptr, r"\HiddenStore");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_VOLUME_INFO_W,
            [path_ptr, 0, volume_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let volume = memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize);
    assert_eq!(
        read_le_u32(&volume, 4) & CE_VOLUME_ATTRIBUTE_HIDDEN,
        CE_VOLUME_ATTRIBUTE_HIDDEN
    );

    kernel.files.mount(MountConfig {
        name: Some("ReadOnlyCard".to_owned()),
        device_name: None,
        bus_name: None,
        guest_root: "\\ReadOnlyCard".to_owned(),
        host_root: Some(readonly_root.clone()),
        total_mbytes: 32,
        free_mbytes: 12,
        writable: false,
        removable: true,
        system: false,
        hidden: false,
        interface_classes: Vec::new(),
        registry_roots: Vec::new(),
        registry_subkey: None,
    });

    memory.write_wide_z(path_ptr, r"\ReadOnlyCard");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_VOLUME_INFO_W,
            [path_ptr, 0, volume_info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let volume = memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize);
    let volume_attributes = read_le_u32(&volume, 4);
    assert_eq!(
        volume_attributes & CE_VOLUME_ATTRIBUTE_READONLY,
        CE_VOLUME_ATTRIBUTE_READONLY
    );
    assert_eq!(
        volume_attributes & CE_VOLUME_ATTRIBUTE_REMOVABLE,
        CE_VOLUME_ATTRIBUTE_REMOVABLE
    );
    assert_eq!(
        volume_attributes & (CE_VOLUME_ATTRIBUTE_HIDDEN | CE_VOLUME_ATTRIBUTE_SYSTEM),
        0
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_GET_VOLUME_INFO,
                info_level_ptr,
                4,
                volume_info_ptr,
                CE_VOLUME_INFO_SIZE,
                bytes_returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, CE_VOLUME_INFO_SIZE);
    let volume = memory.read_bytes(volume_info_ptr, CE_VOLUME_INFO_SIZE as usize);
    let volume_attributes = read_le_u32(&volume, 4);
    assert_eq!(
        volume_attributes & (CE_VOLUME_ATTRIBUTE_READONLY | CE_VOLUME_ATTRIBUTE_REMOVABLE),
        CE_VOLUME_ATTRIBUTE_READONLY | CE_VOLUME_ATTRIBUTE_REMOVABLE
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_fs_io_control_refresh_and_flush_are_no_ops() -> Result<()> {
    const FSCTL_COPY_EXTERNAL_START: u32 = 0x0009_004c;
    const FSCTL_COPY_EXTERNAL_COMPLETE: u32 = 0x0009_0050;
    const FSCTL_REFRESH_VOLUME: u32 = 0x0009_007c;
    const FSCTL_FLUSH_BUFFERS: u32 = 0x0009_0084;
    const FSCTL_STORAGE_MEDIA_CHANGE_EVENT: u32 = 0x0009_00ac;
    const STORAGE_MEDIA_CHANGE_EVENT_DETACHED: u32 = 0;
    const STORAGE_MEDIA_CHANGE_EVENT_ATTACHED: u32 = 1;
    const STORAGE_MEDIA_ATTACH_RESULT_UNCHANGED: u32 = 0;
    const FILE_COPY_EXTERNAL_SIZE: u32 = 536;
    const ERROR_INVALID_PARAMETER: u32 = 87;
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const ERROR_NOT_SUPPORTED: u32 = 50;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("fs_io_control_refresh_flush");
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root("\\SDMMC Disk", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let bytes_returned_ptr = 0x1_8000u32;
    let media_event_ptr = 0x1_8300u32;
    let media_result_ptr = 0x1_8310u32;
    memory.map_words(bytes_returned_ptr, 1);
    memory.map_words(media_event_ptr, 1);
    memory.map_words(media_result_ptr, 1);

    // FSCTL_REFRESH_VOLUME via CeFsIoControlW: no-op, returns true, bytes_returned = 0
    let path_ptr = 0x1_8100u32;
    memory.map_bytes(path_ptr, 32);
    memory.write_wide_z(path_ptr, "\\SDMMC Disk");
    memory.write_word(bytes_returned_ptr, 0xDEAD_BEEF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_REFRESH_VOLUME,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    // CE's mounted-volume media-change path lets FSDs ignore detach and report
    // attach as unchanged. Host-backed mounts have no removable media churn.
    memory.write_word(media_event_ptr, STORAGE_MEDIA_CHANGE_EVENT_DETACHED);
    memory.write_word(media_result_ptr, 0xfeed_cafe);
    memory.write_word(bytes_returned_ptr, 0xDEAD_BEEF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_STORAGE_MEDIA_CHANGE_EVENT,
                media_event_ptr,
                4,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);
    assert_eq!(memory.read_u32(media_result_ptr)?, 0xfeed_cafe);

    memory.write_word(media_event_ptr, STORAGE_MEDIA_CHANGE_EVENT_ATTACHED);
    memory.write_word(media_result_ptr, 0xfeed_cafe);
    memory.write_word(bytes_returned_ptr, 0xDEAD_BEEF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_STORAGE_MEDIA_CHANGE_EVENT,
                media_event_ptr,
                4,
                media_result_ptr,
                4,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u32(media_result_ptr)?,
        STORAGE_MEDIA_ATTACH_RESULT_UNCHANGED
    );
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 4);

    memory.write_word(media_result_ptr, 0xfeed_cafe);
    memory.write_word(bytes_returned_ptr, 0xDEAD_BEEF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_STORAGE_MEDIA_CHANGE_EVENT,
                media_event_ptr,
                4,
                media_result_ptr,
                2,
                bytes_returned_ptr,
                0
            ],
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 4);
    assert_eq!(memory.read_u32(media_result_ptr)?, 0xfeed_cafe);

    let copy_external_ptr = 0x1_8200u32;
    let copy_external_out_ptr = 0x1_8500u32;
    memory.map_bytes(copy_external_ptr, FILE_COPY_EXTERNAL_SIZE);
    memory.map_bytes(copy_external_out_ptr, 4);
    memory.write_word(copy_external_ptr, FILE_COPY_EXTERNAL_SIZE);
    memory.write_bytes(copy_external_ptr + 32, &[0xa5]);
    memory.write_bytes(copy_external_out_ptr, &[0x5a, 0x5a, 0x5a, 0x5a]);
    memory.write_word(bytes_returned_ptr, 0xDEAD_BEEF);
    for ioctl in [FSCTL_COPY_EXTERNAL_START, FSCTL_COPY_EXTERNAL_COMPLETE] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CE_FS_IO_CONTROL_W,
                [
                    path_ptr,
                    ioctl,
                    copy_external_ptr,
                    FILE_COPY_EXTERNAL_SIZE,
                    copy_external_out_ptr,
                    4,
                    bytes_returned_ptr,
                    0
                ],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ));
        assert_eq!(
            kernel.threads.get_last_error(thread_id),
            ERROR_NOT_SUPPORTED
        );
        assert_eq!(memory.read_bytes(copy_external_ptr + 32, 1), vec![0xa5]);
        assert_eq!(
            memory.read_bytes(copy_external_out_ptr, 4),
            vec![0x5a, 0x5a, 0x5a, 0x5a]
        );
        assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0xDEAD_BEEF);
    }

    memory.write_word(copy_external_ptr, FILE_COPY_EXTERNAL_SIZE - 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_COPY_EXTERNAL_START,
                copy_external_ptr,
                FILE_COPY_EXTERNAL_SIZE,
                copy_external_out_ptr,
                4,
                bytes_returned_ptr,
                0
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

    // FSCTL_FLUSH_BUFFERS via CeFsIoControlW: no-op, returns true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_FLUSH_BUFFERS,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    // FSCTL_REFRESH_VOLUME via AFS_FsIoControlW (object store path): no-op too
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_FS_IO_CONTROL_W,
            [0, 0, FSCTL_REFRESH_VOLUME, 0, 0, 0, 0, bytes_returned_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_word(media_event_ptr, STORAGE_MEDIA_CHANGE_EVENT_ATTACHED);
    memory.write_word(media_result_ptr, 0xfeed_cafe);
    memory.write_word(bytes_returned_ptr, 0xDEAD_BEEF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_FS_IO_CONTROL_W,
            [
                0,
                0,
                FSCTL_STORAGE_MEDIA_CHANGE_EVENT,
                media_event_ptr,
                4,
                media_result_ptr,
                4,
                bytes_returned_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u32(media_result_ptr)?,
        STORAGE_MEDIA_ATTACH_RESULT_UNCHANGED
    );
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 4);

    memory.write_word(media_event_ptr, 99);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_STORAGE_MEDIA_CHANGE_EVENT,
                media_event_ptr,
                4,
                media_result_ptr,
                4,
                bytes_returned_ptr,
                0
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

    // Unknown FSCTL returns false and ERROR_NOT_SUPPORTED
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_FS_IO_CONTROL_W,
            [path_ptr, 0x0009_0099, 0, 0, 0, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
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
fn coredll_raw_public_afs_registration_reserves_binds_and_removes_mounts() -> Result<()> {
    const AFS_FLAG_HIDDEN: u32 = 0x0001;
    const AFS_FLAG_SYSTEM: u32 = 0x0020;
    const AFS_FLAG_PERMANENT: u32 = 0x0040;
    const AFS_VERSION: u32 = 0x0000_0004;
    const ERROR_INVALID_INDEX: u32 = 1413;
    const INVALID_MOUNT_INDEX: u32 = u32::MAX;
    const INVALID_FILE_ATTRIBUTES: u32 = u32::MAX;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let name_ptr = 0x3100_0000;
    let path_ptr = 0x3100_0100;
    memory.write_wide_z(name_ptr, "RouteCache");
    memory.write_wide_z(path_ptr, "\\RouteCache");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_AFSNAME,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(INVALID_MOUNT_INDEX),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    let index = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTER_AFSNAME,
        [name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(index),
            ..
        } => index,
        other => panic!("unexpected RegisterAFSName result: {other:?}"),
    };
    assert!(index >= 2);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_AFSNAME,
            [name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(duplicate),
            ..
        } if duplicate == index
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
            ORD_REGISTER_AFSEX,
            [index, 0, 0xfeed_cafe, AFS_VERSION, 0],
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
            ORD_REGISTER_AFSEX,
            [
                index,
                0x1234,
                0xfeed_cafe,
                AFS_VERSION,
                AFS_FLAG_HIDDEN | AFS_FLAG_SYSTEM | AFS_FLAG_PERMANENT,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    let attrs = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_FILE_ATTRIBUTES_W,
        [path_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(attrs),
            ..
        } => attrs,
        other => panic!("unexpected GetFileAttributesW result: {other:?}"),
    };
    assert_ne!(attrs, INVALID_FILE_ATTRIBUTES);
    assert_eq!(
        attrs & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM),
        FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_AFSEX,
            [index, 0x1234, 0xbeef, AFS_VERSION, 0],
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
            ORD_DEREGISTER_AFSNAME,
            [0x7fff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_INDEX
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEREGISTER_AFS,
            [index],
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
            ORD_GET_FILE_ATTRIBUTES_W,
            [path_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(INVALID_FILE_ATTRIBUTES),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_file_handle_set_file_cache_follows_cache_filter_shape() -> Result<()> {
    const FSCTL_SET_FILE_CACHE: u32 = 0x0009_0090;
    const FSCTL_READ_OR_WRITE_SECURITY_DESCRIPTOR: u32 = 0x0009_00a8;
    const FILE_CACHE_ENABLE_STANDARD: u32 = 0;
    const FILE_CACHE_DISABLE_STANDARD: u32 = 2;
    const ERROR_INVALID_HANDLE: u32 = 6;
    const ERROR_INVALID_PARAMETER: u32 = 87;
    const ERROR_NOT_SUPPORTED: u32 = 50;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let root = unique_test_root("file_handle_set_file_cache");
    fs::create_dir_all(&root).unwrap();
    kernel.files = HostFileSystem::new(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x1_9000u32;
    let cache_info_ptr = 0x1_9100u32;
    let bytes_returned_ptr = 0x1_9200u32;
    memory.write_wide_z(path_ptr, "\\cache.bin");
    memory.map_words(cache_info_ptr, 1);
    memory.map_words(bytes_returned_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_ALWAYS,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };

    memory.write_word(bytes_returned_ptr, 0xABCD_1234);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_READ_OR_WRITE_SECURITY_DESCRIPTOR,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
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
    assert_eq!(
        memory.read_u32(bytes_returned_ptr)?,
        0xABCD_1234,
        "CE FSDMGR rejects external security-descriptor FSCTLs before touching byte counts"
    );

    memory.write_word(cache_info_ptr, FILE_CACHE_DISABLE_STANDARD);
    memory.write_word(bytes_returned_ptr, 0xDEAD_BEEF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_FILE_CACHE,
                cache_info_ptr,
                4,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(cache_info_ptr, FILE_CACHE_ENABLE_STANDARD);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_FILE_CACHE,
                cache_info_ptr,
                4,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [file, FSCTL_SET_FILE_CACHE, cache_info_ptr, 0, 0, 0, 0, 0],
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
            ORD_CE_FS_IO_CONTROL_W,
            [
                path_ptr,
                FSCTL_SET_FILE_CACHE,
                cache_info_ptr,
                4,
                0,
                0,
                0,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_FILE_CACHE,
                cache_info_ptr,
                4,
                0,
                0,
                bytes_returned_ptr,
                0
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

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_file_handle_get_stream_information_reports_standard_stream() -> Result<()> {
    const FSCTL_GET_STREAM_INFORMATION: u32 = 0x0009_4038;
    const FILE_STREAM_INFO_STANDARD: u32 = 0;
    const FILE_STREAM_INFO_SIZE: u32 = 52;
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let root = unique_test_root("file_handle_get_stream_information");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("stream.bin"), b"ce stream data").unwrap();
    kernel.files = HostFileSystem::new(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x1_a000u32;
    let stream_info_ptr = 0x1_a100u32;
    let bytes_returned_ptr = 0x1_a200u32;
    memory.write_wide_z(path_ptr, "\\stream.bin");
    memory.map_bytes(stream_info_ptr, FILE_STREAM_INFO_SIZE);
    memory.map_words(bytes_returned_ptr, 1);
    memory.write_word(stream_info_ptr, FILE_STREAM_INFO_STANDARD);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_GET_STREAM_INFORMATION,
                0,
                0,
                stream_info_ptr,
                FILE_STREAM_INFO_SIZE,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, FILE_STREAM_INFO_SIZE);
    let stream_info = memory.read_bytes(stream_info_ptr, FILE_STREAM_INFO_SIZE as usize);
    let stream_u32 =
        |offset: usize| u32::from_le_bytes(stream_info[offset..offset + 4].try_into().unwrap());
    assert_eq!(stream_u32(0), FILE_STREAM_INFO_STANDARD);
    assert_eq!(
        stream_u32(4) & FILE_ATTRIBUTE_ARCHIVE,
        FILE_ATTRIBUTE_ARCHIVE
    );
    assert_eq!(stream_u32(8), 0);
    assert_ne!(stream_u32(12), 0);
    assert_eq!(stream_u32(36), b"ce stream data".len() as u32);
    assert_eq!(stream_u32(40), 0);
    assert_eq!(stream_u32(44), b"ce stream data".len() as u32);
    assert_eq!(stream_u32(48), 0);

    memory.write_word(bytes_returned_ptr, 0xdead_beef);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_GET_STREAM_INFORMATION,
                0,
                0,
                stream_info_ptr,
                FILE_STREAM_INFO_SIZE - 4,
                bytes_returned_ptr,
                0
            ],
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, FILE_STREAM_INFO_SIZE);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_file_handle_compression_reports_uncompressed() -> Result<()> {
    const FSCTL_GET_COMPRESSION: u32 = 0x0009_003c;
    const FSCTL_SET_COMPRESSION: u32 = 0x0009_c040;
    const COMPRESSION_FORMAT_NONE: u16 = 0;
    const COMPRESSION_FORMAT_LZNT1: u16 = 2;
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const ERROR_INVALID_PARAMETER: u32 = 87;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let root = unique_test_root("file_handle_compression");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("plain.bin"), b"plain ce data").unwrap();
    kernel.files = HostFileSystem::new(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x1_ac00u32;
    let format_ptr = 0x1_ad00u32;
    let bytes_returned_ptr = 0x1_ae00u32;
    memory.write_wide_z(path_ptr, "\\plain.bin");
    memory.map_halfwords(format_ptr, 1);
    memory.map_words(bytes_returned_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_EXISTING,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };

    memory.write_halfword(format_ptr, 0xffff);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_GET_COMPRESSION,
                0,
                0,
                format_ptr,
                2,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u16(format_ptr)?, COMPRESSION_FORMAT_NONE);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 2);

    memory.write_halfword(format_ptr, 0xbeef);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_GET_COMPRESSION,
                0,
                0,
                format_ptr,
                1,
                bytes_returned_ptr,
                0
            ],
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 2);
    assert_eq!(memory.read_u16(format_ptr)?, 0xbeef);

    memory.write_halfword(format_ptr, COMPRESSION_FORMAT_NONE);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_COMPRESSION,
                format_ptr,
                2,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_halfword(format_ptr, COMPRESSION_FORMAT_LZNT1);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_COMPRESSION,
                format_ptr,
                2,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_COMPRESSION,
                0,
                0,
                0,
                0,
                bytes_returned_ptr,
                0
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

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_file_handle_query_allocated_ranges_reports_host_extent() -> Result<()> {
    const FSCTL_QUERY_ALLOCATED_RANGES: u32 = 0x0009_40cf;
    const FILE_ALLOCATED_RANGE_BUFFER_SIZE: u32 = 16;
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const ERROR_INVALID_HANDLE: u32 = 6;
    const ERROR_INVALID_PARAMETER: u32 = 87;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let root = unique_test_root("file_handle_query_allocated_ranges");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("ranges.bin"), b"0123456789").unwrap();
    kernel.files = HostFileSystem::new(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x1_c000u32;
    let query_ptr = 0x1_c100u32;
    let out_ptr = 0x1_c200u32;
    let bytes_returned_ptr = 0x1_c300u32;
    memory.write_wide_z(path_ptr, "\\ranges.bin");
    memory.map_words(query_ptr, 4);
    memory.map_words(out_ptr, 4);
    memory.map_words(bytes_returned_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };

    let read_range_u64 = |memory: &TestGuestMemory, addr: u32| -> Result<u64> {
        Ok(u64::from(memory.read_u32(addr)?) | (u64::from(memory.read_u32(addr + 4)?) << 32))
    };

    memory.write_word(query_ptr, 2);
    memory.write_word(query_ptr + 4, 0);
    memory.write_word(query_ptr + 8, 20);
    memory.write_word(query_ptr + 12, 0);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_QUERY_ALLOCATED_RANGES,
                query_ptr,
                FILE_ALLOCATED_RANGE_BUFFER_SIZE,
                out_ptr,
                FILE_ALLOCATED_RANGE_BUFFER_SIZE,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u32(bytes_returned_ptr)?,
        FILE_ALLOCATED_RANGE_BUFFER_SIZE
    );
    assert_eq!(read_range_u64(&memory, out_ptr)?, 2);
    assert_eq!(read_range_u64(&memory, out_ptr + 8)?, 8);

    memory.write_word(out_ptr, 0xaaaa_aaaa);
    memory.write_word(out_ptr + 4, 0xbbbb_bbbb);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_QUERY_ALLOCATED_RANGES,
                query_ptr,
                FILE_ALLOCATED_RANGE_BUFFER_SIZE,
                out_ptr,
                8,
                bytes_returned_ptr,
                0
            ],
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
    assert_eq!(
        memory.read_u32(bytes_returned_ptr)?,
        FILE_ALLOCATED_RANGE_BUFFER_SIZE
    );
    assert_eq!(memory.read_u32(out_ptr)?, 0xaaaa_aaaa);
    assert_eq!(memory.read_u32(out_ptr + 4)?, 0xbbbb_bbbb);

    memory.write_word(query_ptr, 12);
    memory.write_word(query_ptr + 8, 4);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_QUERY_ALLOCATED_RANGES,
                query_ptr,
                FILE_ALLOCATED_RANGE_BUFFER_SIZE,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(query_ptr, 0);
    memory.write_word(query_ptr + 8, 0);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_QUERY_ALLOCATED_RANGES,
                query_ptr,
                FILE_ALLOCATED_RANGE_BUFFER_SIZE,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_QUERY_ALLOCATED_RANGES,
                query_ptr,
                8,
                out_ptr,
                16,
                bytes_returned_ptr,
                0
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
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_QUERY_ALLOCATED_RANGES,
                query_ptr,
                FILE_ALLOCATED_RANGE_BUFFER_SIZE,
                out_ptr,
                FILE_ALLOCATED_RANGE_BUFFER_SIZE,
                bytes_returned_ptr,
                0
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

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_file_handle_set_zero_data_zeros_existing_range() -> Result<()> {
    const FSCTL_SET_ZERO_DATA: u32 = 0x0009_80c8;
    const ERROR_ACCESS_DENIED: u32 = 5;
    const ERROR_INVALID_PARAMETER: u32 = 87;

    let table = CoredllExportTable::default();
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let root = unique_test_root("file_handle_set_zero_data");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("zero.bin"), b"0123456789").unwrap();
    kernel.files = HostFileSystem::new(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let path_ptr = 0x1_b000u32;
    let zero_info_ptr = 0x1_b100u32;
    let bytes_returned_ptr = 0x1_b200u32;
    memory.write_wide_z(path_ptr, "\\zero.bin");
    memory.map_words(zero_info_ptr, 4);
    memory.map_words(bytes_returned_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_EXISTING,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a writable file handle: {other:?}"),
    };

    memory.write_word(zero_info_ptr, 3);
    memory.write_word(zero_info_ptr + 4, 0);
    memory.write_word(zero_info_ptr + 8, 6);
    memory.write_word(zero_info_ptr + 12, 0);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_ZERO_DATA,
                zero_info_ptr,
                16,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(zero_info_ptr, 8);
    memory.write_word(zero_info_ptr + 8, 20);
    memory.write_word(bytes_returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_ZERO_DATA,
                zero_info_ptr,
                16,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(memory.read_u32(bytes_returned_ptr)?, 0);

    memory.write_word(zero_info_ptr, 7);
    memory.write_word(zero_info_ptr + 8, 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                FSCTL_SET_ZERO_DATA,
                zero_info_ptr,
                16,
                0,
                0,
                bytes_returned_ptr,
                0
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
            ORD_CLOSE_HANDLE,
            [file],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        fs::read(root.join("zero.bin")).unwrap(),
        vec![b'0', b'1', b'2', 0, 0, 0, b'6', b'7', 0, 0]
    );

    let readonly = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a read-only file handle: {other:?}"),
    };
    memory.write_word(zero_info_ptr, 0);
    memory.write_word(zero_info_ptr + 8, 2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                readonly,
                FSCTL_SET_ZERO_DATA,
                zero_info_ptr,
                16,
                0,
                0,
                bytes_returned_ptr,
                0
            ],
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
    assert_eq!(
        fs::read(root.join("zero.bin")).unwrap(),
        vec![b'0', b'1', b'2', 0, 0, 0, b'6', b'7', 0, 0]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [readonly],
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
fn coredll_raw_copy_file_w_copies_between_ce_paths() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("copy_file_w_raw");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    fs::write(root.join("source.txt"), b"copy payload").unwrap();
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let src = 0x1_0000;
    let dest = 0x1_0100;
    memory.map_halfwords(src, 64);
    memory.map_halfwords(dest, 64);
    memory.write_wide_z(src, r"\ResidentFlash\source.txt");
    memory.write_wide_z(dest, r"\ResidentFlash\nested\dest.txt");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_COPY_FILE_W,
            [src, dest, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        fs::read(root.join("nested").join("dest.txt")).unwrap(),
        b"copy payload"
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_COPY_FILE_W,
            [src, dest, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_ce_register_file_system_notification_stores_callback() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    assert_eq!(kernel.file_system_notification_callback(), None);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_REGISTER_FILE_SYSTEM_NOTIFICATION,
            [0x8000_1234],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(
        kernel.file_system_notification_callback(),
        Some(0x8000_1234)
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_REGISTER_FILE_SYSTEM_NOTIFICATION,
            [0x8000_5678],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.file_system_notification_callback(),
        Some(0x8000_5678)
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_REGISTER_FILE_SYSTEM_NOTIFICATION,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.file_system_notification_callback(), None);
    Ok(())
}

#[test]
fn coredll_raw_move_file_w_enforces_ce_volume_boundaries() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("move_file_volume_boundaries");
    let _ = fs::remove_dir_all(&root);
    let resident = root.join("resident");
    let storage = root.join("storage");
    fs::create_dir_all(resident.join("docs")).unwrap();
    fs::create_dir_all(&storage).unwrap();
    fs::write(resident.join("move.txt"), b"cross-volume file").unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &resident);
    kernel.files.mount_guest_root("\\Storage Card", &storage);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let src_file = 0x1_0000;
    let dst_file = 0x1_0100;
    let src_dir = 0x1_0200;
    let dst_dir = 0x1_0300;
    let mount_root = 0x1_0400;
    let mount_rename = 0x1_0500;
    memory.map_halfwords(src_file, 96);
    memory.map_halfwords(dst_file, 96);
    memory.map_halfwords(src_dir, 96);
    memory.map_halfwords(dst_dir, 96);
    memory.map_halfwords(mount_root, 96);
    memory.map_halfwords(mount_rename, 96);
    memory.write_wide_z(src_file, r"\ResidentFlash\move.txt");
    memory.write_wide_z(dst_file, r"\Storage Card\moved.txt");
    memory.write_wide_z(src_dir, r"\ResidentFlash\docs");
    memory.write_wide_z(dst_dir, r"\Storage Card\docs");
    memory.write_wide_z(mount_root, r"\ResidentFlash");
    memory.write_wide_z(mount_rename, r"\Storage Card\ResidentFlash");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [src_file, dst_file],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!resident.join("move.txt").exists());
    assert_eq!(
        fs::read(storage.join("moved.txt")).unwrap(),
        b"cross-volume file"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [src_dir, dst_dir],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(resident.join("docs").is_dir());
    assert!(!storage.join("docs").exists());
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SAME_DEVICE
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [mount_root, mount_rename],
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
            ORD_MOVE_FILE_W,
            [src_dir, mount_root],
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

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_delete_and_rename_file_replaces_destination_atomically() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_REMOVED: u32 = 2;
    const FILE_ACTION_RENAMED_OLD_NAME: u32 = 4;
    const FILE_ACTION_RENAMED_NEW_NAME: u32 = 5;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("delete_and_rename_file");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    fs::write(root.join("watch").join("old.bin"), b"old").unwrap();
    fs::write(root.join("watch").join("new.bin"), b"new").unwrap();
    fs::write(root.join("watch").join("keep.bin"), b"keep").unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let old_path = 0x1_0100;
    let new_path = 0x1_0200;
    let keep_path = 0x1_0300;
    let missing_path = 0x1_0400;
    let notification_buffer = 0x3013_1000;
    let returned_ptr = 0x3013_2000;
    let available_ptr = 0x3013_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(old_path, 96);
    memory.map_halfwords(new_path, 96);
    memory.map_halfwords(keep_path, 96);
    memory.map_halfwords(missing_path, 96);
    memory.map_bytes(notification_buffer, 192);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(old_path, r"\ResidentFlash\watch\old.bin");
    memory.write_wide_z(new_path, r"\ResidentFlash\watch\new.bin");
    memory.write_wide_z(keep_path, r"\ResidentFlash\watch\keep.bin");
    memory.write_wide_z(missing_path, r"\ResidentFlash\watch\missing.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_AND_RENAME_FILE,
            [old_path, new_path],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(
        fs::read(root.join("watch").join("old.bin")).unwrap(),
        b"new"
    );
    assert!(!root.join("watch").join("new.bin").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                192,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_REMOVED, "old.bin".to_owned()),
            (FILE_ACTION_RENAMED_OLD_NAME, "new.bin".to_owned()),
            (FILE_ACTION_RENAMED_NEW_NAME, "old.bin".to_owned()),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_AND_RENAME_FILE,
            [keep_path, missing_path],
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
    assert_eq!(
        fs::read(root.join("watch").join("keep.bin")).unwrap(),
        b"keep"
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_cross_volume_move_from_readonly_source_copies_without_delete() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("move_file_readonly_cross_volume");
    let _ = fs::remove_dir_all(&root);
    let resident = root.join("resident");
    let readonly = root.join("readonly");
    fs::create_dir_all(&resident).unwrap();
    fs::create_dir_all(&readonly).unwrap();
    fs::write(readonly.join("source.txt"), b"readonly source").unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &resident);
    kernel.files.mount(MountConfig {
        name: None,
        device_name: None,
        bus_name: None,
        guest_root: "\\ReadOnly".to_owned(),
        host_root: Some(readonly.clone()),
        total_mbytes: 8,
        free_mbytes: 4,
        writable: false,
        removable: true,
        system: false,
        hidden: false,
        interface_classes: Vec::new(),
        registry_roots: Vec::new(),
        registry_subkey: None,
    });

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let source_path = 0x1_0000;
    let dest_path = 0x1_0100;
    let source_watch_path = 0x1_0200;
    let dest_watch_path = 0x1_0300;
    let notification_buffer = 0x3012_1000;
    let returned_ptr = 0x3012_2000;
    let available_ptr = 0x3012_2004;
    memory.map_halfwords(source_path, 96);
    memory.map_halfwords(dest_path, 96);
    memory.map_halfwords(source_watch_path, 32);
    memory.map_halfwords(dest_watch_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(source_path, r"\ReadOnly\source.txt");
    memory.write_wide_z(dest_path, r"\ResidentFlash\moved.txt");
    memory.write_wide_z(source_watch_path, r"\ReadOnly");
    memory.write_wide_z(dest_watch_path, r"\ResidentFlash");

    let source_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            source_watch_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(source_change, u32::MAX);
    let dest_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            dest_watch_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(dest_change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [source_path, dest_path],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(
        fs::read(resident.join("moved.txt")).unwrap(),
        b"readonly source"
    );
    assert_eq!(
        fs::read(readonly.join("source.txt")).unwrap(),
        b"readonly source"
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [dest_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                dest_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "moved.txt".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [source_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                source_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [source_change],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [dest_change],
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
fn coredll_raw_readonly_mount_reports_access_denied_for_mutations() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
    const FILE_NOTIFY_CHANGE_ATTRIBUTES: u32 = 0x0000_0004;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("readonly_mount_mutations");
    let _ = fs::remove_dir_all(&root);
    let writable_root = root.join("resident");
    let readonly_root = root.join("readonly");
    fs::create_dir_all(&writable_root).unwrap();
    fs::create_dir_all(&readonly_root).unwrap();
    fs::write(writable_root.join("source.txt"), b"copy payload").unwrap();
    fs::write(readonly_root.join("existing.txt"), b"readonly payload").unwrap();
    fs::create_dir(readonly_root.join("existing_dir")).unwrap();
    kernel
        .files
        .mount_guest_root("\\ResidentFlash", &writable_root);
    kernel.files.mount(MountConfig {
        name: None,
        device_name: None,
        bus_name: None,
        guest_root: "\\ReadOnly".to_owned(),
        host_root: Some(readonly_root.clone()),
        total_mbytes: 8,
        free_mbytes: 4,
        writable: false,
        removable: true,
        system: false,
        hidden: false,
        interface_classes: Vec::new(),
        registry_roots: Vec::new(),
        registry_subkey: None,
    });

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let create_path = 0x1_0000;
    let source_path = 0x1_0100;
    let copy_dest_path = 0x1_0200;
    let existing_path = 0x1_0300;
    let watch_path = 0x1_0400;
    let create_dir_path = 0x1_0500;
    let existing_dir_path = 0x1_0600;
    let move_dest_path = 0x1_0700;
    let notification_buffer = 0x300a_1000;
    let returned_ptr = 0x300a_2000;
    let available_ptr = 0x300a_2004;
    memory.map_halfwords(create_path, 96);
    memory.map_halfwords(source_path, 96);
    memory.map_halfwords(copy_dest_path, 96);
    memory.map_halfwords(existing_path, 96);
    memory.map_halfwords(watch_path, 32);
    memory.map_halfwords(create_dir_path, 96);
    memory.map_halfwords(existing_dir_path, 96);
    memory.map_halfwords(move_dest_path, 96);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(create_path, r"\ReadOnly\created.txt");
    memory.write_wide_z(source_path, r"\ResidentFlash\source.txt");
    memory.write_wide_z(copy_dest_path, r"\ReadOnly\copied.txt");
    memory.write_wide_z(existing_path, r"\ReadOnly\existing.txt");
    memory.write_wide_z(watch_path, r"\ReadOnly");
    memory.write_wide_z(create_dir_path, r"\ReadOnly\created_dir");
    memory.write_wide_z(existing_dir_path, r"\ReadOnly\existing_dir");
    memory.write_wide_z(move_dest_path, r"\ReadOnly\renamed.txt");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_DIR_NAME
                | FILE_NOTIFY_CHANGE_ATTRIBUTES
                | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_DIRECTORY_W,
            [watch_path, 0],
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
            ORD_REMOVE_DIRECTORY_W,
            [watch_path],
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
    assert!(readonly_root.exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_FILE_W,
            [watch_path],
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
    assert!(readonly_root.exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_ATTRIBUTES_W,
            [watch_path, FILE_ATTRIBUTE_ARCHIVE],
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
            ORD_CREATE_FILE_W,
            [create_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(u32::MAX),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ACCESS_DENIED
    );
    assert!(!readonly_root.join("created.txt").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_COPY_FILE_W,
            [source_path, copy_dest_path, 1],
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
    assert!(!readonly_root.join("copied.txt").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_ATTRIBUTES_W,
            [existing_path, FILE_ATTRIBUTE_ARCHIVE],
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
    assert!(!readonly_root.join("created_dir").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REMOVE_DIRECTORY_W,
            [existing_dir_path],
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
    assert!(readonly_root.join("existing_dir").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [existing_path, move_dest_path],
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
    assert!(readonly_root.join("existing.txt").exists());
    assert!(!readonly_root.join("renamed.txt").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_FILE_W,
            [existing_path],
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
    assert!(readonly_root.join("existing.txt").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_afs_path_ordinals_use_ce_file_namespace() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("afs_path_ordinals");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let dir = 0x1_0000;
    let path = 0x1_0100;
    let moved_path = 0x1_0200;
    let presto_path = 0x1_0280;
    let write_buffer = 0x1_0300;
    let count_ptr = 0x1_0400;
    let find_data_ptr = 0x1_0500;
    memory.map_halfwords(dir, 64);
    memory.map_halfwords(path, 64);
    memory.map_halfwords(moved_path, 64);
    memory.map_halfwords(presto_path, 64);
    memory.map_bytes(write_buffer, 16);
    memory.map_words(count_ptr, 1);
    memory.map_words(find_data_ptr, 11);
    memory.map_halfwords(find_data_ptr + 40, 260);
    memory.write_wide_z(dir, r"\ResidentFlash\afs");
    memory.write_wide_z(path, r"\ResidentFlash\afs\payload.bin");
    memory.write_wide_z(moved_path, r"\ResidentFlash\afs\moved.bin");
    memory.write_wide_z(presto_path, r"\ResidentFlash\afs\presto.bin");
    memory.write_bytes(write_buffer, b"afs-data");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_CREATE_DIRECTORY_W,
            [0, dir, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(root.join("afs").is_dir());

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_AFS_CREATE_FILE_W,
        [
            0,
            0,
            path,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            CREATE_ALWAYS,
            0,
            0,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("AFS_CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_FILE,
            [file, write_buffer, 8, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 8);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        fs::read(root.join("afs").join("payload.bin")).unwrap(),
        b"afs-data"
    );

    let attrs = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_AFS_GET_FILE_ATTRIBUTES_W,
        [0, path],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(attrs),
            ..
        } => attrs,
        other => panic!("AFS_GetFileAttributesW did not return attributes: {other:?}"),
    };
    assert_eq!(attrs, 0x20);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_SET_FILE_ATTRIBUTES_W,
            [0, path, 0x21],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let attrs = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_AFS_GET_FILE_ATTRIBUTES_W,
        [0, path],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(attrs),
            ..
        } => attrs,
        other => panic!("AFS_GetFileAttributesW did not return readonly attributes: {other:?}"),
    };
    assert_eq!(attrs, 0x21);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_SET_FILE_ATTRIBUTES_W,
            [0, path, 0x20],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let find = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_AFS_FIND_FIRST_FILE_W,
        [0, 0, path, find_data_ptr, 560],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("AFS_FindFirstFileW did not return a find handle: {other:?}"),
    };
    assert_ne!(find, u32::MAX);
    assert_eq!(memory.read_u32(find_data_ptr)?, 0x20);
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "payload.bin");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE,
            [find],
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
            ORD_AFS_MOVE_FILE_W,
            [0, path, moved_path],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!root.join("afs").join("payload.bin").exists());
    assert!(root.join("afs").join("moved.bin").exists());
    fs::write(root.join("afs").join("presto.bin"), b"presto-data").unwrap();

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_PRESTO_CHANGO_FILE_NAME,
            [0, moved_path, presto_path],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        fs::read(root.join("afs").join("moved.bin")).unwrap(),
        b"presto-data"
    );
    assert!(!root.join("afs").join("presto.bin").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_DELETE_FILE_W,
            [0, moved_path],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!root.join("afs").join("moved.bin").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_REMOVE_DIRECTORY_W,
            [0, dir],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!root.join("afs").exists());
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_change_notification_handles_signal_and_rearm() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
    const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_REMOVED: u32 = 2;
    const FILE_ACTION_MODIFIED: u32 = 3;
    const FILE_ACTION_CHANGE_COMPLETED: u32 = 0x0001_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    fs::create_dir_all(root.join("watch").join("old_dir")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let child_dir_path = 0x1_0300;
    let write_buffer = 0x1_0400;
    let count_ptr = 0x1_0500;
    let returned_ptr = 0x1_0600;
    let available_ptr = 0x1_0604;
    let renamed_file_path = 0x1_0700;
    let old_dir_path = 0x1_0800;
    let notification_buffer = 0x3000_1000;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_halfwords(child_dir_path, 64);
    memory.map_halfwords(renamed_file_path, 64);
    memory.map_halfwords(old_dir_path, 64);
    memory.map_bytes(write_buffer, 16);
    memory.map_words(count_ptr, 1);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\first.bin");
    memory.write_wide_z(child_dir_path, r"\ResidentFlash\watch\child");
    memory.write_wide_z(renamed_file_path, r"\ResidentFlash\watch\child\renamed.bin");
    memory.write_wide_z(old_dir_path, r"\ResidentFlash\watch\old_dir");
    memory.write_bytes(write_buffer, b"changed!");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_LAST_WRITE
                | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_TIMEOUT),
            ..
        }
    ));

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            file_path,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            CREATE_ALWAYS,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_FILE,
            [file, write_buffer, 8, count_ptr, 0],
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
            ORD_WRITE_FILE,
            [file, write_buffer, 8, count_ptr, 0],
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
            ORD_CLOSE_HANDLE,
            [file],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert!(returned >= 28);
    let notification = memory.read_bytes(notification_buffer, returned as usize);
    assert_eq!(
        parse_file_notification_records(&notification),
        vec![
            (FILE_ACTION_ADDED, "first.bin".to_owned()),
            (FILE_ACTION_MODIFIED, "first.bin".to_owned()),
            (FILE_ACTION_CHANGE_COMPLETED, "first.bin".to_owned()),
        ]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let recursive_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_AFS_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            0,
            0,
            watch_path,
            1,
            FILE_NOTIFY_CHANGE_DIR_NAME
                | FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("AFS_FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(recursive_change, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_DIRECTORY_W,
            [child_dir_path, 0],
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
            [recursive_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                recursive_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "child".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [file_path, renamed_file_path],
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
            [recursive_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                recursive_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_REMOVED, "first.bin".to_owned()),
            (FILE_ACTION_ADDED, r"child\renamed.bin".to_owned()),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REMOVE_DIRECTORY_W,
            [old_dir_path],
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
            [recursive_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                recursive_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "old_dir".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [recursive_change],
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
            ORD_FIND_NEXT_CHANGE_NOTIFICATION,
            [recursive_change],
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
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_duplicate_handle_close_source_preserves_notification_duplicate() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;
    const DUPLICATE_CLOSE_SOURCE: u32 = 0x0000_0001;
    const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("duplicate_change_notification");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let duplicate_ptr = 0x1_0200;
    let notification_buffer = 0x300b_1000;
    let returned_ptr = 0x300b_2000;
    let available_ptr = 0x300b_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_words(duplicate_ptr, 1);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\created.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DUPLICATE_HANDLE,
            [
                u32::MAX,
                change,
                u32::MAX,
                duplicate_ptr,
                0,
                0,
                DUPLICATE_SAME_ACCESS | DUPLICATE_CLOSE_SOURCE,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let duplicate = memory.read_u32(duplicate_ptr)?;
    assert_ne!(duplicate, change);
    assert_ne!(duplicate, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_NEXT_CHANGE_NOTIFICATION,
            [change],
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

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [duplicate, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                duplicate,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "created.bin".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [duplicate],
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
fn coredll_raw_duplicate_handle_retargets_notification_owner() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;
    const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let owner_process = kernel.current_process_id();
    let target_process = owner_process + 1;
    let root = unique_test_root("duplicate_notification_target_owner");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);

    let mut memory = TestGuestMemory::default();
    let owner_thread = 11;
    let target_thread = 22;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let duplicate_ptr = 0x1_0200;
    let notification_buffer = 0x300d_1000;
    let returned_ptr = 0x300d_2000;
    let available_ptr = 0x300d_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_words(duplicate_ptr, 1);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\target.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        owner_thread,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_DUPLICATE_HANDLE,
            [
                u32::MAX,
                change,
                target_process,
                duplicate_ptr,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let duplicate = memory.read_u32(duplicate_ptr)?;
    assert_ne!(duplicate, change);
    assert_ne!(duplicate, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_FIND_NEXT_CHANGE_NOTIFICATION,
            [duplicate],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(owner_thread),
        ERROR_ACCESS_DENIED
    );

    kernel.set_current_process_id(target_process);
    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        target_thread,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            target_thread,
            ORD_CLOSE_HANDLE,
            [file],
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
            target_thread,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [duplicate, 0],
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
            target_thread,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                duplicate,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "target.bin".to_owned())]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            target_thread,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [duplicate],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    kernel.set_current_process_id(owner_process);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_change_notification_handles_are_process_owned() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let owner_process = kernel.current_process_id();
    let root = unique_test_root("notification_owner_process");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let owner_thread = 11;
    let foreign_thread = 22;
    let watch_path = 0x1_0000;
    let duplicate_ptr = 0x1_0100;
    let notification_buffer = 0x300c_1000;
    let returned_ptr = 0x300c_2000;
    let available_ptr = 0x300c_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_words(duplicate_ptr, 1);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        owner_thread,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    kernel.set_current_process_id(owner_process + 1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_FAILED),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_INVALID_HANDLE
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_FIND_NEXT_CHANGE_NOTIFICATION,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_CLOSE_HANDLE,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_DUPLICATE_HANDLE,
            [
                u32::MAX,
                change,
                u32::MAX,
                duplicate_ptr,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );

    kernel.set_current_process_id(owner_process);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(owner_thread), ERROR_SUCCESS);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_afs_change_notification_uses_hproc_owner() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let caller_process = kernel.current_process_id();
    let target_process = caller_process + 1;
    let root = unique_test_root("afs_notification_hproc_owner");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let caller_thread = 11;
    let target_thread = 22;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let notification_buffer = 0x300e_1000;
    let returned_ptr = 0x300e_2000;
    let available_ptr = 0x300e_2004;
    let duplicate_ptr = 0x300e_2008;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 3);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\created.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        caller_thread,
        ORD_AFS_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            0,
            target_process,
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("AFS_FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);
    assert_eq!(kernel.threads.get_last_error(caller_thread), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_FAILED),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(caller_thread),
        ERROR_INVALID_HANDLE
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_CLOSE_HANDLE,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(caller_thread),
        ERROR_ACCESS_DENIED
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_DUPLICATE_HANDLE,
            [u32::MAX, change, u32::MAX, duplicate_ptr, 0, 0, 0x0000_0002],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(caller_thread),
        ERROR_ACCESS_DENIED
    );
    assert_eq!(
        memory.read_u32(duplicate_ptr)?,
        0,
        "foreign duplicate attempts must not publish a target handle"
    );

    kernel.set_current_process_id(target_process);
    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        target_thread,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            target_thread,
            ORD_CLOSE_HANDLE,
            [file],
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
            target_thread,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            target_thread,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "created.bin".to_owned())]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            target_thread,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    kernel.set_current_process_id(caller_process);
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_change_notification_canonicalizes_watch_path() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_canonical");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let notification_buffer = 0x3005_1000;
    let returned_ptr = 0x3005_2000;
    let available_ptr = 0x3005_2004;
    memory.map_halfwords(watch_path, 96);
    memory.map_halfwords(file_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"ResidentFlash\watch\.\nested\..");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\canonical.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "canonical.bin".to_owned())]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_root_change_notification_honors_subtree_flag() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("root_change_notification_subtree");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("ResidentFlash").join("watch")).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let root_path = 0x1_0000;
    let file_path = 0x1_0100;
    let notification_buffer = 0x3007_1000;
    let returned_ptr = 0x3007_2000;
    let available_ptr = 0x3007_2004;
    memory.map_halfwords(root_path, 4);
    memory.map_halfwords(file_path, 96);
    memory.map_bytes(notification_buffer, 160);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(root_path, r"\");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\nested.bin");

    let nonrecursive_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(nonrecursive_change, u32::MAX);
    let recursive_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(recursive_change, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [nonrecursive_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                nonrecursive_change,
                0,
                notification_buffer,
                160,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [recursive_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                recursive_change,
                0,
                notification_buffer,
                160,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(
            FILE_ACTION_ADDED,
            r"ResidentFlash\watch\nested.bin".to_owned()
        )]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [nonrecursive_change],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [recursive_change],
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
fn coredll_raw_nonroot_change_notification_honors_subtree_and_move_boundaries() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_nonroot_subtree");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch").join("child")).unwrap();
    fs::create_dir_all(root.join("watch").join("other")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let old_path = 0x1_0100;
    let new_path = 0x1_0200;
    let notification_buffer = 0x3008_1000;
    let returned_ptr = 0x3008_2000;
    let available_ptr = 0x3008_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(old_path, 96);
    memory.map_halfwords(new_path, 96);
    memory.map_bytes(notification_buffer, 192);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(old_path, r"\ResidentFlash\watch\child\old.bin");
    memory.write_wide_z(new_path, r"\ResidentFlash\watch\other\new.bin");

    let nonrecursive_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(nonrecursive_change, u32::MAX);
    let recursive_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(recursive_change, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [old_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [nonrecursive_change, 0],
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
            [recursive_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                recursive_change,
                0,
                notification_buffer,
                192,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, r"child\old.bin".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [old_path, new_path],
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
            [nonrecursive_change, 0],
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
            [recursive_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                recursive_change,
                0,
                notification_buffer,
                192,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_REMOVED, r"child\old.bin".to_owned()),
            (FILE_ACTION_ADDED, r"other\new.bin".to_owned()),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [nonrecursive_change],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [recursive_change],
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
fn coredll_raw_directory_watch_reports_self_rename_and_remove_as_current_removed() -> Result<()> {
    const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_current_dir_removed");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch").join("victim_rename")).unwrap();
    fs::create_dir_all(root.join("watch").join("victim_remove")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let rename_watch_path = 0x1_0000;
    let renamed_path = 0x1_0100;
    let remove_watch_path = 0x1_0200;
    let notification_buffer = 0x3009_1000;
    let returned_ptr = 0x3009_2000;
    let available_ptr = 0x3009_2004;
    memory.map_halfwords(rename_watch_path, 96);
    memory.map_halfwords(renamed_path, 96);
    memory.map_halfwords(remove_watch_path, 96);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(rename_watch_path, r"\ResidentFlash\watch\victim_rename");
    memory.write_wide_z(renamed_path, r"\ResidentFlash\watch\renamed");
    memory.write_wide_z(remove_watch_path, r"\ResidentFlash\watch\victim_remove");

    let rename_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            rename_watch_path,
            1,
            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(rename_change, u32::MAX);
    let remove_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            remove_watch_path,
            1,
            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(remove_change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [rename_watch_path, renamed_path],
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
            [rename_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                rename_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "\\".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REMOVE_DIRECTORY_W,
            [remove_watch_path],
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
            [remove_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                remove_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "\\".to_owned())]
    );

    for change in [rename_change, remove_change] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
                [change],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_find_close_change_notification_consumes_wrong_handle_type() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_wrong_close");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("ResidentFlash")).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let file_path = 0x1_0000;
    memory.map_halfwords(file_path, 64);
    memory.write_wide_z(file_path, r"\ResidentFlash\not_notify.bin");

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [file],
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
            ORD_CLOSE_HANDLE,
            [file],
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
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_find_next_change_notification_consumes_one_pending_signal() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_find_next");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_a_path = 0x1_0100;
    let file_b_path = 0x1_0200;
    let notification_buffer = 0x3006_1000;
    let returned_ptr = 0x3006_2000;
    let available_ptr = 0x3006_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_a_path, 64);
    memory.map_halfwords(file_b_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_a_path, r"\ResidentFlash\watch\a.bin");
    memory.write_wide_z(file_b_path, r"\ResidentFlash\watch\b.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    for path in [file_a_path, file_b_path] {
        let file = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_FILE_W,
            [path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(handle),
                ..
            } => handle,
            other => panic!("CreateFileW did not return a file handle: {other:?}"),
        };
        assert_ne!(file, u32::MAX);
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CLOSE_HANDLE,
                [file],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_FIND_NEXT_CHANGE_NOTIFICATION,
            [change],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_ADDED, "a.bin".to_owned()),
            (FILE_ACTION_ADDED, "b.bin".to_owned()),
        ]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_file_notification_info_all_zero_args_resets_one_pending_signal() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_info_reset");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_a_path = 0x1_0100;
    let file_b_path = 0x1_0200;
    let notification_buffer = 0x3007_1000;
    let returned_ptr = 0x3007_2000;
    let available_ptr = 0x3007_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_a_path, 64);
    memory.map_halfwords(file_b_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_a_path, r"\ResidentFlash\watch\a.bin");
    memory.write_wide_z(file_b_path, r"\ResidentFlash\watch\b.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    for path in [file_a_path, file_b_path] {
        let file = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_FILE_W,
            [path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(handle),
                ..
            } => handle,
            other => panic!("CreateFileW did not return a file handle: {other:?}"),
        };
        assert_ne!(file, u32::MAX);
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CLOSE_HANDLE,
                [file],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [change, 0, 0, 0, 0, 0],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_ADDED, "a.bin".to_owned()),
            (FILE_ACTION_ADDED, "b.bin".to_owned()),
        ]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_file_notification_info_reset_preserves_hidden_details() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_reset_hidden_details");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_a_path = 0x1_0100;
    let file_b_path = 0x1_0200;
    let notification_buffer = 0x3014_1000;
    let returned_ptr = 0x3014_2000;
    let available_ptr = 0x3014_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_a_path, 64);
    memory.map_halfwords(file_b_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_a_path, r"\ResidentFlash\watch\a.bin");
    memory.write_wide_z(file_b_path, r"\ResidentFlash\watch\b.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    let file_a = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_a_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file_a, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file_a],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [change, 0, 0, 0, 0, 0],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );

    let file_b = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_b_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file_b, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file_b],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_ADDED, "a.bin".to_owned()),
            (FILE_ACTION_ADDED, "b.bin".to_owned()),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_change_notification_without_cegetinfo_signals_without_details() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_no_cegetinfo");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let notification_buffer = 0x3004_1000;
    let returned_ptr = 0x3004_2000;
    let available_ptr = 0x3004_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_bytes(notification_buffer, 64);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\signal-only.bin");
    memory.write_u32(returned_ptr, 0xffff_ffff)?;
    memory.write_u32(available_ptr, 0xffff_ffff)?;

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [watch_path, 0, FILE_NOTIFY_CHANGE_FILE_NAME],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                64,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_change_notification_preserves_unknown_filter_bits() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_NOTIFY_CHANGE_UNKNOWN: u32 = 0x4000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_unknown_filter");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let notification_buffer = 0x3004_3000;
    let returned_ptr = 0x3004_4000;
    let available_ptr = 0x3004_4004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_bytes(notification_buffer, 96);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\unknown-filter.bin");

    let unknown_only = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_UNKNOWN | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(unknown_only, u32::MAX);
    let known_plus_unknown = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_UNKNOWN
                | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(known_plus_unknown, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [unknown_only, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                unknown_only,
                0,
                notification_buffer,
                96,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [known_plus_unknown, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                known_plus_unknown,
                0,
                notification_buffer,
                96,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "unknown-filter.bin".to_owned())]
    );

    for change in [unknown_only, known_plus_unknown] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
                [change],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_change_notification_coalesces_transient_name_churn() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_coalesce");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    fs::write(root.join("watch").join("stable.bin"), b"old").unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let stable_path = 0x1_0100;
    let transient_path = 0x1_0200;
    let write_buffer = 0x1_0300;
    let count_ptr = 0x1_0400;
    let notification_buffer = 0x3003_1000;
    let returned_ptr = 0x3003_2000;
    let available_ptr = 0x3003_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(stable_path, 64);
    memory.map_halfwords(transient_path, 64);
    memory.map_bytes(write_buffer, 16);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(count_ptr, 1);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(stable_path, r"\ResidentFlash\watch\stable.bin");
    memory.write_wide_z(transient_path, r"\ResidentFlash\watch\transient.bin");
    memory.write_bytes(write_buffer, b"new");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME
                | FILE_NOTIFY_CHANGE_LAST_WRITE
                | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    let transient = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [transient_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a transient handle: {other:?}"),
    };
    assert_ne!(transient, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [transient],
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
            ORD_DELETE_FILE_W,
            [transient_path],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );

    let stable = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [stable_path, GENERIC_WRITE, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a stable handle: {other:?}"),
    };
    assert_ne!(stable, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_FILE,
            [stable, write_buffer, 3, count_ptr, 0],
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
            ORD_CLOSE_HANDLE,
            [stable],
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
            ORD_DELETE_FILE_W,
            [stable_path],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "stable.bin".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_changed_file_close_reports_change_completed() -> Result<()> {
    const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_MODIFIED: u32 = 3;
    const FILE_ACTION_CHANGE_COMPLETED: u32 = 0x0001_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_close_completed");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    fs::write(root.join("watch").join("changed.bin"), b"old").unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let write_buffer = 0x1_0200;
    let count_ptr = 0x1_0300;
    let notification_buffer = 0x3007_1000;
    let returned_ptr = 0x3007_2000;
    let available_ptr = 0x3007_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_bytes(write_buffer, 16);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(count_ptr, 1);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\changed.bin");
    memory.write_bytes(write_buffer, b"new");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_FILE,
            [file, write_buffer, 3, count_ptr, 0],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_MODIFIED, "changed.bin".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_CHANGE_COMPLETED, "changed.bin".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_file_notification_info_partially_drains_pending_records() -> Result<()> {
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_partial");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_a_path = 0x1_0100;
    let file_b_path = 0x1_0200;
    let notification_buffer = 0x3002_1000;
    let returned_ptr = 0x3002_2000;
    let available_ptr = 0x3002_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_a_path, 64);
    memory.map_halfwords(file_b_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_a_path, r"\ResidentFlash\watch\a");
    memory.write_wide_z(file_b_path, r"\ResidentFlash\watch\b");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);
    for path in [file_a_path, file_b_path] {
        let file = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_FILE_W,
            [path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(handle),
                ..
            } => handle,
            other => panic!("CreateFileW did not return a file handle: {other:?}"),
        };
        assert_ne!(file, u32::MAX);
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CLOSE_HANDLE,
                [file],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [change, 0, 0, 0, returned_ptr, available_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 32);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INSUFFICIENT_BUFFER
    );

    memory.write_u32(available_ptr, 0xfeed_cafe)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [change, 0, 0, 0, 0x2000_f008, available_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(available_ptr)?,
        32,
        "CE writes available bytes before faulting on a bad returned pointer"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [change, 0, 0, 16, returned_ptr, available_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 32);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                16,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 16);
    assert_eq!(memory.read_u32(available_ptr)?, 16);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_MORE_DATA);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, 16)),
        vec![(FILE_ACTION_ADDED, "a".to_owned())]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 16);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, 16)),
        vec![(FILE_ACTION_ADDED, "b".to_owned())]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );
    memory.write_u32(available_ptr, 0xfeed_cafe)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [change, 0, notification_buffer, 128, 0, available_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(available_ptr)?,
        0xfeed_cafe,
        "CE faults on null lpBytesReturned before touching lpBytesAvailable"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );
    memory.write_u32(available_ptr, 0xfeed_cafe)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                0x2000_f000,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(available_ptr)?, 0xfeed_cafe);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );
    memory.write_u32(returned_ptr, 0xfeed_cafe)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                0x2000_f004,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_file_notification_info_count_fault_drains_copied_records() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_count_fault_drains");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let notification_buffer = 0x300f_1000;
    let returned_ptr = 0x300f_2000;
    let available_ptr = 0x300f_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\created.bin");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [change, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));

    memory.write_u32(available_ptr, 0xfeed_cafe)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                0x2000_f00c,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, 128)),
        vec![(FILE_ACTION_ADDED, "created.bin".to_owned())]
    );
    assert_eq!(
        memory.read_u32(available_ptr)?,
        0,
        "CE writes available bytes before faulting on the returned-count pointer"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    memory.write_u32(returned_ptr, 0xfeed_cafe)?;
    memory.write_u32(available_ptr, 0xfeed_cafe)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NO_MORE_ITEMS
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_file_notification_info_partial_buffer_fault_drains_copied_prefix() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_partial_buffer_fault");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_a_path = 0x1_0100;
    let file_b_path = 0x1_0200;
    let notification_buffer = 0x2010_1000;
    let returned_ptr = 0x3010_2000;
    let available_ptr = 0x3010_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_a_path, 64);
    memory.map_halfwords(file_b_path, 64);
    memory.map_bytes(notification_buffer, 16);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_a_path, r"\ResidentFlash\watch\a");
    memory.write_wide_z(file_b_path, r"\ResidentFlash\watch\b");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    for path in [file_a_path, file_b_path] {
        let file = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_FILE_W,
            [path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(handle),
                ..
            } => handle,
            other => panic!("CreateFileW did not return a file handle: {other:?}"),
        };
        assert_ne!(file, u32::MAX);
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CLOSE_HANDLE,
                [file],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                32,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, 16)),
        vec![(FILE_ACTION_ADDED, "a".to_owned())]
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_TIMEOUT),
            ..
        }
    ));

    memory.map_bytes(notification_buffer + 16, 112);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 16);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, 16)),
        vec![(FILE_ACTION_ADDED, "b".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_file_notification_info_uses_ce_nul_padded_record_lengths() -> Result<()> {
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("change_notification_nul_padded");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("watch")).unwrap();
    kernel.files.mount_guest_root("\\ResidentFlash", &root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let watch_path = 0x1_0000;
    let file_path = 0x1_0100;
    let notification_buffer = 0x300d_1000;
    let returned_ptr = 0x300d_2000;
    let available_ptr = 0x300d_2004;
    memory.map_halfwords(watch_path, 64);
    memory.map_halfwords(file_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(file_path, r"\ResidentFlash\watch\ab");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [file],
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
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                16,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 0);
    assert_eq!(memory.read_u32(available_ptr)?, 20);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INSUFFICIENT_BUFFER
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                20,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, 20);
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_SUCCESS);
    let encoded = memory.read_bytes(notification_buffer, 20);
    assert_eq!(
        parse_file_notification_records(&encoded),
        vec![(FILE_ACTION_ADDED, "ab".to_owned())]
    );
    assert_eq!(
        u32::from_le_bytes([encoded[8], encoded[9], encoded[10], encoded[11]]),
        4
    );
    assert_eq!(&encoded[12..16], &[b'a', 0, b'b', 0]);
    assert_eq!(&encoded[16..20], &[0, 0, 0, 0]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn parse_file_notification_records(bytes: &[u8]) -> Vec<(u32, String)> {
    let mut records = Vec::new();
    let mut offset = 0usize;
    while offset + 12 <= bytes.len() {
        let read_u32 = |relative: usize| {
            u32::from_le_bytes([
                bytes[offset + relative],
                bytes[offset + relative + 1],
                bytes[offset + relative + 2],
                bytes[offset + relative + 3],
            ])
        };
        let next_offset = read_u32(0) as usize;
        let action = read_u32(4);
        let name_len = read_u32(8) as usize;
        if offset + 12 + name_len > bytes.len() {
            break;
        }
        let name_units = bytes[offset + 12..offset + 12 + name_len]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        records.push((action, String::from_utf16_lossy(&name_units)));
        if next_offset == 0 {
            break;
        }
        offset = offset.saturating_add(next_offset);
    }
    records
}

#[test]
fn coredll_raw_root_change_notification_sees_mount_add_remove() -> Result<()> {
    const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("root_change_notification");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("SDMMC")).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let root_path = 0x1_0000;
    let notification_buffer = 0x3001_1000;
    let returned_ptr = 0x3001_2000;
    let available_ptr = 0x3001_2004;
    memory.map_halfwords(root_path, 4);
    memory.write_wide_z(root_path, r"\");
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            0,
            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(change, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_TIMEOUT),
            ..
        }
    ));

    kernel.mount_guest_root(r"\SDMMC Disk", root.join("SDMMC"));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "SDMMC Disk".to_owned())]
    );

    assert!(kernel.unmount_guest_root(r"\SDMMC Disk"));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "SDMMC Disk".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_afs_register_file_system_function_tracks_volume_callback() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let owner_process = kernel.current_process_id();
    let root = unique_test_root("afs_register_file_system_function");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root(r"\SDMMC Disk", &root);
    let volume = kernel.create_volume_handle_for_guest_root(r"\SDMMC Disk")?;
    let mut memory = TestGuestMemory::default();
    let owner_thread = 11;
    let foreign_thread = 22;

    assert_eq!(kernel.afs_file_system_function(volume), None);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION,
            [volume, 0x1234_5678],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(owner_thread), ERROR_SUCCESS);
    assert_eq!(kernel.afs_file_system_function(volume), Some(0x1234_5678));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION,
            [volume, 0x8765_4321],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.afs_file_system_function(volume),
        Some(0x8765_4321),
        "CE forwards replacement registrations to the mounted FSD"
    );

    kernel.set_current_process_id(owner_process + 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION,
            [volume, 0xfeed_cafe],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );
    assert_eq!(kernel.afs_file_system_function(volume), Some(0x8765_4321));

    kernel.set_current_process_id(owner_process);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION,
            [volume, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.afs_file_system_function(volume), None);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION,
            [0xffff_1234, 0x1234],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(owner_thread),
        ERROR_INVALID_HANDLE
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_REGISTER_FILE_SYSTEM_FUNCTION,
            [volume, 0x1111_2222],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.afs_file_system_function(volume), Some(0x1111_2222));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_CLOSE_HANDLE,
            [volume],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.afs_file_system_function(volume), None);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_afs_notify_mounted_fs_tracks_power_flags() -> Result<()> {
    const FSNOTIFY_POWER_ON: u32 = 0x0000_0001;
    const FSNOTIFY_POWER_OFF: u32 = 0x0000_0002;
    const FSNOTIFY_DEVICES_ON: u32 = 0x0000_0004;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let owner_process = kernel.current_process_id();
    let root = unique_test_root("afs_notify_mounted_fs");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root(r"\SDMMC Disk", &root);
    let volume = kernel.create_volume_handle_for_guest_root(r"\SDMMC Disk")?;
    let mut memory = TestGuestMemory::default();
    let owner_thread = 11;
    let foreign_thread = 22;

    assert!(kernel.afs_mounted_fs_notifications(volume).is_empty());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_NOTIFY_MOUNTED_FS,
            [volume, FSNOTIFY_POWER_ON],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(owner_thread), ERROR_SUCCESS);
    assert_eq!(
        kernel.afs_mounted_fs_notifications(volume),
        &[FSNOTIFY_POWER_ON]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_NOTIFY_MOUNTED_FS,
            [volume, FSNOTIFY_POWER_OFF | FSNOTIFY_DEVICES_ON],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.afs_mounted_fs_notifications(volume),
        &[FSNOTIFY_POWER_ON, FSNOTIFY_POWER_OFF | FSNOTIFY_DEVICES_ON],
        "host-backed volumes should retain each valid mounted-FS notify call"
    );

    kernel.set_current_process_id(owner_process + 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_AFS_NOTIFY_MOUNTED_FS,
            [volume, FSNOTIFY_POWER_OFF],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );
    assert_eq!(
        kernel.afs_mounted_fs_notifications(volume),
        &[FSNOTIFY_POWER_ON, FSNOTIFY_POWER_OFF | FSNOTIFY_DEVICES_ON]
    );

    kernel.set_current_process_id(owner_process);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_NOTIFY_MOUNTED_FS,
            [0xffff_1234, FSNOTIFY_POWER_ON],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(owner_thread),
        ERROR_INVALID_HANDLE
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_CLOSE_HANDLE,
            [volume],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.afs_mounted_fs_notifications(volume).is_empty());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_afs_close_all_file_handles_validates_volume() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let owner_process = kernel.current_process_id();
    let root = unique_test_root("afs_close_all_file_handles");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root(r"\SDMMC Disk", &root);
    let volume = kernel.create_volume_handle_for_guest_root(r"\SDMMC Disk")?;
    let mut memory = TestGuestMemory::default();
    let owner_thread = 11;
    let foreign_thread = 22;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_CLOSE_ALL_FILE_HANDLES,
            [volume, 0xffff_ffff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(owner_thread), ERROR_SUCCESS);

    kernel.set_current_process_id(owner_process + 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_AFS_CLOSE_ALL_FILE_HANDLES,
            [volume, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );

    kernel.set_current_process_id(owner_process);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_CLOSE_ALL_FILE_HANDLES,
            [0xffff_1234, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(owner_thread),
        ERROR_INVALID_HANDLE
    );

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_afs_unmount_volume_handle_signals_and_enforces_owner() -> Result<()> {
    const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let owner_process = kernel.current_process_id();
    let root = unique_test_root("afs_unmount_volume_handle");
    let _ = fs::remove_dir_all(&root);
    let sdmmc_root = root.join("SDMMC");
    fs::create_dir_all(sdmmc_root.join("Data")).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root(r"\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let owner_thread = 11;
    let foreign_thread = 22;
    let root_path = 0x1_0000;
    let data_path = 0x1_0100;
    let notification_buffer = 0x3010_1000;
    let returned_ptr = 0x3010_2000;
    let available_ptr = 0x3010_2004;
    memory.map_halfwords(root_path, 4);
    memory.map_halfwords(data_path, 64);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(root_path, r"\");
    memory.write_wide_z(data_path, r"\SDMMC Disk\Data");

    let root_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        owner_thread,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            0,
            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW root did not return a handle: {other:?}"),
    };
    let data_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        owner_thread,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            data_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW data did not return a handle: {other:?}"),
    };
    assert_ne!(root_change, u32::MAX);
    assert_ne!(data_change, u32::MAX);
    let volume = kernel.create_volume_handle_for_guest_root(r"\SDMMC Disk")?;

    kernel.set_current_process_id(owner_process + 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            foreign_thread,
            ORD_AFS_UNMOUNT,
            [volume],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(foreign_thread),
        ERROR_ACCESS_DENIED
    );

    kernel.set_current_process_id(owner_process);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_UNMOUNT,
            [volume],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(owner_thread), ERROR_SUCCESS);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_AFS_UNMOUNT,
            [volume],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(owner_thread),
        ERROR_INVALID_HANDLE
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [root_change, 0],
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
            owner_thread,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                root_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "SDMMC Disk".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [data_change, 0],
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
            owner_thread,
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                data_change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "Data".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            owner_thread,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [root_change],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [data_change],
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
fn coredll_raw_close_handle_on_volume_handle_unmounts_volume_once() -> Result<()> {
    const FILE_NOTIFY_CHANGE_DIR_NAME: u32 = 0x0000_0002;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("close_volume_handle_unmount");
    let _ = fs::remove_dir_all(&root);
    let sdmmc_root = root.join("SDMMC");
    fs::create_dir_all(&sdmmc_root).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root(r"\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let root_path = 0x1_0000;
    let notification_buffer = 0x3011_1000;
    let returned_ptr = 0x3011_2000;
    let available_ptr = 0x3011_2004;
    memory.map_halfwords(root_path, 4);
    memory.map_bytes(notification_buffer, 128);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(root_path, r"\");

    let change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            0,
            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    let volume = kernel.create_volume_handle_for_guest_root(r"\SDMMC Disk")?;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [volume],
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
            ORD_CLOSE_HANDLE,
            [volume],
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
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                change,
                0,
                notification_buffer,
                128,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_REMOVED, "SDMMC Disk".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [change],
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
fn coredll_raw_mounted_volume_change_notifications_are_volume_scoped() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("mounted_volume_change_notification_scope");
    let _ = fs::remove_dir_all(&root);
    let resident_root = root.join("ResidentFlash");
    let sdmmc_root = root.join("SDMMC");
    fs::create_dir_all(resident_root.join("watch")).unwrap();
    fs::create_dir_all(sdmmc_root.join("watch")).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root(r"\ResidentFlash", &resident_root);
    kernel.mount_guest_root(r"\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let root_path = 0x1_0000;
    let resident_watch_path = 0x1_0100;
    let sdmmc_file_path = 0x1_0200;
    let resident_file_path = 0x1_0300;
    let notification_buffer = 0x3011_1000;
    let returned_ptr = 0x3011_2000;
    let available_ptr = 0x3011_2004;
    memory.map_halfwords(root_path, 4);
    memory.map_halfwords(resident_watch_path, 96);
    memory.map_halfwords(sdmmc_file_path, 128);
    memory.map_halfwords(resident_file_path, 128);
    memory.map_bytes(notification_buffer, 256);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(root_path, r"\");
    memory.write_wide_z(resident_watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(sdmmc_file_path, r"\SDMMC Disk\watch\foreign.bin");
    memory.write_wide_z(resident_file_path, r"\ResidentFlash\watch\local.bin");

    let root_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(root_change, u32::MAX);
    let resident_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            resident_watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(resident_change, u32::MAX);

    let sdmmc_file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [sdmmc_file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(sdmmc_file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [sdmmc_file],
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
            [resident_change, 0],
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
            [root_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                root_change,
                0,
                notification_buffer,
                256,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(
            FILE_ACTION_ADDED,
            r"SDMMC Disk\watch\foreign.bin".to_owned()
        )]
    );

    let resident_file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [resident_file_path, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert_ne!(resident_file, u32::MAX);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLOSE_HANDLE,
            [resident_file],
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
            [resident_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                resident_change,
                0,
                notification_buffer,
                256,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(FILE_ACTION_ADDED, "local.bin".to_owned())]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [root_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                root_change,
                0,
                notification_buffer,
                256,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![(
            FILE_ACTION_ADDED,
            r"ResidentFlash\watch\local.bin".to_owned()
        )]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [root_change],
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
            ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            [resident_change],
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
fn coredll_raw_mounted_same_parent_rename_notifications_are_volume_scoped() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_RENAMED_OLD_NAME: u32 = 4;
    const FILE_ACTION_RENAMED_NEW_NAME: u32 = 5;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("mounted_volume_same_parent_rename_scope");
    let _ = fs::remove_dir_all(&root);
    let resident_root = root.join("ResidentFlash");
    let sdmmc_root = root.join("SDMMC");
    fs::create_dir_all(resident_root.join("watch")).unwrap();
    fs::create_dir_all(sdmmc_root.join("watch")).unwrap();
    fs::write(resident_root.join("watch").join("old.bin"), b"resident").unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root(r"\ResidentFlash", &resident_root);
    kernel.mount_guest_root(r"\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let root_path = 0x1_0000;
    let resident_watch_path = 0x1_0100;
    let sdmmc_watch_path = 0x1_0200;
    let old_path = 0x1_0300;
    let new_path = 0x1_0400;
    let notification_buffer = 0x3021_1000;
    let returned_ptr = 0x3021_2000;
    let available_ptr = 0x3021_2004;
    memory.map_halfwords(root_path, 4);
    memory.map_halfwords(resident_watch_path, 96);
    memory.map_halfwords(sdmmc_watch_path, 96);
    memory.map_halfwords(old_path, 128);
    memory.map_halfwords(new_path, 128);
    memory.map_bytes(notification_buffer, 512);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(root_path, r"\");
    memory.write_wide_z(resident_watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(sdmmc_watch_path, r"\SDMMC Disk\watch");
    memory.write_wide_z(old_path, r"\ResidentFlash\watch\old.bin");
    memory.write_wide_z(new_path, r"\ResidentFlash\watch\new.bin");

    let root_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(root_change, u32::MAX);
    let resident_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            resident_watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(resident_change, u32::MAX);
    let sdmmc_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            sdmmc_watch_path,
            0,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(sdmmc_change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [old_path, new_path],
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
            [resident_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                resident_change,
                0,
                notification_buffer,
                512,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_RENAMED_OLD_NAME, "old.bin".to_owned()),
            (FILE_ACTION_RENAMED_NEW_NAME, "new.bin".to_owned()),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [sdmmc_change, 0],
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
            [root_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                root_change,
                0,
                notification_buffer,
                512,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (
                FILE_ACTION_RENAMED_OLD_NAME,
                r"ResidentFlash\watch\old.bin".to_owned()
            ),
            (
                FILE_ACTION_RENAMED_NEW_NAME,
                r"ResidentFlash\watch\new.bin".to_owned()
            ),
        ]
    );

    for change in [root_change, resident_change, sdmmc_change] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
                [change],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_mounted_cross_parent_rename_notifications_are_volume_scoped() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_CEGETINFO: u32 = 0x8000_0000;
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("mounted_volume_cross_parent_rename_scope");
    let _ = fs::remove_dir_all(&root);
    let resident_root = root.join("ResidentFlash");
    let sdmmc_root = root.join("SDMMC");
    fs::create_dir_all(resident_root.join("watch").join("src")).unwrap();
    fs::create_dir_all(resident_root.join("watch").join("dst")).unwrap();
    fs::create_dir_all(sdmmc_root.join("watch").join("src")).unwrap();
    fs::create_dir_all(sdmmc_root.join("watch").join("dst")).unwrap();
    fs::write(
        resident_root.join("watch").join("src").join("move.bin"),
        b"resident",
    )
    .unwrap();
    fs::write(
        sdmmc_root.join("watch").join("src").join("move.bin"),
        b"sdmmc",
    )
    .unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root(r"\ResidentFlash", &resident_root);
    kernel.mount_guest_root(r"\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let root_path = 0x1_0000;
    let resident_watch_path = 0x1_0100;
    let sdmmc_watch_path = 0x1_0200;
    let resident_old_path = 0x1_0300;
    let resident_new_path = 0x1_0400;
    let sdmmc_old_path = 0x1_0500;
    let sdmmc_new_path = 0x1_0600;
    let notification_buffer = 0x3022_1000;
    let returned_ptr = 0x3022_2000;
    let available_ptr = 0x3022_2004;
    memory.map_halfwords(root_path, 4);
    memory.map_halfwords(resident_watch_path, 96);
    memory.map_halfwords(sdmmc_watch_path, 96);
    memory.map_halfwords(resident_old_path, 128);
    memory.map_halfwords(resident_new_path, 128);
    memory.map_halfwords(sdmmc_old_path, 128);
    memory.map_halfwords(sdmmc_new_path, 128);
    memory.map_bytes(notification_buffer, 512);
    memory.map_words(returned_ptr, 2);
    memory.write_wide_z(root_path, r"\");
    memory.write_wide_z(resident_watch_path, r"\ResidentFlash\watch");
    memory.write_wide_z(sdmmc_watch_path, r"\SDMMC Disk\watch");
    memory.write_wide_z(resident_old_path, r"\ResidentFlash\watch\src\move.bin");
    memory.write_wide_z(resident_new_path, r"\ResidentFlash\watch\dst\move.bin");
    memory.write_wide_z(sdmmc_old_path, r"\SDMMC Disk\watch\src\move.bin");
    memory.write_wide_z(sdmmc_new_path, r"\SDMMC Disk\watch\dst\move.bin");

    let root_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            root_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(root_change, u32::MAX);
    let resident_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            resident_watch_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(resident_change, u32::MAX);
    let sdmmc_change = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FIND_FIRST_CHANGE_NOTIFICATION_W,
        [
            sdmmc_watch_path,
            1,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_CEGETINFO,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("FindFirstChangeNotificationW did not return a handle: {other:?}"),
    };
    assert_ne!(sdmmc_change, u32::MAX);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [resident_old_path, resident_new_path],
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
            [resident_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                resident_change,
                0,
                notification_buffer,
                512,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_REMOVED, r"src\move.bin".to_owned()),
            (FILE_ACTION_ADDED, r"dst\move.bin".to_owned()),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [sdmmc_change, 0],
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
            [root_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                root_change,
                0,
                notification_buffer,
                512,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (
                FILE_ACTION_REMOVED,
                r"ResidentFlash\watch\src\move.bin".to_owned()
            ),
            (
                FILE_ACTION_ADDED,
                r"ResidentFlash\watch\dst\move.bin".to_owned()
            ),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_FILE_W,
            [sdmmc_old_path, sdmmc_new_path],
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
            [resident_change, 0],
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
            [sdmmc_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                sdmmc_change,
                0,
                notification_buffer,
                512,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (FILE_ACTION_REMOVED, r"src\move.bin".to_owned()),
            (FILE_ACTION_ADDED, r"dst\move.bin".to_owned()),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            [root_change, 0],
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
            ORD_CE_GET_FILE_NOTIFICATION_INFO,
            [
                root_change,
                0,
                notification_buffer,
                512,
                returned_ptr,
                available_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let returned = memory.read_u32(returned_ptr)?;
    assert_eq!(memory.read_u32(available_ptr)?, 0);
    assert_eq!(
        parse_file_notification_records(&memory.read_bytes(notification_buffer, returned as usize)),
        vec![
            (
                FILE_ACTION_REMOVED,
                r"SDMMC Disk\watch\src\move.bin".to_owned()
            ),
            (
                FILE_ACTION_ADDED,
                r"SDMMC Disk\watch\dst\move.bin".to_owned()
            ),
        ]
    );

    for change in [root_change, resident_change, sdmmc_change] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
                [change],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
    }
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn coredll_raw_read_file_null_buffer_does_not_advance_file_pointer() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("read_file_null_buffer");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    fs::write(sdmmc_root.join("cursor.bin"), b"abcd").unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let read_buffer = 0x1_0200;
    let count_ptr = 0x1_0300;
    memory.write_wide_z(path_ptr, "\\SDMMC Disk\\cursor.bin");
    memory.map_bytes(read_buffer, 4);
    memory.map_words(count_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_FILE,
            [file, read_buffer, 1, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 1);
    assert_eq!(memory.read_bytes(read_buffer, 1), b"a");

    memory.write_word(count_ptr, 0xffff_ffff);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_FILE,
            [file, 0, 2, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_FILE,
            [file, read_buffer, 2, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(count_ptr)?, 2);
    assert_eq!(memory.read_bytes(read_buffer, 2), b"bc");
    Ok(())
}

#[test]
fn coredll_raw_file_mapping_multiple_views_share_flushed_backing() -> Result<()> {
    const INVALID_HANDLE_VALUE: u32 = 0xffff_ffff;
    const PAGE_READWRITE: u32 = 0x04;
    const FILE_MAP_ALL_ACCESS: u32 = 0x000f_001f;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let mapping = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_MAPPING_W,
        [INVALID_HANDLE_VALUE, 0, PAGE_READWRITE, 0, 4096, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileMappingW did not return a handle: {other:?}"),
    };
    assert_ne!(mapping, 0);

    let view_a = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MAP_VIEW_OF_FILE,
        [mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(base),
            ..
        } => base,
        other => panic!("MapViewOfFile A did not return a base: {other:?}"),
    };
    let view_b = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MAP_VIEW_OF_FILE,
        [mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(base),
            ..
        } => base,
        other => panic!("MapViewOfFile B did not return a base: {other:?}"),
    };
    assert_ne!(view_a, 0);
    assert_ne!(view_b, 0);
    assert_ne!(view_a, view_b);
    memory.map_bytes(view_a, 4096);
    memory.map_bytes(view_b, 4096);

    let payload = b"shared-map";
    memory.write_bytes(view_a, payload);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FLUSH_VIEW_OF_FILE,
            [view_a, payload.len() as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(view_b, payload.len()), payload);

    memory.write_bytes(view_b + 16, b"Z");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FLUSH_VIEW_OF_FILE,
            [view_b, 17],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(view_a + 16, 1), b"Z");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_UNMAP_VIEW_OF_FILE,
            [view_b],
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
            ORD_FLUSH_VIEW_OF_FILE,
            [view_b, 0],
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
            ORD_FLUSH_VIEW_OF_FILE,
            [view_a, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    memory.write_bytes(view_a + 32, b"unmap-sync");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_UNMAP_VIEW_OF_FILE,
            [view_a],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let mapping_state = kernel.handles.file_mapping(mapping)?;
    assert_eq!(&mapping_state.data[32..42], b"unmap-sync");
    Ok(())
}

#[test]
fn coredll_raw_read_file_streams_large_host_file_into_guest_memory() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("read_file_large_stream");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    let payload: Vec<u8> = (0..=255).cycle().take(150 * 1024).collect();
    fs::write(sdmmc_root.join("large.bin"), &payload).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let read_buffer = 0x2_0000;
    let count_ptr = 0x5_0000;
    memory.write_wide_z(path_ptr, "\\SDMMC Disk\\large.bin");
    memory.map_bytes(read_buffer, payload.len() as u32);
    memory.map_words(count_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_FILE,
            [file, read_buffer, payload.len() as u32, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(memory.read_u32(count_ptr)?, payload.len() as u32);
    assert_eq!(memory.read_bytes(read_buffer, payload.len()), payload);
    let stats = kernel.file_io_stats();
    assert_eq!(stats.host_file_open_count, 1);
    assert_eq!(stats.host_file_read_count, 1);
    assert_eq!(stats.host_file_read_bytes, payload.len() as u64);
    assert_eq!(stats.max_read_request, payload.len() as u32);
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_write_file_writes_through_host_backing_and_reports_count() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("write_file_host_backing");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    fs::write(sdmmc_root.join("config.bin"), b"0123456789").unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let write_buffer = 0x1_0200;
    let count_ptr = 0x1_0300;
    memory.write_wide_z(path_ptr, "\\SDMMC Disk\\config.bin");
    memory.write_bytes(write_buffer, b"EOF");
    memory.map_words(count_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_EXISTING,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_POINTER,
            [file, 4, 0, 0],
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
            ORD_WRITE_FILE,
            [file, write_buffer, 3, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(memory.read_u32(count_ptr)?, 3);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_HANDLE,
        [file],
    );
    assert_eq!(
        fs::read(sdmmc_root.join("config.bin")).unwrap(),
        b"0123EOF789"
    );
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_device_io_control_file_scatter_gather_transfers_pages() -> Result<()> {
    const IOCTL_FILE_WRITE_GATHER: u32 = 0x0009_0044;
    const IOCTL_FILE_READ_SCATTER: u32 = 0x0009_0048;
    const PAGE_SIZE: u32 = 4096;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("file_scatter_gather");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let write_segments_ptr = 0x1_0200;
    let read_segments_ptr = 0x1_0300;
    let write_a = 0x3000_0000;
    let write_b = 0x3000_2000;
    let read_a = 0x3000_4000;
    let read_b = 0x3000_6000;
    let returned_ptr = 0x1_0400;
    let payload_a = vec![0x31; PAGE_SIZE as usize];
    let payload_b = vec![0x42; PAGE_SIZE as usize];
    memory.write_wide_z(path_ptr, "\\SDMMC Disk\\sg.bin");
    memory.map_words(write_segments_ptr, 4);
    memory.write_word(write_segments_ptr, write_a);
    memory.write_word(write_segments_ptr + 8, write_b);
    memory.map_words(read_segments_ptr, 4);
    memory.write_word(read_segments_ptr, read_a);
    memory.write_word(read_segments_ptr + 8, read_b);
    memory.write_bytes(write_a, &payload_a);
    memory.write_bytes(write_b, &payload_b);
    memory.map_bytes(read_a, PAGE_SIZE);
    memory.map_bytes(read_b, PAGE_SIZE);
    memory.map_words(returned_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            CREATE_ALWAYS,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                IOCTL_FILE_WRITE_GATHER,
                write_segments_ptr,
                PAGE_SIZE * 2,
                0,
                0,
                returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, PAGE_SIZE * 2);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_POINTER,
            [file, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    memory.write_word(returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                IOCTL_FILE_READ_SCATTER,
                read_segments_ptr,
                PAGE_SIZE * 2,
                0,
                0,
                returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, PAGE_SIZE * 2);
    assert_eq!(memory.read_bytes(read_a, PAGE_SIZE as usize), payload_a);
    assert_eq!(memory.read_bytes(read_b, PAGE_SIZE as usize), payload_b);
    assert_eq!(fs::read(sdmmc_root.join("sg.bin")).unwrap().len(), 8192);
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_device_io_control_file_scatter_gather_reserved_offsets() -> Result<()> {
    const IOCTL_FILE_WRITE_GATHER: u32 = 0x0009_0044;
    const IOCTL_FILE_READ_SCATTER: u32 = 0x0009_0048;
    const PAGE_SIZE: u32 = 4096;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("file_scatter_gather_offsets");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let write_segments_ptr = 0x1_0200;
    let read_segments_ptr = 0x1_0300;
    let offset_array_ptr = 0x1_0400;
    let returned_ptr = 0x1_0500;
    let cursor_buffer = 0x1_0600;
    let cursor_count_ptr = 0x1_0700;
    let cursor_read_ptr = 0x1_0800;
    let write_a = 0x3000_0000;
    let write_b = 0x3000_2000;
    let read_a = 0x3000_4000;
    let read_b = 0x3000_6000;
    let payload_a = vec![0x51; PAGE_SIZE as usize];
    let payload_b = vec![0x62; PAGE_SIZE as usize];
    memory.write_wide_z(path_ptr, "\\SDMMC Disk\\sg-offset.bin");
    memory.map_words(write_segments_ptr, 4);
    memory.write_word(write_segments_ptr, write_a);
    memory.write_word(write_segments_ptr + 8, write_b);
    memory.map_words(read_segments_ptr, 4);
    memory.write_word(read_segments_ptr, read_a);
    memory.write_word(read_segments_ptr + 8, read_b);
    memory.map_words(offset_array_ptr, 4);
    memory.write_word(offset_array_ptr, PAGE_SIZE);
    memory.write_word(offset_array_ptr + 4, 0);
    memory.write_word(offset_array_ptr + 8, PAGE_SIZE * 2);
    memory.write_word(offset_array_ptr + 12, 0);
    memory.write_bytes(write_a, &payload_a);
    memory.write_bytes(write_b, &payload_b);
    memory.map_bytes(read_a, PAGE_SIZE);
    memory.map_bytes(read_b, PAGE_SIZE);
    memory.write_bytes(cursor_buffer, b"CUR");
    memory.map_bytes(cursor_read_ptr, 3);
    memory.map_words(returned_ptr, 1);
    memory.map_words(cursor_count_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            CREATE_ALWAYS,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_POINTER,
            [file, 123, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(123),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                IOCTL_FILE_WRITE_GATHER,
                write_segments_ptr,
                PAGE_SIZE * 2,
                offset_array_ptr,
                16,
                returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, PAGE_SIZE * 2);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_FILE,
            [file, cursor_buffer, 3, cursor_count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(cursor_count_ptr)?, 3);

    let file_bytes = fs::read(sdmmc_root.join("sg-offset.bin")).unwrap();
    assert_eq!(&file_bytes[0..3], &[0, 0, 0]);
    assert_eq!(&file_bytes[123..126], b"CUR");
    assert_eq!(
        &file_bytes[PAGE_SIZE as usize..(PAGE_SIZE * 2) as usize],
        &payload_a
    );
    assert_eq!(
        &file_bytes[(PAGE_SIZE * 2) as usize..(PAGE_SIZE * 3) as usize],
        &payload_b
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_POINTER,
            [file, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    memory.write_word(offset_array_ptr, PAGE_SIZE * 2);
    memory.write_word(offset_array_ptr + 8, PAGE_SIZE);
    memory.write_word(returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                IOCTL_FILE_READ_SCATTER,
                read_segments_ptr,
                PAGE_SIZE * 2,
                offset_array_ptr,
                16,
                returned_ptr,
                0,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, PAGE_SIZE * 2);
    assert_eq!(memory.read_bytes(read_a, PAGE_SIZE as usize), payload_b);
    assert_eq!(memory.read_bytes(read_b, PAGE_SIZE as usize), payload_a);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_READ_FILE,
            [file, cursor_read_ptr, 3, cursor_count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(cursor_count_ptr)?, 3);
    assert_eq!(memory.read_bytes(cursor_read_ptr, 3), vec![0, 0, 0]);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_HANDLE,
        [file],
    );
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_device_io_control_file_scatter_gather_ignores_overlapped() -> Result<()> {
    const IOCTL_FILE_WRITE_GATHER: u32 = 0x0009_0044;
    const IOCTL_FILE_READ_SCATTER: u32 = 0x0009_0048;
    const PAGE_SIZE: u32 = 4096;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("file_scatter_gather_overlapped");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let write_segments_ptr = 0x1_0200;
    let read_segments_ptr = 0x1_0300;
    let returned_ptr = 0x1_0400;
    let write_page = 0x3000_0000;
    let read_page = 0x3000_2000;
    let bogus_overlapped_ptr = 0x7fff_0000;
    let payload = vec![0x73; PAGE_SIZE as usize];
    memory.write_wide_z(path_ptr, "\\SDMMC Disk\\sg-overlapped.bin");
    memory.map_words(write_segments_ptr, 2);
    memory.write_word(write_segments_ptr, write_page);
    memory.map_words(read_segments_ptr, 2);
    memory.write_word(read_segments_ptr, read_page);
    memory.write_bytes(write_page, &payload);
    memory.map_bytes(read_page, PAGE_SIZE);
    memory.map_words(returned_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            CREATE_ALWAYS,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                IOCTL_FILE_WRITE_GATHER,
                write_segments_ptr,
                PAGE_SIZE,
                0,
                0,
                returned_ptr,
                bogus_overlapped_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, PAGE_SIZE);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_POINTER,
            [file, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    memory.write_word(returned_ptr, 0xfeed_cafe);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEVICE_IO_CONTROL,
            [
                file,
                IOCTL_FILE_READ_SCATTER,
                read_segments_ptr,
                PAGE_SIZE,
                0,
                0,
                returned_ptr,
                bogus_overlapped_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(returned_ptr)?, PAGE_SIZE);
    assert_eq!(memory.read_bytes(read_page, PAGE_SIZE as usize), payload);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_HANDLE,
        [file],
    );
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_write_file_on_readonly_handle_reports_access_denied() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("write_file_readonly_handle");
    fs::create_dir_all(&root).unwrap();
    let sdmmc_root = root.join("sdmmc");
    fs::create_dir_all(&sdmmc_root).unwrap();
    fs::write(sdmmc_root.join("config.bin"), b"unchanged").unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\SDMMC Disk", &sdmmc_root);

    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_ptr = 0x1_0000;
    let write_buffer = 0x1_0200;
    let count_ptr = 0x1_0300;
    memory.write_wide_z(path_ptr, "\\SDMMC Disk\\config.bin");
    memory.write_bytes(write_buffer, b"nope");
    memory.map_words(count_ptr, 1);

    let file = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [path_ptr, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFileW did not return a file handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WRITE_FILE,
            [file, write_buffer, 4, count_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    assert_eq!(memory.read_u32(count_ptr)?, 0);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ACCESS_DENIED
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_HANDLE,
        [file],
    );
    assert_eq!(
        fs::read(sdmmc_root.join("config.bin")).unwrap(),
        b"unchanged"
    );
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_registry_subkey_create_enum_query_and_delete() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let parent_ptr = 0x2_0000_u32;
    let sub_alpha_ptr = 0x2_0100_u32;
    let sub_beta_ptr = 0x2_0200_u32;
    let hkey_ptr = 0x2_0300_u32;
    let name_buf = 0x2_0400_u32;
    let name_len_ptr = 0x2_0500_u32;
    let info_buf = 0x2_0600_u32;
    memory.write_wide_z(parent_ptr, "Software\\RegSubkeyTest");
    memory.write_wide_z(sub_alpha_ptr, "Alpha");
    memory.write_wide_z(sub_beta_ptr, "Beta");
    memory.map_words(hkey_ptr, 1);
    memory.map_halfwords(name_buf, 32);
    memory.map_words(name_len_ptr, 1);
    memory.map_words(info_buf, 6);

    let open_key = |memory: &mut TestGuestMemory,
                    kernel: &mut CeKernel,
                    root: u32,
                    subkey_ptr: u32,
                    hkey_out: u32| {
        table.dispatch_raw_ordinal_with_memory(
            kernel,
            memory,
            thread_id,
            ORD_REG_CREATE_KEY_EX_W,
            [root, subkey_ptr, 0, 0, 0, 0, 0, hkey_out, 0],
        )
    };

    let parent_ret = open_key(
        &mut memory,
        &mut kernel,
        HKEY_LOCAL_MACHINE,
        parent_ptr,
        hkey_ptr,
    );
    assert!(
        matches!(parent_ret,
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_SUCCESS
        ),
        "opening parent key must succeed"
    );
    let parent_key = memory.read_u32(hkey_ptr)?;

    let _ = open_key(
        &mut memory,
        &mut kernel,
        parent_key,
        sub_alpha_ptr,
        hkey_ptr,
    );
    let alpha_key = memory.read_u32(hkey_ptr)?;
    let _ = open_key(&mut memory, &mut kernel, parent_key, sub_beta_ptr, hkey_ptr);
    let beta_key = memory.read_u32(hkey_ptr)?;

    let subkeys_ptr = info_buf;
    let max_sub_ptr = info_buf + 4;
    let values_ptr = info_buf + 8;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_REG_QUERY_INFO_KEY_W,
                [parent_key, 0, 0, 0, subkeys_ptr, max_sub_ptr, 0, values_ptr, 0, 0, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_SUCCESS
        ),
        "RegQueryInfoKeyW must succeed"
    );
    assert_eq!(
        memory.read_u32(subkeys_ptr)?,
        2,
        "parent must have 2 subkeys"
    );
    assert_eq!(memory.read_u32(values_ptr)?, 0, "parent must have 0 values");
    assert_eq!(
        memory.read_u32(max_sub_ptr)?,
        5,
        "max subkey chars must be 5 (len of \"Alpha\")"
    );

    memory.write_word(name_len_ptr, 32);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_REG_ENUM_KEY_EX_W,
                [parent_key, 0, name_buf, name_len_ptr, 0, 0, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_SUCCESS
        ),
        "RegEnumKeyExW index 0 must succeed"
    );
    let name0 = memory.read_wide_z(name_buf, 32);
    assert!(
        name0 == "alpha" || name0 == "beta",
        "first subkey must be alpha or beta (lowercase), got {name0:?}"
    );

    memory.write_word(name_len_ptr, 32);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_REG_ENUM_KEY_EX_W,
                [parent_key, 1, name_buf, name_len_ptr, 0, 0, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_SUCCESS
        ),
        "RegEnumKeyExW index 1 must succeed"
    );
    let name1 = memory.read_wide_z(name_buf, 32);
    assert!(name1 == "alpha" || name1 == "beta");
    assert_ne!(name0, name1, "two subkey names must differ");

    memory.write_word(name_len_ptr, 32);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_REG_ENUM_KEY_EX_W,
                [parent_key, 2, name_buf, name_len_ptr, 0, 0, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_NO_MORE_ITEMS
        ),
        "RegEnumKeyExW past the last subkey must return ERROR_NO_MORE_ITEMS"
    );

    memory.write_word(name_len_ptr, 3);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_REG_ENUM_KEY_EX_W,
                [parent_key, 0, name_buf, name_len_ptr, 0, 0, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_MORE_DATA
        ),
        "RegEnumKeyExW with undersized buffer must return ERROR_MORE_DATA"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_REG_DELETE_KEY_W,
                [parent_key, sub_alpha_ptr],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_SUCCESS
        ),
        "RegDeleteKeyW must succeed"
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_REG_QUERY_INFO_KEY_W,
            [parent_key, 0, 0, 0, subkeys_ptr, max_sub_ptr, 0, values_ptr, 0, 0, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v == ERROR_SUCCESS
    ));
    assert_eq!(
        memory.read_u32(subkeys_ptr)?,
        1,
        "parent must have 1 subkey after deletion"
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REG_CLOSE_KEY,
        [alpha_key],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REG_CLOSE_KEY,
        [beta_key],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REG_CLOSE_KEY,
        [parent_key],
    );

    Ok(())
}

#[test]
fn coredll_raw_get_file_attributes_ex_w_reads_attribute_data() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("get_file_attr_ex");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root("\\TestVol", &root);
    fs::write(root.join("sample.dat"), b"hello world").unwrap();
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let path_ptr = 0x2_0000_u32;
    let miss_ptr = 0x2_0100_u32;
    let attr_buf = 0x2_0200_u32;
    memory.write_wide_z(path_ptr, r"\TestVol\sample.dat");
    memory.write_wide_z(miss_ptr, r"\TestVol\nosuchfile.dat");
    memory.map_words(attr_buf, 9);

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_FILE_ATTRIBUTES_EX_W,
                [path_ptr, 0, attr_buf],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "GetFileAttributesExW must succeed for an existing file"
    );
    assert_eq!(
        memory.read_u32(attr_buf)?,
        FILE_ATTRIBUTE_ARCHIVE,
        "newly created file must have FILE_ATTRIBUTE_ARCHIVE"
    );
    assert_eq!(
        memory.read_u32(attr_buf + 32)?,
        11,
        "nFileSizeLow must equal the file content byte count"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_FILE_ATTRIBUTES_EX_W,
                [path_ptr, 1, attr_buf],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "GetFileAttributesExW must reject info_level != 0"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_FILE_ATTRIBUTES_EX_W,
                [miss_ptr, 0, attr_buf],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "GetFileAttributesExW must fail for a missing file"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );

    use wince_emulation_v3::ce::file::FILE_ATTRIBUTE_READONLY;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SET_FILE_ATTRIBUTES_W,
                [path_ptr, FILE_ATTRIBUTE_READONLY],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "SetFileAttributesW must succeed"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_ATTRIBUTES_EX_W,
            [path_ptr, 0, attr_buf],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(attr_buf)? & FILE_ATTRIBUTE_READONLY,
        FILE_ATTRIBUTE_READONLY,
        "FILE_ATTRIBUTE_READONLY must be set after SetFileAttributesW"
    );

    // Restore writable so the temp dir cleanup succeeds
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SET_FILE_ATTRIBUTES_W,
        [path_ptr, FILE_ATTRIBUTE_ARCHIVE],
    );
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_file_security_routes_mounted_paths_to_no_acl_manager() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("file_security_routes");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    kernel.files.mount_guest_root("\\TestVol", &root);
    fs::write(root.join("sample.dat"), b"hello").unwrap();
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let path_ptr = 0x2_0000_u32;
    let missing_path_ptr = 0x2_0100_u32;
    let descriptor_ptr = 0x2_0200_u32;
    let needed_ptr = 0x2_0300_u32;
    memory.write_wide_z(path_ptr, r"\TestVol\sample.dat");
    memory.write_wide_z(missing_path_ptr, r"\TestVol\missing.dat");
    memory.write_bytes(descriptor_ptr, &[0xaa; 16]);
    memory.write_word(needed_ptr, 0xfeed_beef);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_SECURITY_W,
            [path_ptr, 0, descriptor_ptr, 16, needed_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );
    assert_eq!(memory.read_u32(needed_ptr)?, 0);
    assert_eq!(memory.read_bytes(descriptor_ptr, 16), vec![0xaa; 16]);

    memory.write_word(needed_ptr, 0xfeed_beef);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_GET_FILE_SECURITY_W,
            [0, missing_path_ptr, 0, descriptor_ptr, 16, needed_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED,
        "CE returns the no-security-manager result once the mounted volume routes, even for a missing subpath"
    );
    assert_eq!(memory.read_u32(needed_ptr)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FILE_SECURITY_W,
            [path_ptr, 0, descriptor_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_SET_FILE_SECURITY_W,
            [0, missing_path_ptr, 0, descriptor_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_file_security_validates_guest_buffers_before_acl_status() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let needed_ptr = 0x2_0000_u32;
    memory.write_word(needed_ptr, 0xfeed_beef);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_SECURITY_W,
            [0, 0, 0, 0, needed_ptr],
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
    assert_eq!(memory.read_u32(needed_ptr)?, 0);

    let path_ptr = 0x2_0100_u32;
    memory.write_wide_z(path_ptr, r"\TestVol\sample.dat");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FILE_SECURITY_W,
            [path_ptr, 0, 0, 0, 0x2000],
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
            ORD_SET_FILE_SECURITY_W,
            [path_ptr, 0, 0, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        wince_emulation_v3::ce::thread::ERROR_NOT_ENOUGH_MEMORY
    );

    Ok(())
}

#[test]
fn coredll_raw_interlocked_operations_read_modify_write_guest_words() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let ptr_a = 0x3000_0000_u32;
    let ptr_b = 0x3000_0004_u32;
    let ptr_c = 0x3000_0008_u32;
    memory.write_word(ptr_a, 10);
    memory.write_word(ptr_b, 100);
    memory.write_word(ptr_c, 5);

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_INTERLOCKED_DECREMENT,
                [ptr_a]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(9),
                ..
            }
        ),
        "InterlockedDecrement must return the new (decremented) value"
    );
    assert_eq!(
        memory.read_u32(ptr_a)?,
        9,
        "memory must hold the new value after decrement"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_INTERLOCKED_EXCHANGE,
                [ptr_b, 42]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(100),
                ..
            }
        ),
        "InterlockedExchange must return the old value"
    );
    assert_eq!(
        memory.read_u32(ptr_b)?,
        42,
        "memory must hold the exchanged value"
    );

    // CE InterlockedTestExchange(lpTarget, oldValue=args[1], newValue=args[2]):
    // if *target == oldValue → store newValue, return old.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_INTERLOCKED_TEST_EXCHANGE,
                [ptr_c, 5, 99]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(5),
                ..
            }
        ),
        "InterlockedTestExchange must return old value when comparand matches"
    );
    assert_eq!(
        memory.read_u32(ptr_c)?,
        99,
        "memory must hold exchange value after CAS success"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_INTERLOCKED_TEST_EXCHANGE,
                [ptr_c, 5, 200]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(99),
                ..
            }
        ),
        "InterlockedTestExchange must return old value when comparand does not match"
    );
    assert_eq!(
        memory.read_u32(ptr_c)?,
        99,
        "memory must be unchanged when CAS comparand does not match"
    );

    Ok(())
}

#[test]
fn coredll_raw_char_case_buff_w_converts_ascii_wide_chars_in_place() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let buf_ptr = 0x3000_0000_u32;
    memory.write_wide_z(buf_ptr, "Hello World");

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CHAR_LOWER_BUFF_W,
                [buf_ptr, 11]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(11),
                ..
            }
        ),
        "CharLowerBuffW must return the char count"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_wide_z(buf_ptr, 12),
        "hello world",
        "buffer must be fully lowercased"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CHAR_UPPER_BUFF_W,
                [buf_ptr, 5]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(5),
                ..
            }
        ),
        "CharUpperBuffW must return the char count"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_wide_z(buf_ptr, 12),
        "HELLO world",
        "only the first 5 chars must be uppercased"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CHAR_LOWER_BUFF_W,
                [0, 5]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ),
        "CharLowerBuffW must return 0 for a null pointer"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    Ok(())
}

#[test]
fn coredll_raw_heap_re_alloc_shrinks_in_place_and_rejects_bad_params() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let heap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_HEAP_CREATE,
        [HEAP_NO_SERIALIZE, 0x1000, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("HeapCreate failed: {other:?}"),
    };

    let ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_HEAP_ALLOC,
        [heap, 0, 32],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(p),
            ..
        } => p,
        other => panic!("HeapAlloc failed: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_HEAP_SIZE,
            [heap, 0, ptr]
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(32),
            ..
        }
    ));

    let ptr2 = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_HEAP_RE_ALLOC,
        [heap, 0, ptr, 16],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(p),
            ..
        } => p,
        other => panic!("HeapReAlloc shrink failed: {other:?}"),
    };
    assert_eq!(ptr2, ptr, "shrink must keep the same pointer");
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_HEAP_SIZE,
                [heap, 0, ptr2]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(16),
                ..
            }
        ),
        "HeapSize must reflect the shrunken allocation"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_HEAP_RE_ALLOC,
                [heap, HEAP_REALLOC_IN_PLACE_ONLY, ptr2, 64]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(0),
                ..
            }
        ),
        "HEAP_REALLOC_IN_PLACE_ONLY must fail when growth requires a move"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_HEAP_RE_ALLOC,
                [0xDEAD_BEEF, 0, ptr2, 8]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(0),
                ..
            }
        ),
        "HeapReAlloc with an invalid heap must return null"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    Ok(())
}

#[test]
fn coredll_raw_serial_comm_control_ordinals_accept_valid_device_handle() -> Result<()> {
    const MS_CTS_ON: u32 = 0x0010;
    const MS_DSR_ON: u32 = 0x0020;
    const MS_RLSD_ON: u32 = 0x0080;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let com_path = 0x2_0000_u32;
    let out_word = 0x2_0100_u32;
    memory.write_wide_z(com_path, "COM1:");
    memory.write_word(out_word, 0);

    let com = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            com_path,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_EXISTING,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("CreateFileW(COM1:) failed: {other:?}"),
    };

    for (name, ord) in [
        ("SetupComm", ORD_SETUP_COMM),
        ("ClearCommBreak", ORD_CLEAR_COMM_BREAK),
        ("SetCommBreak", ORD_SET_COMM_BREAK),
        ("EscapeCommFunction", ORD_ESCAPE_COMM_FUNCTION),
    ] {
        assert!(
            matches!(
                table.dispatch_raw_ordinal_with_memory(
                    &mut kernel,
                    &mut memory,
                    thread_id,
                    ord,
                    [com]
                ),
                CoredllDispatch::Returned {
                    value: CoredllValue::Bool(true),
                    ..
                }
            ),
            "{name} must return true for a valid comm handle"
        );
        assert_eq!(
            kernel.threads.get_last_error(thread_id),
            0,
            "{name} must clear last error"
        );
    }

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SETUP_COMM,
                [0xDEAD_BEEF]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "SetupComm must return false for an invalid handle"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_HANDLE
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_COMM_MODEM_STATUS,
                [com, out_word]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "GetCommModemStatus must return true for a valid handle"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        memory.read_u32(out_word)?,
        MS_CTS_ON | MS_DSR_ON | MS_RLSD_ON,
        "GetCommModemStatus must write asserted serial line bits to out pointer"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_COMM_MODEM_STATUS,
                [com, 0]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "GetCommModemStatus must return false for a null out pointer"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_PARAMETER
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_HANDLE,
        [com],
    );

    Ok(())
}

#[test]
fn coredll_raw_set_file_time_validates_file_handle() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("set_file_time");
    fs::create_dir_all(&root).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    fs::write(root.join("time_test.dat"), b"x").unwrap();
    let path_ptr = 0x2_0000_u32;
    memory.write_wide_z(path_ptr, "\\time_test.dat");

    let fh = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_W,
        [
            path_ptr,
            GENERIC_READ | GENERIC_WRITE,
            0,
            0,
            OPEN_EXISTING,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("CreateFileW failed: {other:?}"),
    };

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SET_FILE_TIME,
                [fh]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "SetFileTime must return true for a valid file handle"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SET_FILE_TIME,
                [0xDEAD_BEEF]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "SetFileTime must return false for an invalid handle"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_HANDLE
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CLOSE_HANDLE,
        [fh],
    );
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn coredll_raw_local_alloc_in_process_and_remote_variants_delegate_to_same_allocator() -> Result<()>
{
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    // LocalAllocInProcess(flags, size) mirrors LocalAlloc — args[0]=flags, args[1]=size
    let ptr_inproc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOCAL_ALLOC_IN_PROCESS,
        [LMEM_ZEROINIT, 24],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(p),
            ..
        } => p,
        other => panic!("LocalAllocInProcess failed: {other:?}"),
    };
    assert_ne!(ptr_inproc, 0);

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_LOCAL_SIZE_IN_PROCESS,
                [ptr_inproc]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(24),
                ..
            }
        ),
        "LocalSizeInProcess must return the allocated size"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_LOCAL_FREE_IN_PROCESS,
                [ptr_inproc]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(0),
                ..
            }
        ),
        "LocalFreeInProcess must return null on success"
    );

    // RemoteLocalAlloc(process, flags, size) — args[0] is the remote process (ignored), args[1]=flags, args[2]=size
    let ptr_remote = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REMOTE_LOCAL_ALLOC,
        [0xDEAD_0000, LMEM_ZEROINIT, 16],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(p),
            ..
        } => p,
        other => panic!("RemoteLocalAlloc failed: {other:?}"),
    };
    assert_ne!(ptr_remote, 0);

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REMOTE_LOCAL_SIZE,
                [0xDEAD_0000, ptr_remote]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(16),
                ..
            }
        ),
        "RemoteLocalSize must return the allocated size"
    );

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REMOTE_LOCAL_FREE,
                [0xDEAD_0000, ptr_remote]
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(0),
                ..
            }
        ),
        "RemoteLocalFree must return null on success"
    );

    Ok(())
}

#[test]
fn coredll_raw_reg_open_key_ex_w_opens_existing_and_rejects_missing() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11_u32;

    let key_ptr = 0x2_0000_u32;
    let miss_ptr = 0x2_0100_u32;
    let hkey_ptr = 0x2_0200_u32;
    memory.write_wide_z(key_ptr, "Software\\OpenTest");
    memory.write_wide_z(miss_ptr, "Software\\NoSuchKey");
    memory.write_word(hkey_ptr, 0);

    // Create the key first so that RegOpenKeyExW has something to open
    let create_hkey_ptr = 0x2_0300_u32;
    memory.write_word(create_hkey_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REG_CREATE_KEY_EX_W,
            [
                HKEY_LOCAL_MACHINE,
                key_ptr,
                0,
                0,
                0,
                0,
                0,
                create_hkey_ptr,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    let create_hkey = memory.read_u32(create_hkey_ptr)?;
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REG_CLOSE_KEY,
        [create_hkey],
    );

    // RegOpenKeyExW on an existing key must return ERROR_SUCCESS and write a valid handle
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REG_OPEN_KEY_EX_W,
                [HKEY_LOCAL_MACHINE, key_ptr, 0, 0, hkey_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(ERROR_SUCCESS),
                ..
            }
        ),
        "RegOpenKeyExW must return ERROR_SUCCESS for an existing key"
    );
    let opened_hkey = memory.read_u32(hkey_ptr)?;
    assert_ne!(opened_hkey, 0, "RegOpenKeyExW must write a valid handle");
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REG_CLOSE_KEY,
        [opened_hkey],
    );

    // RegOpenKeyExW on a missing key must not return ERROR_SUCCESS
    assert!(
        !matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REG_OPEN_KEY_EX_W,
                [HKEY_LOCAL_MACHINE, miss_ptr, 0, 0, hkey_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(ERROR_SUCCESS),
                ..
            }
        ),
        "RegOpenKeyExW must not return ERROR_SUCCESS for a missing key"
    );

    // RegOpenKeyExW with null subkey opens the root key itself
    memory.write_word(hkey_ptr, 0);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REG_OPEN_KEY_EX_W,
                [HKEY_LOCAL_MACHINE, 0, 0, 0, hkey_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(ERROR_SUCCESS),
                ..
            }
        ),
        "RegOpenKeyExW with null subkey must open the root key"
    );
    let root_hkey = memory.read_u32(hkey_ptr)?;
    assert_ne!(root_hkey, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REG_CLOSE_KEY,
        [root_hkey],
    );

    Ok(())
}

#[test]
fn coredll_raw_math_crt_ordinals_return_cemath_f64_values() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let f64_args = |v: f64| -> [u32; 2] {
        let b = v.to_bits();
        [b as u32, (b >> 32) as u32]
    };
    let f64_bin_args = |a: f64, b: f64| -> [u32; 4] {
        let ab = a.to_bits();
        let bb = b.to_bits();
        [ab as u32, (ab >> 32) as u32, bb as u32, (bb >> 32) as u32]
    };

    macro_rules! check_f64 {
        ($ord:expr, $args:expr, $expected:expr) => {{
            let expected: f64 = $expected;
            assert!(
                matches!(
                    table.dispatch_raw_ordinal_with_memory(
                        &mut kernel, &mut memory, thread_id, $ord, $args,
                    ),
                    CoredllDispatch::Returned {
                        value: CoredllValue::CeMath(CeMathValue::F64(v)),
                        ..
                    } if v.to_bits() == expected.to_bits()
                ),
                "ordinal {}: expected F64({})", $ord, expected
            );
        }};
    }

    // Unary trig
    check_f64!(ORD_ACOS, f64_args(1.0), 0.0_f64);
    check_f64!(ORD_ASIN, f64_args(0.0), 0.0_f64);
    check_f64!(ORD_ATAN, f64_args(0.0), 0.0_f64);
    check_f64!(ORD_COS, f64_args(0.0), 1.0_f64);
    check_f64!(ORD_SIN, f64_args(0.0), 0.0_f64);
    check_f64!(ORD_TAN, f64_args(0.0), 0.0_f64);

    // Hyperbolic
    check_f64!(ORD_COSH, f64_args(0.0), 1.0_f64);
    check_f64!(ORD_SINH, f64_args(0.0), 0.0_f64);
    check_f64!(ORD_TANH, f64_args(0.0), 0.0_f64);

    // Rounding / magnitude
    check_f64!(ORD_CEIL, f64_args(1.2), 2.0_f64);
    check_f64!(ORD_FLOOR, f64_args(1.7), 1.0_f64);
    check_f64!(ORD_FABS, f64_args(-5.0), 5.0_f64);

    // Exponential / logarithm
    check_f64!(ORD_EXP, f64_args(0.0), 1.0_f64);
    check_f64!(ORD_LOG, f64_args(1.0), 0.0_f64);
    check_f64!(ORD_LOG10, f64_args(10.0), 1.0_f64);

    // Binary
    check_f64!(ORD_FMOD, f64_bin_args(5.0, 3.0), 2.0_f64);
    check_f64!(ORD_ATAN2, f64_bin_args(0.0, 1.0), 0.0_f64);
    check_f64!(ORD_HYPOT, f64_bin_args(3.0, 4.0), 5.0_f64);

    Ok(())
}

#[test]
fn coredll_raw_64bit_int_ordinals_return_cemath_i64_u64_values() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let i64_args = |v: i64| -> [u32; 2] {
        let b = v as u64;
        [b as u32, (b >> 32) as u32]
    };
    let u64_args = |v: u64| -> [u32; 2] { [v as u32, (v >> 32) as u32] };
    let i64_bin_args = |a: i64, b: i64| -> [u32; 4] {
        let ab = a as u64;
        let bb = b as u64;
        [ab as u32, (ab >> 32) as u32, bb as u32, (bb >> 32) as u32]
    };
    let u64_bin_args =
        |a: u64, b: u64| -> [u32; 4] { [a as u32, (a >> 32) as u32, b as u32, (b >> 32) as u32] };

    // LL_MUL: signed 64-bit multiply
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LL_MUL,
            i64_bin_args(7, -6),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(-42)),
            ..
        }
    ));

    // LL_REM: signed 64-bit remainder
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LL_REM,
            i64_bin_args(-21, 8),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(-5)),
            ..
        }
    ));

    // LL_LSHIFT: logical left shift
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LL_LSHIFT,
            {
                let [lo, hi] = i64_args(1);
                [lo, hi, 10]
            },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(1024)),
            ..
        }
    ));

    // LL_RSHIFT: arithmetic right shift
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LL_RSHIFT,
            {
                let [lo, hi] = i64_args(-1024);
                [lo, hi, 3]
            },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(-128)),
            ..
        }
    ));

    // ULL_DIV: unsigned 64-bit division
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ULL_DIV,
            u64_bin_args(100, 7),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::U64(14)),
            ..
        }
    ));

    // ULL_REM: unsigned 64-bit remainder
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ULL_REM,
            u64_bin_args(100, 7),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::U64(2)),
            ..
        }
    ));

    // ULL_RSHIFT: logical right shift (no sign extension)
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ULL_RSHIFT,
            {
                let [lo, hi] = u64_args(0x8000_0000_0000_0000_u64);
                [lo, hi, 1]
            },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::U64(0x4000_0000_0000_0000)),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_mips_soft_float_ordinals_arithmetic_compare_convert() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let f32_a = || 3.0_f32.to_bits();
    let f32_b = || 4.0_f32.to_bits();
    let f64_pair = |v: f64| -> [u32; 2] {
        let b = v.to_bits();
        [b as u32, (b >> 32) as u32]
    };
    let f64_bin = |a: f64, b: f64| -> [u32; 4] {
        let [alo, ahi] = f64_pair(a);
        let [blo, bhi] = f64_pair(b);
        [alo, ahi, blo, bhi]
    };

    // --- Float (f32) arithmetic ---
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_FPADD, [f32_a(), f32_b()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
        } if v.to_bits() == 7.0_f32.to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_FPSUB, [f32_b(), f32_a()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
        } if v.to_bits() == 1.0_f32.to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_FPDIV, [f32_b(), f32_a()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
        } if v.to_bits() == (4.0_f32 / 3.0_f32).to_bits()
    ));

    // --- Double (f64) arithmetic ---
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DPADD, f64_bin(1.5, 2.5),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
        } if v.to_bits() == 4.0_f64.to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DPSUB, f64_bin(5.0, 3.0),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
        } if v.to_bits() == 2.0_f64.to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DPDIV, f64_bin(9.0, 3.0),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
        } if v.to_bits() == 3.0_f64.to_bits()
    ));

    // --- Float comparisons (f32 args: [lhs_bits, rhs_bits]) ---
    // LTS: 3.0 < 4.0 → true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LTS,
            [f32_a(), f32_b()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    // LES: 3.0 <= 3.0 → true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LES,
            [f32_a(), f32_a()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    // EQS: 3.0 == 4.0 → false
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EQS,
            [f32_a(), f32_b()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    // GTS: 4.0 > 3.0 → true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GTS,
            [f32_b(), f32_a()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    // --- Double comparisons (f64 args: [lhs_lo, lhs_hi, rhs_lo, rhs_hi]) ---
    // EQD: 2.0 == 2.0 → true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_EQD,
            f64_bin(2.0, 2.0),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    // LED: 1.5 <= 2.0 → true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LED,
            f64_bin(1.5, 2.0),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    // GTD: 3.0 > 2.5 → true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GTD,
            f64_bin(3.0, 2.5),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    // NED: 1.0 != 2.0 → true
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NED,
            f64_bin(1.0, 2.0),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    // --- Conversions ---
    // FPTOLI: float to signed long (f32 → i32)
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FPTOLI,
            [(-7.9_f32).to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I32(-7)),
            ..
        }
    ));
    // DPTOUL: double to unsigned long (f64 → u32)
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPTOUL,
            f64_pair(42.9),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::U32(42)),
            ..
        }
    ));
    // ULTOFP: unsigned long to float (u32 → f32)
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_ULTOFP, [100_u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
        } if v.to_bits() == 100.0_f32.to_bits()
    ));

    Ok(())
}

#[test]
fn coredll_raw_crt_file_ordinals_printf_fwrite_fflush_ferror() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("raw_crt_file_ops");
    fs::create_dir_all(&root).unwrap();
    let data_dir = root.join("data");
    fs::create_dir_all(&data_dir).unwrap();
    fs::write(data_dir.join("read.bin"), b"hello").unwrap();
    kernel.set_file_root(&root);
    kernel.mount_guest_root("\\Data", &data_dir);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;
    let path_r = 0x1_0000_u32;
    let mode_r = 0x1_0100_u32;
    let path_w = 0x1_0200_u32;
    let mode_w = 0x1_0300_u32;
    let data_ptr = 0x3000_0000_u32;
    memory.map_bytes(path_r, 32);
    memory.map_bytes(mode_r, 8);
    memory.map_bytes(path_w, 32);
    memory.map_bytes(mode_w, 8);
    memory.map_bytes(data_ptr, 8);
    memory.write_bytes(path_r, b"\\Data\\read.bin\0");
    memory.write_bytes(mode_r, b"rb\0");
    memory.write_bytes(path_w, b"\\Data\\write.bin\0");
    memory.write_bytes(mode_w, b"wb\0");
    memory.write_bytes(data_ptr, b"ABCD\0\0\0\0");

    // PRINTF always returns 0 (stub)
    assert!(matches!(
        table
            .dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_PRINTF, [],),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    // Invalid stream → FERROR=1, FFLUSH=u32::MAX
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FERROR,
                [0u32],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(1),
                ..
            }
        ),
        "ferror on invalid handle must return 1"
    );
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FFLUSH,
                [0u32],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(u32::MAX),
                ..
            }
        ),
        "fflush on invalid handle must return EOF"
    );

    // FWRITE null src → 0 (guard)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FWRITE,
                [0u32, 1, 1, 0],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ),
        "fwrite with null src must return 0"
    );

    // Open read stream — valid handle for FERROR/FFLUSH
    let read_stream = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FOPEN,
        [path_r, mode_r],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("fopen(read) failed: {other:?}"),
    };
    assert_ne!(read_stream, 0);

    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FERROR,
                [read_stream],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ),
        "ferror on valid stream must return 0"
    );
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FFLUSH,
                [read_stream],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ),
        "fflush on valid stream must return 0"
    );

    // Open write stream — FWRITE succeeds
    let write_stream = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FOPEN,
        [path_w, mode_w],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("fopen(write) failed: {other:?}"),
    };
    assert_ne!(write_stream, 0);

    // FWRITE(data_ptr, size=1, count=4, stream) → 4 items written
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FWRITE,
                [data_ptr, 1, 4, write_stream],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(4),
                ..
            }
        ),
        "fwrite must return item count written"
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FCLOSE,
        [read_stream],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_FCLOSE,
        [write_stream],
    );

    Ok(())
}

#[test]
fn coredll_raw_operator_new_delete_array_nothrow_delegate_to_allocator() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    // OPERATOR_NEW_ARRAY_NOTHROW delegates to malloc_raw (same as operator new[])
    let ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_OPERATOR_NEW_ARRAY_NOTHROW,
        [32u32],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("operator new[] nothrow failed: {other:?}"),
    };
    assert_ne!(ptr, 0);

    // OPERATOR_DELETE_ARRAY_NOTHROW delegates to free_raw → returns U32(0)
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_OPERATOR_DELETE_ARRAY_NOTHROW,
            [ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_heap_trace_validate_and_remote_heap_ordinals() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let heap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_HEAP_CREATE,
        [HEAP_NO_SERIALIZE, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("HeapCreate failed: {other:?}"),
    };
    assert_ne!(heap, 0);

    let ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_HEAP_ALLOC,
        [heap, HEAP_ZERO_MEMORY, 24],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("HeapAlloc failed: {other:?}"),
    };
    assert_ne!(ptr, 0);

    let local_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOCAL_ALLOC,
        [LMEM_ZEROINIT, 16],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("LocalAlloc failed: {other:?}"),
    };
    assert_ne!(local_ptr, 0);

    // HEAP_VALIDATE: valid heap + valid ptr → true
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_HEAP_VALIDATE,
                [heap, 0, ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "HeapValidate on valid block must return true"
    );

    // HEAP_VALIDATE: bad heap → false
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_HEAP_VALIDATE,
                [0xDEAD_BEEFu32, 0, ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "HeapValidate on invalid heap must return false"
    );

    // REMOTE_HEAP_ALLOC: args[0]=ignored_process, rest same as heap_alloc
    let remote_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REMOTE_HEAP_ALLOC,
        [0xDEAD_0000, heap, HEAP_ZERO_MEMORY, 16],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("RemoteHeapAlloc failed: {other:?}"),
    };
    assert_ne!(remote_ptr, 0);

    // REMOTE_HEAP_SIZE: args[0]=ignored, args[1]=heap, args[2]=flags, args[3]=ptr
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REMOTE_HEAP_SIZE,
                [0xDEAD_0000, heap, 0, remote_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(16),
                ..
            }
        ),
        "RemoteHeapSize must return 16"
    );

    // REMOTE_HEAP_RE_ALLOC: grow the block
    let grown_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REMOTE_HEAP_RE_ALLOC,
        [0xDEAD_0000, heap, 0, remote_ptr, 32],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("RemoteHeapReAlloc failed: {other:?}"),
    };
    assert_ne!(grown_ptr, 0);

    // REMOTE_HEAP_FREE: args[0]=ignored, args[1]=heap, args[2]=flags, args[3]=ptr
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REMOTE_HEAP_FREE,
                [0xDEAD_0000, heap, 0, grown_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "RemoteHeapFree must return true"
    );

    // REMOTE_LOCAL_RE_ALLOC: args[0]=ignored, args[1]=ptr, args[2]=bytes, args[3]=flags
    let re_local_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REMOTE_LOCAL_RE_ALLOC,
        [0xDEAD_0000, local_ptr, 32, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("RemoteLocalReAlloc failed: {other:?}"),
    };
    assert_ne!(re_local_ptr, 0);

    Ok(())
}

#[test]
fn coredll_raw_mips_soft_float_extended_arithmetic_and_conversions() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let f32_bits = |v: f32| v.to_bits();
    let f64_pair = |v: f64| -> [u32; 2] {
        let b = v.to_bits();
        [b as u32, (b >> 32) as u32]
    };
    let f64_bin = |a: f64, b: f64| -> [u32; 4] {
        let [alo, ahi] = f64_pair(a);
        let [blo, bhi] = f64_pair(b);
        [alo, ahi, blo, bhi]
    };

    // FPMUL: 3.0 * 4.0 = 12.0
    let expected_f32 = 12.0_f32;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_FPMUL,
                [f32_bits(3.0), f32_bits(4.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
            } if v.to_bits() == expected_f32.to_bits()
        ),
        "FPMUL 3.0*4.0 must return F32(12.0)"
    );

    // DPMUL: 2.5 * 4.0 = 10.0
    let expected_f64 = 10.0_f64;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_DPMUL,
                f64_bin(2.5, 4.0),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
            } if v.to_bits() == expected_f64.to_bits()
        ),
        "DPMUL 2.5*4.0 must return F64(10.0)"
    );

    // FPCMP: 3.0 vs 4.0 → -1 (less)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FPCMP,
                [f32_bits(3.0), f32_bits(4.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::Cmp(-1)),
                ..
            }
        ),
        "FPCMP 3.0<4.0 must return Cmp(-1)"
    );

    // FPCMP: 4.0 vs 3.0 → 1 (greater)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FPCMP,
                [f32_bits(4.0), f32_bits(3.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::Cmp(1)),
                ..
            }
        ),
        "FPCMP 4.0>3.0 must return Cmp(1)"
    );

    // DPCMP: 5.0 vs 5.0 → 0 (equal)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_DPCMP,
                f64_bin(5.0, 5.0),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::Cmp(0)),
                ..
            }
        ),
        "DPCMP 5.0==5.0 must return Cmp(0)"
    );

    // FPTOUL: 3.7f32 → U32(3)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_FPTOUL,
                [f32_bits(3.7)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::U32(3)),
                ..
            }
        ),
        "FPTOUL 3.7 must return U32(3)"
    );

    // DPTOLI: -5.9 → I32(-5)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_DPTOLI,
                f64_pair(-5.9),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::I32(-5)),
                ..
            }
        ),
        "DPTOLI -5.9 must return I32(-5)"
    );

    // F_TO_LL: 5.0f32 → I64(5)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_F_TO_LL,
                [f32_bits(5.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::I64(5)),
                ..
            }
        ),
        "F_TO_LL 5.0 must return I64(5)"
    );

    // D_TO_ULL: 10.0 → U64(10)
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_D_TO_ULL,
                f64_pair(10.0),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::U64(10)),
                ..
            }
        ),
        "D_TO_ULL 10.0 must return U64(10)"
    );

    // FPTODP: 3.0f32 → F64(3.0)
    let expected_dp = 3.0_f64;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_FPTODP,
                [f32_bits(3.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
            } if v.to_bits() == expected_dp.to_bits()
        ),
        "FPTODP 3.0f32 must return F64(3.0)"
    );

    // DPTOFP: 3.0f64 → F32(3.0)
    let expected_fp = 3.0_f32;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_DPTOFP,
                f64_pair(3.0),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
            } if v.to_bits() == expected_fp.to_bits()
        ),
        "DPTOFP 3.0f64 must return F32(3.0)"
    );

    // LITOFP: -3i32 → F32(-3.0)
    let expected_litofp = -3.0_f32;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_LITOFP,
                [(-3_i32) as u32],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
            } if v.to_bits() == expected_litofp.to_bits()
        ),
        "LITOFP -3 must return F32(-3.0)"
    );

    // LITODP: 7i32 → F64(7.0)
    let expected_litodp = 7.0_f64;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_LITODP,
                [7_u32],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
            } if v.to_bits() == expected_litodp.to_bits()
        ),
        "LITODP 7 must return F64(7.0)"
    );

    // ULTODP: 200u32 → F64(200.0)
    let expected_ultodp = 200.0_f64;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_ULTODP,
                [200_u32],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
            } if v.to_bits() == expected_ultodp.to_bits()
        ),
        "ULTODP 200 must return F64(200.0)"
    );

    // FMODF: 5.0 % 3.0 = 2.0
    let expected_fmodf = 2.0_f32;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_FMODF,
                [f32_bits(5.0), f32_bits(3.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F32(v)), ..
            } if v.to_bits() == expected_fmodf.to_bits()
        ),
        "FMODF 5.0%3.0 must return F32(2.0)"
    );

    // GES: 4.0 >= 3.0 → true
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GES,
                [f32_bits(4.0), f32_bits(3.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "GES 4.0>=3.0 must return true"
    );

    // GES: 3.0 >= 4.0 → false
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GES,
                [f32_bits(3.0), f32_bits(4.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "GES 3.0>=4.0 must return false"
    );

    // NES: 3.0 != 4.0 → true
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_NES,
                [f32_bits(3.0), f32_bits(4.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "NES 3.0!=4.0 must return true"
    );

    // NES: 3.0 != 3.0 → false
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_NES,
                [f32_bits(3.0), f32_bits(3.0)],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "NES 3.0!=3.0 must return false"
    );

    Ok(())
}

#[test]
fn coredll_raw_crt_atof_iswctype_ll_div_sqrt_pow_and_tls_call() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let f64_pair = |v: f64| -> [u32; 2] {
        let b = v.to_bits();
        [b as u32, (b >> 32) as u32]
    };
    let f64_bin = |a: f64, b: f64| -> [u32; 4] {
        let [alo, ahi] = f64_pair(a);
        let [blo, bhi] = f64_pair(b);
        [alo, ahi, blo, bhi]
    };
    let i64_bin = |a: i64, b: i64| -> [u32; 4] {
        let au = a as u64;
        let bu = b as u64;
        [au as u32, (au >> 32) as u32, bu as u32, (bu >> 32) as u32]
    };

    // ATOF: parse "3.14" from narrow bytes.
    let text_ptr = 0x1_8000_u32;
    memory.map_bytes(text_ptr, 6);
    memory.write_bytes(text_ptr, b"3.14\0");
    let expected_atof = 3.14_f64;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_ATOF,
                [text_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
            } if v.to_bits() == expected_atof.to_bits()
        ),
        "ATOF \"3.14\" must return F64(3.14)"
    );

    // ISWCTYPE: 'A'=0x41 with CTYPE_UPPER(0x0001) → non-zero.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_ISWCTYPE,
                [b'A' as u32, 0x0001],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v != 0
        ),
        "ISWCTYPE 'A' with CTYPE_UPPER must return non-zero"
    );

    // ISWCTYPE: 'a'=0x61 with CTYPE_UPPER(0x0001) → 0 (lowercase not uppercase).
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_ISWCTYPE,
                [b'a' as u32, 0x0001],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ),
        "ISWCTYPE 'a' with CTYPE_UPPER must return 0"
    );

    // ISWCTYPE: '5' with CTYPE_DIGIT(0x0004) → non-zero.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_ISWCTYPE,
                [b'5' as u32, 0x0004],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v != 0
        ),
        "ISWCTYPE '5' with CTYPE_DIGIT must return non-zero"
    );

    // LL_DIV: 21 / 8 = 2 (truncated), quotient returned as I64.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_LL_DIV,
                i64_bin(21, 8),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::I64(2)),
                ..
            }
        ),
        "LL_DIV 21/8 must return I64(2)"
    );

    // LL_DIV by zero → DivideByZero.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_LL_DIV,
                i64_bin(21, 0),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::DivideByZero),
                ..
            }
        ),
        "LL_DIV by zero must return DivideByZero"
    );

    // SQRT: sqrt(9.0) = 3.0
    let expected_sqrt = 3.0_f64;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_SQRT,
                f64_pair(9.0),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
            } if v.to_bits() == expected_sqrt.to_bits()
        ),
        "SQRT 9.0 must return F64(3.0)"
    );

    // POW: 2.0^8.0 = 256.0
    let expected_pow = 256.0_f64;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_POW,
                f64_bin(2.0, 8.0),
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::CeMath(CeMathValue::F64(v)), ..
            } if v.to_bits() == expected_pow.to_bits()
        ),
        "POW 2.0^8.0 must return F64(256.0)"
    );

    // TLS_CALL(ALLOC=0, slot=0) → first available slot (non-zero, >= 4).
    let tls_slot = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_TLS_CALL,
        [0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(s),
            ..
        } => s,
        other => panic!("TLS_CALL alloc returned unexpected: {other:?}"),
    };
    assert_ne!(tls_slot, 0, "TLS_CALL alloc must return non-zero slot");

    // TLS_CALL(FREE=1, slot) → non-zero (true) on success.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_TLS_CALL,
                [1, tls_slot],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(1),
                ..
            }
        ),
        "TLS_CALL free of allocated slot must return 1 (true)"
    );

    // GET_VERSION_EX (without W) same behavior as GET_VERSION_EX_W: writes struct, returns true.
    // Use test heap (0x3000_0000+) so both word and halfword writes auto-allocate.
    let ver_ptr = 0x3001_0000_u32;
    memory.write_word(ver_ptr, 276);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_VERSION_EX,
                [ver_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ),
        "GET_VERSION_EX must return true"
    );

    Ok(())
}

#[test]
fn coredll_raw_flush_view_of_file_maybe_aliases_flush_view_of_file() -> Result<()> {
    const INVALID_HANDLE_VALUE: u32 = 0xffff_ffff;
    const PAGE_READWRITE: u32 = 0x04;
    const FILE_MAP_ALL_ACCESS: u32 = 0x000f_001f;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let mapping = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FILE_MAPPING_W,
        [INVALID_HANDLE_VALUE, 0, PAGE_READWRITE, 0, 4096, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("expected handle: {other:?}"),
    };

    let view = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_MAP_VIEW_OF_FILE,
        [mapping, FILE_MAP_ALL_ACCESS, 0, 0, 4096],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(base),
            ..
        } => base,
        other => panic!("expected base: {other:?}"),
    };
    assert_ne!(view, 0);
    memory.map_bytes(view, 4096);
    memory.write_bytes(view, b"maybe-flush");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FLUSH_VIEW_OF_FILE_MAYBE,
            [view, 11],
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
            ORD_FLUSH_VIEW_OF_FILE_MAYBE,
            [0xDEAD_0000, 0],
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
fn coredll_raw_registry_convenience_apis_get_set_delete_and_exchange() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 22_u32;

    let key_path_ptr = 0x3_0000_u32;
    let value_name_ptr = 0x3_0100_u32;
    let value_str_ptr = 0x3_0200_u32;
    let out32_ptr = 0x3_0300_u32;
    let out_str_ptr = 0x3_0400_u32;
    let old_ptr = 0x3_0500_u32;

    memory.write_wide_z(key_path_ptr, "Software\\CEConv");
    memory.write_wide_z(value_name_ptr, "Count");
    memory.write_word(out32_ptr, 0);

    // RegistrySetDword creates the key and sets a DWORD value.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_SET_DWORD,
            [HKEY_LOCAL_MACHINE, key_path_ptr, value_name_ptr, 42],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));

    // RegistryGetDword reads back the value written above.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_GET_DWORD,
            [HKEY_LOCAL_MACHINE, key_path_ptr, value_name_ptr, out32_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(out32_ptr)?,
        42,
        "RegistryGetDword must return the stored value"
    );

    // RegistrySetString writes a string value.
    let str_name_ptr = 0x3_0600_u32;
    memory.write_wide_z(str_name_ptr, "Label");
    memory.write_wide_z(value_str_ptr, "hello");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_SET_STRING,
            [
                HKEY_LOCAL_MACHINE,
                key_path_ptr,
                str_name_ptr,
                value_str_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));

    // RegistryGetString reads back the string value (capacity = 32 chars).
    memory.map_bytes(out_str_ptr, 64);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_GET_STRING,
            [
                HKEY_LOCAL_MACHINE,
                key_path_ptr,
                str_name_ptr,
                out_str_ptr,
                32
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    // Verify the UTF-16 string starts with 'h' (0x68 0x00).
    let first_two = memory.read_bytes(out_str_ptr, 2);
    assert_eq!(
        first_two,
        [b'h', 0],
        "RegistryGetString must return the stored string"
    );

    // RegistryDeleteValue removes the DWORD value; subsequent get must fail.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_DELETE_VALUE,
            [HKEY_LOCAL_MACHINE, key_path_ptr, value_name_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert!(
        !matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_REGISTRY_GET_DWORD,
                [HKEY_LOCAL_MACHINE, key_path_ptr, value_name_ptr, out32_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(ERROR_SUCCESS),
                ..
            }
        ),
        "RegistryGetDword on deleted value must not return ERROR_SUCCESS"
    );

    // RegistryTestExchangeDword: set to 10, then test-and-exchange with matching value.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_SET_DWORD,
            [HKEY_LOCAL_MACHINE, key_path_ptr, value_name_ptr, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    memory.write_word(old_ptr, 0xFFFF_FFFF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_TEST_EXCHANGE_DWORD,
            [
                HKEY_LOCAL_MACHINE,
                key_path_ptr,
                value_name_ptr,
                10,
                99,
                old_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    // Old value written out must be 10.
    assert_eq!(
        memory.read_u32(old_ptr)?,
        10,
        "RegistryTestExchangeDword must write the old value"
    );
    // New value is 99 (exchange happened because old == test).
    memory.write_word(out32_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_GET_DWORD,
            [HKEY_LOCAL_MACHINE, key_path_ptr, value_name_ptr, out32_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(out32_ptr)?,
        99,
        "RegistryTestExchangeDword must write new value when test matches"
    );

    // Test-and-exchange with non-matching test value must leave value unchanged.
    memory.write_word(old_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTRY_TEST_EXCHANGE_DWORD,
            [
                HKEY_LOCAL_MACHINE,
                key_path_ptr,
                value_name_ptr,
                0,
                77,
                old_ptr
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(ERROR_SUCCESS),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(old_ptr)?,
        99,
        "RegistryTestExchangeDword must report current value even when test fails"
    );
    memory.write_word(out32_ptr, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTRY_GET_DWORD,
        [HKEY_LOCAL_MACHINE, key_path_ptr, value_name_ptr, out32_ptr],
    );
    assert_eq!(
        memory.read_u32(out32_ptr)?,
        99,
        "value must remain 99 when test value did not match"
    );

    Ok(())
}

#[test]
fn coredll_raw_dpa_clone_copies_all_ptrs_to_new_dpa() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    // Create source DPA with cp_grow=4.
    let hdpa = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DPA_CREATE,
        [4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DPA_Create failed: {other:?}"),
    };
    assert_ne!(hdpa, 0);

    // Insert three sentinel pointer values.
    for ptr in [0x1000_u32, 0x2000, 0x3000] {
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_INSERT_PTR,
            [hdpa, 0xffff_ffff, ptr],
        );
    }

    // DPA_Clone(hdpa, 0) — create a new DPA cloning all items.
    let hdpa2 = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DPA_CLONE,
        [hdpa, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DPA_Clone returned unexpected: {other:?}"),
    };
    assert_ne!(hdpa2, 0);
    assert_ne!(hdpa2, hdpa, "clone must be a distinct allocation");

    // Verify all three pointers are present in the clone.
    for (i, expected) in [0x1000_u32, 0x2000, 0x3000].iter().enumerate() {
        let got = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_GET_PTR,
            [hdpa2, i as u32],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(h),
                ..
            } => h,
            other => panic!("DPA_GetPtr({i}) unexpected: {other:?}"),
        };
        assert_eq!(got, *expected, "clone item {i} must match source");
    }

    // Clean up.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DPA_DESTROY,
        [hdpa],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DPA_DESTROY,
        [hdpa2],
    );
    Ok(())
}

#[test]
fn coredll_raw_dpa_dsa_grow_preallocates_guest_backing() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    const DPA_PP_OFFSET: u32 = 4;
    const DPA_CAPACITY_OFFSET: u32 = 12;
    const DPA_GROW_OFFSET: u32 = 16;
    const DSA_PDATA_OFFSET: u32 = 4;
    const DSA_CAPACITY_OFFSET: u32 = 12;

    let hdpa = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DPA_CREATE,
        [4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DPA_Create failed: {other:?}"),
    };
    assert_eq!(memory.read_u32(hdpa + DPA_CAPACITY_OFFSET)?, 0);
    assert_eq!(memory.read_u32(hdpa + DPA_PP_OFFSET)?, 0);
    assert_eq!(memory.read_u32(hdpa + DPA_GROW_OFFSET)?, 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_GROW,
            [hdpa, 5],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(hdpa + DPA_CAPACITY_OFFSET)?,
        8,
        "DPA_Grow rounds the requested capacity up by the stored grow increment"
    );
    let first_pp = memory.read_u32(hdpa + DPA_PP_OFFSET)?;
    assert_ne!(first_pp, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_GROW,
            [hdpa, 3],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(hdpa + DPA_CAPACITY_OFFSET)?,
        8,
        "DPA_Grow does not shrink an already sufficient backing array"
    );
    assert_eq!(memory.read_u32(hdpa + DPA_PP_OFFSET)?, first_pp);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_GROW,
            [hdpa, 0xffff_ffff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(hdpa + DPA_CAPACITY_OFFSET)?,
        8,
        "negative DPA_Grow counts fail without mutating capacity"
    );

    let hdsa = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_CREATE,
        [4, 3],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DSA_Create failed: {other:?}"),
    };
    assert_eq!(memory.read_u32(hdsa + DSA_CAPACITY_OFFSET)?, 0);
    assert_eq!(memory.read_u32(hdsa + DSA_PDATA_OFFSET)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_GROW,
            [hdsa, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(hdsa + DSA_CAPACITY_OFFSET)?,
        6,
        "DSA_Grow rounds item capacity by its grow increment"
    );
    let first_pdata = memory.read_u32(hdsa + DSA_PDATA_OFFSET)?;
    assert_ne!(first_pdata, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_GROW,
            [hdsa, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(hdsa + DSA_CAPACITY_OFFSET)?, 6);
    assert_eq!(memory.read_u32(hdsa + DSA_PDATA_OFFSET)?, first_pdata);

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DPA_DESTROY,
        [hdpa],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_DESTROY,
        [hdsa],
    );
    Ok(())
}

#[test]
fn coredll_raw_dpa_dsa_null_callbacks_do_not_need_unicorn_callouts() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    let hdpa = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DPA_CREATE,
        [4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DPA_Create failed: {other:?}"),
    };
    assert_ne!(hdpa, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_INSERT_PTR,
            [hdpa, 0xffff_ffff, 0x1234_5678],
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
            ORD_DPA_ENUM_CALLBACK,
            [hdpa, 0x1000_0000, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_ENUM_CALLBACK,
            [hdpa, 0, 0xfeed_cafe],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(kernel.memory.allocation(hdpa).is_some());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPA_DESTROY_CALLBACK,
            [hdpa, 0, 0xfeed_cafe],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(
        kernel.memory.allocation(hdpa).is_none(),
        "null DPA_DestroyCallback should still destroy the DPA"
    );

    let hdsa = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_CREATE,
        [4, 4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DSA_Create failed: {other:?}"),
    };
    assert_ne!(hdsa, 0);
    let item_ptr = 0x3_0000_u32;
    memory.write_bytes(item_ptr, &0x0bad_f00d_u32.to_le_bytes());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_INSERT_ITEM,
            [hdsa, 0xffff_ffff, item_ptr],
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
            ORD_DSA_ENUM_CALLBACK,
            [hdsa, 0x1000_0000, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_NOT_SUPPORTED
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_ENUM_CALLBACK,
            [hdsa, 0, 0xfeed_cafe],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(kernel.memory.allocation(hdsa).is_some());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_DESTROY_CALLBACK,
            [hdsa, 0, 0xfeed_cafe],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(
        kernel.memory.allocation(hdsa).is_none(),
        "null DSA_DestroyCallback should still destroy the DSA"
    );

    Ok(())
}

#[test]
fn coredll_raw_dsa_clone_copies_all_items_to_new_dsa() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    // Create DSA: cb_item=4 bytes, cp_grow=4.
    let hdsa = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_CREATE,
        [4, 4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DSA_Create failed: {other:?}"),
    };
    assert_ne!(hdsa, 0);

    // Write item source buffers and insert them.
    // Use write_bytes so read_u8 can recover the bytes during DSA_InsertItem.
    let item_buf = 0x3_0000_u32;
    for val in [0xAABB_u32, 0xCCDD, 0xEEFF] {
        memory.write_bytes(item_buf, &val.to_le_bytes());
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_INSERT_ITEM,
            [hdsa, 0xffff_ffff, item_buf],
        );
    }

    // DSA_Clone(hdsa, 0) — new DSA.
    let hdsa2 = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_CLONE,
        [hdsa, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DSA_Clone returned unexpected: {other:?}"),
    };
    assert_ne!(hdsa2, 0);
    assert_ne!(hdsa2, hdsa, "clone must be a distinct allocation");

    // Verify items in the clone via read_bytes (item data lives in bytes map, not words).
    for (i, expected) in [0xAABB_u32, 0xCCDD, 0xEEFF].iter().enumerate() {
        let ptr = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_GET_ITEM_PTR,
            [hdsa2, i as u32],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(h),
                ..
            } => h,
            other => panic!("DSA_GetItemPtr({i}) unexpected: {other:?}"),
        };
        assert_ne!(ptr, 0);
        let raw = memory.read_bytes(ptr, 4);
        let got = u32::from_le_bytes(raw.try_into().unwrap());
        assert_eq!(
            got, *expected,
            "clone item {i} must match source ({expected:#x} != {got:#x})"
        );
    }

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_DESTROY,
        [hdsa],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_DESTROY,
        [hdsa2],
    );
    Ok(())
}

#[test]
fn coredll_raw_dsa_set_range_overwrites_item_range() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 11;

    // Create DSA: 4-byte items.
    let hdsa = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_CREATE,
        [4, 4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("DSA_Create failed: {other:?}"),
    };

    // Insert 4 items using write_bytes so read_u8 recovers them during DSA_InsertItem.
    let item_buf = 0x3_0000_u32;
    for val in [0x11_u32, 0x22, 0x33, 0x44] {
        memory.write_bytes(item_buf, &val.to_le_bytes());
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_INSERT_ITEM,
            [hdsa, 0xffff_ffff, item_buf],
        );
    }

    // DSA_SetRange(hdsa, 1, 2, item_buf) where item_buf now holds 0xFFFF.
    // Overwrites items[1] and items[2] each to 0xFFFF (2 copies of the same 4-byte value).
    memory.write_bytes(item_buf, &0xFFFF_u32.to_le_bytes());
    let ok = matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_SET_RANGE,
            [hdsa, 1, 2, item_buf],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    );
    assert!(ok, "DSA_SetRange must return TRUE");

    // items[0]=0x11, items[1]=0xFFFF, items[2]=0xFFFF, items[3]=0x44.
    for (i, expected) in [0x11_u32, 0xFFFF, 0xFFFF, 0x44].iter().enumerate() {
        let ptr = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DSA_GET_ITEM_PTR,
            [hdsa, i as u32],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(h),
                ..
            } => h,
            other => panic!("DSA_GetItemPtr({i}) unexpected: {other:?}"),
        };
        let raw = memory.read_bytes(ptr, 4);
        let got = u32::from_le_bytes(raw.try_into().unwrap());
        assert_eq!(
            got, *expected,
            "item[{i}] expected {expected:#x} got {got:#x}"
        );
    }

    // Out-of-range call must return FALSE.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_DSA_SET_RANGE,
                [hdsa, 3, 2, item_buf],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ),
        "DSA_SetRange past end must return FALSE"
    );

    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DSA_DESTROY,
        [hdsa],
    );
    Ok(())
}

fn minimal_1x1_24bpp_bmp() -> Vec<u8> {
    let mut bmp = Vec::new();
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&58u32.to_le_bytes()); // file size
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&54u32.to_le_bytes()); // pixel offset
    bmp.extend_from_slice(&40u32.to_le_bytes()); // BITMAPINFOHEADER size
    bmp.extend_from_slice(&1i32.to_le_bytes()); // width
    bmp.extend_from_slice(&1i32.to_le_bytes()); // height
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&24u16.to_le_bytes()); // bpp
    bmp.extend_from_slice(&0u32.to_le_bytes()); // compression
    bmp.extend_from_slice(&4u32.to_le_bytes()); // image size
    bmp.extend_from_slice(&0i32.to_le_bytes()); // x pixels/m
    bmp.extend_from_slice(&0i32.to_le_bytes()); // y pixels/m
    bmp.extend_from_slice(&0u32.to_le_bytes()); // colors used
    bmp.extend_from_slice(&0u32.to_le_bytes()); // colors important
    bmp.extend_from_slice(&[0x33, 0x66, 0x99, 0x00]); // pixel data + padding
    bmp
}

#[test]
fn coredll_raw_shload_dibitmap_loads_file_and_rejects_missing_path() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("shload_dibitmap");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let bmp = minimal_1x1_24bpp_bmp();
    std::fs::write(root.join("bg.bmp"), &bmp).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 200_u32;
    let path_ptr = 0x1_0000_u32;
    memory.map_halfwords(path_ptr, 64);

    // Valid BMP path → non-zero HBITMAP.
    memory.write_wide_z(path_ptr, r"\bg.bmp");
    let hbm = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SHLOAD_DIBITMAP,
        [path_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("SHLoadDIBitmap unexpected: {other:?}"),
    };
    assert_ne!(
        hbm, 0,
        "SHLoadDIBitmap must return a valid HBITMAP for an existing BMP"
    );

    // Missing path → 0.
    memory.write_wide_z(path_ptr, r"\missing.bmp");
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SHLOAD_DIBITMAP,
                [path_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(0),
                ..
            }
        ),
        "SHLoadDIBitmap with missing path must return 0"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    Ok(())
}

#[test]
fn coredll_raw_load_image_w_loads_bitmap_file_like_ce() -> Result<()> {
    const DIBSECTION_SIZE: u32 = 84;
    const IMAGE_BITMAP: u32 = 0;
    const LR_LOADFROMFILE: u32 = 0x10;
    const LR_CREATEDIBSECTION: u32 = 0x2000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("load_image_bitmap_file");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let bmp = minimal_1x1_24bpp_bmp();
    std::fs::write(root.join("frame.bmp"), &bmp).unwrap();
    kernel.set_file_root(&root);
    let mut memory = TestGuestMemory::default();
    let thread_id = 201_u32;
    let path_ptr = 0x1_0000_u32;
    let object_out = 0x1_0200_u32;
    memory.map_halfwords(path_ptr, 64);
    memory.map_bytes(object_out, DIBSECTION_SIZE);

    memory.write_wide_z(path_ptr, r"\frame.bmp");
    let hbm = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [0, path_ptr, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("LoadImageW unexpected: {other:?}"),
    };
    assert_ne!(
        hbm, 0,
        "LoadImageW must return a valid HBITMAP for LR_LOADFROMFILE bitmap paths"
    );
    let bitmap = kernel
        .resources
        .bitmap(hbm)
        .expect("LoadImageW returns a bitmap handle");
    assert_eq!(bitmap.width, 1);
    assert_eq!(bitmap.height, 1);
    assert_eq!(bitmap.planes, 1);
    assert_eq!(bitmap.bits_pixel, 24);
    assert!(bitmap.bits_owned);
    assert!(!bitmap.bits_writable);
    assert!(!bitmap.dib_section);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_OBJECT_W,
                [hbm, DIBSECTION_SIZE, object_out],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(24),
                ..
            }
        ),
        "LoadImageW without LR_CREATEDIBSECTION should expose a BITMAP-sized object"
    );

    memory.write_wide_z(path_ptr, r"\frame.bmp");
    let dib_hbm = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_IMAGE_W,
        [
            0,
            path_ptr,
            IMAGE_BITMAP,
            0,
            0,
            LR_LOADFROMFILE | LR_CREATEDIBSECTION,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("LoadImageW LR_CREATEDIBSECTION unexpected: {other:?}"),
    };
    assert_ne!(
        dib_hbm, 0,
        "LoadImageW must still return a HBITMAP with LR_CREATEDIBSECTION"
    );
    let dib_bitmap = kernel
        .resources
        .bitmap(dib_hbm)
        .expect("LoadImageW LR_CREATEDIBSECTION returns a bitmap handle");
    assert!(dib_bitmap.dib_section);
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_GET_OBJECT_W,
                [dib_hbm, DIBSECTION_SIZE, object_out],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(DIBSECTION_SIZE),
                ..
            }
        ),
        "LR_CREATEDIBSECTION should make GetObjectW return the DIBSECTION size"
    );
    let dib_object = memory.read_bytes(object_out, DIBSECTION_SIZE as usize);
    assert_eq!(
        i32::from_le_bytes(dib_object[28..32].try_into().unwrap()),
        1
    );
    assert_eq!(
        i32::from_le_bytes(dib_object[32..36].try_into().unwrap()),
        1
    );
    assert_eq!(
        u16::from_le_bytes(dib_object[36..38].try_into().unwrap()),
        1
    );
    assert_eq!(
        u16::from_le_bytes(dib_object[38..40].try_into().unwrap()),
        24
    );

    memory.write_wide_z(path_ptr, r"\missing.bmp");
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_LOAD_IMAGE_W,
                [0, path_ptr, IMAGE_BITMAP, 0, 0, LR_LOADFROMFILE],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(0),
                ..
            }
        ),
        "LoadImageW with LR_LOADFROMFILE and a missing bitmap must return 0"
    );
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_FILE_NOT_FOUND
    );
    Ok(())
}
