use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_CLOSE_HANDLE, ORD_CREATE_FILE_W, ORD_FIND_CLOSE, ORD_FIND_FIRST_FILE_W,
            ORD_FLUSH_FILE_BUFFERS, ORD_FREE, ORD_GET_FILE_SIZE, ORD_GET_MODULE_FILE_NAME_W,
            ORD_GET_PROCESS_HEAP, ORD_HEAP_ALLOC, ORD_HEAP_CREATE, ORD_HEAP_DESTROY, ORD_HEAP_FREE,
            ORD_HEAP_SIZE, ORD_LOCAL_ALLOC, ORD_LOCAL_FREE, ORD_LOCAL_RE_ALLOC, ORD_LOCAL_SIZE,
            ORD_MALLOC, ORD_MEMCPY, ORD_MEMSET, ORD_OPERATOR_NEW, ORD_READ_FILE,
            ORD_SET_FILE_POINTER, ORD_VIRTUAL_ALLOC, ORD_VIRTUAL_FREE, ORD_WCSDUP, ORD_WCSNICMP,
            ORD_WCSRCHR, ORD_WRITE_FILE,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE, OPEN_EXISTING},
        kernel::CeKernel,
        memory::{HEAP_NO_SERIALIZE, HEAP_ZERO_MEMORY, LMEM_ZEROINIT, MEM_COMMIT, MEM_RELEASE},
    },
    config::RuntimeConfig,
};

mod support;
use support::{TestGuestMemory, unique_test_root};

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
            ORD_VIRTUAL_FREE,
            [virtual_base, 0, MEM_RELEASE],
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
    memory.map_halfwords(find_data_ptr + 44, 260);

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
    assert_eq!(memory.read_wide_z(find_data_ptr + 44, 260), "SDMMC Disk");
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
    assert_eq!(memory.read_wide_z(find_data_ptr + 44, 260), "mapinfo.bin");
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
