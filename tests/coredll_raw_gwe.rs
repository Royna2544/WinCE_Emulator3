use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_ADJUST_WINDOW_RECT_EX, ORD_APPEND_MENU_W, ORD_BEGIN_PAINT, ORD_BIT_BLT,
            ORD_BRING_WINDOW_TO_TOP, ORD_CHECK_MENU_ITEM, ORD_CHECK_MENU_RADIO_ITEM,
            ORD_CHILD_WINDOW_FROM_POINT, ORD_CLIENT_TO_SCREEN, ORD_COMBINE_RGN, ORD_COPY_RECT,
            ORD_CREATE_COMPATIBLE_BITMAP, ORD_CREATE_COMPATIBLE_DC,
            ORD_CREATE_DIALOG_INDIRECT_PARAM_W, ORD_CREATE_DIBSECTION, ORD_CREATE_FONT_INDIRECT_W,
            ORD_CREATE_MENU, ORD_CREATE_MUTEX_W, ORD_CREATE_PALETTE, ORD_CREATE_PATTERN_BRUSH,
            ORD_CREATE_PEN_INDIRECT, ORD_CREATE_POPUP_MENU, ORD_CREATE_RECT_RGN,
            ORD_CREATE_SOLID_BRUSH, ORD_CREATE_WINDOW_EX_W, ORD_DEF_WINDOW_PROC_W,
            ORD_DELETE_OBJECT, ORD_DESTROY_ICON, ORD_DESTROY_WINDOW, ORD_DISPATCH_MESSAGE_W,
            ORD_DRAW_MENU_BAR, ORD_ENABLE_MENU_ITEM, ORD_ENABLE_WINDOW, ORD_END_PAINT,
            ORD_EQUAL_RECT, ORD_EXT_TEXT_OUT_W, ORD_FILL_RECT, ORD_FIND_RESOURCE_W,
            ORD_FIND_WINDOW_W, ORD_GET_ACTIVE_WINDOW, ORD_GET_ASSOCIATED_MENU,
            ORD_GET_ASYNC_KEY_STATE, ORD_GET_ASYNC_SHIFT_FLAGS, ORD_GET_CAPTURE,
            ORD_GET_CLASS_INFO_W, ORD_GET_CLASS_NAME_W, ORD_GET_CLIENT_RECT, ORD_GET_CURSOR_POS,
            ORD_GET_DC, ORD_GET_DEVICE_CAPS, ORD_GET_DIALOG_BASE_UNITS, ORD_GET_DIBCOLOR_TABLE,
            ORD_GET_DLG_CTRL_ID, ORD_GET_DLG_ITEM, ORD_GET_DLG_ITEM_INT, ORD_GET_DLG_ITEM_TEXT_W,
            ORD_GET_FOCUS, ORD_GET_FOREGROUND_KEYBOARD_TARGET, ORD_GET_FOREGROUND_WINDOW,
            ORD_GET_KEY_STATE, ORD_GET_KEYBOARD_TARGET, ORD_GET_MENU, ORD_GET_MENU_ITEM_INFO_W,
            ORD_GET_MESSAGE_POS, ORD_GET_MESSAGE_QUEUE_READY_TIME_STAMP, ORD_GET_MESSAGE_SOURCE,
            ORD_GET_MESSAGE_W, ORD_GET_MESSAGE_WNO_WAIT, ORD_GET_NEAREST_PALETTE_INDEX,
            ORD_GET_NEXT_DLG_GROUP_ITEM, ORD_GET_NEXT_DLG_TAB_ITEM, ORD_GET_PALETTE_ENTRIES,
            ORD_GET_PARENT, ORD_GET_QUEUE_STATUS, ORD_GET_RGN_BOX, ORD_GET_ROP2,
            ORD_GET_STOCK_OBJECT, ORD_GET_SUB_MENU, ORD_GET_SYS_COLOR, ORD_GET_SYS_COLOR_BRUSH,
            ORD_GET_SYSTEM_INFO, ORD_GET_SYSTEM_METRICS, ORD_GET_SYSTEM_PALETTE_ENTRIES,
            ORD_GET_TEXT_ALIGN, ORD_GET_TEXT_COLOR, ORD_GET_TEXT_EXTENT_EX_POINT_W,
            ORD_GET_TEXT_FACE_W, ORD_GET_TEXT_METRICS_W, ORD_GET_UPDATE_RECT, ORD_GET_UPDATE_RGN,
            ORD_GET_WINDOW, ORD_GET_WINDOW_LONG_W, ORD_GET_WINDOW_RECT, ORD_GET_WINDOW_RGN,
            ORD_GET_WINDOW_TEXT_LENGTH_W, ORD_GET_WINDOW_TEXT_W, ORD_GET_WINDOW_THREAD_PROCESS_ID,
            ORD_GLOBAL_MEMORY_STATUS, ORD_IN_SEND_MESSAGE, ORD_INFLATE_RECT, ORD_INSERT_MENU_W,
            ORD_INTERSECT_RECT, ORD_INVALIDATE_RECT, ORD_IS_CHILD, ORD_IS_DIALOG_MESSAGE_W,
            ORD_IS_RECT_EMPTY, ORD_IS_WINDOW, ORD_IS_WINDOW_ENABLED, ORD_IS_WINDOW_VISIBLE,
            ORD_KEYBD_EVENT, ORD_KILL_TIMER, ORD_LOAD_ICON_W, ORD_LOAD_RESOURCE, ORD_LOAD_STRING_W,
            ORD_MAP_DIALOG_RECT, ORD_MAP_WINDOW_POINTS, ORD_MESSAGE_BOX_W, ORD_MOVE_WINDOW,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX, ORD_OFFSET_RECT, ORD_PAT_BLT, ORD_PEEK_MESSAGE_W,
            ORD_POLYGON, ORD_POLYLINE, ORD_POST_KEYBD_MESSAGE, ORD_POST_MESSAGE_W,
            ORD_POST_QUIT_MESSAGE, ORD_POST_THREAD_MESSAGE_W, ORD_PT_IN_RECT, ORD_PT_IN_REGION,
            ORD_REALIZE_PALETTE, ORD_RECT_IN_REGION, ORD_REDRAW_WINDOW, ORD_REGISTER_CLASS_W,
            ORD_REGISTER_GESTURE, ORD_RELEASE_CAPTURE, ORD_RELEASE_DC, ORD_RELEASE_MUTEX,
            ORD_REMOVE_MENU, ORD_ROUND_RECT, ORD_SCREEN_TO_CLIENT, ORD_SELECT_OBJECT,
            ORD_SELECT_PALETTE, ORD_SEND_DLG_ITEM_MESSAGE_W, ORD_SEND_MESSAGE_TIMEOUT,
            ORD_SEND_MESSAGE_W, ORD_SEND_NOTIFY_MESSAGE_W, ORD_SET_ACTIVE_WINDOW,
            ORD_SET_ASSOCIATED_MENU, ORD_SET_BK_COLOR, ORD_SET_BRUSH_ORG_EX, ORD_SET_CAPTURE,
            ORD_SET_DIBCOLOR_TABLE, ORD_SET_DIBITS_TO_DEVICE, ORD_SET_DLG_ITEM_INT,
            ORD_SET_DLG_ITEM_TEXT_W, ORD_SET_FOCUS, ORD_SET_FOREGROUND_WINDOW,
            ORD_SET_KEYBOARD_TARGET, ORD_SET_MENU, ORD_SET_MENU_ITEM_INFO_W,
            ORD_SET_PALETTE_ENTRIES, ORD_SET_PARENT, ORD_SET_RECT, ORD_SET_RECT_EMPTY,
            ORD_SET_ROP2, ORD_SET_TEXT_ALIGN, ORD_SET_TEXT_COLOR, ORD_SET_TIMER,
            ORD_SET_WINDOW_LONG_W, ORD_SET_WINDOW_POS, ORD_SET_WINDOW_RGN, ORD_SET_WINDOW_TEXT_W,
            ORD_SHOW_WINDOW, ORD_SIZEOF_RESOURCE, ORD_SLEEP, ORD_STRETCH_BLT, ORD_STRETCH_DIBITS,
            ORD_SYSTEM_PARAMETERS_INFO_W, ORD_TRACK_POPUP_MENU_EX, ORD_TRANSPARENT_IMAGE,
            ORD_UNION_RECT, ORD_UPDATE_WINDOW, ORD_VALIDATE_RECT, ORD_WINDOW_FROM_POINT,
        },
        framebuffer::{Framebuffer, FramebufferRect, PixelFormat, VirtualFramebuffer},
        gwe::{
            BS_DEFPUSHBUTTON, BS_PUSHBUTTON, DC_HASDEFID, DLGC_BUTTON, DLGC_DEFPUSHBUTTON,
            DLGC_UNDEFPUSHBUTTON, DM_GETDEFID, DM_SETDEFID, GW_CHILD, GW_HWNDFIRST, GW_HWNDNEXT,
            GW_HWNDPREV, GW_OWNER, GWL_USERDATA, HWND_BROADCAST, KEY_SHIFT_ANY_SHIFT_FLAG,
            KEY_STATE_DOWN_FLAG, KEY_STATE_GET_ASYNC_DOWN_FLAG, KEY_STATE_PREV_DOWN_FLAG,
            MSGSRC_HARDWARE_KEYBOARD, MSGSRC_SOFTWARE_POST, MSGSRC_SOFTWARE_SEND, Message,
            PeekFlags, Point, QS_PAINT, QS_POSTMESSAGE, QS_SENDMESSAGE, QS_TIMER, Rect,
            SM_CXBORDER, SM_CXSCREEN, SM_CYSCREEN, SMF_NOTIFY_MESSAGE, SMF_SENDER_NO_WAIT,
            SMF_TIMEOUT, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            SWP_SHOWWINDOW, VK_LSHIFT, VK_SHIFT, WA_ACTIVE, WA_INACTIVE, WM_ACTIVATE,
            WM_CANCELMODE, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_ENABLE, WM_ERASEBKGND, WM_GETDLGCODE,
            WM_GETTEXT, WM_GETTEXTLENGTH, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN,
            WM_MOVE, WM_NCDESTROY, WM_PAINT, WM_QUIT, WM_SETFOCUS, WM_SETTEXT, WM_SHOWWINDOW,
            WM_SIZE, WM_TIMER, WM_USER, WM_WINDOWPOSCHANGED, WS_CHILD, WS_DISABLED, WS_GROUP,
            WS_POPUP, WS_TABSTOP, WS_VISIBLE,
        },
        kernel::CeKernel,
        memory::PROCESS_HEAP_HANDLE,
        resource::ResourceId,
        thread::{
            ERROR_ALREADY_EXISTS, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER,
            ERROR_INVALID_WINDOW_HANDLE,
        },
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
            value: CoredllValue::Handle(handle),
            ..
        } if handle == system_font
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
fn coredll_raw_destroy_icon_accepts_loaded_icon_handles() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let icon = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_ICON_W,
        [0, 32512],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(icon),
            ..
        } => icon,
        other => panic!("LoadIconW did not return a handle: {other:?}"),
    };
    assert_ne!(icon, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [icon],
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
            ORD_DESTROY_ICON,
            [0],
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
fn coredll_raw_polyline_paints_attached_framebuffer_with_selected_pen() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let mut framebuffer = VirtualFramebuffer::new(8, 6, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();
    let thread_id = 9;
    let points_ptr = 0x1_0000;
    let logpen_ptr = 0x1_0100;
    memory.map_bytes(points_ptr, 24);
    memory.map_bytes(logpen_ptr, 16);
    for (index, (x, y)) in [(1u32, 1u32), (4, 1), (4, 3)].into_iter().enumerate() {
        let point = points_ptr + (index as u32) * 8;
        memory.write_word(point, x);
        memory.write_word(point + 4, y);
    }
    memory.write_bytes(
        logpen_ptr,
        &[
            0x00, 0x00, 0x00, 0x00, // lopnStyle = PS_SOLID
            0x01, 0x00, 0x00, 0x00, // lopnWidth.x = 1
            0x00, 0x00, 0x00, 0x00, // lopnWidth.y
            0x00, 0xff, 0x00, 0x00, // lopnColor = green
        ],
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
        other => panic!("CreatePenIndirect did not return a pen: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [hdc, pen],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_POLYLINE,
            [hdc, points_ptr, 3],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let bpp = PixelFormat::Rgb565.bytes_per_pixel();
    let offset = |x: usize, y: usize| y * framebuffer.stride() + x * bpp;
    assert_eq!(
        &framebuffer.pixels()[offset(1, 1)..offset(1, 1) + 2],
        &[0xe0, 0x07]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(4, 1)..offset(4, 1) + 2],
        &[0xe0, 0x07]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(4, 3)..offset(4, 3) + 2],
        &[0xe0, 0x07]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(0, 0)..offset(0, 0) + 2],
        &[0x00, 0x00]
    );
    assert!(!framebuffer.dirty_rects().is_empty());

    Ok(())
}

