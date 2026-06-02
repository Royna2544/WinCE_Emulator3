use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_CHAR_LOWER_W, ORD_CHAR_UPPER_W, ORD_CLOSE_HANDLE, ORD_CREATE_FILE_W,
            ORD_DEVICE_IO_CONTROL, ORD_FIND_CLOSE, ORD_FIND_FIRST_FILE_W, ORD_FLUSH_FILE_BUFFERS,
            ORD_FLUSH_INSTRUCTION_CACHE, ORD_FREE, ORD_GET_FILE_SIZE, ORD_GET_MODULE_FILE_NAME_W,
            ORD_GET_PROCESS_HEAP, ORD_HEAP_ALLOC, ORD_HEAP_CREATE, ORD_HEAP_DESTROY, ORD_HEAP_FREE,
            ORD_HEAP_SIZE, ORD_IS_BAD_READ_PTR, ORD_IS_BAD_WRITE_PTR, ORD_LOCAL_ALLOC,
            ORD_LOCAL_FREE, ORD_LOCAL_RE_ALLOC, ORD_LOCAL_SIZE, ORD_MALLOC, ORD_MEMCPY, ORD_MEMSET,
            ORD_MULTI_BYTE_TO_WIDE_CHAR, ORD_OPERATOR_DELETE, ORD_OPERATOR_NEW, ORD_READ_FILE,
            ORD_REG_CLOSE_KEY, ORD_REG_CREATE_KEY_EX_W, ORD_REG_DELETE_VALUE_W,
            ORD_REG_ENUM_VALUE_W, ORD_REG_QUERY_VALUE_EX_W, ORD_REG_SET_VALUE_EX_W,
            ORD_SET_FILE_POINTER, ORD_VIRTUAL_ALLOC, ORD_VIRTUAL_FREE, ORD_WCSDUP, ORD_WCSNCPY,
            ORD_WCSNICMP, ORD_WCSRCHR, ORD_WIDE_CHAR_TO_MULTI_BYTE, ORD_WRITE_FILE, ORD_WSPRINTF_W,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE, OPEN_EXISTING},
        kernel::CeKernel,
        memory::{HEAP_NO_SERIALIZE, HEAP_ZERO_MEMORY, LMEM_ZEROINIT, MEM_COMMIT, MEM_RELEASE},
        registry::{ERROR_SUCCESS, HKEY_CURRENT_USER, REG_DWORD},
        thread::ERROR_NOT_SUPPORTED,
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
    fs::write(sdmmc_root.join("mapinfo.bin"), b"mounted").unwrap();
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

    memory.map_bytes(0x6000, 8);
    memory.map_bytes(0x6010, 8);
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
            ORD_MEMSET,
            [0x6010, 0x2a, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x6010),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(0x6010, 8), b"****EFGH");

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
    assert_eq!(memory.read_u32(find_data_ptr)?, 0x10);
    assert_eq!(memory.read_wide_z(find_data_ptr + 40, 260), "SDMMC Disk");
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
