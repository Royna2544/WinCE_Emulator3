use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_ADJUST_WINDOW_RECT_EX, ORD_BEGIN_PAINT, ORD_BIT_BLT, ORD_CHECK_MENU_RADIO_ITEM,
            ORD_CLIENT_TO_SCREEN, ORD_COPY_RECT, ORD_CREATE_COMPATIBLE_BITMAP,
            ORD_CREATE_COMPATIBLE_DC, ORD_CREATE_DIBSECTION, ORD_CREATE_MUTEX_W,
            ORD_CREATE_PALETTE, ORD_CREATE_PEN_INDIRECT, ORD_CREATE_SOLID_BRUSH,
            ORD_CREATE_WINDOW_EX_W, ORD_DELETE_OBJECT, ORD_DESTROY_WINDOW, ORD_ENABLE_WINDOW,
            ORD_END_PAINT, ORD_EQUAL_RECT, ORD_FILL_RECT, ORD_FIND_RESOURCE_W, ORD_FIND_WINDOW_W,
            ORD_GET_ACTIVE_WINDOW, ORD_GET_CAPTURE, ORD_GET_CLASS_INFO_W, ORD_GET_CLASS_NAME_W,
            ORD_GET_CLIENT_RECT, ORD_GET_CURSOR_POS, ORD_GET_DC, ORD_GET_DEVICE_CAPS,
            ORD_GET_FOCUS, ORD_GET_FOREGROUND_WINDOW, ORD_GET_MESSAGE_SOURCE, ORD_GET_MESSAGE_W,
            ORD_GET_NEAREST_PALETTE_INDEX, ORD_GET_PALETTE_ENTRIES, ORD_GET_PARENT,
            ORD_GET_QUEUE_STATUS, ORD_GET_STOCK_OBJECT, ORD_GET_SYS_COLOR, ORD_GET_SYS_COLOR_BRUSH,
            ORD_GET_SYSTEM_INFO, ORD_GET_SYSTEM_METRICS, ORD_GET_SYSTEM_PALETTE_ENTRIES,
            ORD_GET_UPDATE_RECT, ORD_GET_WINDOW, ORD_GET_WINDOW_LONG_W, ORD_GET_WINDOW_RECT,
            ORD_GET_WINDOW_TEXT_LENGTH_W, ORD_GET_WINDOW_TEXT_W, ORD_GLOBAL_MEMORY_STATUS,
            ORD_IN_SEND_MESSAGE, ORD_INFLATE_RECT, ORD_INTERSECT_RECT, ORD_INVALIDATE_RECT,
            ORD_IS_RECT_EMPTY, ORD_IS_WINDOW, ORD_IS_WINDOW_ENABLED, ORD_IS_WINDOW_VISIBLE,
            ORD_KILL_TIMER, ORD_LOAD_RESOURCE, ORD_LOAD_STRING_W, ORD_MAP_WINDOW_POINTS,
            ORD_MESSAGE_BOX_W, ORD_MOVE_WINDOW, ORD_OFFSET_RECT, ORD_PEEK_MESSAGE_W, ORD_POLYGON,
            ORD_POST_MESSAGE_W, ORD_POST_QUIT_MESSAGE, ORD_POST_THREAD_MESSAGE_W, ORD_PT_IN_RECT,
            ORD_REALIZE_PALETTE, ORD_REGISTER_CLASS_W, ORD_REGISTER_GESTURE, ORD_RELEASE_CAPTURE,
            ORD_RELEASE_DC, ORD_RELEASE_MUTEX, ORD_ROUND_RECT, ORD_SCREEN_TO_CLIENT,
            ORD_SELECT_OBJECT, ORD_SELECT_PALETTE, ORD_SEND_MESSAGE_TIMEOUT, ORD_SET_ACTIVE_WINDOW,
            ORD_SET_BK_COLOR, ORD_SET_CAPTURE, ORD_SET_FOCUS, ORD_SET_FOREGROUND_WINDOW,
            ORD_SET_PALETTE_ENTRIES, ORD_SET_PARENT, ORD_SET_RECT, ORD_SET_RECT_EMPTY,
            ORD_SET_TIMER, ORD_SET_WINDOW_LONG_W, ORD_SET_WINDOW_POS, ORD_SET_WINDOW_TEXT_W,
            ORD_SHOW_WINDOW, ORD_SIZEOF_RESOURCE, ORD_SLEEP, ORD_SYSTEM_PARAMETERS_INFO_W,
            ORD_UNION_RECT, ORD_UPDATE_WINDOW, ORD_VALIDATE_RECT,
        },
        framebuffer::{Framebuffer, FramebufferRect, PixelFormat, VirtualFramebuffer},
        gwe::{
            GW_CHILD, GW_HWNDFIRST, GW_HWNDNEXT, GW_HWNDPREV, GW_OWNER, GWL_USERDATA,
            HWND_BROADCAST, MSGSRC_SOFTWARE_POST, Point, QS_PAINT, QS_POSTMESSAGE, SM_CXBORDER,
            SM_CXSCREEN, SM_CYSCREEN, WM_MOVE, WM_PAINT, WM_QUIT, WM_SHOWWINDOW, WM_SIZE, WM_TIMER,
            WM_USER, WM_WINDOWPOSCHANGED, WS_VISIBLE,
        },
        kernel::CeKernel,
        memory::PROCESS_HEAP_HANDLE,
        resource::ResourceId,
        thread::{ERROR_ALREADY_EXISTS, ERROR_INVALID_PARAMETER},
    },
    config::RuntimeConfig,
};