#[test]
fn coredll_raw_polyline_with_null_pen_does_not_touch_framebuffer() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let mut framebuffer = VirtualFramebuffer::new(8, 6, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();
    let thread_id = 9;
    let points_ptr = 0x1_0000;
    memory.map_bytes(points_ptr, 16);
    for (index, (x, y)) in [(1u32, 1u32), (4, 1)].into_iter().enumerate() {
        let point = points_ptr + (index as u32) * 8;
        memory.write_word(point, x);
        memory.write_word(point + 4, y);
    }

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
    let null_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_STOCK_OBJECT,
        [8],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetStockObject(NULL_PEN) did not return a handle: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [hdc, null_pen],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_POLYLINE,
            [hdc, points_ptr, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert!(framebuffer.pixels().iter().all(|byte| *byte == 0));
    assert!(framebuffer.dirty_rects().is_empty());

    Ok(())
}

fn create_selected_rgb565_dib(
    table: &CoredllExportTable,
    kernel: &mut CeKernel,
    memory: &mut TestGuestMemory,
    thread_id: u32,
    width: i32,
    height: i32,
) -> (u32, u32, u32) {
    let (mem_dc, _bitmap, bits_ptr, stride) =
        create_selected_rgb565_dib_with_bitmap(table, kernel, memory, thread_id, width, height);
    (mem_dc, bits_ptr, stride)
}

fn create_selected_rgb565_dib_with_bitmap(
    table: &CoredllExportTable,
    kernel: &mut CeKernel,
    memory: &mut TestGuestMemory,
    thread_id: u32,
    width: i32,
    height: i32,
) -> (u32, u32, u32, u32) {
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
    let info = 0x1_0000;
    let bits_out = 0x1_0100;
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
    (mem_dc, bitmap, bits_ptr, stride)
}

fn rgb565_at(memory: &TestGuestMemory, bits_ptr: u32, stride: u32, x: u32, y: u32) -> u16 {
    memory
        .read_u16(bits_ptr + y * stride + x * 2)
        .expect("pixel should be readable")
}

fn write_rgb565(
    memory: &mut TestGuestMemory,
    bits_ptr: u32,
    stride: u32,
    x: u32,
    y: u32,
    raw: u16,
) {
    memory
        .write_u16(bits_ptr + y * stride + x * 2, raw)
        .expect("pixel should be writable")
}

#[test]
fn coredll_raw_fill_rect_paints_selected_memory_dib() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr, 1);
    memory.write_word(rect_ptr + 4, 2);
    memory.write_word(rect_ptr + 8, 5);
    memory.write_word(rect_ptr + 12, 5);
    let green_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_SOLID_BRUSH,
        [0x0000_ff00],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateSolidBrush did not return a brush: {other:?}"),
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FILL_RECT,
            [mem_dc, rect_ptr, green_brush],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 1, 2), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 4, 4), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_ext_text_out_opaque_fills_selected_memory_dib_with_bk_color() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr, 2);
    memory.write_word(rect_ptr + 4, 1);
    memory.write_word(rect_ptr + 8, 6);
    memory.write_word(rect_ptr + 12, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_BK_COLOR,
            [mem_dc, 0x0000_ff00],
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
            ORD_EXT_TEXT_OUT_W,
            [mem_dc, 0, 0, 0x0002, rect_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 1), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 3), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 1, 1), 0x0000);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 6, 3), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_pat_blt_patcopy_paints_selected_memory_dib() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let blue_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_SOLID_BRUSH,
        [0x00ff_0000],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateSolidBrush did not return a brush: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, blue_brush],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PAT_BLT,
            [mem_dc, 2, 1, 3, 4, 0x00f0_0021],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 1), 0x001f);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 4, 4), 0x001f);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 1, 1), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_pat_blt_tiles_pattern_brush_on_selected_memory_dib() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (dst_dc, dst_bits, dst_stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 6, 5);
    let (_pattern_dc, pattern_bitmap, pattern_bits, pattern_stride) =
        create_selected_rgb565_dib_with_bitmap(&table, &mut kernel, &mut memory, thread_id, 2, 2);
    write_rgb565(&mut memory, pattern_bits, pattern_stride, 0, 0, 0xf800);
    write_rgb565(&mut memory, pattern_bits, pattern_stride, 1, 0, 0x07e0);
    write_rgb565(&mut memory, pattern_bits, pattern_stride, 0, 1, 0x001f);
    write_rgb565(&mut memory, pattern_bits, pattern_stride, 1, 1, 0xffff);
    let pattern_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_PATTERN_BRUSH,
        [pattern_bitmap],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreatePatternBrush did not return a brush: {other:?}"),
    };
    assert_ne!(pattern_brush, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [dst_dc, pattern_brush],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    let old_origin = 0x1_0400;
    memory.map_words(old_origin, 2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_BRUSH_ORG_EX,
            [dst_dc, 1, 1, old_origin],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(old_origin)?, 0);
    assert_eq!(memory.read_i32(old_origin + 4)?, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PAT_BLT,
            [dst_dc, 1, 1, 3, 3, 0x00f0_0021],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, dst_bits, dst_stride, 1, 1), 0xf800);
    assert_eq!(rgb565_at(&memory, dst_bits, dst_stride, 2, 1), 0x07e0);
    assert_eq!(rgb565_at(&memory, dst_bits, dst_stride, 1, 2), 0x001f);
    assert_eq!(rgb565_at(&memory, dst_bits, dst_stride, 2, 2), 0xffff);
    assert_eq!(rgb565_at(&memory, dst_bits, dst_stride, 0, 0), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_polyline_uses_pen_width_on_selected_memory_dib() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let points_ptr = 0x1_0200;
    let logpen_ptr = 0x1_0300;
    memory.map_bytes(points_ptr, 16);
    memory.map_bytes(logpen_ptr, 16);
    memory.write_point(points_ptr, 1, 3);
    memory.write_point(points_ptr + 8, 6, 3);
    memory.write_bytes(
        logpen_ptr,
        &[
            0x00, 0x00, 0x00, 0x00, // PS_SOLID
            0x03, 0x00, 0x00, 0x00, // width 3
            0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, // red
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
        other => panic!("CreatePenIndirect did not return a pen: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, pen],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POLYLINE,
            [mem_dc, points_ptr, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 3), 0xf800);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 2), 0xf800);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 4), 0xf800);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 0), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_set_get_rop2_round_trips_dc_state() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, _bits_ptr, _stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 4, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ROP2,
            [mem_dc],
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
            ORD_SET_ROP2,
            [mem_dc, 7],
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
            ORD_GET_ROP2,
            [mem_dc],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(7),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_polyline_applies_rop2_xorpen_on_selected_memory_dib() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let points_ptr = 0x1_0200;
    let logpen_ptr = 0x1_0300;
    memory.map_bytes(points_ptr, 16);
    memory.map_bytes(logpen_ptr, 16);
    memory.write_point(points_ptr, 1, 3);
    memory.write_point(points_ptr + 8, 6, 3);
    write_rgb565(&mut memory, bits_ptr, stride, 3, 3, 0xffff);
    memory.write_bytes(
        logpen_ptr,
        &[
            0x00, 0x00, 0x00, 0x00, // PS_SOLID
            0x01, 0x00, 0x00, 0x00, // width 1
            0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, // red
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
        other => panic!("CreatePenIndirect did not return a pen: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, pen],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_ROP2,
            [mem_dc, 7],
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
            ORD_POLYLINE,
            [mem_dc, points_ptr, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 3), 0x07ff);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 3), 0xf800);

    Ok(())
}

#[test]
fn coredll_raw_polygon_fills_selected_memory_dib_with_selected_brush() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let points_ptr = 0x1_0200;
    memory.map_bytes(points_ptr, 32);
    for (index, (x, y)) in [(1, 1), (6, 1), (6, 5), (1, 5)].into_iter().enumerate() {
        memory.write_point(points_ptr + index as u32 * 8, x, y);
    }
    let blue_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_SOLID_BRUSH,
        [0x00ff_0000],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateSolidBrush did not return a brush: {other:?}"),
    };
    let null_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_STOCK_OBJECT,
        [8],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetStockObject(NULL_PEN) did not return a handle: {other:?}"),
    };
    for object in [blue_brush, null_pen] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SELECT_OBJECT,
                [mem_dc, object],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(handle),
                ..
            } if handle != 0
        ));
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POLYGON,
            [mem_dc, points_ptr, 4],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 2), 0x001f);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 4), 0x001f);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_polygon_uses_winding_fill_for_repeated_edges() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let points_ptr = 0x1_0200;
    let points = [
        (1, 1),
        (6, 1),
        (6, 5),
        (1, 5),
        (1, 1),
        (6, 1),
        (6, 5),
        (1, 5),
    ];
    memory.map_bytes(points_ptr, (points.len() * 8) as u32);
    for (index, (x, y)) in points.into_iter().enumerate() {
        memory.write_point(points_ptr + index as u32 * 8, x, y);
    }
    let blue_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_SOLID_BRUSH,
        [0x00ff_0000],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateSolidBrush did not return a brush: {other:?}"),
    };
    let null_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_STOCK_OBJECT,
        [8],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetStockObject(NULL_PEN) did not return a handle: {other:?}"),
    };
    for object in [blue_brush, null_pen] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_SELECT_OBJECT,
                [mem_dc, object],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(handle),
                ..
            } if handle != 0
        ));
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POLYGON,
            [mem_dc, points_ptr, points.len() as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 2), 0x001f);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 4), 0x001f);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x0000);

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
fn coredll_raw_select_object_returns_restorable_dc_defaults() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let black_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_STOCK_OBJECT,
        [7],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetStockObject(BLACK_PEN) did not return a handle: {other:?}"),
    };
    let white_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_STOCK_OBJECT,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetStockObject(WHITE_BRUSH) did not return a handle: {other:?}"),
    };
    let default_palette = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_STOCK_OBJECT,
        [15],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetStockObject(DEFAULT_PALETTE) did not return a handle: {other:?}"),
    };
    assert_eq!(kernel.resources.gdi_object_kind(default_palette), "palette");

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
    memory.map_bytes(info, 40);
    memory.map_words(bits_out, 1);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&1i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-1i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&16u16.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);

    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
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

    let old_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SELECT_OBJECT,
        [mem_dc, bitmap],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("SelectObject(bitmap) did not return a handle: {other:?}"),
    };
    assert_ne!(old_bitmap, 0);
    assert_eq!(kernel.resources.gdi_object_kind(old_bitmap), "bitmap");
    assert_eq!(kernel.resources.selected_bitmap(mem_dc), Some(bitmap));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, old_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == bitmap
    ));
    assert_eq!(kernel.resources.selected_bitmap(mem_dc), None);

    let logpen_ptr = 0x1_0200;
    memory.map_bytes(logpen_ptr, 16);
    memory.write_bytes(
        logpen_ptr,
        &[
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff,
            0x00, 0x00,
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
        other => panic!("CreatePenIndirect did not return a pen: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, pen],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == black_pen
    ));

    let brush = match table.dispatch_raw_ordinal_with_memory(
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
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [mem_dc, brush],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == white_brush
    ));

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
    assert_ne!(
        bitmap,
        0,
        "last_error={}",
        kernel.threads.get_last_error(thread_id)
    );
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
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
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
fn coredll_raw_dib_color_table_drives_8bpp_blit() -> Result<()> {
    const SRCCOPY: u32 = 0x00cc_0020;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let mut framebuffer = VirtualFramebuffer::new(2, 1, PixelFormat::Rgb565)?;
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "FRAME",
        "frame",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 2, 1),
    );
    let screen_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [hwnd],
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
    let colors = 0x1_0200;
    let colors_out = 0x1_0300;
    memory.map_words(info, 10);
    memory.map_bytes(info, 40);
    memory.map_words(bits_out, 1);
    memory.map_bytes(colors, 8);
    memory.map_bytes(colors_out, 8);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-1i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&8u16.to_le_bytes());
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
    assert_ne!(
        bitmap,
        0,
        "last_error={}",
        kernel.threads.get_last_error(thread_id)
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
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));

    memory.write_bytes(colors, &[0, 0, 255, 0, 0, 255, 0, 0]);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_DIBCOLOR_TABLE,
            [mem_dc, 0, 2, colors],
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
            ORD_GET_DIBCOLOR_TABLE,
            [mem_dc, 0, 2, colors_out],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(
        memory.read_bytes(colors_out, 8),
        [0, 0, 255, 0, 0, 255, 0, 0]
    );

    let bits_ptr = memory.read_u32(bits_out)?;
    memory.map_bytes(bits_ptr, 4);
    memory.write_bytes(bits_ptr, &[0, 1, 0, 0]);
    let _ = framebuffer.take_dirty_rects();
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
fn coredll_raw_create_dibsection_accepts_1bpp_pal_colors_and_blits() -> Result<()> {
    const DIB_PAL_COLORS: u32 = 1;
    const SRCCOPY: u32 = 0x00cc_0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let mut framebuffer = VirtualFramebuffer::new(2, 2, PixelFormat::Rgb565)?;
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
    memory.map_bytes(info, 40);
    memory.map_words(bits_out, 1);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-2i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&1u16.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);

    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, info, DIB_PAL_COLORS, bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection did not return a bitmap: {other:?}"),
    };
    assert_ne!(
        bitmap,
        0,
        "last_error={}",
        kernel.threads.get_last_error(thread_id)
    );
    let object = kernel.resources.bitmap(bitmap).expect("bitmap object");
    assert_eq!(object.bits_pixel, 1);
    assert_eq!(
        object.color_table,
        vec![[0, 0, 0, 0], [0xff, 0xff, 0xff, 0]]
    );

    let bits_ptr = memory.read_u32(bits_out)?;
    memory.map_bytes(bits_ptr, 8);
    memory.write_bytes(
        bits_ptr,
        &[
            0b0100_0000,
            0,
            0,
            0, // black, white
            0b1000_0000,
            0,
            0,
            0, // white, black
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
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_BIT_BLT,
            [screen_dc, 0, 0, 2, 2, mem_dc, 0, 0, SRCCOPY],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let stride = framebuffer.stride();
    let offset = |x: usize, y: usize| y * stride + x * 2;
    assert_eq!(
        &framebuffer.pixels()[offset(0, 0)..offset(0, 0) + 2],
        &[0x00, 0x00]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(1, 0)..offset(1, 0) + 2],
        &[0xff, 0xff]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(0, 1)..offset(0, 1) + 2],
        &[0xff, 0xff]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(1, 1)..offset(1, 1) + 2],
        &[0x00, 0x00]
    );
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(0, 0, 2, 2)]
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
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
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

    let stride = framebuffer.stride();
    let offset = |x: usize, y: usize| y * stride + x * 2;
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
fn coredll_raw_bitblt_copies_between_memory_dcs() -> Result<()> {
    const SRCCOPY: u32 = 0x00cc_0020;

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
    let src_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("CreateCompatibleDC(src) did not return a handle: {other:?}"),
    };
    let dst_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("CreateCompatibleDC(dst) did not return a handle: {other:?}"),
    };

    let src_info = 0x1_0000;
    let src_bits_out = 0x1_0100;
    let dst_info = 0x1_0200;
    let dst_bits_out = 0x1_0300;
    memory.map_bytes(src_info, 40);
    memory.map_words(src_bits_out, 1);
    memory.map_bytes(dst_info, 40);
    memory.map_words(dst_bits_out, 1);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-1i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&32u16.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    memory.write_bytes(src_info, &header);
    memory.write_word(src_info, 40);
    memory.write_bytes(dst_info, &header);
    memory.write_word(dst_info, 40);

    let src_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, src_info, 0, src_bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection(src) did not return a bitmap: {other:?}"),
    };
    let dst_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, dst_info, 0, dst_bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection(dst) did not return a bitmap: {other:?}"),
    };
    let src_bits = memory.read_u32(src_bits_out)?;
    let dst_bits = memory.read_u32(dst_bits_out)?;
    memory.map_bytes(src_bits, 8);
    memory.map_bytes(dst_bits, 8);
    memory.write_bytes(
        src_bits,
        &[
            0x00, 0x00, 0xff, 0xff, // red
            0x00, 0xff, 0x00, 0xff, // green
        ],
    );
    memory.write_bytes(dst_bits, &[0; 8]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [src_dc, src_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [dst_dc, dst_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BIT_BLT,
            [dst_dc, 0, 0, 2, 1, src_dc, 0, 0, SRCCOPY],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(
        memory.read_bytes(dst_bits, 8),
        vec![
            0x00, 0x00, 0xff, 0xff, // red
            0x00, 0xff, 0x00, 0xff, // green
        ]
    );

    Ok(())
}

#[test]
fn coredll_raw_stretchblt_uses_stretch_abi_and_scales_between_memory_dcs() -> Result<()> {
    const SRCCOPY: u32 = 0x00cc_0020;

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
    let src_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("CreateCompatibleDC(src) did not return a handle: {other:?}"),
    };
    let dst_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("CreateCompatibleDC(dst) did not return a handle: {other:?}"),
    };

    let src_info = 0x1_0000;
    let src_bits_out = 0x1_0100;
    let dst_info = 0x1_0200;
    let dst_bits_out = 0x1_0300;
    memory.map_bytes(src_info, 40);
    memory.map_words(src_bits_out, 1);
    memory.map_bytes(dst_info, 40);
    memory.map_words(dst_bits_out, 1);
    let mut src_header = [0u8; 40];
    src_header[0..4].copy_from_slice(&40u32.to_le_bytes());
    src_header[4..8].copy_from_slice(&2i32.to_le_bytes());
    src_header[8..12].copy_from_slice(&(-1i32).to_le_bytes());
    src_header[12..14].copy_from_slice(&1u16.to_le_bytes());
    src_header[14..16].copy_from_slice(&32u16.to_le_bytes());
    src_header[16..20].copy_from_slice(&0u32.to_le_bytes());
    let mut dst_header = src_header;
    dst_header[4..8].copy_from_slice(&4i32.to_le_bytes());
    memory.write_bytes(src_info, &src_header);
    memory.write_word(src_info, 40);
    memory.write_bytes(dst_info, &dst_header);
    memory.write_word(dst_info, 40);

    let src_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, src_info, 0, src_bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection(src) did not return a bitmap: {other:?}"),
    };
    let dst_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, dst_info, 0, dst_bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection(dst) did not return a bitmap: {other:?}"),
    };
    let src_bits = memory.read_u32(src_bits_out)?;
    let dst_bits = memory.read_u32(dst_bits_out)?;
    memory.write_bytes(
        src_bits,
        &[
            0x00, 0x00, 0xff, 0xff, // red
            0x00, 0xff, 0x00, 0xff, // green
        ],
    );
    memory.write_bytes(dst_bits, &[0; 16]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [src_dc, src_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [dst_dc, dst_bitmap],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_STRETCH_BLT,
            [dst_dc, 0, 0, 4, 1, src_dc, 0, 0, 2, 1, SRCCOPY],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(
        memory.read_bytes(dst_bits, 16),
        vec![
            0x00, 0x00, 0xff, 0xff, // red
            0x00, 0x00, 0xff, 0xff, // red
            0x00, 0xff, 0x00, 0xff, // green
            0x00, 0xff, 0x00, 0xff, // green
        ]
    );

    Ok(())
}

#[test]
fn coredll_raw_transparent_image_copies_selected_bitmap_with_color_key() -> Result<()> {
    const MAGENTA_COLORREF: u32 = 0x00ff_00ff;

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
            0xff, 0x00, 0xff, 0xff, // transparent magenta
            0x00, 0xff, 0x00, 0xff, // green
            0xff, 0x00, 0x00, 0xff, // blue
            0x00, 0x00, 0xff, 0xff, // red
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
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_TRANSPARENT_IMAGE,
            [screen_dc, 1, 1, 2, 2, mem_dc, 0, 0, 2, 2, MAGENTA_COLORREF],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let stride = framebuffer.stride();
    let offset = |x: usize, y: usize| y * stride + x * 2;
    assert_eq!(
        &framebuffer.pixels()[offset(1, 1)..offset(1, 1) + 2],
        &[0x00, 0x00]
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
        &[0x00, 0xf8]
    );
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(1, 1, 2, 2)]
    );

    Ok(())
}

