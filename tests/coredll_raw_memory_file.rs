use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_AFS_CREATE_DIRECTORY_W, ORD_AFS_CREATE_FILE_W, ORD_AFS_DELETE_FILE_W,
            ORD_AFS_FIND_FIRST_CHANGE_NOTIFICATION_W, ORD_AFS_FIND_FIRST_FILE_W,
            ORD_AFS_FS_IO_CONTROL_W, ORD_AFS_GET_DISK_FREE_SPACE, ORD_AFS_GET_FILE_ATTRIBUTES_W,
            ORD_AFS_MOVE_FILE_W, ORD_AFS_PRESTO_CHANGO_FILE_NAME, ORD_AFS_REMOVE_DIRECTORY_W,
            ORD_AFS_SET_FILE_ATTRIBUTES_W, ORD_ATOI, ORD_CE_FS_IO_CONTROL_W,
            ORD_CE_GET_FILE_NOTIFICATION_INFO, ORD_CE_GET_VOLUME_INFO_W, ORD_CHAR_LOWER_W,
            ORD_CHAR_UPPER_W, ORD_CLOSE_HANDLE, ORD_COPY_FILE_W, ORD_CREATE_DIRECTORY_W,
            ORD_CREATE_FILE_MAPPING_W, ORD_CREATE_FILE_W, ORD_DELETE_FILE_W, ORD_DEVICE_IO_CONTROL,
            ORD_FCLOSE, ORD_FEOF, ORD_FGETS, ORD_FIND_CLOSE, ORD_FIND_CLOSE_CHANGE_NOTIFICATION,
            ORD_FIND_FIRST_CHANGE_NOTIFICATION_W, ORD_FIND_FIRST_FILE_W,
            ORD_FIND_NEXT_CHANGE_NOTIFICATION, ORD_FIND_NEXT_FILE_W, ORD_FLUSH_FILE_BUFFERS,
            ORD_FLUSH_INSTRUCTION_CACHE, ORD_FLUSH_VIEW_OF_FILE, ORD_FOPEN, ORD_FREAD, ORD_FREE,
            ORD_FSEEK, ORD_FTELL, ORD_GET_DISK_FREE_SPACE_EX_W, ORD_GET_FILE_ATTRIBUTES_W,
            ORD_GET_FILE_SIZE, ORD_GET_MODULE_FILE_NAME_W, ORD_GET_PROCESS_HEAP, ORD_HEAP_ALLOC,
            ORD_HEAP_CREATE, ORD_HEAP_DESTROY, ORD_HEAP_FREE, ORD_HEAP_SIZE, ORD_IS_BAD_READ_PTR,
            ORD_IS_BAD_WRITE_PTR, ORD_IS_VALID_LOCALE, ORD_LOCAL_ALLOC, ORD_LOCAL_FREE,
            ORD_LOCAL_RE_ALLOC, ORD_LOCAL_SIZE, ORD_MALLOC, ORD_MAP_VIEW_OF_FILE, ORD_MEMCMP,
            ORD_MEMCPY, ORD_MEMMOVE, ORD_MEMSET, ORD_MOVE_FILE_W, ORD_MSIZE,
            ORD_MULTI_BYTE_TO_WIDE_CHAR, ORD_OPERATOR_DELETE, ORD_OPERATOR_DELETE_ARRAY,
            ORD_OPERATOR_NEW, ORD_OPERATOR_NEW_ARRAY, ORD_RAND, ORD_READ_FILE, ORD_REALLOC,
            ORD_REG_CLOSE_KEY, ORD_REG_CREATE_KEY_EX_W, ORD_REG_DELETE_VALUE_W,
            ORD_REG_ENUM_VALUE_W, ORD_REG_QUERY_VALUE_EX_W, ORD_REG_SET_VALUE_EX_W,
            ORD_REMOVE_DIRECTORY_W, ORD_SECURITY_GEN_COOKIE, ORD_SECURITY_GEN_COOKIE2,
            ORD_SET_FILE_POINTER, ORD_SNPRINTF, ORD_SNWPRINTF, ORD_SPRINTF, ORD_SRAND, ORD_STRCAT,
            ORD_STRCPY, ORD_STRING_CB_CAT_W, ORD_STRING_CCH_CAT_W, ORD_STRING_CCH_LENGTH_W,
            ORD_STRTOK, ORD_STRTOUL, ORD_STRUPR, ORD_SWPRINTF, ORD_TOLOWER, ORD_TOUPPER,
            ORD_UNMAP_VIEW_OF_FILE, ORD_VIRTUAL_ALLOC, ORD_VIRTUAL_FREE, ORD_VSNPRINTF,
            ORD_VSNWPRINTF, ORD_VSPRINTF, ORD_VSWPRINTF, ORD_WAIT_FOR_SINGLE_OBJECT, ORD_WCSCHR,
            ORD_WCSCPY, ORD_WCSDUP, ORD_WCSICMP, ORD_WCSLEN, ORD_WCSNCMP, ORD_WCSNCPY,
            ORD_WCSNICMP, ORD_WCSPBRK, ORD_WCSRCHR, ORD_WCSSTR, ORD_WCSTOUL, ORD_WFOPEN,
            ORD_WIDE_CHAR_TO_MULTI_BYTE, ORD_WRITE_FILE, ORD_WSPRINTF_W, ORD_WTOL, ORD_WVSPRINTF_W,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE, OPEN_EXISTING},
        kernel::CeKernel,
        memory::{HEAP_NO_SERIALIZE, HEAP_ZERO_MEMORY, LMEM_ZEROINIT, MEM_COMMIT, MEM_RELEASE},
        registry::{
            ERROR_MORE_DATA, ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, HKEY_CURRENT_USER, REG_DWORD,
        },
        thread::{
            ERROR_ACCESS_DENIED, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER,
            ERROR_NO_MORE_FILES, ERROR_NOT_SUPPORTED,
        },
        timer::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
};