mod support;
use support::TestGuestMemory;

#[test]
fn coredll_raw_gwe_rect_helpers_match_win32_semantics() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let a = 0x1_0000;
    let b = 0x1_0020;
    let c = 0x1_0040;
    memory.map_words(a, 4);
    memory.map_words(b, 4);
    memory.map_words(c, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_RECT,
            [a, 10, 20, 70, 90],
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
            ORD_ADJUST_WINDOW_RECT_EX,
            [a, 0x5200_0000, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(a)?, 10);
    assert_eq!(memory.read_i32(a + 4)?, 20);
    assert_eq!(memory.read_i32(a + 8)?, 70);
    assert_eq!(memory.read_i32(a + 12)?, 90);
    let menu = kernel
        .resources
        .create_menu(0, ResourceId::Integer(8501), None);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CHECK_MENU_RADIO_ITEM,
            [menu, 8502, 8504, 8503, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let menu_state = kernel.resources.menu(menu).unwrap();
    assert_eq!(menu_state.checked_items.get(&8502), Some(&false));
    assert_eq!(menu_state.checked_items.get(&8503), Some(&true));
    assert_eq!(menu_state.checked_items.get(&8504), Some(&false));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYS_COLOR,
            [0x4000_000f],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x00c0_c0c0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYS_COLOR_BRUSH,
            [0x4000_000f],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    let system_font = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_STOCK_OBJECT,
        [13],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetStockObject did not return a handle: {other:?}"),
    };
    assert_ne!(system_font, 0);
    let desktop_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("GetDC did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [desktop_dc, system_font],
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
            ORD_SET_BK_COLOR,
            [desktop_dc, 0x0078_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x00ff_ffff),
            ..
        }
    ));
    let polygon_points = 0x1_0060;
    memory.map_words(polygon_points, 8);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POLYGON,
            [desktop_dc, polygon_points, 4],
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
            ORD_ROUND_RECT,
            [desktop_dc, 0, 0, 20, 20, 4, 4],
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
            ORD_COPY_RECT,
            [b, a],
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
            ORD_EQUAL_RECT,
            [a, b],
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
            ORD_OFFSET_RECT,
            [b, (-5i32) as u32, 3],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(b)?, 5);
    assert_eq!(memory.read_i32(b + 4)?, 23);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INFLATE_RECT,
            [b, 5, 2],
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
            ORD_PT_IN_RECT,
            [b, 10, 22],
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
            ORD_INTERSECT_RECT,
            [c, a, b],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(c)?, 10);
    assert_eq!(memory.read_i32(c + 4)?, 21);
    assert_eq!(memory.read_i32(c + 8)?, 70);
    assert_eq!(memory.read_i32(c + 12)?, 90);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_UNION_RECT,
            [c, a, b],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(c)?, 0);
    assert_eq!(memory.read_i32(c + 4)?, 20);
    assert_eq!(memory.read_i32(c + 8)?, 70);
    assert_eq!(memory.read_i32(c + 12)?, 95);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_RECT_EMPTY,
            [c],
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
            ORD_IS_RECT_EMPTY,
            [c],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    Ok(())
}