#[test]
fn coredll_raw_transparent_image_composites_between_memory_dcs() -> Result<()> {
    const MAGENTA_COLORREF: u32 = 0x00ff_00ff;
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
    let src_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("CreateCompatibleDC(src) did not return a handle: {other:?}"),
    };
    let dst_dc = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("CreateCompatibleDC(dst) did not return a handle: {other:?}"),
    };

    let info = 0x1_0000;
    let src_bits_out = 0x1_0100;
    let dst_bits_out = 0x1_0110;
    memory.map_words(info, 10);
    memory.map_bytes(info, 40);
    memory.map_words(src_bits_out, 1);
    memory.map_words(dst_bits_out, 1);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-2i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&32u16.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);

    let src_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, info, 0, src_bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection(src) did not return a bitmap: {other:?}"),
    };
    let dst_bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIBSECTION,
        [screen_dc, info, 0, dst_bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection(dst) did not return a bitmap: {other:?}"),
    };
    let src_bits = memory.read_u32(src_bits_out)?;
    let dst_bits = memory.read_u32(dst_bits_out)?;
    memory.map_bytes(src_bits, 16);
    memory.map_bytes(dst_bits, 16);
    memory.write_bytes(
        src_bits,
        &[
            0xff, 0x00, 0xff, 0xff, // transparent magenta
            0x00, 0xff, 0x00, 0xff, // green
            0xff, 0x00, 0x00, 0xff, // blue
            0x00, 0x00, 0xff, 0xff, // red
        ],
    );
    memory.write_bytes(dst_bits, &[0xff; 16]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [src_dc, src_bitmap],
        ),
        CoredllDispatch::Returned { .. }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [dst_dc, dst_bitmap],
        ),
        CoredllDispatch::Returned { .. }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            None,
            thread_id,
            ORD_TRANSPARENT_IMAGE,
            [dst_dc, 0, 0, 2, 2, src_dc, 0, 0, 2, 2, MAGENTA_COLORREF],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
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
            [screen_dc, 1, 1, 2, 2, dst_dc, 0, 0, SRCCOPY],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let stride = framebuffer.stride();
    let offset = |x: usize, y: usize| y * stride + x * 2;
    assert_eq!(
        &framebuffer.pixels()[offset(1, 1)..offset(1, 1) + 2],
        &[0xff, 0xff]
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
        &[0x00, 0xf8]
    );

    Ok(())
}

#[test]
fn coredll_raw_direct_dib_calls_paint_attached_framebuffer() -> Result<()> {
    const DIB_RGB_COLORS: u32 = 0;
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

    let info = 0x1_0000;
    let bits = 0x1_0100;
    memory.map_words(info, 10);
    memory.map_bytes(info, 40);
    memory.map_bytes(bits, 16);
    let mut header = [0u8; 40];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-2i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&32u16.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);
    memory.write_bytes(
        bits,
        &[
            0x00, 0x00, 0xff, 0xff, // red
            0x00, 0xff, 0x00, 0xff, // green
            0xff, 0x00, 0x00, 0xff, // blue
            0xff, 0xff, 0xff, 0xff, // white
        ],
    );

    let stretch_result = table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel,
        &mut memory,
        Some(&mut framebuffer),
        thread_id,
        ORD_STRETCH_DIBITS,
        [
            screen_dc,
            1,
            0,
            2,
            2,
            0,
            0,
            2,
            2,
            bits,
            info,
            DIB_RGB_COLORS,
            SRCCOPY,
        ],
    );
    assert!(
        matches!(
            stretch_result,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(2),
                ..
            }
        ),
        "unexpected StretchDIBits result: {stretch_result:?}, hdc=0x{screen_dc:08x}, last_error={}",
        kernel.threads.get_last_error(thread_id)
    );

    let stride = framebuffer.stride();
    let offset = |x: usize, y: usize| y * stride + x * 2;
    assert_eq!(
        &framebuffer.pixels()[offset(1, 0)..offset(1, 0) + 2],
        &[0x00, 0xf8]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(2, 0)..offset(2, 0) + 2],
        &[0xe0, 0x07]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(1, 1)..offset(1, 1) + 2],
        &[0x1f, 0x00]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(2, 1)..offset(2, 1) + 2],
        &[0xff, 0xff]
    );
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(1, 0, 2, 2)]
    );

    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_SET_DIBITS_TO_DEVICE,
            [
                screen_dc,
                0,
                2,
                2,
                2,
                0,
                0,
                0,
                2,
                bits,
                info,
                DIB_RGB_COLORS
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ));
    assert_eq!(
        &framebuffer.pixels()[offset(0, 2)..offset(0, 2) + 2],
        &[0x00, 0xf8]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(1, 2)..offset(1, 2) + 2],
        &[0xe0, 0x07]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(0, 3)..offset(0, 3) + 2],
        &[0x1f, 0x00]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(1, 3)..offset(1, 3) + 2],
        &[0xff, 0xff]
    );
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(0, 2, 2, 2)]
    );

    Ok(())
}

#[test]
fn coredll_raw_direct_8bpp_dib_uses_bitmapinfo_color_table() -> Result<()> {
    const DIB_RGB_COLORS: u32 = 0;
    const SRCCOPY: u32 = 0x00cc_0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let mut framebuffer = VirtualFramebuffer::new(2, 1, PixelFormat::Rgb565)?;
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

    let info = 0x1_0000;
    let bits = 0x1_0100;
    memory.map_words(info, 10);
    memory.map_bytes(info, 48);
    memory.map_bytes(bits, 4);
    let mut header = [0u8; 48];
    header[0..4].copy_from_slice(&40u32.to_le_bytes());
    header[4..8].copy_from_slice(&2i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-1i32).to_le_bytes());
    header[12..14].copy_from_slice(&1u16.to_le_bytes());
    header[14..16].copy_from_slice(&8u16.to_le_bytes());
    header[16..20].copy_from_slice(&0u32.to_le_bytes());
    header[32..36].copy_from_slice(&2u32.to_le_bytes());
    header[40..48].copy_from_slice(&[0, 0, 255, 0, 0, 255, 0, 0]);
    memory.write_bytes(info, &header);
    memory.write_word(info, 40);
    memory.write_word(info + 32, 2);
    memory.write_bytes(bits, &[0, 1, 0, 0]);

    let stretch_result = table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel,
        &mut memory,
        Some(&mut framebuffer),
        thread_id,
        ORD_STRETCH_DIBITS,
        [
            screen_dc,
            0,
            0,
            2,
            1,
            0,
            0,
            2,
            1,
            bits,
            info,
            DIB_RGB_COLORS,
            SRCCOPY,
        ],
    );
    assert!(
        matches!(
            stretch_result,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(1),
                ..
            }
        ),
        "unexpected StretchDIBits result: {stretch_result:?}, hdc=0x{screen_dc:08x}, last_error={}",
        kernel.threads.get_last_error(thread_id)
    );

    assert_eq!(&framebuffer.pixels()[0..2], &[0x00, 0xf8]);
    assert_eq!(&framebuffer.pixels()[2..4], &[0xe0, 0x07]);
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(0, 0, 2, 1)]
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
fn coredll_raw_text_metrics_use_selected_logfont() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let logfont_ptr = 0x1_0000;
    let metrics_ptr = 0x1_0100;
    let face_ptr = 0x1_0200;
    memory.map_bytes(logfont_ptr, 92);
    memory.map_bytes(metrics_ptr, 60);
    memory.map_halfwords(face_ptr, 32);

    let mut logfont = [0u8; 92];
    logfont[0..4].copy_from_slice(&(-18i32).to_le_bytes());
    logfont[4..8].copy_from_slice(&(9i32).to_le_bytes());
    logfont[16..20].copy_from_slice(&(700i32).to_le_bytes());
    logfont[20] = 1;
    logfont[21] = 1;
    logfont[22] = 1;
    logfont[23] = 0;
    logfont[27] = 2;
    for (index, unit) in "Courier New".encode_utf16().enumerate() {
        let offset = 28 + index * 2;
        logfont[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
    }
    memory.write_bytes(logfont_ptr, &logfont);

    let font = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_FONT_INDIRECT_W,
        [logfont_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateFontIndirectW did not return a handle: {other:?}"),
    };
    assert_ne!(font, 0);

    let hdc = match table.dispatch_raw_ordinal_with_memory(
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_OBJECT,
            [hdc, font],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(_),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_TEXT_COLOR,
            [hdc, 0x0011_2233],
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
            ORD_SET_TEXT_ALIGN,
            [hdc, 6],
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
            ORD_GET_TEXT_METRICS_W,
            [hdc, metrics_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let metrics = memory.read_bytes(metrics_ptr, 60);
    let i32_at = |offset: usize| {
        i32::from_le_bytes([
            metrics[offset],
            metrics[offset + 1],
            metrics[offset + 2],
            metrics[offset + 3],
        ])
    };
    assert_eq!(i32_at(0), 18);
    assert_eq!(i32_at(20), 9);
    assert_eq!(i32_at(28), 700);
    assert_eq!(metrics[52], 1);
    assert_eq!(metrics[53], 1);
    assert_eq!(metrics[54], 1);
    assert_eq!(metrics[55], 2);
    assert_eq!(metrics[56], 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_TEXT_COLOR,
            [hdc],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x0011_2233),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_TEXT_ALIGN,
            [hdc],
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
            ORD_GET_TEXT_FACE_W,
            [hdc, 32, face_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(11),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(face_ptr, 32), "Courier New");

    Ok(())
}

#[test]
fn coredll_raw_get_text_extent_ex_point_fills_fit_dx_and_size() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let text_ptr = 0x1_0000;
    let fit_ptr = 0x1_0100;
    let dx_ptr = 0x1_0120;
    let size_ptr = 0x1_0140;
    memory.map_halfwords(text_ptr, 5);
    memory.map_words(fit_ptr, 1);
    memory.map_words(dx_ptr, 4);
    memory.map_words(size_ptr, 2);
    memory.write_wide_z(text_ptr, "abcd");

    let hdc = match table.dispatch_raw_ordinal_with_memory(
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_TEXT_EXTENT_EX_POINT_W,
            [hdc, text_ptr, 4, 20, fit_ptr, dx_ptr, size_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(fit_ptr)?, 2);
    assert_eq!(memory.read_u32(dx_ptr)?, 8);
    assert_eq!(memory.read_u32(dx_ptr + 4)?, 16);
    assert_eq!(memory.read_u32(dx_ptr + 8)?, 24);
    assert_eq!(memory.read_u32(dx_ptr + 12)?, 32);
    assert_eq!(memory.read_i32(size_ptr)?, 32);
    assert_eq!(memory.read_i32(size_ptr + 4)?, 16);

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
            value: CoredllValue::Handle(handle),
            ..
        } if handle != 0
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
fn coredll_raw_window_from_point_hits_visible_thread_windows() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let parent = kernel.create_window_ex_w_with_rect(
        thread_id,
        "PARENT",
        "parent",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(10, 20, 300, 200),
    );
    let child = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CHILD",
        "child",
        Some(parent),
        2,
        WS_VISIBLE | WS_CHILD,
        0,
        Rect::from_origin_size(5, 6, 70, 80),
    );
    let sibling = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CHILD",
        "sibling",
        Some(parent),
        3,
        WS_CHILD,
        0,
        Rect::from_origin_size(100, 60, 40, 40),
    );
    assert!(kernel.gwe.is_window_visible(parent));
    assert!(kernel.gwe.is_window_visible(child));
    assert!(!kernel.gwe.is_window_visible(sibling));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WINDOW_FROM_POINT,
            [22, 34],
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
            ORD_CHILD_WINDOW_FROM_POINT,
            [parent, 6, 7],
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
            ORD_CHILD_WINDOW_FROM_POINT,
            [parent, 110, 70],
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
            ORD_WINDOW_FROM_POINT,
            [900, 900],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_is_window_visible_observes_hidden_ancestors() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let parent = kernel.create_window_ex_w_with_rect(
        thread_id,
        "PARENT",
        "parent",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(10, 20, 300, 200),
    );
    let child = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CHILD",
        "child",
        Some(parent),
        2,
        WS_VISIBLE | WS_CHILD,
        0,
        Rect::from_origin_size(5, 6, 70, 80),
    );

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
            ORD_SHOW_WINDOW,
            [parent, 0],
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
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WINDOW_FROM_POINT,
            [22, 34],
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
            ORD_CHILD_WINDOW_FROM_POINT,
            [parent, 6, 7],
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
            ORD_SHOW_WINDOW,
            [parent, 1],
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
            ORD_WINDOW_FROM_POINT,
            [22, 34],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == child
    ));

    Ok(())
}