mod support;
use support::{TestGuestMemory, unique_test_root};

#[test]
fn coredll_raw_string_conversion_ordinals_round_trip_ascii() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
fn coredll_raw_snprintf_formats_with_count_limit() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
fn coredll_raw_copy_file_w_copies_between_ce_paths() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("copy_file_w_raw");
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
fn coredll_raw_afs_path_ordinals_use_ce_file_namespace() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    assert!(!root.join("afs").join("moved.bin").exists());
    assert!(root.join("afs").join("presto.bin").exists());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_AFS_DELETE_FILE_W,
            [0, presto_path],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!root.join("afs").join("presto.bin").exists());

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
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_REMOVED: u32 = 2;
    const FILE_ACTION_MODIFIED: u32 = 3;
    const FILE_ACTION_RENAMED_OLD_NAME: u32 = 4;
    const FILE_ACTION_RENAMED_NEW_NAME: u32 = 5;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE,
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
            FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_FILE_NAME,
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
            (FILE_ACTION_RENAMED_OLD_NAME, "first.bin".to_owned()),
            (
                FILE_ACTION_RENAMED_NEW_NAME,
                r"child\renamed.bin".to_owned()
            ),
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
fn coredll_raw_change_notification_coalesces_transient_name_churn() -> Result<()> {
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE,
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
fn coredll_raw_file_notification_info_partially_drains_pending_records() -> Result<()> {
    const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_ACTION_ADDED: u32 = 1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
        [watch_path, 0, FILE_NOTIFY_CHANGE_FILE_NAME],
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
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_REMOVED: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
        [root_path, 0, FILE_NOTIFY_CHANGE_DIR_NAME],
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
fn coredll_raw_read_file_null_buffer_does_not_advance_file_pointer() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
fn coredll_raw_write_file_on_readonly_handle_reports_access_denied() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