#[test]
fn coredll_raw_system_parameters_info_returns_ce_strings_and_work_area() -> Result<()> {
    const SPI_GETWORKAREA: u32 = 0x0030;
    const SPI_GETOEMINFO: u32 = 0x0102;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 5;
    let text = 0x1000;
    let rect = 0x1200;
    memory.map_halfwords(text, 256);
    memory.map_words(rect, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SYSTEM_PARAMETERS_INFO_W,
            [SPI_GETOEMINFO, 256, text, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(text, 256), "iNavi GN 2010");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SYSTEM_PARAMETERS_INFO_W,
            [SPI_GETWORKAREA, 0, rect, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(rect)?, 0);
    assert_eq!(memory.read_u32(rect + 4)?, 0);
    assert_eq!(memory.read_u32(rect + 8)?, 800);
    assert_eq!(memory.read_u32(rect + 12)?, 480);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_fill_rect_paints_attached_framebuffer() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let mut framebuffer = VirtualFramebuffer::new(8, 6, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();
    let thread_id = 9;
    let rect_ptr = 0x1_0000;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr, 1);
    memory.write_word(rect_ptr + 4, 2);
    memory.write_word(rect_ptr + 8, 4);
    memory.write_word(rect_ptr + 12, 5);

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
    let red_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_SOLID_BRUSH,
        [0x0000_00ff],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateSolidBrush did not return a brush: {other:?}"),
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_FILL_RECT,
            [hdc, rect_ptr, red_brush],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));

    let painted = (2 * framebuffer.stride()) + PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(&framebuffer.pixels()[painted..painted + 2], &[0x00, 0xf8]);
    let untouched = PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[untouched..untouched + 2],
        &[0x00, 0x00]
    );
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(1, 2, 3, 3)]
    );

    Ok(())
}

#[test]
fn coredll_raw_create_compatible_dc_accepts_null_screen_dc() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let mem_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
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

    assert_ne!(mem_dc, 0);
    assert!(kernel.resources.is_memory_dc(mem_dc));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_bitblt_decodes_16bpp_bitfields_dib() -> Result<()> {
    const BI_BITFIELDS: u32 = 3;
    const SRCCOPY: u32 = 0x00cc_0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let mut framebuffer = VirtualFramebuffer::new(3, 2, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();

    let screen_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("GetDC did not return a handle: {other:?}"),
    };
    let mem_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
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

    let info = 0x1_0000;
    let bits_out = 0x1_0100;
    memory.map_bytes(info, 52);
    memory.map_words(bits_out, 1);
    let mut header = [0u8; 52];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-1i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&16u16.to_le_bytes());
    header[16..20].copy_from_slice(&BI_BITFIELDS.to_le_bytes());
    header[40..44].copy_from_slice(&0x0000_7c00u32.to_le_bytes());
    header[44..48].copy_from_slice(&0x0000_03e0u32.to_le_bytes());
    header[48..52].copy_from_slice(&0x0000_001fu32.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);
    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [0, info, 0, bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection did not return a bitmap: {other:?}"),
    };
    assert_ne!(bitmap, 0);
    let object = kernel.resources.bitmap(bitmap).expect("bitmap object");
    assert_eq!(
        object.rgb_masks,
        Some([0x0000_7c00, 0x0000_03e0, 0x0000_001f])
    );

    let bits_ptr = memory.read_u32(bits_out)?;
    memory.map_bytes(bits_ptr, 4);
    memory.write_bytes(
        bits_ptr,
        &[
            0x00, 0x7c, // red in RGB555
            0xe0, 0x03, // green in RGB555
        ],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_BIT_BLT,
            [screen_dc, 0, 0, 2, 1, mem_dc, 0, 0, SRCCOPY],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(&framebuffer.pixels()[0..2], &[0x00, 0xf8]);
    assert_eq!(&framebuffer.pixels()[2..4], &[0xe0, 0x07]);
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(0, 0, 2, 1)]
    );

    Ok(())
}

#[test]
fn coredll_raw_delete_object_frees_owned_dib_section_bits() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let info = 0x1_0000;
    let bits_out = 0x1_0100;
    memory.map_words(info, 10);
    memory.map_bytes(info, 40);
    memory.map_words(bits_out, 1);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&3i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-2i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&32u16.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);

    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [0, info, 0, bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection did not return a bitmap: {other:?}"),
    };
    let bits_ptr = memory.read_u32(bits_out)?;
    assert_ne!(bitmap, 0);
    assert_ne!(bits_ptr, 0);
    assert_eq!(
        kernel
            .resources
            .bitmap(bitmap)
            .map(|bitmap| bitmap.bits_owned),
        Some(true)
    );
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, bits_ptr)
            .is_some()
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_OBJECT,
            [bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert!(kernel.resources.bitmap(bitmap).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, bits_ptr)
            .is_none()
    );

    Ok(())
}

#[test]
fn coredll_raw_delete_object_frees_owned_compatible_bitmap_bits() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let screen_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("GetDC did not return a handle: {other:?}"),
    };
    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_BITMAP,
        [screen_dc, 3, 2],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateCompatibleBitmap did not return a bitmap: {other:?}"),
    };
    assert_ne!(bitmap, 0);

    let object = kernel.resources.bitmap(bitmap).expect("bitmap object");
    assert!(object.bits_owned);
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, object.bits_ptr)
            .is_some()
    );
    let bits_ptr = object.bits_ptr;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_OBJECT,
            [bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert!(kernel.resources.bitmap(bitmap).is_none());
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, bits_ptr)
            .is_none()
    );

    Ok(())
}