#[test]
fn coredll_raw_update_window_observes_hidden_ancestors() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let rect_ptr = 0x2_1000;
    let msg_ptr = 0x2_1100;
    memory.map_words(rect_ptr, 4);
    memory.map_words(msg_ptr, 7);

    let parent = kernel.create_window_ex_w_with_rect(
        thread_id,
        "PARENT",
        "parent",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(10, 20, 300, 200),
    );
    let child = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CHILD",
        "child",
        Some(parent),
        2,
        WS_VISIBLE | WS_CHILD,
        0,
        Rect::from_origin_size(5, 6, 70, 80),
    );
    assert!(kernel.gwe.validate_window(child));
    assert!(kernel.gwe.invalidate_window(child, None, true));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [parent, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window_visible(child));
    assert!(kernel.gwe.update_rect(child).is_some());

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
    assert!(
        kernel.gwe.update_rect(child).is_some(),
        "UpdateWindow must not send WM_PAINT to an effectively hidden child"
    );
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, child, WM_PAINT, WM_PAINT, 0],
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
            [parent, 5],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(kernel.gwe.is_window_visible(child));
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
    assert!(kernel.gwe.update_rect(child).is_none());

    Ok(())
}

#[test]
fn coredll_raw_map_dialog_rect_uses_dialog_base_units() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let rect_ptr = 0x1_8000;
    memory.map_words(rect_ptr, 4);

    let base_units = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DIALOG_BASE_UNITS,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } => value,
        other => panic!("GetDialogBaseUnits did not return units: {other:?}"),
    };
    let base_x = (base_units & 0xffff) as i32;
    let base_y = ((base_units >> 16) & 0xffff) as i32;
    assert!(base_x > 0);
    assert!(base_y > 0);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "DIALOG",
        "dialog",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 80),
    );
    memory.write_word(rect_ptr, 4);
    memory.write_word(rect_ptr + 4, 8);
    memory.write_word(rect_ptr + 8, 14);
    memory.write_word(rect_ptr + 12, 19);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MAP_DIALOG_RECT,
            [hwnd, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 4 * base_x / 4);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 8 * base_y / 8);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 14 * base_x / 4);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 19 * base_y / 8);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    memory.write_word(rect_ptr, 10);
    memory.write_word(rect_ptr + 4, 11);
    memory.write_word(rect_ptr + 8, 12);
    memory.write_word(rect_ptr + 12, 13);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MAP_DIALOG_RECT,
            [0x0bad_cafe, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 10);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 11);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 12);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 13);
    Ok(())
}

#[test]
fn coredll_raw_get_next_dialog_items_follow_tab_and_group_order() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let dialog = kernel.create_window_ex_w_with_rect(
        thread_id,
        "DIALOG",
        "dialog",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 200, 120),
    );
    let first = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "first",
        Some(dialog),
        10,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | WS_GROUP,
        0,
        Rect::from_origin_size(0, 0, 20, 20),
    );
    let second = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "second",
        Some(dialog),
        11,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP,
        0,
        Rect::from_origin_size(20, 0, 20, 20),
    );
    let disabled = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "disabled",
        Some(dialog),
        12,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | WS_DISABLED,
        0,
        Rect::from_origin_size(40, 0, 20, 20),
    );
    let hidden = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "hidden",
        Some(dialog),
        13,
        WS_CHILD | WS_TABSTOP,
        0,
        Rect::from_origin_size(60, 0, 20, 20),
    );
    let third = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "third",
        Some(dialog),
        14,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | WS_GROUP,
        0,
        Rect::from_origin_size(80, 0, 20, 20),
    );
    let fourth = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "fourth",
        Some(dialog),
        15,
        WS_VISIBLE | WS_CHILD,
        0,
        Rect::from_origin_size(100, 0, 20, 20),
    );

    assert!(kernel.gwe.is_window(disabled));
    assert!(kernel.gwe.is_window(hidden));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_TAB_ITEM,
            [dialog, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == first
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_TAB_ITEM,
            [dialog, first, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == second
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_TAB_ITEM,
            [dialog, second, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == third
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_TAB_ITEM,
            [dialog, first, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == third
    ));

    assert_eq!(kernel.enable_window(dialog, false), Some(true));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_TAB_ITEM,
            [dialog, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.enable_window(dialog, true), Some(false));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_GROUP_ITEM,
            [dialog, first, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == second
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_GROUP_ITEM,
            [dialog, second, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == first
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_NEXT_DLG_GROUP_ITEM,
            [dialog, third, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == fourth
    ));

    Ok(())
}

#[test]
fn coredll_raw_is_dialog_message_dispatches_dialog_owned_messages() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let msg_ptr = 0x3520;
    memory.map_words(msg_ptr, 7);

    let dialog = kernel.create_window_ex_w_with_rect(
        thread_id,
        "DIALOG",
        "dialog",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 200, 120),
    );
    write_raw_message(&mut memory, msg_ptr, dialog, WM_CLOSE, 0, 0)?;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_DIALOG_MESSAGE_W,
            [dialog, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window(dialog));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_create_dialog_does_not_post_init_dialog() -> Result<()> {
    const WM_INITDIALOG_RAW: u32 = 0x0110;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let template = 0x3600;
    let dlgproc = 0x1234_5678;

    memory.write_word(template, WS_POPUP | WS_VISIBLE);
    memory.write_word(template + 4, 0);
    memory.write_halfword(template + 8, 0);
    memory.write_halfword(template + 10, 4);
    memory.write_halfword(template + 12, 8);
    memory.write_halfword(template + 14, 120);
    memory.write_halfword(template + 16, 64);
    memory.write_halfword(template + 18, 0);
    memory.write_halfword(template + 20, 0);
    memory.write_halfword(template + 22, 0);

    let dialog = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_DIALOG_INDIRECT_PARAM_W,
        [0, template, 0, dlgproc, 0xfeed_beef],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateDialogIndirectParamW did not return an HWND: {other:?}"),
    };
    assert_ne!(dialog, 0);
    assert_eq!(
        kernel
            .gwe
            .get_window_long(dialog, wince_emulation_v3::ce::gwe::GWL_WNDPROC),
        Some(dlgproc)
    );

    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(dialog),
                WM_INITDIALOG_RAW,
                WM_INITDIALOG_RAW,
                PeekFlags::NO_REMOVE,
            )
            .is_none(),
        "WM_INITDIALOG must be delivered synchronously by dialog creation, not left queued"
    );

    Ok(())
}

#[test]
fn coredll_raw_is_dialog_message_ignores_unrelated_windows() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let msg_ptr = 0x3540;
    memory.map_words(msg_ptr, 7);

    let dialog = kernel.create_window_ex_w(thread_id, "DIALOG", "dialog", None, 1, WS_VISIBLE, 0);
    let other = kernel.create_window_ex_w(thread_id, "OTHER", "other", None, 2, WS_VISIBLE, 0);
    write_raw_message(&mut memory, msg_ptr, other, WM_CLOSE, 0, 0)?;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_DIALOG_MESSAGE_W,
            [dialog, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(kernel.gwe.is_window(dialog));
    assert!(kernel.gwe.is_window(other));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_is_dialog_message_tabs_to_next_dialog_item() -> Result<()> {
    const VK_TAB: u32 = 0x09;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let msg_ptr = 0x3560;
    memory.map_words(msg_ptr, 7);

    let dialog = kernel.create_window_ex_w_with_rect(
        thread_id,
        "DIALOG",
        "dialog",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 200, 120),
    );
    let first = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "first",
        Some(dialog),
        10,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP,
        0,
        Rect::from_origin_size(0, 0, 20, 20),
    );
    let second = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "second",
        Some(dialog),
        11,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP,
        0,
        Rect::from_origin_size(20, 0, 20, 20),
    );
    let _ = kernel.set_focus(Some(first));
    write_raw_message(&mut memory, msg_ptr, first, WM_KEYDOWN, VK_TAB, 0)?;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_DIALOG_MESSAGE_W,
            [dialog, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.gwe.get_focus(), Some(second));
    assert!(kernel.gwe.is_window(dialog));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(kernel.post_message_w_for_thread(thread_id, first, WM_KEYDOWN, VK_LSHIFT, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEY_STATE,
            [VK_SHIFT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(state),
            ..
        } if state & 0x8000 != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ASYNC_SHIFT_FLAGS,
            [VK_SHIFT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(flags),
            ..
        } if flags & KEY_SHIFT_ANY_SHIFT_FLAG != 0
            && flags & KEY_STATE_DOWN_FLAG != 0
            && flags & KEY_STATE_GET_ASYNC_DOWN_FLAG != 0
    ));
    assert!(kernel.post_message_w_for_thread(thread_id, first, WM_KEYDOWN, VK_LSHIFT, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ASYNC_KEY_STATE,
            [VK_SHIFT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(state),
            ..
        } if state & 0x8000 != 0 && state & 0x0001 != 0
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ASYNC_KEY_STATE,
            [VK_SHIFT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(state),
            ..
        } if state & 0x8000 != 0 && state & 0x0001 == 0
    ));
    write_raw_message(&mut memory, msg_ptr, second, WM_KEYDOWN, VK_TAB, 0)?;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_DIALOG_MESSAGE_W,
            [dialog, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.gwe.get_focus(), Some(first));

    assert!(kernel.post_message_w_for_thread(thread_id, first, WM_KEYUP, VK_LSHIFT, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEY_STATE,
            [VK_SHIFT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(state),
            ..
        } if state & 0x8000 == 0
    ));

    Ok(())
}

#[test]
fn coredll_raw_post_keybd_message_posts_key_and_characters() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let msg_ptr = 0x35a0;
    let chars_ptr = 0x35e0;
    memory.map_words(msg_ptr, 7);
    memory.map_words(chars_ptr, 2);
    memory.write_word(chars_ptr, u32::from('A'));
    memory.write_word(chars_ptr + 4, u32::from('b'));

    let hwnd = kernel.create_window_ex_w(thread_id, "KEYTARGET", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POST_KEYBD_MESSAGE,
            [hwnd, u32::from('A'), KEY_STATE_DOWN_FLAG, 2, 0, chars_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_KEYDOWN,
        u32::from('A'),
        1,
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
            value: CoredllValue::U32(MSGSRC_HARDWARE_KEYBOARD),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEY_STATE,
            [u32::from('A')],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(state),
            ..
        } if state & 0x8000 != 0
    ));
    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_CHAR,
        u32::from('A'),
        1,
    );
    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_CHAR,
        u32::from('b'),
        1,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POST_KEYBD_MESSAGE,
            [hwnd, u32::from('A'), KEY_STATE_PREV_DOWN_FLAG, 0, 0, 0,],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_KEYUP,
        u32::from('A'),
        0xc000_0001,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEY_STATE,
            [u32::from('A')],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(state),
            ..
        } if state & 0x8000 == 0
    ));

    Ok(())
}

#[test]
fn coredll_raw_keybd_event_targets_focus_window() -> Result<()> {
    const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
    const KEYEVENTF_KEYUP: u32 = 0x0002;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let msg_ptr = 0x3620;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "KEYTARGET", "", None, 0, 0, 0);
    let _ = kernel.set_focus(Some(hwnd));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KEYBD_EVENT,
            [0x25, 0x4b, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_KEYDOWN,
        0x25,
        1 | (0x4b << 16),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KEYBD_EVENT,
            [0x25, 0x4b, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_KEYUP,
        0x25,
        0xc14b_0001,
    );

    Ok(())
}

#[test]
fn coredll_raw_keyboard_target_routes_hardware_keyboard_input() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let msg_ptr = 0x3680;
    memory.map_words(msg_ptr, 7);

    let focus_hwnd = kernel.create_window_ex_w(thread_id, "KEYFOCUS", "", None, 0, 0, 0);
    let target_hwnd = kernel.create_window_ex_w(thread_id, "KEYTARGET", "", None, 0, 0, 0);
    let _ = kernel.set_focus(Some(focus_hwnd));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_KEYBOARD_TARGET,
            [target_hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEYBOARD_TARGET,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == target_hwnd
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FOREGROUND_KEYBOARD_TARGET,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == target_hwnd
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KEYBD_EVENT,
            [u32::from('K'), 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        target_hwnd,
        WM_KEYDOWN,
        u32::from('K'),
        1,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_KEYBOARD_TARGET,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == target_hwnd
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEYBOARD_TARGET,
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
            ORD_KEYBD_EVENT,
            [u32::from('F'), 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_next_filtered_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        focus_hwnd,
        WM_KEYDOWN,
        u32::from('F'),
        1,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_KEYBOARD_TARGET,
            [0x7fff_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
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
        [
            0, class_ptr, title_ptr, WS_CHILD, 5, 6, 70, 80, parent, 2, 0, 0,
        ],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [parent, 1],
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, child, WM_PAINT, WM_PAINT, 0],
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
            [parent, 0],
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
            ORD_INVALIDATE_RECT,
            [child, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

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
            [parent, 1],
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
            ORD_SHOW_WINDOW,
            [parent, 0],
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
        [child, 77, 1, 0x0012_3450],
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
    assert_eq!(memory.read_u32(timer_msg_ptr + 12)?, 0x0012_3450);
    let other_timer_hwnd = kernel.gwe.create_window(thread_id, "TimerPeer", "peer");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_KILL_TIMER,
            [other_timer_hwnd, timer_id],
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
            [parent, 1],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_ASSOCIATED_MENU,
            [0xffff_ffff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
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
        WM_ACTIVATE,
        WA_ACTIVE,
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
fn coredll_raw_windowposchanged_carries_guest_windowpos_payload() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 3;
    let msg_ptr = 0x7100;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "WINPOS", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_POS,
            [hwnd, 0, 12, 34, 120, 90, 0x0004 | 0x0010],
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
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_WINDOWPOSCHANGED);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0);
    let window_pos_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(window_pos_ptr, 0);
    assert_eq!(memory.read_u32(window_pos_ptr)?, hwnd);
    assert_eq!(memory.read_u32(window_pos_ptr + 4)?, 0);
    assert_eq!(memory.read_i32(window_pos_ptr + 8)?, 12);
    assert_eq!(memory.read_i32(window_pos_ptr + 12)?, 34);
    assert_eq!(memory.read_i32(window_pos_ptr + 16)?, 120);
    assert_eq!(memory.read_i32(window_pos_ptr + 20)?, 90);
    assert_eq!(memory.read_u32(window_pos_ptr + 24)?, 0x0004 | 0x0010);
    assert!(
        kernel
            .memory
            .heap_size(PROCESS_HEAP_HANDLE, 0, window_pos_ptr)
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
            .heap_size(PROCESS_HEAP_HANDLE, 0, window_pos_ptr)
            .is_none()
    );

    Ok(())
}

#[test]
fn coredll_raw_set_window_pos_show_hide_queues_windowpos_without_rect_change() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 3;
    let msg_ptr = 0x7200;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "SHOWPOS", "", None, 0, 0, 0);
    let show_flags = SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_POS,
            [hwnd, 0, 0, 0, 0, 0, show_flags],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_WINDOWPOSCHANGED);
    let show_pos_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(show_pos_ptr, 0);
    assert_eq!(memory.read_u32(show_pos_ptr)?, hwnd);
    assert_eq!(memory.read_u32(show_pos_ptr + 24)?, show_flags);
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

    let hide_flags = SWP_HIDEWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_POS,
            [hwnd, 0, 0, 0, 0, 0, hide_flags],
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
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_WINDOWPOSCHANGED);
    let hide_pos_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(hide_pos_ptr, 0);
    assert_eq!(memory.read_u32(hide_pos_ptr)?, hwnd);
    assert_eq!(memory.read_u32(hide_pos_ptr + 24)?, hide_flags);

    Ok(())
}

#[test]
fn coredll_raw_hidden_move_defers_size_move_until_show_window() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 3;
    let msg_ptr = 0x7280;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "HIDDENMOVE", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MOVE_WINDOW,
            [hwnd, 12, 34, 120, 90, 0],
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
        WM_WINDOWPOSCHANGED,
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_MOVE, WM_SIZE, 0],
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
            [hwnd, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
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
        0x0022_000c,
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
        0x005a_0078,
    );

    Ok(())
}

#[test]
fn coredll_raw_show_window_queues_direct_visibility_windowpos_under_hidden_parent() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 3;
    let msg_ptr = 0x7300;
    memory.map_words(msg_ptr, 7);

    let parent = kernel.create_window_ex_w(thread_id, "HIDDENPARENT", "", None, 0, 0, 0);
    let child =
        kernel.create_window_ex_w(thread_id, "VISIBLECHILD", "", Some(parent), 0, WS_CHILD, 0);
    assert!(!kernel.gwe.is_window_visible(child));

    assert!(!kernel.gwe.show_window(child, true));
    assert!(kernel.gwe.window(child).expect("child").visible);
    assert!(!kernel.gwe.is_window_visible(child));

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
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_SHOWWINDOW,
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, child);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_WINDOWPOSCHANGED);
    let window_pos_ptr = memory.read_u32(msg_ptr + 12)?;
    assert_ne!(window_pos_ptr, 0);
    assert_eq!(memory.read_u32(window_pos_ptr)?, child);
    assert_eq!(
        memory.read_u32(window_pos_ptr + 24)?,
        SWP_HIDEWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
    );

    Ok(())
}

#[test]
fn coredll_raw_focus_and_activation_queue_ce_messages() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 3;
    let msg_ptr = 0x7600;
    memory.map_words(msg_ptr, 7);

    let first = kernel.create_window_ex_w(thread_id, "FIRST", "", None, 0, 0, 0);
    let second = kernel.create_window_ex_w(thread_id, "SECOND", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FOCUS,
            [first],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        first,
        WM_ACTIVATE,
        WA_ACTIVE,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        first,
        WM_SETFOCUS,
        0,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FOCUS,
            [second],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == first
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        first,
        WM_KILLFOCUS,
        second,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        first,
        WM_ACTIVATE,
        WA_INACTIVE,
        second,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        second,
        WM_ACTIVATE,
        WA_ACTIVE,
        first,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        second,
        WM_SETFOCUS,
        first,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FOREGROUND_WINDOW,
            [first],
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
        second,
        WM_ACTIVATE,
        WA_INACTIVE,
        first,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        first,
        WM_ACTIVATE,
        WA_ACTIVE,
        second,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        second,
        WM_KILLFOCUS,
        first,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        first,
        WM_SETFOCUS,
        second,
        0,
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
fn coredll_raw_get_message_prioritizes_paint_over_generated_timer() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 4;
    let msg_ptr = 0x8200;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel
        .gwe
        .create_window_ex(thread_id, "PAINT_TIMER", "", None, 0, WS_VISIBLE, 0);
    let timer_id = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_SET_TIMER,
        [hwnd, 91, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(id),
            ..
        } => id,
        other => panic!("SetTimer did not return an id: {other:?}"),
    };
    assert_eq!(timer_id, 91);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_PAINT);

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
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_TIMER);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, timer_id);

    Ok(())
}

#[test]
fn coredll_raw_get_message_delivers_invalidated_visible_child_paint() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 4;
    let msg_ptr = 0x8200;
    memory.map_words(msg_ptr, 7);

    let parent = kernel
        .gwe
        .create_window_ex(thread_id, "PARENT", "", None, 0, WS_VISIBLE, 0);
    let child = kernel.gwe.create_window_ex(
        thread_id,
        "CHILD",
        "",
        Some(parent),
        0,
        WS_CHILD | WS_VISIBLE,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VALIDATE_RECT,
            [parent, 0],
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
            ORD_VALIDATE_RECT,
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
            ORD_INVALIDATE_RECT,
            [child, 0, 1],
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
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, child);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_PAINT);

    Ok(())
}

#[test]
fn coredll_raw_hidden_invalidated_child_does_not_signal_paint_until_visible() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 4;
    let msg_ptr = 0x8400;
    memory.map_words(msg_ptr, 7);

    let parent = kernel
        .gwe
        .create_window_ex(thread_id, "HIDDEN_PARENT_PAINT", "", None, 0, 0, 0);
    let child = kernel.gwe.create_window_ex(
        thread_id,
        "HIDDEN_CHILD_PAINT",
        "",
        Some(parent),
        0,
        WS_CHILD | WS_VISIBLE,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_QUEUE_STATUS,
            [QS_PAINT],
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
            ORD_INVALIDATE_RECT,
            [child, 0, 1],
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
            [QS_PAINT],
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
            [msg_ptr, 0, WM_PAINT, WM_PAINT, 0],
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
            [parent, 1],
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
            ORD_GET_QUEUE_STATUS,
            [QS_PAINT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(v),
            ..
        } if v == ((QS_PAINT << 16) | QS_PAINT)
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

    assert!(
        kernel
            .gwe
            .queue_sent_message_for_window(hwnd, Message::new(hwnd, WM_USER + 16, 0x16, 0x17, 0))
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_QUEUE_STATUS,
            [QS_SENDMESSAGE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(v),
            ..
        } if v == ((QS_SENDMESSAGE << 16) | QS_SENDMESSAGE)
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        hwnd,
        WM_USER + 16,
        0x16,
        0x17,
    );
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_SOURCE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(MSGSRC_SOFTWARE_SEND),
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
fn coredll_raw_send_notify_message_is_async_across_threads() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let caller_thread = 44;
    let receiver_thread = 45;
    let msg_ptr = 0xa000;
    memory.map_words(msg_ptr, 7);

    let same_thread_hwnd =
        kernel.create_window_ex_w(caller_thread, "NOTIFY_SAME", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SEND_NOTIFY_MESSAGE_W,
            [same_thread_hwnd, WM_CLOSE, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window(same_thread_hwnd));
    assert!(
        kernel
            .gwe
            .window(same_thread_hwnd)
            .is_some_and(|window| window.destroy_message_sent)
    );
    assert!(!kernel.gwe.in_send_message(caller_thread));

    let cross_thread_hwnd =
        kernel.create_window_ex_w(receiver_thread, "NOTIFY_CROSS", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SEND_NOTIFY_MESSAGE_W,
            [cross_thread_hwnd, WM_CLOSE, 0x46, 0x47],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.gwe.is_window(cross_thread_hwnd));
    assert!(!kernel.gwe.in_send_message(caller_thread));
    assert!(!kernel.gwe.in_send_message(receiver_thread));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_GET_QUEUE_STATUS,
            [QS_SENDMESSAGE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(v),
            ..
        } if v == ((QS_SENDMESSAGE << 16) | QS_SENDMESSAGE)
    ));
    let sent = kernel.gwe.sent_message(1).expect("queued notify send");
    assert_eq!(sent.sender_thread_id, None);
    assert_ne!(sent.flags & SMF_SENDER_NO_WAIT, 0);
    assert_ne!(sent.flags & SMF_NOTIFY_MESSAGE, 0);

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        cross_thread_hwnd,
        WM_CLOSE,
        0x46,
        0x47,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_IN_SEND_MESSAGE,
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
            receiver_thread,
            ORD_GET_MESSAGE_SOURCE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(MSGSRC_SOFTWARE_SEND),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(!kernel.gwe.in_send_message(receiver_thread));
    assert!(!kernel.gwe.is_window(cross_thread_hwnd));
    assert!(
        kernel
            .gwe
            .window(cross_thread_hwnd)
            .is_some_and(|window| window.destroy_message_sent)
    );

    Ok(())
}

#[test]
fn coredll_raw_dispatch_completes_queued_cross_thread_send() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 46;
    let receiver_thread = 47;
    let msg_ptr = 0xb000;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(receiver_thread, "SYNC_SEND", "", None, 0, 0, 0);
    let send_id = kernel
        .begin_cross_thread_send_message_w(
            sender_thread,
            hwnd,
            WM_ERASEBKGND,
            0x1234,
            0x5678,
            Some(500),
        )
        .expect("queued cross-thread send");
    assert!(!kernel.gwe.in_send_message(sender_thread));
    assert!(!kernel.gwe.in_send_message(receiver_thread));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_GET_QUEUE_STATUS,
            [QS_SENDMESSAGE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(v),
            ..
        } if v == ((QS_SENDMESSAGE << 16) | QS_SENDMESSAGE)
    ));

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        hwnd,
        WM_ERASEBKGND,
        0x1234,
        0x5678,
    );
    assert!(kernel.gwe.in_send_message(receiver_thread));
    assert_eq!(
        kernel.gwe.active_sent_message_id(receiver_thread),
        Some(send_id)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert!(!kernel.gwe.in_send_message(receiver_thread));
    assert_eq!(kernel.take_completed_send_message_result(send_id), Some(1));

    Ok(())
}

#[test]
fn coredll_raw_send_message_cross_thread_queues_transaction() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 54;
    let receiver_thread = 55;
    let msg_ptr = 0xb300;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(receiver_thread, "RAW_SYNC_SEND", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_SEND_MESSAGE_W,
            [hwnd, WM_ERASEBKGND, 0x54, 0x55],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let sent = kernel.gwe.sent_message(1).expect("queued raw send");
    assert_eq!(sent.sender_thread_id, Some(sender_thread));
    assert_eq!(sent.receiver_thread_id, receiver_thread);
    assert_eq!(sent.flags, 0);
    assert!(!kernel.gwe.in_send_message(sender_thread));
    assert!(!kernel.gwe.in_send_message(receiver_thread));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_GET_QUEUE_STATUS,
            [QS_SENDMESSAGE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(v),
            ..
        } if v == ((QS_SENDMESSAGE << 16) | QS_SENDMESSAGE)
    ));

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        hwnd,
        WM_ERASEBKGND,
        0x54,
        0x55,
    );
    assert!(kernel.gwe.in_send_message(receiver_thread));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert!(!kernel.gwe.in_send_message(receiver_thread));
    assert_eq!(kernel.take_completed_send_message_result(1), Some(1));

    let direct_hwnd =
        kernel.create_window_ex_w(receiver_thread, "RAW_DEF_WINDOW_PROC", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_DEF_WINDOW_PROC_W,
            [direct_hwnd, WM_ERASEBKGND, 0x56, 0x57],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert!(kernel.gwe.sent_message(2).is_none());

    Ok(())
}

#[test]
fn coredll_raw_send_dlg_item_message_uses_sendmessage_queue() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 56;
    let receiver_thread = 57;
    let msg_ptr = 0xb380;
    memory.map_words(msg_ptr, 7);

    let dialog = kernel.create_window_ex_w(sender_thread, "DLG_SEND_PARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w_with_rect(
        receiver_thread,
        "DLG_SEND_CHILD",
        "",
        Some(dialog),
        202,
        WS_CHILD | WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 20, 20),
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_SEND_DLG_ITEM_MESSAGE_W,
            [dialog, 202, WM_ERASEBKGND, 0x56, 0x57],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let sent = kernel.gwe.sent_message(1).expect("queued dlg item send");
    assert_eq!(sent.sender_thread_id, Some(sender_thread));
    assert_eq!(sent.receiver_thread_id, receiver_thread);
    assert_eq!(sent.message.hwnd, child);
    assert_eq!(sent.message.msg, WM_ERASEBKGND);

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        child,
        WM_ERASEBKGND,
        0x56,
        0x57,
    );
    assert!(kernel.gwe.in_send_message(receiver_thread));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.take_completed_send_message_result(1), Some(1));

    Ok(())
}

#[test]
fn coredll_raw_get_message_expires_timed_out_cross_thread_send() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 48;
    let receiver_thread = 49;
    let msg_ptr = 0xb100;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(receiver_thread, "SYNC_SEND_TIMEOUT", "", None, 0, 0, 0);
    let send_id = kernel
        .begin_cross_thread_send_message_w(sender_thread, hwnd, WM_USER + 57, 0x57, 0x58, Some(0))
        .expect("queued cross-thread send");
    assert!(kernel.gwe.sent_message(send_id).is_some());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_GET_MESSAGE_W,
            [msg_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(kernel.take_completed_send_message_result(send_id), Some(0));
    assert_eq!(kernel.gwe.stats().send_transaction_timeout_count, 1);

    Ok(())
}

#[test]
fn coredll_raw_send_message_timeout_zero_cross_thread_expires_transaction() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 50;
    let receiver_thread = 51;
    let result_ptr = 0xb200;
    memory.map_words(result_ptr, 1);
    memory.write_u32(result_ptr, 0xfeed_cafe)?;

    let hwnd =
        kernel.create_window_ex_w(receiver_thread, "SYNC_SEND_TIMEOUT_RAW", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_SEND_MESSAGE_TIMEOUT,
            [hwnd, WM_USER + 58, 0x58, 0x59, 0, 0, result_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(result_ptr)?, 0xfeed_cafe);
    assert!(kernel.gwe.sent_message(1).is_none());
    assert_eq!(kernel.gwe.stats().send_transaction_timeout_count, 1);
    assert!(kernel.gwe.get_message(receiver_thread).is_none());

    Ok(())
}