#[test]
fn coredll_raw_bitblt_copies_selected_dib_to_attached_framebuffer() -> Result<()> {
    const SRCCOPY: u32 = 0x00cc_0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let mut framebuffer = VirtualFramebuffer::new(4, 4, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();

    let screen_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("GetDC did not return a handle: {other:?}"),
    };
    let mem_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [screen_dc],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateCompatibleDC did not return a handle: {other:?}"),
    };

    let info = 0x1_0000;
    let bits_out = 0x1_0100;
    memory.map_words(info, 10);
    memory.map_bytes(info, 40);
    memory.map_words(bits_out, 1);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-2i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&32u16.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);
    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, info, 0, bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection did not return a bitmap: {other:?}"),
    };
    let bits_ptr = memory.read_u32(bits_out)?;
    memory.map_bytes(bits_ptr, 16);
    memory.write_bytes(
        bits_ptr,
        &[
            0x00, 0x00, 0xff, 0xff, // red
            0x00, 0xff, 0x00, 0xff, // green
            0xff, 0x00, 0x00, 0xff, // blue
            0xff, 0xff, 0xff, 0xff, // white
        ],
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_BIT_BLT,
            [screen_dc, 1, 1, 2, 2, mem_dc, 0, 0, SRCCOPY],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let offset = |x: usize, y: usize| y * framebuffer.stride() + x * 2;
    assert_eq!(
        &framebuffer.pixels()[offset(1, 1)..offset(1, 1) + 2],
        &[0x00, 0xf8]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(2, 1)..offset(2, 1) + 2],
        &[0xe0, 0x07]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(1, 2)..offset(1, 2) + 2],
        &[0x1f, 0x00]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(2, 2)..offset(2, 2) + 2],
        &[0xff, 0xff]
    );
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(1, 1, 2, 2)]
    );

    Ok(())
}

#[test]
fn coredll_raw_create_pen_indirect_reads_logpen() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let logpen_ptr = 0x1_0000;
    memory.map_bytes(logpen_ptr, 16);
    memory.write_bytes(
        logpen_ptr,
        &[
            0x02, 0x00, 0x00, 0x00, // lopnStyle
            0x07, 0x00, 0x00, 0x00, // lopnWidth.x
            0x00, 0x00, 0x00, 0x00, // lopnWidth.y
            0x56, 0x34, 0x12, 0x00, // lopnColor
        ],
    );

    let pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_PEN_INDIRECT,
        [logpen_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreatePenIndirect did not return a handle: {other:?}"),
    };
    assert_ne!(pen, 0);
    let object = kernel.resources.pen(pen).expect("pen object");
    assert_eq!(object.style, 2);
    assert_eq!(object.width, 7);
    assert_eq!(object.color, 0x0012_3456);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_PEN_INDIRECT,
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

    Ok(())
}