#[test]
fn coredll_raw_send_message_timeout_nonzero_cross_thread_queues_transaction() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 52;
    let receiver_thread = 53;
    let result_ptr = 0xb240;
    let msg_ptr = 0xb280;
    memory.map_words(result_ptr, 1);
    memory.map_words(msg_ptr, 7);
    memory.write_u32(result_ptr, 0xfeed_cafe)?;

    let hwnd = kernel.create_window_ex_w(
        receiver_thread,
        "SYNC_SEND_TIMEOUT_RAW_WAIT",
        "",
        None,
        0,
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_SEND_MESSAGE_TIMEOUT,
            [hwnd, WM_ERASEBKGND, 0x5a, 0x5b, 0, 250, result_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(result_ptr)?, 0xfeed_cafe);
    let sent = kernel.gwe.sent_message(1).expect("queued timeout send");
    assert_eq!(sent.sender_thread_id, Some(sender_thread));
    assert_eq!(sent.receiver_thread_id, receiver_thread);
    assert_ne!(sent.flags & SMF_TIMEOUT, 0);
    assert_eq!(sent.timeout_ms, Some(250));

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        hwnd,
        WM_ERASEBKGND,
        0x5a,
        0x5b,
    );
    assert!(kernel.gwe.in_send_message(receiver_thread));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_DISPATCH_MESSAGE_W,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.take_completed_send_message_result(1), Some(1));

    Ok(())
}

#[test]
fn coredll_raw_send_message_records_nc_destroy_lifecycle() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 46;

    assert_eq!(WM_NCDESTROY, 0x7fff);
    let hwnd = kernel.create_window_ex_w(thread_id, "NC_DESTROY", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [hwnd, WM_DESTROY, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(kernel.gwe.is_window(hwnd));
    assert!(
        kernel
            .gwe
            .window(hwnd)
            .is_some_and(|window| window.destroy_message_sent && !window.nc_destroy_message_sent)
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [hwnd, WM_NCDESTROY, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(kernel.gwe.is_window(hwnd));
    assert!(
        kernel
            .gwe
            .window(hwnd)
            .is_some_and(|window| window.destroy_message_sent && window.nc_destroy_message_sent)
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_WINDOW,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window(hwnd));
    assert!(
        kernel
            .gwe
            .window(hwnd)
            .is_some_and(|window| window.destroy_message_sent && window.nc_destroy_message_sent)
    );

    Ok(())
}

#[test]
fn coredll_raw_get_window_thread_process_id_reports_owner_ids() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 31;
    let process_id = 0x44;
    let process_id_ptr = 0x9c00;
    memory.map_words(process_id_ptr, 1);
    kernel.set_current_process_id(process_id);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "OWNER_IDS",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 80),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW_THREAD_PROCESS_ID,
            [hwnd, process_id_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(owner_thread),
            ..
        } if owner_thread == thread_id
    ));
    assert_eq!(memory.read_u32(process_id_ptr)?, process_id);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW_THREAD_PROCESS_ID,
            [hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(owner_thread),
            ..
        } if owner_thread == thread_id
    ));

    assert!(kernel.gwe.destroy_window(hwnd, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW_THREAD_PROCESS_ID,
            [hwnd, process_id_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_is_child_checks_descendant_relationships() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 32;

    let parent = kernel.create_window_ex_w(thread_id, "PARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w(thread_id, "CHILD", "", Some(parent), 1, 0, 0);
    let grandchild = kernel.create_window_ex_w(thread_id, "GRANDCHILD", "", Some(child), 2, 0, 0);
    let sibling = kernel.create_window_ex_w(thread_id, "SIBLING", "", Some(parent), 3, 0, 0);

    for (candidate, expected) in [
        (child, true),
        (grandchild, true),
        (parent, false),
        (sibling, true),
    ] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_IS_CHILD,
                [parent, candidate],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(value),
                ..
            } if value == expected
        ));
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_CHILD,
            [child, sibling],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert!(kernel.gwe.destroy_window(child, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_CHILD,
            [parent, grandchild],
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
            ORD_IS_CHILD,
            [0xffff_ffff, sibling],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_create_window_distinguishes_owner_from_child_parent() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 32;
    let owner_class_ptr = 0x7400;
    let owned_class_ptr = 0x7440;
    let child_class_ptr = 0x7480;
    let title_ptr = 0x74c0;
    let rect_ptr = 0x7500;
    memory.write_wide_z(owner_class_ptr, "OWNER_TOP");
    memory.write_wide_z(owned_class_ptr, "OWNED_TOP");
    memory.write_wide_z(child_class_ptr, "REAL_CHILD");
    memory.write_wide_z(title_ptr, "");
    memory.map_words(rect_ptr, 4);

    let owner = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [
            0,
            owner_class_ptr,
            title_ptr,
            0,
            30,
            40,
            200,
            120,
            0,
            0,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create owner hwnd: {other:?}"),
    };
    let owned = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [
            0,
            owned_class_ptr,
            title_ptr,
            WS_POPUP,
            5,
            6,
            70,
            80,
            owner,
            0,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create owned hwnd: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [owned, GW_OWNER],
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
            ORD_GET_PARENT,
            [owned],
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
            ORD_GET_WINDOW,
            [owner, GW_CHILD],
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
            ORD_GET_WINDOW_RECT,
            [owned, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr).unwrap(), 5);
    assert_eq!(memory.read_i32(rect_ptr + 4).unwrap(), 6);

    let child = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [
            0,
            child_class_ptr,
            title_ptr,
            WS_CHILD,
            5,
            6,
            70,
            80,
            owner,
            0,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create child hwnd: {other:?}"),
    };
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
        } if hwnd == owner
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
            ORD_GET_WINDOW,
            [owner, GW_CHILD],
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
            ORD_GET_WINDOW_RECT,
            [child, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr).unwrap(), 35);
    assert_eq!(memory.read_i32(rect_ptr + 4).unwrap(), 46);

    Ok(())
}

#[test]
fn coredll_raw_window_menu_state_preserves_child_control_ids() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 33;
    let frame_class_ptr = 0x7600;
    let child_class_ptr = 0x7640;
    let title_ptr = 0x7680;
    memory.write_wide_z(frame_class_ptr, "MENU_FRAME");
    memory.write_wide_z(child_class_ptr, "MENU_CHILD");
    memory.write_wide_z(title_ptr, "");
    let menu = kernel
        .resources
        .create_menu(0, ResourceId::Integer(9001), None);
    let replacement_menu = kernel
        .resources
        .create_menu(0, ResourceId::Integer(9002), None);

    let frame = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [
            0,
            frame_class_ptr,
            title_ptr,
            0,
            10,
            20,
            220,
            140,
            0,
            menu,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create menu frame: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MENU,
            [frame],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == menu
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DRAW_MENU_BAR,
            [frame],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let child_id = 0x1201;
    let child = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [
            0,
            child_class_ptr,
            title_ptr,
            WS_CHILD,
            3,
            4,
            70,
            30,
            frame,
            child_id,
            0,
            0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create menu child: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DLG_CTRL_ID,
            [child],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(id),
            ..
        } if id == child_id
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MENU,
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
            ORD_SET_MENU,
            [frame, replacement_menu],
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
            ORD_GET_MENU,
            [frame],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == replacement_menu
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_ASSOCIATED_MENU,
            [frame, replacement_menu],
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
            ORD_GET_ASSOCIATED_MENU,
            [frame],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == replacement_menu
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_MENU,
            [frame, 0],
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
            ORD_GET_MENU,
            [frame],
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
            ORD_GET_ASSOCIATED_MENU,
            [frame],
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
            ORD_DRAW_MENU_BAR,
            [0xffff_ffff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_menu_items_round_trip_through_ce_menuiteminfo() -> Result<()> {
    const MF_BYPOSITION: u32 = 0x0000_0400;
    const MF_POPUP: u32 = 0x0000_0010;
    const MF_CHECKED: u32 = 0x0000_0008;
    const MF_GRAYED: u32 = 0x0000_0001;
    const MF_DISABLED: u32 = 0x0000_0002;
    const MIIM_STATE: u32 = 0x0000_0001;
    const MIIM_ID: u32 = 0x0000_0002;
    const MIIM_TYPE: u32 = 0x0000_0010;
    const MIIM_DATA: u32 = 0x0000_0020;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 34;
    let open_text_ptr = 0x1_7900;
    let inserted_text_ptr = 0x1_7940;
    let popup_text_ptr = 0x1_7980;
    let updated_text_ptr = 0x1_79c0;
    let info_ptr = 0x1_7a00;
    let out_text_ptr = 0x1_7a80;
    memory.write_wide_z(open_text_ptr, "Open");
    memory.write_wide_z(inserted_text_ptr, "Inserted");
    memory.write_wide_z(popup_text_ptr, "More");
    memory.write_wide_z(updated_text_ptr, "Route");
    memory.map_words(info_ptr, 11);
    memory.map_halfwords(out_text_ptr, 32);

    let menu = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_MENU,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateMenu did not return a handle: {other:?}"),
    };
    let popup = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_POPUP_MENU,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreatePopupMenu did not return a handle: {other:?}"),
    };
    assert_ne!(menu, 0);
    assert_ne!(popup, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_APPEND_MENU_W,
            [menu, 0, 1001, open_text_ptr],
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
            ORD_APPEND_MENU_W,
            [menu, MF_POPUP, popup, popup_text_ptr],
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
            ORD_INSERT_MENU_W,
            [menu, 1, MF_BYPOSITION, 1002, inserted_text_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    memory.write_word(info_ptr, 44);
    memory.write_word(info_ptr + 4, MIIM_TYPE | MIIM_STATE | MIIM_ID | MIIM_DATA);
    memory.write_word(info_ptr + 36, out_text_ptr);
    memory.write_word(info_ptr + 40, 32);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MENU_ITEM_INFO_W,
            [menu, 1, 1, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr + 16)?, 1002);
    assert_eq!(memory.read_wide_z(out_text_ptr, 32), "Inserted");
    assert_eq!(
        memory.read_u32(info_ptr + 40)?,
        "Inserted".encode_utf16().count() as u32
    );

    memory.write_word(info_ptr, 44);
    memory.write_word(info_ptr + 4, MIIM_TYPE | MIIM_STATE | MIIM_DATA);
    memory.write_word(info_ptr + 8, 0);
    memory.write_word(info_ptr + 12, MF_CHECKED);
    memory.write_word(info_ptr + 16, 0);
    memory.write_word(info_ptr + 20, 0);
    memory.write_word(info_ptr + 24, 0);
    memory.write_word(info_ptr + 28, 0);
    memory.write_word(info_ptr + 32, 0xfeed_beef);
    memory.write_word(info_ptr + 36, updated_text_ptr);
    memory.write_word(info_ptr + 40, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_MENU_ITEM_INFO_W,
            [menu, 1, 1, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    memory.write_wide_z(out_text_ptr, "");
    memory.write_word(info_ptr, 44);
    memory.write_word(info_ptr + 4, MIIM_TYPE | MIIM_STATE | MIIM_ID | MIIM_DATA);
    memory.write_word(info_ptr + 36, out_text_ptr);
    memory.write_word(info_ptr + 40, 32);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MENU_ITEM_INFO_W,
            [menu, 1, 1, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(info_ptr + 12)?, MF_CHECKED);
    assert_eq!(memory.read_u32(info_ptr + 16)?, 1002);
    assert_eq!(memory.read_u32(info_ptr + 32)?, 0xfeed_beef);
    assert_eq!(memory.read_wide_z(out_text_ptr, 32), "Route");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENABLE_MENU_ITEM,
            [menu, 1, MF_BYPOSITION | MF_DISABLED | MF_GRAYED],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    memory.write_word(info_ptr, 44);
    memory.write_word(info_ptr + 4, MIIM_STATE);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MENU_ITEM_INFO_W,
            [menu, 1, 1, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + 12)? & (MF_CHECKED | MF_DISABLED | MF_GRAYED),
        MF_CHECKED | MF_DISABLED | MF_GRAYED
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENABLE_MENU_ITEM,
            [menu, 1, MF_BYPOSITION],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(previous),
            ..
        } if previous == (MF_DISABLED | MF_GRAYED)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CHECK_MENU_ITEM,
            [menu, 1, MF_BYPOSITION],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(MF_CHECKED),
            ..
        }
    ));
    memory.write_word(info_ptr, 44);
    memory.write_word(info_ptr + 4, MIIM_STATE);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MENU_ITEM_INFO_W,
            [menu, 1, 1, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_u32(info_ptr + 12)? & (MF_CHECKED | MF_DISABLED | MF_GRAYED),
        0
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_SUB_MENU,
            [menu, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == popup
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REMOVE_MENU,
            [menu, 1, MF_BYPOSITION],
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
            ORD_GET_SUB_MENU,
            [menu, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } if handle == popup
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MENU_ITEM_INFO_W,
            [menu, 1002, 0, info_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_track_popup_menu_records_attempt_without_fake_selection() -> Result<()> {
    const TPM_LEFTBUTTON: u32 = 0x0000;
    const TPM_RETURNCMD: u32 = 0x0100;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 35;
    let tpm_ptr = 0x1_7b00;
    memory.map_words(tpm_ptr, 5);
    memory.write_word(tpm_ptr, 20);
    memory.write_word(tpm_ptr + 4, 10);
    memory.write_word(tpm_ptr + 8, 20);
    memory.write_word(tpm_ptr + 12, 110);
    memory.write_word(tpm_ptr + 16, 140);

    let hwnd = kernel.create_window_ex_w(thread_id, "POPUP_OWNER", "", None, 0, 0, 0);
    let popup = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_POPUP_MENU,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreatePopupMenu did not return a handle: {other:?}"),
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_LEFTBUTTON, 320, 200, hwnd, tpm_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let tracking = kernel
        .resources
        .last_popup_tracking()
        .expect("popup tracking should be recorded");
    assert_eq!(tracking.menu, popup);
    assert_eq!(tracking.flags, TPM_LEFTBUTTON);
    assert_eq!(tracking.x, 320);
    assert_eq!(tracking.y, 200);
    assert_eq!(tracking.hwnd, hwnd);
    assert_eq!(tracking.exclude_rect, Some([10, 20, 110, 140]));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 321, 201, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let tracking = kernel
        .resources
        .last_popup_tracking()
        .expect("return-command tracking should still be recorded");
    assert_eq!(tracking.flags, TPM_RETURNCMD);
    assert_eq!(tracking.exclude_rect, None);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [0xffff_ffff, 0, 0, 0, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
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
            ORD_TRACK_POPUP_MENU_EX,
            [popup, 0, 0, 0, 0xffff_ffff, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_WINDOW_HANDLE
    );
    memory.write_word(tpm_ptr, 16);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, 0, 0, 0, hwnd, tpm_ptr],
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
fn coredll_raw_set_parent_relinks_tree_and_clears_invalid_focus() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 31;
    let msg_ptr = 0x7570;
    memory.map_words(msg_ptr, 7);

    let old_parent = kernel.create_window_ex_w(thread_id, "OLDPARENT", "", None, 0, 0, 0);
    let hidden_parent = kernel.create_window_ex_w(thread_id, "HIDDENPARENT", "", None, 0, 0, 0);
    let child =
        kernel.create_window_ex_w(thread_id, "REPARENTCHILD", "", Some(old_parent), 7, 0, 0);
    let grandchild =
        kernel.create_window_ex_w(thread_id, "REPARENTGRANDCHILD", "", Some(child), 8, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FOCUS,
            [grandchild],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        grandchild,
        WM_ACTIVATE,
        WA_ACTIVE,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        grandchild,
        WM_SETFOCUS,
        0,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_PARENT,
            [child, hidden_parent],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == old_parent
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        grandchild,
        WM_KILLFOCUS,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        grandchild,
        WM_ACTIVATE,
        WA_INACTIVE,
        0,
    );
    assert_eq!(kernel.gwe.get_focus(), None);
    assert!(!kernel.gwe.active_window_is_within(child));
    assert!(!kernel.gwe.is_window_visible(child));
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
        } if hwnd == hidden_parent
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [hidden_parent, GW_CHILD],
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
            ORD_SET_PARENT,
            [hidden_parent, grandchild],
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
    assert_eq!(kernel.gwe.get_parent(hidden_parent), None);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_PARENT,
            [child, 0xdead_beef],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_WINDOW_HANDLE
    );
    assert_eq!(kernel.gwe.get_parent(child), Some(hidden_parent));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_PARENT,
            [child, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == hidden_parent
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(kernel.gwe.get_parent(child), None);

    Ok(())
}

#[test]
fn coredll_raw_disable_or_hide_clears_focus_and_activation() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 3;
    let msg_ptr = 0x7650;
    memory.map_words(msg_ptr, 7);

    let parent = kernel.create_window_ex_w(thread_id, "FOCUSPARENT", "", None, 0, 0, 0);
    let child =
        kernel.create_window_ex_w(thread_id, "FOCUSCHILD", "", Some(parent), 0, WS_CHILD, 0);
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
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_ACTIVATE,
        WA_ACTIVE,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_SETFOCUS,
        0,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENABLE_WINDOW,
            [parent, 0],
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
        parent,
        WM_CANCELMODE,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ENABLE,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_KILLFOCUS,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_ACTIVATE,
        WA_INACTIVE,
        0,
    );
    assert_eq!(kernel.gwe.get_focus(), None);
    assert!(!kernel.gwe.active_window_is_within(parent));
    assert_eq!(kernel.gwe.get_active_window(), None);

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
    assert_eq!(kernel.gwe.get_focus(), None);
    assert_eq!(kernel.gwe.get_active_window(), None);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_ACTIVE_WINDOW,
            [parent],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.gwe.get_active_window(), None);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, 0, 0, PeekFlags::REMOVE.bits()],
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
            [parent, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ENABLE,
        1,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [parent, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
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
        parent,
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
        parent,
        WM_ACTIVATE,
        WA_ACTIVE,
        0,
    );
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
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
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
        child,
        WM_WINDOWPOSCHANGED,
        0,
        0,
    );
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
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ACTIVATE,
        WA_INACTIVE,
        child,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_ACTIVATE,
        WA_ACTIVE,
        parent,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_SETFOCUS,
        0,
        0,
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
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_SHOWWINDOW,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
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
        child,
        WM_KILLFOCUS,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_ACTIVATE,
        WA_INACTIVE,
        0,
    );
    assert_eq!(kernel.gwe.get_focus(), None);
    assert!(!kernel.gwe.active_window_is_within(child));

    let child2 =
        kernel.create_window_ex_w(thread_id, "FOCUSCHILD2", "", Some(parent), 0, WS_CHILD, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [child2, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child2,
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
        child2,
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
        parent,
        WM_ACTIVATE,
        WA_ACTIVE,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_FOCUS,
            [child2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ACTIVATE,
        WA_INACTIVE,
        child2,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child2,
        WM_ACTIVATE,
        WA_ACTIVE,
        parent,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child2,
        WM_SETFOCUS,
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_POS,
            [child2, 0, 0, 0, 0, 0, SWP_HIDEWINDOW],
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
        child2,
        WM_SHOWWINDOW,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child2,
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
        child2,
        WM_KILLFOCUS,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child2,
        WM_ACTIVATE,
        WA_INACTIVE,
        0,
    );
    assert_eq!(kernel.gwe.get_focus(), None);
    assert!(!kernel.gwe.active_window_is_within(child2));

    Ok(())
}

#[test]
fn coredll_raw_enable_window_queues_ce_enable_messages() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 33;
    let msg_ptr = 0x7a00;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "ENABLE", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENABLE_WINDOW,
            [hwnd, 0],
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
        WM_CANCELMODE,
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
        WM_ENABLE,
        0,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENABLE_WINDOW,
            [hwnd, 0],
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, 0, 0, 0],
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
            [hwnd, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
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
        WM_ENABLE,
        1,
        0,
    );

    Ok(())
}