#[test]
fn coredll_raw_palette_entries_round_trip_and_select() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let log_palette_ptr = 0x1_1000;
    let out_ptr = 0x1_1040;
    let update_ptr = 0x1_1080;
    memory.map_bytes(log_palette_ptr, 12);
    memory.map_bytes(out_ptr, 16);
    memory.map_bytes(update_ptr, 4);
    memory.write_bytes(
        log_palette_ptr,
        &[
            0x00, 0x03, // palVersion 0x0300
            0x02, 0x00, // palNumEntries
            0x10, 0x20, 0x30, 0x00, 0xe0, 0xc0, 0xa0, 0x00,
        ],
    );
    memory.write_bytes(update_ptr, &[0x12, 0x34, 0x56, 0x00]);

    let palette = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_PALETTE,
        [log_palette_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreatePalette did not return a handle: {other:?}"),
    };
    assert_ne!(palette, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PALETTE_ENTRIES,
            [palette, 0, 2, out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(
        memory.read_bytes(out_ptr, 8),
        [0x10, 0x20, 0x30, 0x00, 0xe0, 0xc0, 0xa0, 0x00]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_PALETTE_ENTRIES,
            [palette, 1, 1, update_ptr],
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
            ORD_GET_PALETTE_ENTRIES,
            [palette, 1, 1, out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(out_ptr, 4), [0x12, 0x34, 0x56, 0x00]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEAREST_PALETTE_INDEX,
            [palette, 0x0056_3412],
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
            ORD_GET_SYSTEM_PALETTE_ENTRIES,
            [0x0200_0001, 10, 2, out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(
        memory.read_bytes(out_ptr, 8),
        [10, 10, 10, 0, 11, 11, 11, 0]
    );

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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_PALETTE,
            [hdc, palette, 0],
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
            ORD_REALIZE_PALETTE,
            [hdc],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_gwe_rejects_empty_class_names() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let empty_class_ptr = 0x1_0000;
    let wndclass_ptr = 0x1_0040;
    let wndclass_out_ptr = 0x1_0080;
    memory.write_wide_z(empty_class_ptr, "");
    memory.map_bytes(wndclass_ptr, 40);
    memory.map_bytes(wndclass_out_ptr, 40);
    let mut wndclass = [0; 40];
    wndclass[4..8].copy_from_slice(&0x0040_1234u32.to_le_bytes());
    wndclass[36..40].copy_from_slice(&empty_class_ptr.to_le_bytes());
    memory.write_bytes(wndclass_ptr, &wndclass);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_CLASS_W,
            [wndclass_ptr],
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
            ORD_GET_CLASS_INFO_W,
            [0, empty_class_ptr, wndclass_out_ptr],
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
            ORD_CREATE_WINDOW_EX_W,
            [0, empty_class_ptr, 0, WS_VISIBLE, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert!(kernel.gwe.class_info("").is_none());
    Ok(())
}

#[test]
fn coredll_raw_register_gesture_records_guest_registration() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let handle = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTER_GESTURE,
        [0x390, 0x9210, 0x3001_496c, 0xa0bc],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("RegisterGesture did not return a handle: {other:?}"),
    };
    assert_ne!(handle, 0);
    let registration = kernel.gwe.gesture_registration(0x390).unwrap();
    assert_eq!(registration.id, 0x390);
    assert_eq!(registration.handle, handle);
    assert_eq!(registration.arg1, 0x9210);
    assert_eq!(registration.arg2, 0x3001_496c);
    assert_eq!(registration.arg3, 0xa0bc);
    let allocation = kernel.memory.allocation(handle).unwrap();
    assert!(allocation.actual_size >= 0x400);
    assert!(allocation.zeroed);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REGISTER_GESTURE,
            [0, 0x9210, 0x3001_496c, 0xa0bc],
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
fn coredll_raw_gwe_ordinals_manage_hwnd_rects_points_and_resources() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let parent = kernel.create_window_ex_w(9, "PARENT", "parent", None, 1, 0, 0);
    assert!(kernel.gwe.move_window(parent, 10, 20, 300, 200, true));
    let class_ptr = 0x1_0000;
    let title_ptr = 0x1_0040;
    let wndclass_ptr = 0x1_0080;
    let wndclass_out_ptr = 0x1_00c0;
    let mutex_name_ptr = 0x1_0100;
    memory.write_wide_z(class_ptr, "CHILD");
    memory.write_wide_z(title_ptr, "child");
    memory.write_wide_z(mutex_name_ptr, "gwe-smoke-mutex");
    memory.map_bytes(wndclass_ptr, 40);
    memory.map_bytes(wndclass_out_ptr, 40);
    let mut wndclass = [0; 40];
    wndclass[4..8].copy_from_slice(&0x0040_1234u32.to_le_bytes());
    wndclass[36..40].copy_from_slice(&class_ptr.to_le_bytes());
    memory.write_bytes(wndclass_ptr, &wndclass);
    let class_atom = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_REGISTER_CLASS_W,
        [wndclass_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(atom),
            ..
        } => atom,
        other => panic!("RegisterClassW did not register raw class: {other:?}"),
    };
    assert!(class_atom >= 0xc000);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_METRICS,
            [SM_CXSCREEN],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(800),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_METRICS,
            [SM_CYSCREEN],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(480),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_METRICS,
            [SM_CXBORDER],
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
            ORD_GET_CLASS_INFO_W,
            [0, class_atom, wndclass_out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_bytes(wndclass_out_ptr, 40), wndclass);
    let mutex = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MUTEX_W,
        [0, 1, mutex_name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateMutexW did not create raw mutex: {other:?}"),
    };
    assert_ne!(mutex, 0);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let existing_mutex = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MUTEX_W,
        [0, 0, mutex_name_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateMutexW did not reopen raw mutex: {other:?}"),
    };
    assert_eq!(existing_mutex, mutex);
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_ALREADY_EXISTS
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_MUTEX,
            [mutex],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let child = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [0, class_ptr, title_ptr, 0, 5, 6, 70, 80, parent, 2, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create raw hwnd: {other:?}"),
    };
    let child_window = kernel.gwe.window(child).unwrap();
    assert_eq!(child_window.class_name, "child");
    assert_eq!(child_window.wndproc, 0x0040_1234);
    let sibling = kernel.create_window_ex_w(thread_id, "CHILD", "sibling", Some(parent), 3, 0, 0);
    let desktop = kernel.gwe.get_desktop_window();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [desktop, GW_CHILD],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == parent
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [parent, GW_HWNDFIRST],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == parent
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [parent, GW_CHILD],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [child, GW_HWNDNEXT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == sibling
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [sibling, GW_HWNDPREV],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [child, GW_OWNER],
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
            ORD_SET_WINDOW_POS,
            [sibling, 0, 0, 0, 0, 0, 0x13],
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
            ORD_GET_WINDOW,
            [parent, GW_CHILD],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == sibling
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_POS,
            [child, 0, 0, 0, 0, 0, 0x13],
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
            ORD_SET_PARENT,
            [child, sibling],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == parent
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PARENT,
            [child],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == sibling
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_PARENT,
            [child, parent],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == sibling
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_WINDOW_W,
            [class_atom, title_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_POS,
            [child, 0, 5, 6, 70, 80, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let rect_ptr = 0x3000;
    memory.map_words(rect_ptr, 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW_RECT,
            [child, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 15);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 26);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 85);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 106);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CLIENT_RECT,
            [child, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 70);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 80);

    let point_ptr = 0x3040;
    memory.write_point(point_ptr, 7, 8);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CLIENT_TO_SCREEN,
            [child, point_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(point_ptr)?, 22);
    assert_eq!(memory.read_i32(point_ptr + 4)?, 34);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SCREEN_TO_CLIENT,
            [child, point_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(point_ptr)?, 7);
    assert_eq!(memory.read_i32(point_ptr + 4)?, 8);

    kernel.gwe.set_cursor_pos(Point { x: 123, y: 456 });
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CURSOR_POS,
            [point_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(point_ptr)?, 123);
    assert_eq!(memory.read_i32(point_ptr + 4)?, 456);

    memory.write_point(point_ptr, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MAP_WINDOW_POINTS,
            [child, parent, point_ptr, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x0006_0005),
            ..
        }
    ));
    assert_eq!(memory.read_i32(point_ptr)?, 5);
    assert_eq!(memory.read_i32(point_ptr + 4)?, 6);

    let msg_ptr = 0x3080;
    memory.map_words(msg_ptr, 7);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POST_MESSAGE_W,
            [HWND_BROADCAST, WM_USER + 77, 0xaa, 0xbb],
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
            [msg_ptr, parent, WM_USER + 77, WM_USER + 77, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, parent);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_USER + 77);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POST_MESSAGE_W,
            [0, WM_USER + 88, 0xcc, 0xdd],
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
            [msg_ptr, 0, WM_USER + 88, WM_USER + 88, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, 0);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_USER + 88);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POST_MESSAGE_W,
            [child, WM_USER + 99, 0x11, 0x22],
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
            [msg_ptr, child, WM_USER + 99, WM_USER + 99, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, child);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_USER + 99);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x11);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 0x22);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, child, WM_USER + 99, WM_USER + 99],
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
            ORD_POST_QUIT_MESSAGE,
            [0x33],
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
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, WM_QUIT, WM_QUIT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_QUIT);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x33);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [child, 1],
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
            ORD_UPDATE_WINDOW,
            [child],
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
            [msg_ptr, child, WM_PAINT, WM_PAINT, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, child);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_PAINT);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_UPDATE_RECT,
            [child, rect_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 70);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 80);

    let paint_ptr = 0x3400;
    memory.map_words(paint_ptr, 16);
    memory.map_bytes(paint_ptr, 64);
    let paint_hdc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_BEGIN_PAINT,
        [child, paint_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hdc),
            ..
        } => hdc,
        other => panic!("BeginPaint did not return a virtual HDC: {other:?}"),
    };
    assert_ne!(paint_hdc, 0);
    assert_eq!(memory.read_u32(paint_ptr)?, paint_hdc);
    assert_eq!(memory.read_u32(paint_ptr + 4)?, 1);
    assert_eq!(memory.read_i32(paint_ptr + 8)?, 0);
    assert_eq!(memory.read_i32(paint_ptr + 12)?, 0);
    assert_eq!(memory.read_i32(paint_ptr + 16)?, 70);
    assert_eq!(memory.read_i32(paint_ptr + 20)?, 80);
    assert_eq!(memory.read_u32(paint_ptr + 24)?, 0);
    assert_eq!(memory.read_u32(paint_ptr + 28)?, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_UPDATE_RECT,
            [child, rect_ptr, 0],
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
            ORD_END_PAINT,
            [child, paint_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_word(rect_ptr, 3);
    memory.write_word(rect_ptr + 4, 4);
    memory.write_word(rect_ptr + 8, 30);
    memory.write_word(rect_ptr + 12, 40);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_INVALIDATE_RECT,
            [child, rect_ptr, 0],
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
            ORD_GET_MESSAGE_W,
            [msg_ptr, child, WM_PAINT, WM_PAINT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_PAINT);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VALIDATE_RECT,
            [child, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(child),
                WM_PAINT,
                WM_PAINT,
                wince_emulation_v3::ce::gwe::PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [child, 0],
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
            ORD_MOVE_WINDOW,
            [child, 20, 30, 90, 100, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let new_title_ptr = 0x1_30c0;
    let title_buffer = 0x3200;
    let class_buffer = 0x3240;
    memory.write_wide_z(new_title_ptr, "renamed child");
    memory.map_halfwords(title_buffer, 32);
    memory.map_halfwords(class_buffer, 32);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_TEXT_W,
            [child, new_title_ptr],
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
            ORD_GET_WINDOW_TEXT_LENGTH_W,
            [child],
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
            ORD_GET_WINDOW_TEXT_W,
            [child, title_buffer, 32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(13),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(title_buffer, 32), "renamed child");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CLASS_NAME_W,
            [child, class_buffer, 32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(5),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(class_buffer, 32), "child");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_LONG_W,
            [child, GWL_USERDATA as u32, 0x1234],
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
            ORD_GET_WINDOW_LONG_W,
            [child, GWL_USERDATA as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x1234),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PARENT,
            [child],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == parent
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FOCUS,
            [child],
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
            ORD_GET_FOCUS,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ACTIVE_WINDOW,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FOREGROUND_WINDOW,
            [parent],
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
            ORD_GET_FOREGROUND_WINDOW,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == parent
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_ACTIVE_WINDOW,
            [child],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == parent
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ACTIVE_WINDOW,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CAPTURE,
            [child],
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
            ORD_GET_CAPTURE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_CAPTURE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let child_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [child],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hdc),
            ..
        } => hdc,
        other => panic!("GetDC did not return a handle: {other:?}"),
    };
    assert_ne!(child_dc, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DEVICE_CAPS,
            [child_dc, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(width),
            ..
        } if width == 800
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RELEASE_DC,
            [child, child_dc],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let timer_id = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SET_TIMER,
        [child, 77, 1, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(id),
            ..
        } => id,
        other => panic!("SetTimer did not return an id: {other:?}"),
    };
    assert_eq!(timer_id, 77);
    assert!(matches!(
        table
            .dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_SLEEP, [2],),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let timer_msg_ptr = 0x3340;
    memory.map_words(timer_msg_ptr, 7);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [timer_msg_ptr, child, WM_TIMER, WM_TIMER, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(timer_msg_ptr)?, child);
    assert_eq!(memory.read_u32(timer_msg_ptr + 4)?, WM_TIMER);
    assert_eq!(memory.read_u32(timer_msg_ptr + 8)?, timer_id);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KILL_TIMER,
            [child, timer_id],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let memory_status_ptr = 0x3280;
    let system_info_ptr = 0x32c0;
    memory.map_words(memory_status_ptr, 8);
    memory.map_words(system_info_ptr, 9);
    memory.map_halfwords(system_info_ptr, 18);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GLOBAL_MEMORY_STATUS,
            [memory_status_ptr],
        ),
        CoredllDispatch::Returned { .. }
    ));
    assert_eq!(memory.read_u32(memory_status_ptr)?, 32);
    assert!(memory.read_u32(memory_status_ptr + 8)? > 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SYSTEM_INFO,
            [system_info_ptr],
        ),
        CoredllDispatch::Returned { .. }
    ));
    assert_eq!(memory.read_u32(system_info_ptr + 4)?, 4096);
    assert_eq!(memory.read_u32(system_info_ptr + 24)?, 4000);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MESSAGE_BOX_W,
            [0, title_ptr, title_ptr, 0],
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
            ORD_SHOW_WINDOW,
            [child, 1],
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
            ORD_SHOW_WINDOW,
            [child, 1],
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
            ORD_IS_WINDOW_VISIBLE,
            [child],
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
            ORD_ENABLE_WINDOW,
            [child, 0],
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
            ORD_IS_WINDOW_ENABLED,
            [child],
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
            ORD_ENABLE_WINDOW,
            [child, 1],
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
            ORD_IS_WINDOW_ENABLED,
            [child],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let resource = kernel.resources.register(
        0x4000,
        ResourceId::Integer(10),
        ResourceId::Integer(6),
        0x5000,
        32,
    );
    kernel.set_process_module_base(0x4000);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_RESOURCE_W,
            [0x4000, 10, 6],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(found),
            ..
        } if found == resource
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FIND_RESOURCE_W,
            [0, 10, 6],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(found),
            ..
        } if found == resource
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_RESOURCE,
            [0x4000, resource],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0x5000),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SIZEOF_RESOURCE,
            [0x4000, resource],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(32),
            ..
        }
    ));
    kernel
        .resources
        .register_string(0x4000, 42, "route ready", None);
    let string_ptr = 0x3080;
    memory.map_halfwords(string_ptr, 16);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_STRING_W,
            [0x4000, 42, string_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(11),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(string_ptr, 16), "route ready");
    memory.write_wide_z(string_ptr, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_STRING_W,
            [0, 42, string_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(11),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(string_ptr, 16), "route ready");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_WINDOW,
            [child],
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
            ORD_IS_WINDOW,
            [child],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_window_state_changes_queue_lifecycle_messages() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 3;
    let msg_ptr = 0x7000;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "PANEL", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [hwnd, 1],
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
            ORD_MOVE_WINDOW,
            [hwnd, 10, 20, 100, 80, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_SHOWWINDOW,
        1,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_WINDOWPOSCHANGED,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_MOVE,
        0,
        0x0014_000a,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_SIZE,
        0,
        0x0050_0064,
    );

    Ok(())
}