#[test]
fn coredll_raw_is_window_enabled_observes_disabled_ancestors() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 35;
    let msg_ptr = 0x7b00;
    memory.map_words(msg_ptr, 7);

    let parent = kernel.create_window_ex_w(thread_id, "PARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w(
        thread_id,
        "CHILD",
        "",
        Some(parent),
        1,
        WS_CHILD | WS_TABSTOP,
        0,
    );
    let grandchild = kernel.create_window_ex_w(
        thread_id,
        "GRANDCHILD",
        "",
        Some(child),
        2,
        WS_CHILD | WS_TABSTOP,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_WINDOW_ENABLED,
            [grandchild],
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
            [parent, 0],
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
        parent,
        WM_CANCELMODE,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ENABLE,
        0,
        0,
    );
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, 0, 0, 0],
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
            [parent, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ENABLE,
        1,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_WINDOW_ENABLED,
            [grandchild],
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
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_CANCELMODE,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        child,
        WM_ENABLE,
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_WINDOW_ENABLED,
            [grandchild],
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
            [parent, 0],
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
        parent,
        WM_CANCELMODE,
        0,
        0,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ENABLE,
        0,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENABLE_WINDOW,
            [parent, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        parent,
        WM_ENABLE,
        1,
        0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_WINDOW_ENABLED,
            [grandchild],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_bring_window_to_top_updates_z_order_and_activation() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 34;
    let msg_ptr = 0x7c00;
    memory.map_words(msg_ptr, 7);

    let first = kernel.create_window_ex_w(thread_id, "ZFIRST", "", None, 0, 0, 0);
    let second = kernel.create_window_ex_w(thread_id, "ZSECOND", "", None, 0, 0, 0);
    let third = kernel.create_window_ex_w(thread_id, "ZTHIRD", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW,
            [first, GW_HWNDFIRST],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == third
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BRING_WINDOW_TO_TOP,
            [third],
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
            [first, GW_HWNDFIRST],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == third
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        third,
        WM_ACTIVATE,
        WA_ACTIVE,
        0,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BRING_WINDOW_TO_TOP,
            [second],
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
            [first, GW_HWNDFIRST],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == second
    ));
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        third,
        WM_ACTIVATE,
        WA_INACTIVE,
        second,
    );
    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        thread_id,
        msg_ptr,
        second,
        WM_ACTIVATE,
        WA_ACTIVE,
        third,
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BRING_WINDOW_TO_TOP,
            [0xffff_ffff],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

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
        WM_PAINT,
        0,
        0,
    );

    Ok(())
}

#[test]
fn coredll_raw_post_quit_uses_queue_state_not_filtered_post() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 77;
    let msg_ptr = 0x9a00;
    memory.map_words(msg_ptr, 7);
    let hwnd = kernel.create_window_ex_w(thread_id, "QUITFILTER", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POST_QUIT_MESSAGE,
            [0x4d],
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
            [msg_ptr, hwnd, WM_USER + 77, WM_USER + 77, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, 0);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_QUIT);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x4d);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_W,
            [msg_ptr, hwnd, WM_USER + 88, WM_USER + 88],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, 0);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_QUIT);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x4d);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_QUEUE_STATUS,
            [QS_POSTMESSAGE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(QS_POSTMESSAGE),
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
            value: CoredllValue::U32(0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_get_message_no_wait_uses_gwe_queue_without_blocking() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 78;
    let msg_ptr = 0x9c00;
    memory.map_words(msg_ptr, 7);
    let hwnd = kernel.create_window_ex_w(thread_id, "NOWAIT", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_WNO_WAIT,
            [msg_ptr, hwnd, WM_USER + 78, WM_USER + 78],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_USER + 78, 0x78, 0x87));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_WNO_WAIT,
            [msg_ptr, hwnd, WM_USER + 78, WM_USER + 78],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_USER + 78);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x78);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 0x87);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_WNO_WAIT,
            [msg_ptr, hwnd, WM_USER + 78, WM_USER + 78],
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
            ORD_POST_QUIT_MESSAGE,
            [0x78],
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
            ORD_GET_MESSAGE_WNO_WAIT,
            [msg_ptr, hwnd, WM_USER + 79, WM_USER + 79],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, 0);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_QUIT);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x78);

    Ok(())
}

#[test]
fn coredll_raw_dialog_buttons_report_default_codes_and_ids() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let dialog = kernel.create_window_ex_w_with_rect(
        thread_id,
        "DIALOG",
        "dialog",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 200, 120),
    );
    let plain = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "plain",
        Some(dialog),
        10,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON,
        0,
        Rect::from_origin_size(0, 0, 40, 20),
    );
    let default = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "default",
        Some(dialog),
        11,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_DEFPUSHBUTTON,
        0,
        Rect::from_origin_size(50, 0, 40, 20),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [plain, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(code),
            ..
        } if code == (DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [default, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(code),
            ..
        } if code == (DLGC_BUTTON | DLGC_DEFPUSHBUTTON)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [dialog, DM_GETDEFID, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(result),
            ..
        } if result == ((DC_HASDEFID << 16) | 11)
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [dialog, DM_SETDEFID, 10, 0],
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
            ORD_SEND_MESSAGE_W,
            [plain, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(code),
            ..
        } if code == (DLGC_BUTTON | DLGC_DEFPUSHBUTTON)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [default, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(code),
            ..
        } if code == (DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON)
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [dialog, DM_GETDEFID, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(result),
            ..
        } if result == ((DC_HASDEFID << 16) | 10)
    ));

    Ok(())
}

#[test]
fn coredll_raw_msgwait_ignores_desktop_waitall_flag_bit_on_ce() -> Result<()> {
    const DESKTOP_MWMO_WAITALL: u32 = 0x0001;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 80;
    let handles_ptr = 0xa100;
    memory.map_words(handles_ptr, 1);

    let event = kernel.create_event_w(None, false, true);
    memory.write_u32(handles_ptr, event)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [1, handles_ptr, 0, 0, DESKTOP_MWMO_WAITALL],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    let hwnd = kernel.create_window_ex_w(thread_id, "MSGWAIT_CE_FLAGS", "", None, 0, 0, 0);
    assert_eq!(
        kernel.set_timer_for_thread(thread_id, Some(hwnd), Some(80), 0, None),
        80
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX,
            [0, 0, 0, QS_TIMER, DESKTOP_MWMO_WAITALL],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.gwe.get_message(thread_id).unwrap().msg, WM_TIMER);

    Ok(())
}

#[test]
fn coredll_raw_message_pos_and_ready_timestamp_follow_pulled_queue_entry() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 79;
    let msg_ptr = 0x9e00;
    memory.map_words(msg_ptr, 7);
    let hwnd = kernel.create_window_ex_w(thread_id, "MSGENTRYMETA", "", None, 0, 0, 0);
    let client_pos = make_test_lparam(10, 11);
    let screen_pos = make_test_lparam(110, 211);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_POS,
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
            ORD_GET_MESSAGE_QUEUE_READY_TIME_STAMP,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    kernel.gwe.post_message(
        thread_id,
        Message::new(hwnd, WM_LBUTTONDOWN, 1, client_pos, 0x1234).with_mouse_pos(screen_pos),
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_QUEUE_READY_TIME_STAMP,
            [hwnd],
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
            ORD_GET_MESSAGE_W,
            [msg_ptr, hwnd, WM_LBUTTONDOWN, WM_LBUTTONDOWN],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_LBUTTONDOWN);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, client_pos);
    assert_eq!(memory.read_u32(msg_ptr + 16)?, 0x1234);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_MESSAGE_POS,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value == screen_pos
    ));

    Ok(())
}

#[test]
fn coredll_raw_validate_rect_preserves_remaining_update_bounds() -> Result<()> {
    const RDW_VALIDATE: u32 = 0x0008;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 12;
    let rect_ptr = 0xaa00;
    memory.map_words(rect_ptr, 4);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "VALIDATE_RECT",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 80),
    );
    assert!(kernel.gwe.validate_window(hwnd));
    assert!(kernel.gwe.invalidate_window(hwnd, None, true));

    memory.write_word(rect_ptr, 0);
    memory.write_word(rect_ptr + 4, 0);
    memory.write_word(rect_ptr + 8, 100);
    memory.write_word(rect_ptr + 12, 20);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_VALIDATE_RECT,
            [hwnd, rect_ptr],
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
            ORD_GET_UPDATE_RECT,
            [hwnd, rect_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 20);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 100);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 80);

    let region = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [0, 60, 100, 80],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn did not return a region: {other:?}"),
    };
    assert_ne!(region, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REDRAW_WINDOW,
            [hwnd, 0, region, RDW_VALIDATE],
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
            ORD_GET_UPDATE_RECT,
            [hwnd, rect_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 20);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 100);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 60);

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
            ORD_GET_UPDATE_RECT,
            [hwnd, rect_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_get_update_rgn_copies_pending_paint_bounds() -> Result<()> {
    const NULLREGION: u32 = 1;
    const SIMPLEREGION: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 13;

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "UPDATE_RGN",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 80),
    );
    assert!(kernel.gwe.validate_window(hwnd));
    let region = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [1, 2, 3, 4],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn did not return a region: {other:?}"),
    };
    assert_ne!(region, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_UPDATE_RGN,
            [hwnd, region, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(NULLREGION),
            ..
        }
    ));
    assert_eq!(
        kernel.resources.region(region).map(|region| region.rect),
        Some(Rect::default())
    );

    assert!(
        kernel
            .gwe
            .invalidate_window(hwnd, Some(Rect::from_origin_size(10, 12, 30, 22)), true)
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_UPDATE_RGN,
            [hwnd, region, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(SIMPLEREGION),
            ..
        }
    ));
    assert_eq!(
        kernel.resources.region(region).map(|region| region.rect),
        Some(Rect::from_origin_size(10, 12, 30, 22))
    );
    assert!(kernel.gwe.update_rect(hwnd).is_some());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_UPDATE_RGN,
            [0xffff_ffff, region, 0],
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
            ORD_GET_UPDATE_RGN,
            [hwnd, 0xffff_ffff, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_set_window_rgn_honors_redraw_flag() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 14;
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "WINDOW_RGN",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 80),
    );
    assert!(kernel.gwe.validate_window(hwnd));

    let region = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [5, 6, 70, 60],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn did not return a region: {other:?}"),
    };
    assert_ne!(region, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_RGN,
            [hwnd, region, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        kernel.gwe.window_region(hwnd),
        Some(Rect {
            left: 5,
            top: 6,
            right: 70,
            bottom: 60,
        })
    );
    assert!(kernel.gwe.update_rect(hwnd).is_none());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_RGN,
            [hwnd, 0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.gwe.window_region(hwnd), None);
    assert_eq!(
        kernel.gwe.update_rect(hwnd).map(|update| update.rect),
        Some(Rect::from_origin_size(0, 0, 100, 80))
    );

    Ok(())
}