#[test]
fn coredll_raw_visible_create_uses_default_rect_and_exposes_paint() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 4;
    let class_ptr = 0x8000;
    let title_ptr = 0x8040;
    let msg_ptr = 0x8080;
    memory.write_wide_z(class_ptr, "FRAME");
    memory.write_wide_z(title_ptr, "");
    memory.map_words(msg_ptr, 7);

    let hwnd = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [0, class_ptr, title_ptr, WS_VISIBLE, 0, 0, 0, 0, 0, 0, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create visible raw hwnd: {other:?}"),
    };
    assert_eq!(
        kernel.gwe.get_window_rect(hwnd).unwrap(),
        wince_emulation_v3::ce::gwe::Rect::from_origin_size(0, 0, 800, 480)
    );

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_WINDOWPOSCHANGED,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_SIZE,
        0,
        0x01e0_0320,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_PAINT,
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VALIDATE_RECT,
            [hwnd, 0],
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
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_message_ipc_state_tracks_source_send_and_timeout() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 5;
    let msg_ptr = 0x9000;
    let result_ptr = 0x9040;
    memory.map_words(msg_ptr, 7);
    memory.map_words(result_ptr, 1);

    let hwnd = kernel.create_window_ex_w(thread_id, "IPC", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_SOURCE,
            [],
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
            ORD_POST_MESSAGE_W,
            [hwnd, WM_USER + 5, 0xaa, 0xbb],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_USER + 5,
        0xaa,
        0xbb,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_SOURCE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(MSGSRC_SOFTWARE_POST),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IN_SEND_MESSAGE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    kernel.gwe.begin_send_message(thread_id);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IN_SEND_MESSAGE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    kernel.gwe.end_send_message(thread_id);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_TIMEOUT,
            [hwnd, WM_USER + 6, 0x66, 0x77, 0, 100, result_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(result_ptr)?, 0);
    assert!(!kernel.gwe.in_send_message(thread_id));

    Ok(())
}

#[test]
fn coredll_raw_queue_status_tracks_thread_posts_and_paint() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 6;
    let msg_ptr = 0x9800;
    memory.map_words(msg_ptr, 7);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_QUEUE_STATUS,
            [QS_POSTMESSAGE],
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
            ORD_POST_THREAD_MESSAGE_W,
            [thread_id, WM_USER + 61, 0x61, 0x62],
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
            ORD_GET_QUEUE_STATUS,
            [QS_POSTMESSAGE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(v),
            ..
        } if v == ((QS_POSTMESSAGE << 16) | QS_POSTMESSAGE)
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        0,
        WM_USER + 61,
        0x61,
        0x62,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_QUEUE_STATUS,
            [QS_POSTMESSAGE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    let hwnd = kernel.create_window_ex_w(thread_id, "PAINTSTATUS", "", None, 0, WS_VISIBLE, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_QUEUE_STATUS,
            [QS_PAINT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(v),
            ..
        } if v == ((QS_PAINT << 16) | QS_PAINT)
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_WINDOWPOSCHANGED,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_SIZE,
        0,
        0x01e0_0320,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_PAINT,
        0,
        0,
    );

    Ok(())
}

#[test]
fn coredll_raw_destroy_parent_invalidates_children_and_purges_messages() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 6;
    let msg_ptr = 0xa000;
    memory.map_words(msg_ptr, 7);

    let parent = kernel.create_window_ex_w(thread_id, "PARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w(thread_id, "CHILD", "", Some(parent), 1, 0, 0);
    assert!(kernel.post_message_w_for_thread(thread_id, child, WM_USER + 6, 1, 2));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_WINDOW,
            [parent],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window(parent));
    assert!(!kernel.gwe.is_window(child));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

fn assert_next_message(
    table: &CoredllExportTable,
    kernel: &mut CeKernel,
    memory: &mut TestGuestMemory,
    thread_id: u32,
    msg_ptr: u32,
    hwnd: u32,
    msg: u32,
    wparam: u32,
    lparam: u32,
) {
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            kernel,
            memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr).unwrap(), hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4).unwrap(), msg);
    assert_eq!(memory.read_u32(msg_ptr + 8).unwrap(), wparam);
    assert_eq!(memory.read_u32(msg_ptr + 12).unwrap(), lparam);
}