#[test]
fn coredll_raw_combine_rgn_diff_preserves_holes() -> Result<()> {
    const COMPLEXREGION: u32 = 3;
    const RGN_DIFF: u32 = 4;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 14;
    let rect_ptr = 0xc000;
    memory.map_words(rect_ptr, 4);

    let outer = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [0, 0, 100, 100],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn(outer) did not return a region: {other:?}"),
    };
    let inner = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [25, 25, 75, 75],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn(inner) did not return a region: {other:?}"),
    };
    let dest = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [0, 0, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn(dest) did not return a region: {other:?}"),
    };

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_COMBINE_RGN,
            [dest, outer, inner, RGN_DIFF],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(COMPLEXREGION),
            ..
        }
    ));
    assert_eq!(kernel.resources.region(dest).unwrap().rects.len(), 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PT_IN_REGION,
            [dest, 10, 10],
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
            ORD_PT_IN_REGION,
            [dest, 50, 50],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    memory.write_word(rect_ptr, 40);
    memory.write_word(rect_ptr + 4, 40);
    memory.write_word(rect_ptr + 8, 45);
    memory.write_word(rect_ptr + 12, 45);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RECT_IN_REGION,
            [dest, rect_ptr],
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
            ORD_GET_RGN_BOX,
            [dest, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(COMPLEXREGION),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 100);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 100);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "COMPLEX_WINDOW_RGN",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 100),
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_WINDOW_RGN,
            [hwnd, dest, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));

    let roundtrip = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [0, 0, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn(roundtrip) did not return a region: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_WINDOW_RGN,
            [hwnd, roundtrip],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(COMPLEXREGION),
            ..
        }
    ));
    assert_eq!(kernel.resources.region(roundtrip).unwrap().rects.len(), 4);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PT_IN_REGION,
            [roundtrip, 50, 50],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_get_update_queries_consume_pending_erase_only() -> Result<()> {
    const SIMPLEREGION: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 14;
    let rect_ptr = 0xb000;
    let paint_ptr = 0xb100;
    memory.map_words(rect_ptr, 4);
    memory.map_words(paint_ptr, 16);
    memory.map_bytes(paint_ptr, 64);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "UPDATE_ERASE",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 80),
    );
    assert!(kernel.gwe.validate_window(hwnd));
    assert!(
        kernel
            .gwe
            .invalidate_window(hwnd, Some(Rect::from_origin_size(5, 6, 30, 20)), true)
    );
    assert_eq!(
        kernel.gwe.update_rect(hwnd),
        Some(wince_emulation_v3::ce::gwe::PaintUpdate {
            rect: Rect::from_origin_size(5, 6, 30, 20),
            erase: true,
        })
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_UPDATE_RECT,
            [hwnd, rect_ptr, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 5);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 6);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 35);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 26);
    assert_eq!(
        kernel.gwe.update_rect(hwnd),
        Some(wince_emulation_v3::ce::gwe::PaintUpdate {
            rect: Rect::from_origin_size(5, 6, 30, 20),
            erase: false,
        })
    );

    let paint_hdc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_BEGIN_PAINT,
        [hwnd, paint_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hdc),
            ..
        } => hdc,
        other => panic!("BeginPaint did not return a virtual HDC: {other:?}"),
    };
    assert_ne!(paint_hdc, 0);
    assert_eq!(memory.read_u32(paint_ptr + 4)?, 0);
    assert!(kernel.gwe.update_rect(hwnd).is_none());

    assert!(
        kernel
            .gwe
            .invalidate_window(hwnd, Some(Rect::from_origin_size(10, 12, 30, 22)), true)
    );
    let region = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [0, 0, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn did not return a region: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_UPDATE_RGN,
            [hwnd, region, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(SIMPLEREGION),
            ..
        }
    ));
    assert_eq!(
        kernel.resources.region(region).map(|region| region.rect),
        Some(Rect::from_origin_size(10, 12, 30, 22))
    );
    assert_eq!(
        kernel.gwe.update_rect(hwnd),
        Some(wince_emulation_v3::ce::gwe::PaintUpdate {
            rect: Rect::from_origin_size(10, 12, 30, 22),
            erase: false,
        })
    );

    Ok(())
}

#[test]
fn coredll_raw_redraw_window_invalidates_regions_children_and_updates_now() -> Result<()> {
    const RDW_INVALIDATE: u32 = 0x0001;
    const RDW_ERASE: u32 = 0x0004;
    const RDW_VALIDATE: u32 = 0x0008;
    const RDW_NOERASE: u32 = 0x0020;
    const RDW_ALLCHILDREN: u32 = 0x0080;
    const RDW_UPDATENOW: u32 = 0x0100;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 12;
    let rect_ptr = 0xb000;
    let paint_ptr = 0xb100;
    let msg_ptr = 0xb200;
    memory.map_words(rect_ptr, 4);
    memory.map_words(paint_ptr, 16);
    memory.map_bytes(paint_ptr, 64);
    memory.map_words(msg_ptr, 7);

    let parent = kernel.create_window_ex_w_with_rect(
        thread_id,
        "REDRAW_PARENT",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 80),
    );
    let child = kernel.create_window_ex_w_with_rect(
        thread_id,
        "REDRAW_CHILD",
        "",
        Some(parent),
        1,
        WS_VISIBLE | WS_CHILD,
        0,
        Rect::from_origin_size(10, 10, 50, 40),
    );
    assert!(kernel.gwe.validate_window(parent));
    assert!(kernel.gwe.validate_window(child));

    let region = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [4, 5, 14, 16],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn did not return a region: {other:?}"),
    };
    assert_ne!(region, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REDRAW_WINDOW,
            [parent, 0, region, RDW_INVALIDATE | RDW_NOERASE],
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
            ORD_GET_UPDATE_RECT,
            [parent, rect_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 4);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 5);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 14);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 16);

    memory.write_word(rect_ptr, 10);
    memory.write_word(rect_ptr + 4, 12);
    memory.write_word(rect_ptr + 8, 25);
    memory.write_word(rect_ptr + 12, 30);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REDRAW_WINDOW,
            [parent, rect_ptr, 0, RDW_INVALIDATE | RDW_ERASE],
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
            ORD_GET_UPDATE_RECT,
            [parent, rect_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 4);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 5);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 25);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 30);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_BEGIN_PAINT,
            [parent, paint_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hdc),
            ..
        } if hdc != 0
    ));
    assert_eq!(memory.read_u32(paint_ptr + 4)?, 1);
    assert_eq!(memory.read_i32(paint_ptr + 8)?, 4);
    assert_eq!(memory.read_i32(paint_ptr + 12)?, 5);
    assert_eq!(memory.read_i32(paint_ptr + 16)?, 25);
    assert_eq!(memory.read_i32(paint_ptr + 20)?, 30);

    assert!(kernel.gwe.validate_window(parent));
    assert!(kernel.gwe.validate_window(child));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REDRAW_WINDOW,
            [parent, 0, 0, RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_NOERASE],
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
            ORD_GET_UPDATE_RECT,
            [parent, rect_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_i32(rect_ptr)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 4)?, 0);
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 100);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 80);
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
    assert_eq!(memory.read_i32(rect_ptr + 8)?, 50);
    assert_eq!(memory.read_i32(rect_ptr + 12)?, 40);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_REDRAW_WINDOW,
            [parent, 0, 0, RDW_VALIDATE | RDW_ALLCHILDREN],
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
            ORD_GET_UPDATE_RECT,
            [parent, rect_ptr, 0],
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
            ORD_REDRAW_WINDOW,
            [parent, 0, 0, RDW_INVALIDATE | RDW_UPDATENOW],
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
            ORD_GET_UPDATE_RECT,
            [parent, rect_ptr, 0],
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, parent, WM_PAINT, WM_PAINT, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

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
    let grandchild = kernel.create_window_ex_w(thread_id, "GRANDCHILD", "", Some(child), 2, 0, 0);
    assert!(kernel.post_message_w_for_thread(thread_id, grandchild, WM_USER + 6, 1, 2));
    assert!(
        kernel
            .gwe
            .queue_sent_message_for_window(child, Message::new(child, WM_USER + 7, 3, 4, 0))
    );
    assert_eq!(kernel.set_timer(Some(parent), Some(71), 0), 71);
    assert_eq!(kernel.set_timer(Some(child), Some(72), 0), 72);
    assert_eq!(kernel.set_timer(Some(grandchild), Some(73), 0), 73);
    assert_eq!(kernel.timers.timer_count(), 3);

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
    assert!(
        kernel
            .gwe
            .window(parent)
            .is_some_and(|window| window.destroy_message_sent)
    );
    assert!(
        kernel
            .gwe
            .window(child)
            .is_some_and(|window| window.destroy_message_sent)
    );
    assert!(
        kernel
            .gwe
            .window(grandchild)
            .is_some_and(|window| window.destroy_message_sent)
    );
    let parent_order = kernel
        .gwe
        .window(parent)
        .and_then(|window| window.destroy_message_order)
        .expect("parent destroy order");
    let child_order = kernel
        .gwe
        .window(child)
        .and_then(|window| window.destroy_message_order)
        .expect("child destroy order");
    let grandchild_order = kernel
        .gwe
        .window(grandchild)
        .and_then(|window| window.destroy_message_order)
        .expect("grandchild destroy order");
    assert!(grandchild_order < child_order);
    assert!(child_order < parent_order);
    assert!(!kernel.gwe.is_window(parent));
    assert!(!kernel.gwe.is_window(child));
    assert!(!kernel.gwe.is_window(grandchild));
    assert_eq!(kernel.timers.timer_count(), 0);
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
fn coredll_raw_dialog_item_int_uses_child_window_text_and_ok_flag() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 6;
    let ok_ptr = 0xb000;
    memory.map_words(ok_ptr, 1);

    let dialog = kernel.create_window_ex_w(thread_id, "Dialog", "", None, 0, 0, 0);
    let edit = kernel.create_window_ex_w(thread_id, "edit", "", Some(dialog), 102, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_DLG_ITEM_INT,
            [dialog, 102, 42, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.gwe.get_window_text(edit, 8).as_deref(), Some("42"));

    memory.write_word(ok_ptr, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DLG_ITEM_INT,
            [dialog, 102, ok_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(42),
            ..
        }
    ));
    assert_eq!(memory.read_u32(ok_ptr)?, 1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_DLG_ITEM_INT,
            [dialog, 102, (-17i32) as u32, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.gwe.get_window_text(edit, 8).as_deref(), Some("-17"));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DLG_ITEM_INT,
            [dialog, 102, ok_ptr, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(value),
            ..
        } if value == (-17i32) as u32
    ));
    assert_eq!(memory.read_u32(ok_ptr)?, 1);

    assert!(kernel.gwe.set_window_text(edit, "not a number"));
    memory.write_word(ok_ptr, 1);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DLG_ITEM_INT,
            [dialog, 102, ok_ptr, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(ok_ptr)?, 0);

    Ok(())
}

#[test]
fn coredll_raw_dialog_controls_support_text_and_message_forwarding() -> Result<()> {
    const BM_GETCHECK: u32 = 0x00f0;
    const BM_SETCHECK: u32 = 0x00f1;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 7;
    let text_ptr = 0x1_0000;
    let buffer_ptr = 0x1_0100;
    memory.map_halfwords(text_ptr, 64);
    memory.map_halfwords(buffer_ptr, 64);

    let dialog = kernel.create_window_ex_w(thread_id, "Dialog", "", None, 0, 0, 0);
    let edit = kernel.create_window_ex_w(thread_id, "edit", "initial", Some(dialog), 102, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DLG_ITEM,
            [dialog, 102],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(found),
            ..
        } if found == edit
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DLG_ITEM,
            [dialog, 999],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));

    memory.write_wide_z(text_ptr, "hello");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_DLG_ITEM_TEXT_W,
            [dialog, 102, text_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.gwe.get_window_text(edit, 32).as_deref(),
        Some("hello")
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_DLG_ITEM_TEXT_W,
            [dialog, 102, buffer_ptr, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(5),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(buffer_ptr, 16), "hello");

    memory.write_wide_z(text_ptr, "via message");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_DLG_ITEM_MESSAGE_W,
            [dialog, 102, WM_SETTEXT, 0, text_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(
        kernel.gwe.get_window_text(edit, 32).as_deref(),
        Some("via message")
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_DLG_ITEM_MESSAGE_W,
            [dialog, 102, WM_GETTEXTLENGTH, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(11),
            ..
        }
    ));
    memory.write_wide_z(buffer_ptr, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_DLG_ITEM_MESSAGE_W,
            [dialog, 102, WM_GETTEXT, 16, buffer_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(11),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(buffer_ptr, 16), "via message");

    memory.write_wide_z(buffer_ptr, "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_MESSAGE_W,
            [edit, WM_GETTEXT, 5, buffer_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(4),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(buffer_ptr, 5), "via ");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SEND_DLG_ITEM_MESSAGE_W,
            [dialog, 102, BM_SETCHECK, 1, 0],
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
            ORD_SEND_DLG_ITEM_MESSAGE_W,
            [dialog, 102, BM_GETCHECK, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
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
    let actual_lparam = memory.read_u32(msg_ptr + 12).unwrap();
    if msg == WM_WINDOWPOSCHANGED && lparam == 0 {
        assert_ne!(actual_lparam, 0);
    } else {
        assert_eq!(actual_lparam, lparam);
    }
}

fn assert_next_filtered_message(
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, msg, msg, PeekFlags::REMOVE.bits()],
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

fn write_raw_message(
    memory: &mut TestGuestMemory,
    msg_ptr: u32,
    hwnd: u32,
    msg: u32,
    wparam: u32,
    lparam: u32,
) -> Result<()> {
    memory.write_u32(msg_ptr, hwnd)?;
    memory.write_u32(msg_ptr + 4, msg)?;
    memory.write_u32(msg_ptr + 8, wparam)?;
    memory.write_u32(msg_ptr + 12, lparam)?;
    memory.write_u32(msg_ptr + 16, 0)?;
    memory.write_u32(msg_ptr + 20, 0)?;
    memory.write_u32(msg_ptr + 24, 0)?;
    Ok(())
}

fn make_test_lparam(x: i32, y: i32) -> u32 {
    ((y as u16 as u32) << 16) | (x as u16 as u32)
}
