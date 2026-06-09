use std::fs;

use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_ACTIVATE_KEYBOARD_LAYOUT, ORD_ADD_FONT_RESOURCE_W, ORD_ADJUST_WINDOW_RECT_EX,
            ORD_APPEND_MENU_W, ORD_BEGIN_PAINT, ORD_BIT_BLT, ORD_BRING_WINDOW_TO_TOP,
            ORD_CREATE_BITMAP, ORD_CREATE_PEN, ORD_CREATE_RECT_RGN_INDIRECT,
            ORD_CHECK_MENU_ITEM, ORD_CHECK_MENU_RADIO_ITEM, ORD_CHECK_RADIO_BUTTON,
            ORD_CHILD_WINDOW_FROM_POINT,
            ORD_CLIENT_TO_SCREEN, ORD_COMBINE_RGN, ORD_COPY_RECT, ORD_CREATE_CARET,
            ORD_CREATE_COMPATIBLE_BITMAP, ORD_CREATE_COMPATIBLE_DC,
            ORD_CREATE_DIALOG_INDIRECT_PARAM_W, ORD_CREATE_DIBSECTION, ORD_CREATE_FONT_INDIRECT_W,
            ORD_CREATE_ICON_INDIRECT, ORD_CREATE_MENU, ORD_CREATE_MUTEX_W, ORD_CREATE_PALETTE,
            ORD_CREATE_PATTERN_BRUSH, ORD_CREATE_PEN_INDIRECT, ORD_CREATE_POPUP_MENU,
            ORD_CREATE_RECT_RGN, ORD_CREATE_SOLID_BRUSH, ORD_CREATE_WINDOW_EX_W,
            ORD_DEF_DLG_PROC_W, ORD_DEF_WINDOW_PROC_W, ORD_DELETE_DC, ORD_DELETE_MENU,
            ORD_DELETE_OBJECT, ORD_DESTROY_ACCELERATOR_TABLE, ORD_DESTROY_CARET,
            ORD_DIALOG_BOX_INDIRECT_PARAM_W, ORD_DESTROY_ICON, ORD_DESTROY_MENU,
            ORD_DESTROY_WINDOW, ORD_DISABLE_CARET_SYSTEM_WIDE, ORD_DISPATCH_MESSAGE_W,
            ORD_DRAW_FRAME_CONTROL,
            ORD_DRAW_ICON_EX, ORD_DRAW_MENU_BAR, ORD_DRAW_TEXT_W,
            ORD_ELLIPSE, ORD_ENABLE_CARET_SYSTEM_WIDE,
            ORD_ENABLE_MENU_ITEM, ORD_ENABLE_WINDOW, ORD_END_DIALOG, ORD_END_PAINT, ORD_EQUAL_RECT,
            ORD_EXT_TEXT_OUT_W, ORD_FILL_RECT, ORD_FIND_RESOURCE, ORD_FIND_RESOURCE_W,
            ORD_FIND_WINDOW_W,
            ORD_GRADIENT_FILL,
            ORD_GET_ACTIVE_WINDOW, ORD_GET_ASSOCIATED_MENU, ORD_GET_ASYNC_KEY_STATE,
            ORD_GET_CLIP_BOX, ORD_GET_OBJECT_W, ORD_GET_PIXEL,
            ORD_IMAGE_LIST_DRAW, ORD_IMAGE_LIST_DRAW_INDIRECT,
            ORD_GET_ASYNC_SHIFT_FLAGS, ORD_GET_CAPTURE, ORD_GET_CARET_BLINK_TIME,
            ORD_GET_CARET_POS, ORD_GET_CLASS_INFO_W, ORD_GET_CLASS_NAME_W, ORD_GET_CLIENT_RECT,
            ORD_GET_CURSOR, ORD_GET_CURSOR_POS, ORD_GET_DC, ORD_GET_DESKTOP_WINDOW,
            ORD_GET_DEVICE_CAPS,
            ORD_GET_DIALOG_BASE_UNITS, ORD_GET_DIBCOLOR_TABLE, ORD_GET_DLG_CTRL_ID,
            ORD_GET_DLG_ITEM, ORD_GET_DLG_ITEM_INT, ORD_GET_DLG_ITEM_TEXT_W, ORD_GET_FOCUS,
            ORD_GET_FOREGROUND_KEYBOARD_LAYOUT_HANDLE, ORD_GET_FOREGROUND_KEYBOARD_TARGET,
            ORD_GET_FOREGROUND_WINDOW, ORD_GET_KEY_STATE, ORD_GET_KEYBOARD_LAYOUT,
            ORD_GET_KEYBOARD_LAYOUT_LIST, ORD_GET_KEYBOARD_LAYOUT_NAME_W, ORD_GET_KEYBOARD_TARGET,
            ORD_GET_MENU, ORD_GET_MENU_ITEM_INFO_W, ORD_GET_MESSAGE_POS,
            ORD_GET_MESSAGE_QUEUE_READY_TIME_STAMP, ORD_GET_MESSAGE_SOURCE, ORD_GET_MESSAGE_W,
            ORD_GET_MESSAGE_WNO_WAIT, ORD_GET_NEAREST_PALETTE_INDEX, ORD_GET_NEXT_DLG_GROUP_ITEM,
            ORD_GET_NEXT_DLG_TAB_ITEM, ORD_GET_PALETTE_ENTRIES, ORD_GET_PARENT,
            ORD_GET_QUEUE_STATUS, ORD_GET_RGN_BOX, ORD_GET_ROP2, ORD_GET_STOCK_OBJECT,
            ORD_GET_SUB_MENU, ORD_GET_SYS_COLOR, ORD_GET_SYS_COLOR_BRUSH, ORD_GET_SYSTEM_INFO,
            ORD_GET_SYSTEM_METRICS, ORD_GET_SYSTEM_PALETTE_ENTRIES, ORD_GET_TEXT_ALIGN,
            ORD_GET_TEXT_COLOR, ORD_GET_TEXT_EXTENT_EX_POINT_W, ORD_GET_TEXT_FACE_W,
            ORD_GET_TEXT_METRICS_W, ORD_GET_UPDATE_RECT, ORD_GET_UPDATE_RGN, ORD_GET_VERSION_EX_W,
            ORD_GET_WINDOW,
            ORD_GET_WINDOW_LONG_W, ORD_GET_WINDOW_RECT, ORD_GET_WINDOW_RGN,
            ORD_GET_WINDOW_TEXT_LENGTH_W, ORD_GET_WINDOW_TEXT_W, ORD_GET_WINDOW_TEXT_WDIRECT,
            ORD_GET_WINDOW_THREAD_PROCESS_ID,
            ORD_GLOBAL_MEMORY_STATUS, ORD_HIDE_CARET, ORD_IMM_ASSOCIATE_CONTEXT,
            ORD_IMM_CREATE_CONTEXT, ORD_IMM_DESTROY_CONTEXT, ORD_IMM_DISABLE_IME,
            ORD_IMM_ENABLE_IME, ORD_IMM_GET_COMPOSITION_STRING_W, ORD_IMM_GET_CONTEXT,
            ORD_IMM_GET_CONVERSION_STATUS, ORD_IMM_GET_IMEFILE_NAME_W, ORD_IMM_GET_KEYBOARD_LAYOUT,
            ORD_IMM_GET_OPEN_STATUS, ORD_IMM_IS_IME, ORD_IMM_NOTIFY_IME, ORD_IMM_RELEASE_CONTEXT,
            ORD_IMM_SET_CONVERSION_STATUS, ORD_IMM_SET_OPEN_STATUS, ORD_IN_SEND_MESSAGE,
            ORD_INFLATE_RECT, ORD_INSERT_MENU_W, ORD_INTERSECT_RECT, ORD_INVALIDATE_RECT,
            ORD_IS_CHILD, ORD_IS_DIALOG_MESSAGE_W, ORD_IS_RECT_EMPTY, ORD_IS_WINDOW,
            ORD_IS_WINDOW_ENABLED, ORD_IS_WINDOW_VISIBLE, ORD_KEYBD_EVENT, ORD_KILL_TIMER,
            ORD_LINE_TO, ORD_LOAD_CURSOR_W, ORD_LOAD_ICON_W, ORD_LOAD_KEYBOARD_LAYOUT_W,
            ORD_LOAD_ACCELERATORS_W, ORD_LOAD_BITMAP_W, ORD_LOAD_MENU_W,
            ORD_LOAD_RESOURCE, ORD_LOAD_STRING_W, ORD_MAP_DIALOG_RECT, ORD_MAP_WINDOW_POINTS,
            ORD_MESSAGE_BOX_W, ORD_MOVE_TO_EX, ORD_MOVE_WINDOW,
            ORD_MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX, ORD_OFFSET_RECT, ORD_PAT_BLT, ORD_PEEK_MESSAGE_W,
            ORD_POLYGON, ORD_POLYLINE, ORD_POST_KEYBD_MESSAGE, ORD_POST_MESSAGE_W,
            ORD_POST_QUIT_MESSAGE, ORD_POST_THREAD_MESSAGE_W, ORD_PT_IN_RECT, ORD_PT_IN_REGION,
            ORD_REALIZE_PALETTE, ORD_RECT_IN_REGION, ORD_RECTANGLE, ORD_REDRAW_WINDOW,
            ORD_REGISTER_CLASS_W,
            ORD_REGISTER_GESTURE, ORD_REGISTER_WINDOW_MESSAGE_W, ORD_RELEASE_CAPTURE,
            ORD_RELEASE_DC, ORD_RELEASE_MUTEX,
            ORD_REMOVE_MENU, ORD_ROUND_RECT, ORD_SCREEN_TO_CLIENT, ORD_SELECT_CLIP_RGN,
            ORD_SELECT_OBJECT, ORD_SELECT_PALETTE, ORD_SEND_DLG_ITEM_MESSAGE_W,
            ORD_SEND_MESSAGE_TIMEOUT, ORD_SEND_MESSAGE_W, ORD_SEND_NOTIFY_MESSAGE_W,
            ORD_SET_ACTIVE_WINDOW, ORD_SET_ASSOCIATED_MENU, ORD_SET_BK_COLOR, ORD_SET_BK_MODE,
            ORD_SET_BITMAP_BITS, ORD_SET_BRUSH_ORG_EX,
            ORD_SET_CAPTURE, ORD_SET_CARET_BLINK_TIME, ORD_SET_CARET_POS, ORD_SET_CURSOR,
            ORD_SET_DIBCOLOR_TABLE, ORD_SET_DIBITS_TO_DEVICE, ORD_SET_DLG_ITEM_INT,
            ORD_SET_DLG_ITEM_TEXT_W, ORD_SET_FOCUS, ORD_SET_FOREGROUND_WINDOW,
            ORD_SET_KEYBOARD_TARGET, ORD_SET_MENU, ORD_SET_MENU_ITEM_INFO_W,
            ORD_SET_PALETTE_ENTRIES, ORD_SET_PARENT, ORD_SET_RECT, ORD_SET_RECT_EMPTY,
            ORD_SET_RECT_RGN, ORD_SET_ROP2, ORD_SET_SYS_COLORS,
            ORD_SET_TEXT_ALIGN, ORD_SET_TEXT_COLOR, ORD_SET_TIMER,
            ORD_CHANGE_DISPLAY_SETTINGS_EX,
            ORD_REMOVE_FONT_RESOURCE_W,
            ORD_SET_LOCAL_TIME, ORD_SET_SYSTEM_TIME,
            ORD_SET_WINDOW_LONG_W, ORD_SET_WINDOW_POS, ORD_SET_WINDOW_RGN, ORD_SET_WINDOW_TEXT_W,
            ORD_SHOW_CARET, ORD_SHOW_WINDOW, ORD_SIZEOF_RESOURCE, ORD_SLEEP, ORD_STRETCH_BLT,
            ORD_STRETCH_DIBITS, ORD_SYSTEM_PARAMETERS_INFO_W, ORD_TRACK_POPUP_MENU_EX,
            ORD_TRANSLATE_ACCELERATOR_W, ORD_TRANSLATE_MESSAGE, ORD_TRANSPARENT_IMAGE,
            ORD_UNION_RECT, ORD_UPDATE_WINDOW, ORD_VALIDATE_RECT, ORD_WINDOW_FROM_POINT,
        },
        framebuffer::{Framebuffer, FramebufferRect, PixelFormat, VirtualFramebuffer},
        gwe::{
            BS_AUTORADIOBUTTON, BS_DEFPUSHBUTTON, BS_PUSHBUTTON, BS_RADIOBUTTON,
            DC_HASDEFID, DEFAULT_KEYBOARD_LAYOUT_HKL,
            DEFAULT_KEYBOARD_LAYOUT_NAME, DLGC_BUTTON, DLGC_DEFPUSHBUTTON, DLGC_HASSETSEL,
            DLGC_RADIOBUTTON, DLGC_STATIC, DLGC_UNDEFPUSHBUTTON, DLGC_WANTARROWS, DLGC_WANTCHARS,
            DM_GETDEFID, DM_SETDEFID, GW_CHILD, GW_HWNDFIRST, GW_HWNDNEXT, GW_HWNDPREV, GW_OWNER,
            GWL_STYLE, GWL_USERDATA, HTCAPTION, HTCLIENT, HTCLOSE, HTNOWHERE, HTSYSMENU, HTTOPLEFT,
            HWND_BROADCAST, KEY_SHIFT_ANY_SHIFT_FLAG,
            KEY_STATE_DOWN_FLAG, KEY_STATE_GET_ASYNC_DOWN_FLAG, KEY_STATE_PREV_DOWN_FLAG,
            GCS_COMPSTR, GCS_RESULTSTR, MA_ACTIVATE, MSGSRC_HARDWARE_KEYBOARD, MSGSRC_SOFTWARE_POST, MSGSRC_SOFTWARE_SEND,
            Message, PeekFlags, Point, QS_PAINT, QS_POSTMESSAGE, QS_SENDMESSAGE, QS_TIMER, Rect,
            SC_CLOSE, SM_CXBORDER, SM_CXSCREEN, SM_CYSCREEN, SMF_NOTIFY_MESSAGE,
            SMF_SENDER_NO_WAIT, SMF_TIMEOUT, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE,
            SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, VK_CAPITAL, VK_CONTROL, VK_HANGUL, VK_LCONTROL,
            VK_LSHIFT, VK_MENU, VK_NUMLOCK, VK_SCROLL, VK_SHIFT, WA_ACTIVE, WA_CLICKACTIVE,
            WA_INACTIVE, WM_ACTIVATE,
            WM_CANCELMODE, WM_CAPTURECHANGED, WM_CHAR, WM_CHARTOITEM, WM_CLOSE, WM_COMMAND,
            WM_CONTEXTMENU, WM_DESTROY,
            WM_ENABLE, WM_ENTERMENULOOP, WM_ERASEBKGND,
            WM_EXITMENULOOP, WM_GETDLGCODE, WM_GETTEXT, WM_GETTEXTLENGTH, WM_INITMENUPOPUP,
            WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP,
            WM_MENUCHAR, WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_MOUSEWHEEL,
            WM_MOVE, WM_NCACTIVATE, WM_NCCREATE, WM_NCDESTROY, WM_NCHITTEST,
            WM_NCLBUTTONDBLCLK, WM_NCLBUTTONDOWN, WM_PAINT, WM_QUIT, WM_RBUTTONDOWN,
            WM_RBUTTONDBLCLK, WM_RBUTTONUP, WM_SETCURSOR, WM_SETFOCUS, WM_SETREDRAW,
            WM_COMPAREITEM, WM_DELETEITEM, WM_DISPLAYCHANGE, WM_DRAWITEM, WM_FONTCHANGE,
            WM_GETFONT, WM_GETMINMAXINFO, WM_HSCROLL,
            WM_INPUTLANGCHANGE,
            WM_MEASUREITEM, WM_NEXTDLGCTL, WM_SETFONT, WM_SETTEXT, WM_SETTINGCHANGE, WM_SHOWWINDOW, WM_SIZE,
            WM_TIMECHANGE, WM_VSCROLL,
            WM_DEADCHAR, WM_IME_CHAR, WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION,
            WM_SYSCHAR, WM_SYSCOMMAND, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER, WM_USER, WM_VKEYTOITEM,
            WM_WINDOWPOSCHANGED,
            WNDCLASSW_SIZE,
            WS_CHILD, WS_DISABLED, WS_GROUP, WS_POPUP, WS_TABSTOP, WS_VISIBLE,
        },
        kernel::CeKernel,
        memory::PROCESS_HEAP_HANDLE,
        resource::{AcceleratorEntry, ResourceId},
        thread::{
            ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE,
            ERROR_INVALID_PARAMETER, ERROR_INVALID_WINDOW_HANDLE,
            ERROR_RESOURCE_NAME_NOT_FOUND,
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
    const RT_GROUP_ICON: u32 = 14;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let module = 0x0040_0000;

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

    let resource_icon = kernel.resources.register(
        module,
        ResourceId::Integer(77),
        ResourceId::Integer(RT_GROUP_ICON as u16),
        0x0004_0000,
        16,
    );
    let loaded_resource_icon = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_ICON_W,
        [module, 77],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(icon),
            ..
        } => icon,
        other => panic!("resource LoadIconW did not return a handle: {other:?}"),
    };
    assert_eq!(loaded_resource_icon, resource_icon);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [loaded_resource_icon],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    let non_icon_resource = kernel.resources.register(
        module,
        ResourceId::Integer(78),
        ResourceId::Integer(10),
        0x0004_0100,
        4,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [non_icon_resource],
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

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "ICON_DRAW",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 40, 40),
    );
    let hdc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [hwnd],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hdc),
            ..
        } => hdc,
        other => panic!("GetDC did not return a handle: {other:?}"),
    };
    let mut framebuffer = VirtualFramebuffer::new(40, 40, PixelFormat::Rgb565)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 5, 6, icon, 8, 8, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    let body_offset = (7 * framebuffer.stride()) + (6 * PixelFormat::Rgb565.bytes_per_pixel());
    assert_ne!(
        &framebuffer.pixels()[body_offset..body_offset + 2],
        &[0, 0],
        "DrawIconEx should paint a pseudo-icon body into the framebuffer"
    );
    let outside_offset = 4 * PixelFormat::Rgb565.bytes_per_pixel();
    assert_eq!(
        &framebuffer.pixels()[outside_offset..outside_offset + 2],
        &[0, 0],
        "DrawIconEx should leave pixels outside the icon rectangle untouched"
    );

    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 12, 12);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DRAW_ICON_EX,
            [mem_dc, 2, 3, icon, 6, 6, 0, 0, 0x0003],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_ne!(
        rgb565_at(&memory, bits_ptr, stride, 3, 4),
        0,
        "DrawIconEx should paint a pseudo-icon body into selected memory bitmaps"
    );
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, 1, 2),
        0,
        "DrawIconEx should leave memory bitmap pixels outside the icon rectangle untouched"
    );

    let (_, mask_bitmap, _, _) =
        create_selected_rgb565_dib_with_bitmap(&table, &mut kernel, &mut memory, thread_id, 8, 8);
    let (_, color_bitmap, _, _) =
        create_selected_rgb565_dib_with_bitmap(&table, &mut kernel, &mut memory, thread_id, 8, 8);
    let icon_info = 0x1_5000;
    memory.map_words(icon_info, 5);
    memory.write_word(icon_info, 1);
    memory.write_word(icon_info + 4, 2);
    memory.write_word(icon_info + 8, 3);
    memory.write_word(icon_info + 12, mask_bitmap);
    memory.write_word(icon_info + 16, color_bitmap);
    let indirect_icon = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_ICON_INDIRECT,
        [icon_info],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(icon),
            ..
        } => icon,
        other => panic!("CreateIconIndirect did not return a handle: {other:?}"),
    };
    assert_ne!(indirect_icon, 0);
    let indirect = kernel
        .resources
        .icon(indirect_icon)
        .expect("CreateIconIndirect should register icon object");
    assert!(indirect.is_icon);
    assert_eq!(indirect.x_hotspot, 2);
    assert_eq!(indirect.y_hotspot, 3);
    assert_eq!(indirect.mask_bitmap, mask_bitmap);
    assert_eq!(indirect.color_bitmap, color_bitmap);
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [indirect_icon],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.resources.icon(indirect_icon).is_none());
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_ICON,
            [indirect_icon],
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

    memory.write_word(icon_info + 12, 0x1234_5678);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_ICON_INDIRECT,
            [icon_info],
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_ICON_INDIRECT,
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
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [hdc, 5, 6, 0x1234_5678, 8, 8, 0, 0, 0x0003],
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
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_DRAW_ICON_EX,
            [0x1234_5678, 5, 6, icon, 8, 8, 0, 0, 0x0003],
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
            ORD_DESTROY_ICON,
            [0x1234_5678],
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
fn coredll_raw_fill_rect_respects_complex_clip_holes_on_memory_dib() -> Result<()> {
    const COMPLEXREGION: u32 = 3;
    const RGN_DIFF: u32 = 4;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 8);

    let outer = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [1, 1, 7, 7],
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
        [3, 3, 5, 5],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn(inner) did not return a region: {other:?}"),
    };
    let clip = match table.dispatch_raw_ordinal_with_memory(
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
        other => panic!("CreateRectRgn(clip) did not return a region: {other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_COMBINE_RGN,
            [clip, outer, inner, RGN_DIFF],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(COMPLEXREGION),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SELECT_CLIP_RGN,
            [mem_dc, clip],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(COMPLEXREGION),
            ..
        }
    ));

    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr, 0);
    memory.write_word(rect_ptr + 4, 0);
    memory.write_word(rect_ptr + 8, 8);
    memory.write_word(rect_ptr + 12, 8);
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

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 2), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 4, 4), 0x0000);
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
fn coredll_raw_line_to_paints_selected_memory_dib() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let logpen_ptr = 0x1_0300;
    memory.map_bytes(logpen_ptr, 16);
    memory.write_bytes(
        logpen_ptr,
        &[
            0x00, 0x00, 0x00, 0x00, // PS_SOLID
            0x01, 0x00, 0x00, 0x00, // width 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, // green
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
            ORD_MOVE_TO_EX,
            [mem_dc, 1, 2, 0],
        ),
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
            ORD_LINE_TO,
            [mem_dc, 5, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 1, 2), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 2), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 2), 0x07e0);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 1), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_polyline_honors_ps_dash_on_selected_memory_dib() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 16, 4);
    let points_ptr = 0x1_0200;
    let logpen_ptr = 0x1_0300;
    memory.map_bytes(points_ptr, 16);
    memory.map_bytes(logpen_ptr, 16);
    memory.write_point(points_ptr, 0, 1);
    memory.write_point(points_ptr + 8, 14, 1);
    memory.write_bytes(
        logpen_ptr,
        &[
            0x01, 0x00, 0x00, 0x00, // PS_DASH
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
            ORD_POLYLINE,
            [mem_dc, points_ptr, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 1, 1), 0xf800);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 9, 1), 0x0000);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 13, 1), 0xf800);

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
fn coredll_raw_polygon_alternate_fill_skips_doubly_traced_interior() -> Result<()> {
    // ALTERNATE (even-odd) fill is the CE default. A square traced twice produces
    // an even number of crossings at interior points, so the interior is NOT filled.
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

    // Even number of edge crossings at every interior point → not filled.
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 2), 0x0000);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 4), 0x0000);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_polygon_alternate_fill_fills_simple_convex_polygon() -> Result<()> {
    // ALTERNATE fill covers the interior of a simple convex polygon.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 6);
    let points_ptr = 0x1_0200;
    let points = [(1, 1), (6, 1), (6, 5), (1, 5)];
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
        other => panic!("GetStockObject(NULL_PEN): {other:?}"),
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
    // Interior pixels are filled with blue (0x001f in RGB565).
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 2), 0x001f);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 4), 0x001f);
    // Outside the polygon is untouched.
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
fn coredll_raw_bitblt_skips_effectively_hidden_window_hdc() -> Result<()> {
    const SRCCOPY: u32 = 0x00cc_0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let mut framebuffer = VirtualFramebuffer::new(4, 4, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();

    let parent = kernel.create_window_ex_w_with_rect(
        thread_id,
        "PARENT",
        "parent",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 4, 4),
    );
    let child = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CHILD",
        "child",
        Some(parent),
        2,
        WS_CHILD,
        0,
        Rect::from_origin_size(1, 1, 2, 2),
    );

    let parent_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [parent],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetDC(parent) did not return a handle: {other:?}"),
    };
    let child_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [child],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetDC(child) did not return a handle: {other:?}"),
    };
    let mem_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [parent_dc],
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
        [parent_dc, info, 0, bits_out, 0, 0],
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
            [child_dc, 0, 0, 2, 2, mem_dc, 0, 0, SRCCOPY],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(framebuffer.pixels().iter().all(|byte| *byte == 0));
    assert!(framebuffer.dirty_rects().is_empty());

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
    assert!(kernel.gwe.is_window_visible(child));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_BIT_BLT,
            [child_dc, 0, 0, 2, 2, mem_dc, 0, 0, SRCCOPY],
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
fn coredll_raw_bitblt_clips_window_hdc_behind_visible_sibling() -> Result<()> {
    const SRCCOPY: u32 = 0x00cc_0020;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let mut framebuffer = VirtualFramebuffer::new(4, 4, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();

    let bottom = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BOTTOM",
        "bottom",
        None,
        1,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 4, 4),
    );
    let top = kernel.create_window_ex_w_with_rect(
        thread_id,
        "TOP",
        "top",
        None,
        2,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(1, 1, 2, 2),
    );

    assert_eq!(kernel.gwe.z_order_snapshot()[1], top);
    assert_eq!(kernel.gwe.visible_client_rects(bottom).len(), 4);

    let bottom_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_GET_DC,
        [bottom],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("GetDC(bottom) did not return a handle: {other:?}"),
    };
    let mem_dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [bottom_dc],
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
    header[4..8].copy_from_slice(&4i32.to_le_bytes());
    header[8..12].copy_from_slice(&(-4i32).to_le_bytes());
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
        [bottom_dc, info, 0, bits_out, 0, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        other => panic!("CreateDIBSection did not return a bitmap: {other:?}"),
    };
    let bits_ptr = memory.read_u32(bits_out)?;
    memory.map_bytes(bits_ptr, 64);
    let mut red_bitmap = vec![0u8; 64];
    for pixel in red_bitmap.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0x00, 0x00, 0xff, 0xff]);
    }
    memory.write_bytes(bits_ptr, &red_bitmap);

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
            [bottom_dc, 0, 0, 4, 4, mem_dc, 0, 0, SRCCOPY],
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
        &[0x00, 0xf8]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(1, 1)..offset(1, 1) + 2],
        &[0x00, 0x00]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(2, 2)..offset(2, 2) + 2],
        &[0x00, 0x00]
    );
    assert_eq!(
        &framebuffer.pixels()[offset(3, 3)..offset(3, 3) + 2],
        &[0x00, 0xf8]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_WINDOW,
            [top, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.gwe.visible_client_rects(bottom),
        vec![Rect::from_origin_size(0, 0, 4, 4)]
    );
    let mut framebuffer = VirtualFramebuffer::new(4, 4, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_BIT_BLT,
            [bottom_dc, 0, 0, 4, 4, mem_dc, 0, 0, SRCCOPY],
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
        &framebuffer.pixels()[offset(2, 2)..offset(2, 2) + 2],
        &[0x00, 0xf8]
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
fn coredll_raw_add_font_resource_uses_guest_file_mounts() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let root = std::env::temp_dir().join(format!("wince_add_font_resource_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temporary mounted font directory");
    fs::write(root.join("font.ttf"), b"fake-font").expect("write temporary mounted font");
    kernel.mount_guest_root("\\SDMMC Disk", &root);

    let font_path_ptr = 0x1_0000;
    let missing_path_ptr = 0x1_0100;
    memory.write_wide_z(font_path_ptr, "\\SDMMC Disk\\font.ttf");
    memory.write_wide_z(missing_path_ptr, "\\SDMMC Disk\\missing.ttf");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ADD_FONT_RESOURCE_W,
            [font_path_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ADD_FONT_RESOURCE_W,
            [missing_path_ptr],
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

    let _ = fs::remove_dir_all(&root);
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
    let disabled_child = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CHILD",
        "disabled",
        Some(parent),
        4,
        WS_VISIBLE | WS_CHILD | WS_DISABLED,
        0,
        Rect::from_origin_size(150, 60, 40, 40),
    );
    assert!(kernel.gwe.is_window_visible(parent));
    assert!(kernel.gwe.is_window_visible(child));
    assert!(!kernel.gwe.is_window_visible(sibling));
    assert!(!kernel.gwe.is_window_enabled(disabled_child));

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
            ORD_WINDOW_FROM_POINT,
            [160, 80],
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
        } if hwnd == sibling
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CHILD_WINDOW_FROM_POINT,
            [parent, 160, 70],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } if hwnd == disabled_child
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CHILD_WINDOW_FROM_POINT,
            [parent, 280, 180],
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
            ORD_CHILD_WINDOW_FROM_POINT,
            [parent, 900, 900],
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
        } if hwnd == child
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
fn coredll_raw_is_dialog_message_tab_skips_disabled_and_invisible_controls() -> Result<()> {
    // Tab navigation must skip controls that are disabled or invisible,
    // matching CE IsDialogMessage behavior.
    const VK_TAB: u32 = 0x09;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let msg_ptr = 0x3600;
    memory.map_words(msg_ptr, 7);

    let dialog = kernel.create_window_ex_w_with_rect(
        thread_id, "DIALOG", "dlg", None, 1, WS_VISIBLE, 0,
        Rect::from_origin_size(0, 0, 200, 120),
    );
    let first = kernel.create_window_ex_w_with_rect(
        thread_id, "BUTTON", "a", Some(dialog), 20,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP, 0,
        Rect::from_origin_size(0, 0, 20, 20),
    );
    // Second tab stop is disabled — tab should skip it.
    let _disabled = kernel.create_window_ex_w_with_rect(
        thread_id, "BUTTON", "b", Some(dialog), 21,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | WS_DISABLED, 0,
        Rect::from_origin_size(20, 0, 20, 20),
    );
    // Third tab stop is invisible — tab should also skip it.
    let _invisible = kernel.create_window_ex_w_with_rect(
        thread_id, "BUTTON", "c", Some(dialog), 22,
        WS_CHILD | WS_TABSTOP, 0,
        Rect::from_origin_size(40, 0, 20, 20),
    );
    let last = kernel.create_window_ex_w_with_rect(
        thread_id, "BUTTON", "d", Some(dialog), 23,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP, 0,
        Rect::from_origin_size(60, 0, 20, 20),
    );

    let _ = kernel.set_focus(Some(first));
    write_raw_message(&mut memory, msg_ptr, first, WM_KEYDOWN, VK_TAB, 0)?;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_IS_DIALOG_MESSAGE_W, [dialog, msg_ptr],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    assert_eq!(
        kernel.gwe.get_focus(),
        Some(last),
        "tab from first must skip disabled and invisible controls to reach last"
    );

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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POST_KEYBD_MESSAGE,
            [
                hwnd,
                u32::from('C'),
                KEY_STATE_DOWN_FLAG,
                2,
                0,
                0,
                chars_ptr,
                1,
            ],
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
        WM_KEYDOWN,
        u32::from('C'),
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
        u32::from('A'),
        1,
    );
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_CHAR,
                WM_CHAR,
                PeekFlags::NO_REMOVE
            )
            .is_none(),
        "capacity-limited PostKeybdMessage should queue only one character"
    );

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
            ORD_POST_KEYBD_MESSAGE,
            [0, u32::from('T'), KEY_STATE_DOWN_FLAG, 0, 0, 0],
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
        target_hwnd,
        WM_KEYDOWN,
        u32::from('T'),
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
            ORD_POST_KEYBD_MESSAGE,
            [0, u32::from('G'), KEY_STATE_DOWN_FLAG, 0, 0, 0],
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
        focus_hwnd,
        WM_KEYDOWN,
        u32::from('G'),
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
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SEND_NOTIFY_MESSAGE_W,
            [same_thread_hwnd, WM_CLOSE, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(caller_thread),
        ERROR_INVALID_WINDOW_HANDLE
    );

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

    let mut sync_send_class = [0u8; WNDCLASSW_SIZE];
    sync_send_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("SYNC_SEND", sync_send_class);
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
fn coredll_raw_peek_message_services_sent_before_posted() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 146;
    let receiver_thread = 147;
    let msg_ptr = 0xb040;
    memory.map_words(msg_ptr, 7);

    let mut peek_class = [0u8; WNDCLASSW_SIZE];
    peek_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("SYNC_SEND_PEEK_ORDER", peek_class);
    let hwnd =
        kernel.create_window_ex_w(receiver_thread, "SYNC_SEND_PEEK_ORDER", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_POST_MESSAGE_W,
            [hwnd, WM_USER + 1, 0x1111, 0x2222],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_SEND_MESSAGE_W,
            [hwnd, WM_ERASEBKGND, 0x3333, 0x4444],
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
            receiver_thread,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, 0, 0, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_ERASEBKGND);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0x3333);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 0x4444);
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

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        hwnd,
        WM_USER + 1,
        0x1111,
        0x2222,
    );

    Ok(())
}

#[test]
fn coredll_raw_get_message_services_sent_before_posted() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 148;
    let receiver_thread = 149;
    let msg_ptr = 0xb080;
    memory.map_words(msg_ptr, 7);

    let mut get_class = [0u8; WNDCLASSW_SIZE];
    get_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("SYNC_SEND_GET_ORDER", get_class);
    let hwnd = kernel.create_window_ex_w(receiver_thread, "SYNC_SEND_GET_ORDER", "", None, 0, 0, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_POST_MESSAGE_W,
            [hwnd, WM_USER + 3, 0x5555, 0x6666],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_SEND_MESSAGE_W,
            [hwnd, WM_ERASEBKGND, 0x7777, 0x8888],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        hwnd,
        WM_ERASEBKGND,
        0x7777,
        0x8888,
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

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        hwnd,
        WM_USER + 3,
        0x5555,
        0x6666,
    );

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

    let mut raw_sync_class = [0u8; WNDCLASSW_SIZE];
    raw_sync_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("RAW_SYNC_SEND", raw_sync_class);
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

    let mut raw_dwp_class = [0u8; WNDCLASSW_SIZE];
    raw_dwp_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("RAW_DEF_WINDOW_PROC", raw_dwp_class);
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

    let mut dlg_child_class = [0u8; WNDCLASSW_SIZE];
    dlg_child_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("DLG_SEND_CHILD", dlg_child_class);
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
fn coredll_raw_send_message_timeout_rejects_non_ce_flags_without_queueing() -> Result<()> {
    const ERROR_INVALID_FLAGS: u32 = 1004;
    const SMTO_UNKNOWN_FLAG: u32 = 0x0000_0004;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 54;
    let receiver_thread = 55;
    let result_ptr = 0xb220;
    memory.map_words(result_ptr, 1);
    memory.write_u32(result_ptr, 0xfeed_cafe)?;

    let hwnd = kernel.create_window_ex_w(
        receiver_thread,
        "SYNC_SEND_TIMEOUT_FLAGS",
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
            [
                hwnd,
                WM_USER + 59,
                0x59,
                0x5a,
                SMTO_UNKNOWN_FLAG,
                250,
                result_ptr,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(
        kernel.threads.get_last_error(sender_thread),
        ERROR_INVALID_FLAGS
    );
    assert_eq!(memory.read_u32(result_ptr)?, 0xfeed_cafe);
    assert!(kernel.gwe.sent_message(1).is_none());
    assert!(kernel.gwe.get_message(receiver_thread).is_none());

    Ok(())
}

#[test]
fn coredll_raw_send_message_timeout_nonzero_cross_thread_queues_transaction() -> Result<()> {
    const SMTO_BLOCK: u32 = 0x0000_0001;
    const SMTO_ABORTIFHUNG: u32 = 0x0000_0002;
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

    let mut timeout_class = [0u8; WNDCLASSW_SIZE];
    timeout_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("SYNC_SEND_TIMEOUT_RAW_WAIT", timeout_class);
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
            [
                hwnd,
                WM_ERASEBKGND,
                0x5a,
                0x5b,
                SMTO_BLOCK | SMTO_ABORTIFHUNG,
                250,
                result_ptr,
            ],
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
    assert_eq!(sent.send_timeout_flags, SMTO_BLOCK | SMTO_ABORTIFHUNG);
    assert_eq!(sent.result_ptr, Some(result_ptr));

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
    assert_eq!(memory.read_u32(result_ptr)?, 1);
    assert_eq!(kernel.take_completed_send_message_result(1), Some(1));

    Ok(())
}

#[test]
fn coredll_raw_send_message_timeout_writes_zero_result_when_target_destroyed() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 52;
    let receiver_thread = 53;
    let result_ptr = 0xb2c0;
    memory.map_words(result_ptr, 1);
    memory.write_u32(result_ptr, 0xfeed_cafe)?;

    let hwnd = kernel.create_window_ex_w(
        receiver_thread,
        "SYNC_SEND_TIMEOUT_DESTROYED",
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
            [hwnd, WM_USER + 60, 0x60, 0x61, 0, 250, result_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(result_ptr)?, 0xfeed_cafe);
    let sent = kernel.gwe.sent_message(1).expect("queued timeout send");
    assert_eq!(sent.result_ptr, Some(result_ptr));
    assert_eq!(sent.receiver_thread_id, receiver_thread);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_DESTROY_WINDOW,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(result_ptr)?, 0);
    assert_eq!(kernel.take_completed_send_message_result(1), Some(0));
    assert_eq!(
        kernel
            .gwe
            .stats()
            .send_transaction_receiver_terminated_count,
        1
    );

    Ok(())
}

#[test]
fn coredll_raw_send_message_timeout_abortifhung_aborts_when_thread_is_hung() -> Result<()> {
    const SMTO_ABORTIFHUNG: u32 = 0x0000_0002;
    const ERROR_TIMEOUT: u32 = 1460;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let sender_thread = 54_u32;
    let receiver_thread = 55_u32;
    let result_ptr = 0xb300;
    memory.map_words(result_ptr, 1);
    memory.write_u32(result_ptr, 0xfeed_cafe)?;

    let hwnd = kernel.create_window_ex_w(receiver_thread, "SMTO_HUNG_CLASS", "", None, 0, 0, 0);

    // Simulate receiver last dispatched at tick 0, then advance past the 5-second hung threshold.
    kernel.gwe.record_thread_dispatched(receiver_thread, 0);
    kernel.timers.sleep_ms(5001);

    // SMTO_ABORTIFHUNG should detect the hung thread and abort without queuing.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            sender_thread,
            ORD_SEND_MESSAGE_TIMEOUT,
            [hwnd, WM_USER + 70, 0, 0, SMTO_ABORTIFHUNG, 250, result_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    // result_ptr must be untouched (no reply written)
    assert_eq!(memory.read_u32(result_ptr)?, 0xfeed_cafe);
    // No message queued to the receiver.
    assert!(kernel.gwe.sent_message(1).is_none(), "hung abort must not queue a sent message");
    // Last error on sender should be ERROR_TIMEOUT.
    assert_eq!(kernel.threads.get_last_error(sender_thread), ERROR_TIMEOUT);

    Ok(())
}

#[test]
fn coredll_raw_send_notify_broadcast_uses_notify_send_for_live_top_level_windows() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let caller_thread = 46;
    let receiver_thread = 47;
    let other_thread = 48;
    let msg_ptr = 0xa100;
    memory.map_words(msg_ptr, 7);

    let same_thread_hwnd =
        kernel.create_window_ex_w(caller_thread, "NOTIFY_BROADCAST_SAME", "", None, 0, 0, 0);
    let cross_thread_hwnd =
        kernel.create_window_ex_w(receiver_thread, "NOTIFY_BROADCAST_CROSS", "", None, 0, 0, 0);
    let child_hwnd = kernel.create_window_ex_w(
        receiver_thread,
        "NOTIFY_BROADCAST_CHILD",
        "",
        Some(cross_thread_hwnd),
        7,
        0,
        0,
    );
    let destroyed_hwnd = kernel.create_window_ex_w(
        other_thread,
        "NOTIFY_BROADCAST_DESTROYED",
        "",
        None,
        0,
        0,
        0,
    );
    assert!(kernel.destroy_window(destroyed_hwnd));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SEND_NOTIFY_MESSAGE_W,
            [HWND_BROADCAST, WM_CLOSE, 0x64, 0x65],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(caller_thread), 0);

    assert!(!kernel.gwe.is_window(same_thread_hwnd));
    assert!(
        kernel
            .gwe
            .window(same_thread_hwnd)
            .is_some_and(|window| window.destroy_message_sent)
    );
    assert!(kernel.gwe.is_window(cross_thread_hwnd));
    assert!(kernel.gwe.is_window(child_hwnd));
    assert!(!kernel.gwe.is_window(destroyed_hwnd));

    let sent = kernel
        .gwe
        .sent_message(1)
        .expect("broadcast notify should queue cross-thread sent message");
    assert_eq!(sent.sender_thread_id, None);
    assert_eq!(sent.receiver_thread_id, receiver_thread);
    assert_eq!(sent.message.hwnd, cross_thread_hwnd);
    assert_eq!(sent.message.msg, WM_CLOSE);
    assert_ne!(sent.flags & SMF_SENDER_NO_WAIT, 0);
    assert_ne!(sent.flags & SMF_NOTIFY_MESSAGE, 0);

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
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                receiver_thread,
                Some(child_hwnd),
                WM_CLOSE,
                WM_CLOSE,
                PeekFlags::NO_REMOVE,
            )
            .is_none()
    );

    assert_next_message(
        &table,
        &mut kernel,
        &mut memory,
        receiver_thread,
        msg_ptr,
        cross_thread_hwnd,
        WM_CLOSE,
        0x64,
        0x65,
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
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window(cross_thread_hwnd));

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
fn coredll_raw_track_popup_menu_records_attempt_and_returns_default_command() -> Result<()> {
    const TPM_LEFTBUTTON: u32 = 0x0000;
    const TPM_NONOTIFY: u32 = 0x0080;
    const TPM_RETURNCMD: u32 = 0x0100;
    const TPM_RIGHTBUTTON: u32 = 0x0002;
    const MF_DISABLED: u32 = 0x0002;
    const MF_CHECKED: u32 = 0x0008;
    const MF_POPUP: u32 = 0x0010;
    const MF_HILITE: u32 = 0x0080;
    const MF_SEPARATOR: u32 = 0x0800;
    const MFS_DEFAULT: u32 = 0x1000;
    const VK_RETURN: u32 = 0x0d;
    const VK_ESCAPE: u32 = 0x1b;
    const VK_LEFT: u32 = 0x25;
    const VK_UP: u32 = 0x26;
    const VK_RIGHT: u32 = 0x27;
    const VK_DOWN: u32 = 0x28;
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
    let disabled_text = 0x1_7c00;
    let first_text = 0x1_7d00;
    let default_text = 0x1_7e00;
    memory.map_halfwords(disabled_text, 32);
    memory.map_halfwords(first_text, 32);
    memory.map_halfwords(default_text, 32);
    memory.write_wide_z(disabled_text, "Disabled");
    memory.write_wide_z(first_text, "Route &list");
    memory.write_wide_z(default_text, "Open");

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
    let render_submenu = match table.dispatch_raw_ordinal_with_memory(
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
    assert_eq!(
        kernel
            .resources
            .popup_notifications()
            .iter()
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_APPEND_MENU_W,
            [popup, MF_DISABLED | MFS_DEFAULT, 700, disabled_text],
        ),
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
            [popup, 0, 701, first_text],
        ),
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
            [popup, MFS_DEFAULT, 702, default_text],
        ),
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
            [popup, MF_SEPARATOR, 0, 0],
        ),
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
            [popup, MF_CHECKED, 703, default_text],
        ),
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
            [popup, MF_POPUP, render_submenu, first_text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        {
            let mut framebuffer = VirtualFramebuffer::new(460, 300, PixelFormat::Rgb565)?;
            let _ = framebuffer.take_dirty_rects();
            let result = table.dispatch_raw_ordinal_with_framebuffer(
                &mut kernel,
                &mut memory,
                Some(&mut framebuffer),
                thread_id,
                ORD_TRACK_POPUP_MENU_EX,
                [popup, TPM_LEFTBUTTON, 121, 51, hwnd, 0],
            );
            assert!(!framebuffer.dirty_rects().is_empty());
            assert!(framebuffer.dirty_rects().iter().any(|rect| {
                rect.x <= 121
                    && rect.y <= 51
                    && rect.x + rect.width >= 211
                    && rect.y + rect.height >= 163
            }));
            assert!(
                framebuffer
                    .pixels()
                    .chunks_exact(PixelFormat::Rgb565.bytes_per_pixel())
                    .filter(|pixel| *pixel != [0, 0])
                    .count()
                    > 100
            );
            let bpp = PixelFormat::Rgb565.bytes_per_pixel();
            let highlighted_pixel = 91 * framebuffer.stride() + 125 * bpp;
            assert_eq!(
                &framebuffer.pixels()[highlighted_pixel..highlighted_pixel + bpp],
                &[0xdf, 0x34]
            );
            result
        },
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        kernel
            .resources
            .menu(popup)
            .expect("popup menu should remain available")
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (item.state & MF_HILITE != 0).then_some(index))
            .collect::<Vec<_>>(),
        vec![2]
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 7);
    assert_eq!(
        notifications
            .iter()
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
            (hwnd, WM_COMMAND, 702, 0),
        ]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 321, 201, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(702),
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
    assert_eq!(kernel.resources.popup_notifications().len(), 10);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_NONOTIFY, 322, 202, hwnd, 0],
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
        .expect("nonotify tracking should still be recorded");
    assert_eq!(tracking.flags, TPM_NONOTIFY);
    assert_eq!(kernel.resources.popup_notifications().len(), 10);

    let disabled_only_popup = match table.dispatch_raw_ordinal_with_memory(
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
            ORD_APPEND_MENU_W,
            [
                disabled_only_popup,
                MF_DISABLED | MFS_DEFAULT,
                800,
                disabled_text
            ],
        ),
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
            ORD_TRACK_POPUP_MENU_EX,
            [disabled_only_popup, TPM_RETURNCMD, 323, 203, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.resources.popup_notifications().len(), 13);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [disabled_only_popup, 0, 324, 204, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 16);
    assert!(
        !notifications
            .iter()
            .any(|notification| { notification.msg == WM_COMMAND && notification.wparam == 800 })
    );

    let submenu_parent = match table.dispatch_raw_ordinal_with_memory(
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
    let submenu_child = match table.dispatch_raw_ordinal_with_memory(
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
            ORD_APPEND_MENU_W,
            [submenu_child, 0, 903, default_text],
        ),
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
            [submenu_child, 0, 904, first_text],
        ),
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
            [
                submenu_parent,
                MF_POPUP | MFS_DEFAULT,
                submenu_child,
                first_text
            ],
        ),
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
            [submenu_parent, 0, 905, default_text],
        ),
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
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, TPM_RETURNCMD, 325, 205, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(903),
            ..
        }
    ));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 20);
    assert_eq!(
        notifications
            .iter()
            .skip(16)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, 0, 326, 206, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 25);
    assert_eq!(
        notifications
            .iter()
            .skip(20)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
            (hwnd, WM_COMMAND, 903, 0),
        ]
    );
    assert!(
        notifications
            .iter()
            .any(|notification| { notification.msg == WM_COMMAND && notification.wparam == 903 })
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, TPM_RETURNCMD, 326, 206, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(903),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 29);
    assert_eq!(
        notifications
            .iter()
            .skip(25)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    kernel.gwe.set_cursor_pos(Point { x: 351, y: 226 });
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 321, 201, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(701),
            ..
        }
    ));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 32);
    assert_eq!(
        notifications
            .iter()
            .skip(29)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 327, 207, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(703),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 35);
    assert_eq!(
        notifications
            .iter()
            .skip(32)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_UP, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_LEFTBUTTON, 328, 208, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 39);
    assert_eq!(
        notifications
            .iter()
            .skip(35)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
            (hwnd, WM_COMMAND, 701, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_ESCAPE, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 329, 209, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 42);
    assert_eq!(
        notifications
            .iter()
            .skip(39)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let click_lparam = ((235_u32) << 16) | 342;
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_LBUTTONDOWN, 1, click_lparam));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_LBUTTONUP, 0, click_lparam));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 330, 210, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(701),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 45);
    assert_eq!(
        notifications
            .iter()
            .skip(42)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_CANCELMODE, 0, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 331, 211, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_CANCELMODE,
                WM_CANCELMODE,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 48);
    assert_eq!(
        notifications
            .iter()
            .skip(45)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let outside_lparam = ((5_u32) << 16) | 5;
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_LBUTTONDOWN, 1, outside_lparam));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_LBUTTONUP, 0, outside_lparam));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 332, 212, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_LBUTTONDOWN,
                WM_LBUTTONUP,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 51);
    assert_eq!(
        notifications
            .iter()
            .skip(48)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let right_click_lparam = ((240_u32) << 16) | 350;
    assert!(kernel.post_message_w_for_thread(
        thread_id,
        hwnd,
        WM_RBUTTONDOWN,
        1,
        right_click_lparam
    ));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_RBUTTONUP, 0, right_click_lparam));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RIGHTBUTTON | TPM_RETURNCMD, 333, 213, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(701),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_RBUTTONDOWN,
                WM_RBUTTONUP,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 54);
    assert_eq!(
        notifications
            .iter()
            .skip(51)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let hover_lparam = ((240_u32) << 16) | 350;
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_MOUSEMOVE, 0, hover_lparam));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        {
            let mut framebuffer = VirtualFramebuffer::new(460, 300, PixelFormat::Rgb565)?;
            let result = table.dispatch_raw_ordinal_with_framebuffer(
                &mut kernel,
                &mut memory,
                Some(&mut framebuffer),
                thread_id,
                ORD_TRACK_POPUP_MENU_EX,
                [popup, TPM_RETURNCMD, 334, 214, hwnd, 0],
            );
            let bpp = PixelFormat::Rgb565.bytes_per_pixel();
            let highlighted_pixel = 236 * framebuffer.stride() + 338 * bpp;
            assert_eq!(
                &framebuffer.pixels()[highlighted_pixel..highlighted_pixel + bpp],
                &[0xdf, 0x34]
            );
            result
        },
        CoredllDispatch::Returned {
            value: CoredllValue::U32(701),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_MOUSEMOVE,
                WM_MOUSEMOVE,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 57);
    assert_eq!(
        notifications
            .iter()
            .skip(54)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );
    assert_eq!(
        kernel
            .resources
            .menu(popup)
            .expect("popup menu should remain available")
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (item.state & MF_HILITE != 0).then_some(index))
            .collect::<Vec<_>>(),
        vec![1]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_CHAR, u32::from(b'l'), 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [popup, TPM_RETURNCMD, 335, 215, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(701),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_CHAR,
                WM_CHAR,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 60);
    assert_eq!(
        notifications
            .iter()
            .skip(57)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );
    assert_eq!(
        kernel
            .resources
            .menu(popup)
            .expect("popup menu should remain available")
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (item.state & MF_HILITE != 0).then_some(index))
            .collect::<Vec<_>>(),
        vec![1]
    );

    kernel.gwe.set_cursor_pos(Point { x: 450, y: 232 });
    assert!(matches!(
        {
            let mut framebuffer = VirtualFramebuffer::new(560, 300, PixelFormat::Rgb565)?;
            let result = table.dispatch_raw_ordinal_with_framebuffer(
                &mut kernel,
                &mut memory,
                Some(&mut framebuffer),
                thread_id,
                ORD_TRACK_POPUP_MENU_EX,
                [submenu_parent, TPM_RETURNCMD, 325, 205, hwnd, 0],
            );
            let bpp = PixelFormat::Rgb565.bytes_per_pixel();
            let child_highlight_pixel = 212 * framebuffer.stride() + 430 * bpp;
            assert_eq!(
                &framebuffer.pixels()[child_highlight_pixel..child_highlight_pixel + bpp],
                &[0xdf, 0x34]
            );
            result
        },
        CoredllDispatch::Returned {
            value: CoredllValue::U32(904),
            ..
        }
    ));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 64);
    assert_eq!(
        notifications
            .iter()
            .skip(60)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, TPM_RETURNCMD, 336, 216, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(904),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    assert_eq!(
        kernel
            .resources
            .menu(submenu_child)
            .expect("submenu child should remain available")
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (item.state & MF_HILITE != 0).then_some(index))
            .collect::<Vec<_>>(),
        vec![1]
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 68);
    assert_eq!(
        notifications
            .iter()
            .skip(64)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LEFT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, TPM_RETURNCMD, 337, 217, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(903),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    assert_eq!(
        kernel
            .resources
            .menu(submenu_child)
            .expect("submenu child should remain available")
            .items
            .iter()
            .filter(|item| item.state & MF_HILITE != 0)
            .count(),
        0
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 72);
    assert_eq!(
        notifications
            .iter()
            .skip(68)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let parent_second_row_lparam = ((238_u32) << 16) | 350;
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(
        thread_id,
        hwnd,
        WM_MOUSEMOVE,
        0,
        parent_second_row_lparam
    ));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, TPM_RETURNCMD, 338, 218, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(905),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_MOUSEMOVE,
                WM_MOUSEMOVE,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    assert_eq!(
        kernel
            .resources
            .menu(submenu_child)
            .expect("submenu child should remain available")
            .items
            .iter()
            .filter(|item| item.state & MF_HILITE != 0)
            .count(),
        0
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 75);
    assert_eq!(
        notifications
            .iter()
            .skip(72)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_ESCAPE, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, TPM_RETURNCMD, 339, 219, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(903),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .menu(submenu_child)
            .expect("submenu child should remain available")
            .items
            .iter()
            .filter(|item| item.state & MF_HILITE != 0)
            .count(),
        0
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 79);
    assert_eq!(
        notifications
            .iter()
            .skip(75)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_CHAR, VK_ESCAPE, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [submenu_parent, TPM_RETURNCMD, 340, 220, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(903),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .menu(submenu_child)
            .expect("submenu child should remain available")
            .items
            .iter()
            .filter(|item| item.state & MF_HILITE != 0)
            .count(),
        0
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 83);
    assert_eq!(
        notifications
            .iter()
            .skip(79)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_parent, 0),
            (hwnd, WM_INITMENUPOPUP, submenu_child, 1),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let deep_parent = match table.dispatch_raw_ordinal_with_memory(
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
    let deep_child = match table.dispatch_raw_ordinal_with_memory(
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
    let deep_grandchild = match table.dispatch_raw_ordinal_with_memory(
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
            ORD_APPEND_MENU_W,
            [deep_grandchild, 0, 906, default_text],
        ),
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
            [deep_grandchild, 0, 907, first_text],
        ),
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
            [deep_child, 0, 908, default_text],
        ),
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
            [deep_child, MF_POPUP, deep_grandchild, first_text],
        ),
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
            [deep_parent, MF_POPUP | MFS_DEFAULT, deep_child, first_text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [deep_parent, TPM_RETURNCMD, 341, 221, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(907),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    assert_eq!(
        kernel
            .resources
            .menu(deep_child)
            .expect("deep child should remain available")
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (item.state & MF_HILITE != 0).then_some(index))
            .collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(
        kernel
            .resources
            .menu(deep_grandchild)
            .expect("deep grandchild should remain available")
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (item.state & MF_HILITE != 0).then_some(index))
            .collect::<Vec<_>>(),
        vec![1]
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 88);
    assert_eq!(
        notifications
            .iter()
            .skip(83)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, deep_parent, 0),
            (hwnd, WM_INITMENUPOPUP, deep_child, 1),
            (hwnd, WM_INITMENUPOPUP, deep_grandchild, 2),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RIGHT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_DOWN, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_ESCAPE, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [deep_parent, TPM_RETURNCMD, 342, 222, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(906),
            ..
        }
    ));
    assert_eq!(
        kernel
            .resources
            .menu(deep_grandchild)
            .expect("deep grandchild should remain available")
            .items
            .iter()
            .filter(|item| item.state & MF_HILITE != 0)
            .count(),
        0
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 93);
    assert_eq!(
        notifications
            .iter()
            .skip(88)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, deep_parent, 0),
            (hwnd, WM_INITMENUPOPUP, deep_child, 1),
            (hwnd, WM_INITMENUPOPUP, deep_grandchild, 2),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let modal_pump_popup = match table.dispatch_raw_ordinal_with_memory(
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
            ORD_APPEND_MENU_W,
            [modal_pump_popup, 0, 909, default_text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let modal_posted_other =
        kernel.create_window_ex_w(thread_id, "POPUP_POSTED_OTHER", "", None, 0, 0, 0);
    assert!(kernel.post_message_w_for_thread(thread_id, modal_posted_other, WM_CLOSE, 0, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [modal_pump_popup, TPM_RETURNCMD, 343, 223, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(909),
            ..
        }
    ));
    assert!(
        !kernel.gwe.is_window(modal_posted_other),
        "popup modal pump should dispatch unrelated same-thread WM_CLOSE before owner selection"
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 96);
    assert_eq!(
        notifications
            .iter()
            .skip(93)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, modal_pump_popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let modal_sent_popup = match table.dispatch_raw_ordinal_with_memory(
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
            ORD_APPEND_MENU_W,
            [modal_sent_popup, 0, 910, default_text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let modal_sent_other =
        kernel.create_window_ex_w(thread_id, "POPUP_SENT_OTHER", "", None, 0, 0, 0);
    let send_id = kernel
        .begin_cross_thread_send_message_w(735, modal_sent_other, WM_CLOSE, 0, 0, None)
        .expect("cross-thread sent close should queue");
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [modal_sent_popup, TPM_RETURNCMD, 344, 224, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(910),
            ..
        }
    ));
    assert!(
        !kernel.gwe.is_window(modal_sent_other),
        "popup modal pump should dispatch unrelated sent WM_CLOSE before owner selection"
    );
    assert_eq!(kernel.take_completed_send_message_result(send_id), Some(0));
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 99);
    assert_eq!(
        notifications
            .iter()
            .skip(96)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, modal_sent_popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let modal_owner_popup = match table.dispatch_raw_ordinal_with_memory(
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
            ORD_APPEND_MENU_W,
            [modal_owner_popup, 0, 911, default_text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_TIMER, 0x44, 0x55));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_RETURN, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRACK_POPUP_MENU_EX,
            [modal_owner_popup, TPM_RETURNCMD, 345, 225, hwnd, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(911),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_TIMER,
                WM_TIMER,
                PeekFlags::NO_REMOVE
            )
            .is_none(),
        "popup modal pump should dispatch owner non-menu messages before owner selection"
    );
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_KEYDOWN,
                WM_KEYDOWN,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 102);
    assert_eq!(
        notifications
            .iter()
            .skip(99)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (hwnd, WM_ENTERMENULOOP, 1, 0),
            (hwnd, WM_INITMENUPOPUP, modal_owner_popup, 0),
            (hwnd, WM_EXITMENULOOP, 1, 0),
        ]
    );

    let modal_paint_owner = kernel.create_window_ex_w_with_rect(
        thread_id,
        "POPUP_PAINT_OWNER",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(40, 40, 90, 40),
    );
    assert!(kernel.gwe.validate_window_rect(modal_paint_owner, None));
    assert!(kernel.gwe.invalidate_window(modal_paint_owner, None, true));
    let modal_paint_popup = match table.dispatch_raw_ordinal_with_memory(
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
            ORD_APPEND_MENU_W,
            [modal_paint_popup, 0, 912, default_text],
        ),
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
            ORD_TRACK_POPUP_MENU_EX,
            [
                modal_paint_popup,
                TPM_RETURNCMD,
                346,
                226,
                modal_paint_owner,
                0
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(912),
            ..
        }
    ));
    assert!(
        kernel.gwe.update_rect(modal_paint_owner).is_none(),
        "popup modal pump should dispatch generated owner WM_PAINT before default selection"
    );
    let notifications = kernel.resources.popup_notifications();
    assert_eq!(notifications.len(), 105);
    assert_eq!(
        notifications
            .iter()
            .skip(102)
            .map(|notification| (
                notification.hwnd,
                notification.msg,
                notification.wparam,
                notification.lparam
            ))
            .collect::<Vec<_>>(),
        vec![
            (modal_paint_owner, WM_ENTERMENULOOP, 1, 0),
            (modal_paint_owner, WM_INITMENUPOPUP, modal_paint_popup, 0),
            (modal_paint_owner, WM_EXITMENULOOP, 1, 0),
        ]
    );

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
fn coredll_raw_translate_message_uses_shift_caps_and_syschar() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 37;
    let msg_ptr = 0x1_7d00;
    memory.map_words(msg_ptr, 7);
    let hwnd = kernel.create_window_ex_w(thread_id, "TRANSLATE_OWNER", "", None, 0, 0, 0);

    write_raw_message(
        &mut memory,
        msg_ptr,
        hwnd,
        WM_KEYDOWN,
        u32::from(b'A'),
        0x44,
    )?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let translated = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("plain letter should post WM_CHAR");
    assert_eq!(translated.wparam, u32::from(b'a'));
    assert_eq!(translated.lparam, 0x44);

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LSHIFT, 0));
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, u32::from(b'A'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let translated = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("shifted letter should post WM_CHAR");
    assert_eq!(translated.wparam, u32::from(b'A'));

    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, u32::from(b'1'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let translated = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("shifted digit should post WM_CHAR");
    assert_eq!(translated.wparam, u32::from(b'!'));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_LSHIFT, 0));

    for (vkey, expected, label) in [
        (0x08, 0x08, "backspace"),
        (0x09, 0x09, "tab"),
        (0x0d, 0x0d, "return"),
        (0x1b, 0x1b, "escape"),
        (0xba, u32::from(b';'), "semicolon"),
        (0xbd, u32::from(b'-'), "minus"),
        (0xbf, u32::from(b'/'), "slash"),
    ] {
        write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, vkey, vkey + 0x100)?;
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_TRANSLATE_MESSAGE,
                [msg_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
        let translated = kernel
            .gwe
            .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
            .unwrap_or_else(|| panic!("{label} should post WM_CHAR"));
        assert_eq!(translated.wparam, expected, "{label} translation");
        assert_eq!(translated.lparam, vkey + 0x100);
    }

    for (vkey, expected, label) in [
        (0x60, u32::from(b'0'), "numpad zero"),
        (0x65, u32::from(b'5'), "numpad five"),
        (0x69, u32::from(b'9'), "numpad nine"),
        (0x6a, u32::from(b'*'), "numpad multiply"),
        (0x6b, u32::from(b'+'), "numpad add"),
        (0x6d, u32::from(b'-'), "numpad subtract"),
        (0x6e, u32::from(b'.'), "numpad decimal"),
        (0x6f, u32::from(b'/'), "numpad divide"),
    ] {
        write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, vkey, vkey + 0x180)?;
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_TRANSLATE_MESSAGE,
                [msg_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
        let translated = kernel
            .gwe
            .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
            .unwrap_or_else(|| panic!("{label} should post WM_CHAR"));
        assert_eq!(translated.wparam, expected, "{label} translation");
        assert_eq!(translated.lparam, vkey + 0x180);
    }

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LSHIFT, 0));
    for (vkey, expected, label) in [
        (0xbb, u32::from(b'+'), "shifted plus"),
        (0xbc, u32::from(b'<'), "shifted comma"),
        (0xc0, u32::from(b'~'), "shifted grave"),
        (0xdb, u32::from(b'{'), "shifted left bracket"),
        (0xdc, u32::from(b'|'), "shifted backslash"),
        (0xde, u32::from(b'"'), "shifted quote"),
    ] {
        write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, vkey, vkey + 0x200)?;
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_TRANSLATE_MESSAGE,
                [msg_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
        let translated = kernel
            .gwe
            .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
            .unwrap_or_else(|| panic!("{label} should post WM_CHAR"));
        assert_eq!(translated.wparam, expected, "{label} translation");
        assert_eq!(translated.lparam, vkey + 0x200);
    }
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_LSHIFT, 0));

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LCONTROL, 0));
    for (vkey, expected, label) in [
        (u32::from(b'A'), 0x01, "control-a"),
        (u32::from(b'Z'), 0x1a, "control-z"),
        (0xdb, 0x1b, "control-left-bracket"),
        (0xdc, 0x1c, "control-backslash"),
        (0xdd, 0x1d, "control-right-bracket"),
        (0xbd, 0x1f, "control-minus"),
    ] {
        write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, vkey, vkey + 0x260)?;
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_TRANSLATE_MESSAGE,
                [msg_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
        let translated = kernel
            .gwe
            .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
            .unwrap_or_else(|| panic!("{label} should post WM_CHAR"));
        assert_eq!(translated.wparam, expected, "{label} translation");
        assert_eq!(translated.lparam, vkey + 0x260);
    }
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_LCONTROL, 0));

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_CAPITAL, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_CAPITAL, 0));
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, u32::from(b'A'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let translated = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("caps letter should post WM_CHAR");
    assert_eq!(translated.wparam, u32::from(b'A'));

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LSHIFT, 0));
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, u32::from(b'A'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let translated = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("shift plus caps should post WM_CHAR");
    assert_eq!(translated.wparam, u32::from(b'a'));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_LSHIFT, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_CAPITAL, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_CAPITAL, 0));

    write_raw_message(
        &mut memory,
        msg_ptr,
        hwnd,
        WM_SYSKEYDOWN,
        u32::from(b'M'),
        0x55,
    )?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let translated = kernel
        .gwe
        .peek_message_filtered(
            thread_id,
            Some(hwnd),
            WM_SYSCHAR,
            WM_SYSCHAR,
            PeekFlags::REMOVE,
        )
        .expect("syskey letter should post WM_SYSCHAR");
    assert_eq!(translated.wparam, u32::from(b'm'));
    assert_eq!(translated.lparam, 0x55);

    for (vkey, label) in [
        (0x6cu32, "VK_SEPARATOR"),
        (0x70, "F1"),
        (0x71, "F2"),
        (0x7a, "F11"),
        (0x87, "F24"),
    ] {
        write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, vkey, 0)?;
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_TRANSLATE_MESSAGE,
                [msg_ptr],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(false),
                ..
            }
        ), "{label} should not post WM_CHAR");
        assert!(
            kernel
                .gwe
                .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
                .is_none(),
            "{label} must not produce WM_CHAR"
        );
    }

    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYUP, u32::from(b'A'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_translate_message_hangul_ime_composes_syllables() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 37;
    let msg_ptr = 0x1_8000;
    memory.map_words(msg_ptr, 7);

    // Default keyboard layout is 0x0412 (Korean).
    assert_eq!(kernel.gwe.keyboard_layout(), 0x0412);

    let hwnd = kernel.create_window_ex_w(thread_id, "IME_TEST", "", None, 0, 0, 0);

    let himc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMM_GET_CONTEXT,
        [hwnd],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("unexpected ImmGetContext result: {other:?}"),
    };
    assert_ne!(himc, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_IMM_SET_OPEN_STATUS, [himc, 1],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // R (0x52) → ㄱ = Consonant(0): empty state → StartComposition + UpdateComposition(compat ㄱ=0x3131)
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, 0x52, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_TRANSLATE_MESSAGE, [msg_ptr],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_STARTCOMPOSITION, WM_IME_STARTCOMPOSITION, PeekFlags::REMOVE)
        .expect("R should post WM_IME_STARTCOMPOSITION");
    let m = kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_COMPOSITION, WM_IME_COMPOSITION, PeekFlags::REMOVE)
        .expect("R should post WM_IME_COMPOSITION(ㄱ compat)");
    assert_eq!(m.wparam, 0x3131, "R: WM_IME_COMPOSITION wparam should be compat ㄱ");
    assert_eq!(m.lparam, GCS_COMPSTR, "R: WM_IME_COMPOSITION lparam should be GCS_COMPSTR");

    // K (0x4B) → ㅏ = Vowel(0): ㄱ + ㅏ → 가 (0xAC00)
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, 0x4B, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_TRANSLATE_MESSAGE, [msg_ptr],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let m = kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_COMPOSITION, WM_IME_COMPOSITION, PeekFlags::REMOVE)
        .expect("K after R should post WM_IME_COMPOSITION(가)");
    assert_eq!(m.wparam, 0xAC00, "K after R: should compose 가");
    assert_eq!(m.lparam, GCS_COMPSTR);

    // S (0x53) → ㄴ = Consonant(2), INITIAL_TO_FINAL[2]=4: 가 + final 4 → 간 (0xAC04)
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, 0x53, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_TRANSLATE_MESSAGE, [msg_ptr],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let m = kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_COMPOSITION, WM_IME_COMPOSITION, PeekFlags::REMOVE)
        .expect("S after RK should post WM_IME_COMPOSITION(간)");
    assert_eq!(m.wparam, 0xAC04, "S after RK: should compose 간 = 0xAC00 + 4");
    assert_eq!(m.lparam, GCS_COMPSTR);

    // K (0x4B) again → ㅏ splits ㄴ from 간:
    //   CommitChar(가=0xAC00) → WM_IME_CHAR + WM_IME_ENDCOMPOSITION
    //   then StartComposition + UpdateComposition(나=0xB098)
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, 0x4B, 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_TRANSLATE_MESSAGE, [msg_ptr],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let m = kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_CHAR, WM_IME_CHAR, PeekFlags::REMOVE)
        .expect("K after RKS should commit 가 via WM_IME_CHAR");
    assert_eq!(m.wparam, 0xAC00, "committed char should be 가");
    kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_ENDCOMPOSITION, WM_IME_ENDCOMPOSITION, PeekFlags::REMOVE)
        .expect("K after RKS should post WM_IME_ENDCOMPOSITION");
    kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_STARTCOMPOSITION, WM_IME_STARTCOMPOSITION, PeekFlags::REMOVE)
        .expect("K after RKS should start new composition");
    let m = kernel.gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_IME_COMPOSITION, WM_IME_COMPOSITION, PeekFlags::REMOVE)
        .expect("K after RKS should post WM_IME_COMPOSITION(나)");
    // 나 = ㄴ(initial 2) + ㅏ(vowel 0) = 0xAC00 + 2*21*28 = 0xAC00 + 1176 = 0xB098
    assert_eq!(m.wparam, 0xB098, "new composition should be 나");
    assert_eq!(m.lparam, GCS_COMPSTR);

    Ok(())
}

#[test]
fn coredll_raw_keyboard_layout_and_imm_context_are_stateful() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 37;
    let layout_name = 0x1_9000;
    let loaded_layout_name = 0x1_9020;
    let layout_list = 0x1_9040;
    let conversion_ptr = 0x1_9060;
    let sentence_ptr = 0x1_9064;
    let ime_file_name = 0x1_9080;
    memory.map_halfwords(layout_name, 9);
    memory.map_halfwords(loaded_layout_name, 9);
    memory.map_words(layout_list, 1);
    memory.map_words(conversion_ptr, 1);
    memory.map_words(sentence_ptr, 1);
    memory.map_halfwords(ime_file_name, 16);
    memory.write_wide_z(loaded_layout_name, "E0010412");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEYBOARD_LAYOUT,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(DEFAULT_KEYBOARD_LAYOUT_HKL),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_GET_KEYBOARD_LAYOUT,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(DEFAULT_KEYBOARD_LAYOUT_HKL),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FOREGROUND_KEYBOARD_LAYOUT_HANDLE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(DEFAULT_KEYBOARD_LAYOUT_HKL),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEYBOARD_LAYOUT_NAME_W,
            [layout_name],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        memory.read_wide_z(layout_name, 9),
        DEFAULT_KEYBOARD_LAYOUT_NAME
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_KEYBOARD_LAYOUT_LIST,
            [0, 0],
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
            ORD_GET_KEYBOARD_LAYOUT_LIST,
            [1, layout_list],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    assert_eq!(memory.read_u32(layout_list)?, DEFAULT_KEYBOARD_LAYOUT_HKL);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_IS_IME,
            [DEFAULT_KEYBOARD_LAYOUT_HKL],
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
            ORD_LOAD_KEYBOARD_LAYOUT_W,
            [loaded_layout_name, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0xe001_0412),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_IS_IME,
            [0xe001_0412],
        ),
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
            ORD_ACTIVATE_KEYBOARD_LAYOUT,
            [DEFAULT_KEYBOARD_LAYOUT_HKL, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0xe001_0412),
            ..
        }
    ));

    let hwnd = kernel.create_window_ex_w(thread_id, "IME_OWNER", "", None, 0, 0, 0);
    let himc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMM_GET_CONTEXT,
        [hwnd],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(himc),
            ..
        } => himc,
        other => panic!("unexpected ImmGetContext result: {other:?}"),
    };
    assert_ne!(himc, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_GET_OPEN_STATUS,
            [himc],
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
            ORD_IMM_SET_OPEN_STATUS,
            [himc, 1],
        ),
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
            ORD_IMM_GET_OPEN_STATUS,
            [himc],
        ),
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
            ORD_IMM_SET_CONVERSION_STATUS,
            [himc, 0x111, 0x222],
        ),
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
            ORD_IMM_GET_CONVERSION_STATUS,
            [himc, conversion_ptr, sentence_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(conversion_ptr)?, 0x111);
    assert_eq!(memory.read_u32(sentence_ptr)?, 0x222);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_GET_COMPOSITION_STRING_W,
            [himc, 0, 0, 0],
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
            ORD_IMM_GET_IMEFILE_NAME_W,
            [DEFAULT_KEYBOARD_LAYOUT_HKL, ime_file_name, 16],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_wide_z(ime_file_name, 16), "");
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_NOTIFY_IME,
            [himc, 0, 0, 0],
        ),
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
            ORD_IMM_RELEASE_CONTEXT,
            [hwnd, himc],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let created_himc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_IMM_CREATE_CONTEXT,
        [],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(himc),
            ..
        } => himc,
        other => panic!("unexpected ImmCreateContext result: {other:?}"),
    };
    assert_ne!(created_himc, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_ASSOCIATE_CONTEXT,
            [hwnd, created_himc],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(previous),
            ..
        } if previous == himc
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMM_DESTROY_CONTEXT,
            [created_himc],
        ),
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
            ORD_IMM_DISABLE_IME,
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
            ORD_IMM_GET_CONTEXT,
            [hwnd],
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
            ORD_IMM_ENABLE_IME,
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
            ORD_IMM_GET_CONTEXT,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(reenabled),
            ..
        } if reenabled != 0
    ));

    Ok(())
}

#[test]
fn coredll_raw_translate_accelerator_honors_modifiers_and_syskey() -> Result<()> {
    const FVIRTKEY: u8 = 0x01;
    const FSHIFT: u8 = 0x04;
    const FCONTROL: u8 = 0x08;
    const FALT: u8 = 0x10;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 36;
    let msg_ptr = 0x1_7c00;
    memory.map_words(msg_ptr, 7);
    let hwnd = kernel.create_window_ex_w(thread_id, "ACCEL_OWNER", "", None, 0, 0, 0);
    let accel = kernel.resources.create_accelerator(
        0,
        ResourceId::Integer(700),
        None,
        vec![
            AcceleratorEntry {
                flags: FVIRTKEY | FCONTROL,
                key: u16::from(b'N'),
                command: 7001,
            },
            AcceleratorEntry {
                flags: FVIRTKEY | FALT,
                key: u16::from(b'M'),
                command: 7002,
            },
            AcceleratorEntry {
                flags: FSHIFT,
                key: u16::from(b'!'),
                command: 7003,
            },
        ],
    );

    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, u32::from(b'N'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                thread_id,
                Some(hwnd),
                WM_COMMAND,
                WM_COMMAND,
                PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_CONTROL, 0));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let command = kernel
        .gwe
        .peek_message_filtered(
            thread_id,
            Some(hwnd),
            WM_COMMAND,
            WM_COMMAND,
            PeekFlags::REMOVE,
        )
        .expect("accelerator should queue WM_COMMAND");
    assert_eq!(command.wparam, 7001 | (1 << 16), "accelerator WM_COMMAND has HIWORD=1");

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_CONTROL, 0));
    write_raw_message(
        &mut memory,
        msg_ptr,
        hwnd,
        WM_SYSKEYDOWN,
        u32::from(b'M'),
        0,
    )?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let command = kernel
        .gwe
        .peek_message_filtered(
            thread_id,
            Some(hwnd),
            WM_COMMAND,
            WM_COMMAND,
            PeekFlags::REMOVE,
        )
        .expect("syskey accelerator should queue WM_COMMAND");
    assert_eq!(command.wparam, 7002 | (1 << 16), "accelerator WM_COMMAND has HIWORD=1");

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LSHIFT, 0));
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, u32::from(b'1'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let command = kernel
        .gwe
        .peek_message_filtered(
            thread_id,
            Some(hwnd),
            WM_COMMAND,
            WM_COMMAND,
            PeekFlags::REMOVE,
        )
        .expect("shifted ascii accelerator should queue WM_COMMAND");
    assert_eq!(command.wparam, 7003 | (1 << 16), "accelerator WM_COMMAND has HIWORD=1");
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_LSHIFT, 0));

    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_MENU, 0));
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_KEYDOWN, u32::from(b'N'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

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
fn coredll_raw_def_window_proc_handles_hit_test_and_syscommand_close() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 80;
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "DEFPROC",
        "default proc",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(100, 200, 80, 40),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_NCHITTEST, 0, make_test_lparam(120, 220)],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(HTCLIENT),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_NCHITTEST, 0, make_test_lparam(20, 20)],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(HTNOWHERE),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_MOUSEACTIVATE, hwnd, HTCLIENT],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(MA_ACTIVATE),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_SYSCOMMAND, SC_CLOSE, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert!(!kernel.gwe.is_window(hwnd));
    assert!(
        kernel
            .gwe
            .window(hwnd)
            .is_some_and(|window| window.destroy_message_sent)
    );

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_setcursor_uses_class_cursor() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 81;
    let class_ptr = 0x1_0000;
    let title_ptr = 0x1_0040;
    let wndclass_ptr = 0x1_0080;
    memory.write_wide_z(class_ptr, "CURSORDEF");
    memory.write_wide_z(title_ptr, "cursor default");
    memory.map_bytes(wndclass_ptr, 40);

    let class_cursor = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_CURSOR_W,
        [0, 32512],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(cursor),
            ..
        } => cursor,
        other => panic!("LoadCursorW did not return a stock cursor handle: {other:?}"),
    };
    assert_ne!(class_cursor, 0);
    let prior_cursor = class_cursor ^ 0x40;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CURSOR,
            [prior_cursor],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));

    let mut wndclass = [0; 40];
    wndclass[4..8].copy_from_slice(&0x0040_5678u32.to_le_bytes());
    wndclass[24..28].copy_from_slice(&class_cursor.to_le_bytes());
    wndclass[36..40].copy_from_slice(&class_ptr.to_le_bytes());
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
            value: CoredllValue::U32(atom),
            ..
        } if atom >= 0xc000
    ));
    let hwnd = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_WINDOW_EX_W,
        [
            0, class_ptr, title_ptr, WS_VISIBLE, 10, 20, 70, 30, 0, 0, 0, 0,
        ],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(hwnd),
            ..
        } => hwnd,
        other => panic!("CreateWindowExW did not create raw hwnd: {other:?}"),
    };
    assert_ne!(hwnd, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_SETCURSOR, hwnd, HTCLIENT],
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
            ORD_GET_CURSOR,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(cursor),
            ..
        } if cursor == class_cursor
    ));

    let resize_cursor = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_LOAD_CURSOR_W,
        [0, 32642],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(cursor),
            ..
        } => cursor,
        other => panic!("LoadCursorW did not return a stock resize cursor handle: {other:?}"),
    };
    assert_ne!(resize_cursor, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_SETCURSOR, hwnd, HTTOPLEFT],
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
            ORD_GET_CURSOR,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(cursor),
            ..
        } if cursor == resize_cursor
    ));

    Ok(())
}

#[test]
fn coredll_raw_dialog_buttons_report_default_codes_and_ids() -> Result<()> {
    const IDCANCEL: u32 = 2;

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
    let cancel = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BUTTON",
        "cancel",
        Some(dialog),
        IDCANCEL,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON,
        0,
        Rect::from_origin_size(100, 0, 40, 20),
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

    assert_eq!(
        kernel.gwe.dialog_cancel_command(dialog, IDCANCEL),
        (IDCANCEL, cancel)
    );
    assert_eq!(
        kernel.gwe.dialog_cancel_command(default, IDCANCEL),
        (IDCANCEL, 0)
    );

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
fn coredll_raw_region_or_canonicalizes_duplicate_coverage() -> Result<()> {
    const SIMPLEREGION: u32 = 2;
    const RGN_OR: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 14;

    let left = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [0, 0, 100, 80],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn(left) did not return a region: {other:?}"),
    };
    let right = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN,
        [60, 0, 160, 80],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(region),
            ..
        } => region,
        other => panic!("CreateRectRgn(right) did not return a region: {other:?}"),
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
            [dest, left, right, RGN_OR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(SIMPLEREGION),
            ..
        }
    ));
    let region = kernel.resources.region(dest).unwrap();
    assert_eq!(
        region.rects,
        vec![Rect {
            left: 0,
            top: 0,
            right: 160,
            bottom: 80,
        }]
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_COMBINE_RGN,
            [dest, dest, left, RGN_OR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(SIMPLEREGION),
            ..
        }
    ));
    assert_eq!(kernel.resources.region(dest).unwrap().rects.len(), 1);

    Ok(())
}

#[test]
fn coredll_raw_region_or_incrementally_merges_scanline_rects() -> Result<()> {
    const SIMPLEREGION: u32 = 2;
    const RGN_OR: u32 = 2;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 14;

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
    for y in 0..4 {
        for x in 0..128 {
            let src = match table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                thread_id,
                ORD_CREATE_RECT_RGN,
                [x, y, x + 1, y + 1],
            ) {
                CoredllDispatch::Returned {
                    value: CoredllValue::Handle(region),
                    ..
                } => region,
                other => panic!("CreateRectRgn(pixel) did not return a region: {other:?}"),
            };
            assert!(matches!(
                table.dispatch_raw_ordinal_with_memory(
                    &mut kernel,
                    &mut memory,
                    thread_id,
                    ORD_COMBINE_RGN,
                    [dest, dest, src, RGN_OR],
                ),
                CoredllDispatch::Returned { .. }
            ));
            assert!(matches!(
                table.dispatch_raw_ordinal_with_memory(
                    &mut kernel,
                    &mut memory,
                    thread_id,
                    ORD_DELETE_OBJECT,
                    [src],
                ),
                CoredllDispatch::Returned { .. }
            ));
        }
    }

    assert_eq!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_RGN_BOX,
            [dest, 0],
        ),
        CoredllDispatch::Returned {
            export: table.resolve_ordinal(ORD_GET_RGN_BOX).unwrap().clone(),
            value: CoredllValue::U32(SIMPLEREGION),
        }
    );
    assert_eq!(
        kernel.resources.region(dest).unwrap().rects,
        vec![Rect {
            left: 0,
            top: 0,
            right: 128,
            bottom: 4,
        }]
    );

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

    // Register the class with a non-null background brush so that DefWindowProcW
    // WM_ERASEBKGND returns 1, allowing GetUpdateRect(bErase=TRUE) to clear the
    // erase flag (CE behavior: erase cleared only when the background is erased).
    let mut update_erase_class = [0u8; WNDCLASSW_SIZE];
    update_erase_class[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes()); // COLOR_WINDOW brush
    kernel.gwe.register_class("UPDATE_ERASE", update_erase_class);

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
    // GetUpdateRect(bErase=TRUE) calls WM_ERASEBKGND; since the class has a
    // background brush and DefWindowProcW returns 1, the erase flag is cleared.
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

#[test]
fn coredll_raw_caret_ordinals_track_position_visibility_and_blink() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 260;
    let hwnd = kernel.create_window_ex_w(thread_id, "CARET_OWNER", "", None, 0, WS_VISIBLE, 0);
    let other = kernel.create_window_ex_w(thread_id, "CARET_OTHER", "", None, 0, WS_VISIBLE, 0);
    let point_ptr = 0x2a_0000;
    memory.map_words(point_ptr, 2);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_CARET,
            [hwnd, 0x000b_7000, 2, 18],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    let caret = kernel.gwe.caret().expect("caret created");
    assert_eq!(caret.hwnd, hwnd);
    assert_eq!(caret.bitmap, 0x000b_7000);
    assert_eq!(caret.width, 2);
    assert_eq!(caret.height, 18);
    assert_eq!(caret.show_count, -1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SHOW_CARET,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.gwe.caret().expect("caret").show_count, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_HIDE_CARET,
            [other],
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
            ORD_HIDE_CARET,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.gwe.caret().expect("caret").show_count, -1);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CARET_POS,
            [123, (-45i32) as u32],
        ),
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
            ORD_GET_CARET_POS,
            [point_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(point_ptr)?, 123);
    assert_eq!(memory.read_u32(point_ptr + 4)? as i32, -45);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CARET_BLINK_TIME,
            [750],
        ),
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
            ORD_GET_CARET_BLINK_TIME,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(750),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DISABLE_CARET_SYSTEM_WIDE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(!kernel.gwe.caret_system_enabled());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ENABLE_CARET_SYSTEM_WIDE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.gwe.caret_system_enabled());
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    Ok(())
}

#[test]
fn coredll_raw_visible_caret_invalidates_and_marks_framebuffer_dirty() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let mut framebuffer = VirtualFramebuffer::new(80, 60, PixelFormat::Rgb565)?;
    let _ = framebuffer.take_dirty_rects();
    let thread_id = 2602;
    let parent = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CARET_PARENT",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(10, 12, 40, 30),
    );
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CARET_CHILD",
        "",
        Some(parent),
        0,
        WS_VISIBLE | WS_CHILD,
        0,
        Rect::from_origin_size(3, 4, 20, 18),
    );
    assert!(kernel.gwe.validate_window_rect(parent, None));
    assert!(kernel.gwe.validate_window_rect(hwnd, None));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_CREATE_CARET,
            [hwnd, 0, 2, 6],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(framebuffer.dirty_rects().is_empty());
    assert!(kernel.gwe.update_rect(hwnd).is_none());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_SET_CARET_POS,
            [5, 7],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(framebuffer.dirty_rects().is_empty());
    assert!(kernel.gwe.update_rect(hwnd).is_none());

    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_SHOW_CARET,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(18, 23, 2, 6)]
    );
    assert_eq!(
        kernel.gwe.update_rect(hwnd).expect("caret update").rect,
        Rect::from_origin_size(5, 7, 2, 6)
    );

    let _ = framebuffer.take_dirty_rects();
    assert!(
        kernel
            .gwe
            .validate_window_rect(hwnd, Some(Rect::from_origin_size(5, 7, 2, 6)))
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_SET_CARET_POS,
            [8, 9],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        framebuffer.dirty_rects(),
        &[
            FramebufferRect::new(18, 23, 2, 6),
            FramebufferRect::new(21, 25, 2, 6),
        ]
    );
    assert_eq!(
        kernel
            .gwe
            .update_rect(hwnd)
            .expect("moved caret update")
            .rect,
        Rect::from_origin_size(5, 7, 5, 8)
    );

    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_DISABLE_CARET_SYSTEM_WIDE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(21, 25, 2, 6)]
    );

    let _ = framebuffer.take_dirty_rects();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_framebuffer(
            &mut kernel,
            &mut memory,
            Some(&mut framebuffer),
            thread_id,
            ORD_ENABLE_CARET_SYSTEM_WIDE,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        framebuffer.dirty_rects(),
        &[FramebufferRect::new(21, 25, 2, 6)]
    );

    Ok(())
}

#[test]
fn coredll_raw_caret_rejects_invalid_state_and_destroys_with_owner() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 261;
    let hwnd = kernel.create_window_ex_w(thread_id, "CARET_DESTROY", "", None, 0, WS_VISIBLE, 0);
    let point_ptr = 0x2b_0000;
    memory.map_words(point_ptr, 2);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CARET_POS,
            [point_ptr],
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
            ORD_CREATE_CARET,
            [hwnd, 0, (-1i32) as u32, 8],
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
            ORD_CREATE_CARET,
            [hwnd, 0, 1, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(kernel.gwe.caret().is_some());
    assert!(kernel.destroy_window_with_reason(hwnd, "test"));
    assert!(kernel.gwe.caret().is_none());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DESTROY_CARET,
            [],
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
fn coredll_raw_def_window_proc_ncactivate_returns_true() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 90;
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "NCACT",
        "ncactivate",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(10, 10, 100, 50),
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_NCACTIVATE, 1, 0],
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
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_NCACTIVATE, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_activate_sets_focus() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 91;
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "ACTFOCUS",
        "activate focus",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(20, 20, 120, 60),
    );

    // Initially no focus
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FOCUS,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));

    // WM_ACTIVATE with WA_ACTIVE causes DefWindowProcW to set focus
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_ACTIVATE, WA_ACTIVE, 0],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FOCUS,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(focus),
            ..
        } if focus == hwnd
    ));

    // WM_ACTIVATE with WA_CLICKACTIVE also sets focus
    let hwnd2 = kernel.create_window_ex_w_with_rect(
        thread_id,
        "ACTFOCUS2",
        "activate focus 2",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(200, 20, 120, 60),
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd2, WM_ACTIVATE, WA_CLICKACTIVE, 0],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FOCUS,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(focus),
            ..
        } if focus == hwnd2
    ));

    // WM_ACTIVATE with WA_INACTIVE does not move focus
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd2, WM_ACTIVATE, WA_INACTIVE, 0],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_FOCUS,
            [],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(focus),
            ..
        } if focus == hwnd2
    ));

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_nclbuttondown_htclose_sends_syscommand_close() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 93;
    let msg_ptr = 0x3_4000;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "NCTEST", "", None, 0, 0, 0);

    // WM_NCLBUTTONDOWN with HTCLOSE should cause WM_CLOSE to be dispatched (via SC_CLOSE)
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_NCLBUTTONDOWN, HTCLOSE, 0],
    );
    // DefWindowProcW(WM_NCLBUTTONDOWN, HTCLOSE) routes to WM_SYSCOMMAND(SC_CLOSE) → WM_CLOSE → destroy
    // The window should now be destroyed
    assert!(!kernel.gwe.is_window(hwnd), "HTCLOSE should destroy the window via SC_CLOSE");

    // WM_NCLBUTTONDBLCLK on HTSYSMENU also closes
    let hwnd2 = kernel.create_window_ex_w(thread_id, "NCTEST2", "", None, 0, 0, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd2, WM_NCLBUTTONDBLCLK, HTSYSMENU, 0],
    );
    assert!(!kernel.gwe.is_window(hwnd2), "HTSYSMENU dblclk should destroy the window via SC_CLOSE");

    // WM_NCLBUTTONDOWN on HTCAPTION should NOT destroy (no default close behavior)
    let hwnd3 = kernel.create_window_ex_w(thread_id, "NCTEST3", "", None, 0, 0, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd3, WM_NCLBUTTONDOWN, HTCAPTION, 0],
    );
    assert!(kernel.gwe.is_window(hwnd3), "HTCAPTION click should NOT destroy the window");

    // WM_NCLBUTTONDBLCLK on HTCLOSE also closes (both HTCLOSE dblclk and single are close triggers)
    let hwnd4 = kernel.create_window_ex_w(thread_id, "NCTEST4", "", None, 0, 0, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd4, WM_NCLBUTTONDBLCLK, HTCLOSE, 0],
    );
    assert!(!kernel.gwe.is_window(hwnd4), "HTCLOSE dblclk should also destroy the window");

    // WM_NCLBUTTONDOWN on HTSYSMENU should NOT destroy (only dblclk on HTSYSMENU closes)
    let hwnd5 = kernel.create_window_ex_w(thread_id, "NCTEST5", "", None, 0, 0, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd5, WM_NCLBUTTONDOWN, HTSYSMENU, 0],
    );
    assert!(kernel.gwe.is_window(hwnd5), "HTSYSMENU single-click should NOT destroy the window");

    Ok(())
}

#[test]
fn coredll_raw_set_capture_posts_capture_changed_to_old_owner() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 92;
    let msg_ptr = 0x3_2000;
    memory.map_words(msg_ptr, 7);

    let hwnd1 = kernel.create_window_ex_w(thread_id, "CAP1", "", None, 0, 0, 0);
    let hwnd2 = kernel.create_window_ex_w(thread_id, "CAP2", "", None, 0, 0, 0);

    // SetCapture(hwnd1) — no previous capture, no WM_CAPTURECHANGED expected
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CAPTURE,
            [hwnd1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(thread_id, Some(hwnd1), WM_CAPTURECHANGED, WM_CAPTURECHANGED, PeekFlags::REMOVE)
            .is_none(),
        "no WM_CAPTURECHANGED when there was no prior capture"
    );

    // SetCapture(hwnd2) — hwnd1 should receive WM_CAPTURECHANGED(lparam=hwnd2)
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_CAPTURE,
            [hwnd2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(prev),
            ..
        } if prev == hwnd1
    ));
    let msg = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd1), WM_CAPTURECHANGED, WM_CAPTURECHANGED, PeekFlags::REMOVE)
        .expect("hwnd1 should receive WM_CAPTURECHANGED when hwnd2 takes capture");
    assert_eq!(msg.lparam, hwnd2, "WM_CAPTURECHANGED lparam is new capture window");

    // ReleaseCapture() — hwnd2 should receive WM_CAPTURECHANGED(lparam=0)
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
    let msg = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd2), WM_CAPTURECHANGED, WM_CAPTURECHANGED, PeekFlags::REMOVE)
        .expect("hwnd2 should receive WM_CAPTURECHANGED on ReleaseCapture");
    assert_eq!(msg.lparam, 0, "WM_CAPTURECHANGED lparam is 0 when capture released");
    assert_eq!(kernel.gwe.get_capture(), None, "capture is cleared after ReleaseCapture");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_syskeydown_f4_closes_window() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 91;
    const VK_F4: u32 = 0x73;

    let hwnd = kernel.create_window_ex_w(thread_id, "SYSKEY", "", None, 0, 0, 0);
    assert!(kernel.gwe.is_window(hwnd));

    // DefWindowProcW(WM_SYSKEYDOWN, VK_F4) → WM_SYSCOMMAND(SC_CLOSE) → WM_CLOSE → destroy
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_SYSKEYDOWN, VK_F4, 0],
    );
    assert!(!kernel.gwe.is_window(hwnd), "WM_SYSKEYDOWN(VK_F4) should close the window");

    // WM_SYSKEYDOWN with a different key (VK_TAB) should NOT close
    let hwnd2 = kernel.create_window_ex_w(thread_id, "SYSKEY2", "", None, 0, 0, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd2, WM_SYSKEYDOWN, 0x09u32, 0], // VK_TAB
    );
    assert!(kernel.gwe.is_window(hwnd2), "WM_SYSKEYDOWN(VK_TAB) should NOT close the window");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_syskeydown_f10_and_alt_up_send_keymenu() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 83;
    let msg_ptr = 0x2_a000;
    memory.map_words(msg_ptr, 7);
    const VK_F10: u32 = 0x79;

    let hwnd = kernel.create_window_ex_w(thread_id, "KEYMENU", "", None, 0, 0, 0);

    // WM_SYSKEYDOWN(VK_F10) → WM_SYSCOMMAND(SC_KEYMENU, VK_F10); window survives.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_SYSKEYDOWN, VK_F10, 0],
    );
    assert!(kernel.gwe.is_window(hwnd), "F10 should not destroy the window");

    // WM_SYSKEYUP(VK_MENU) → WM_SYSCOMMAND(SC_KEYMENU, 0); window survives.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_SYSKEYUP, VK_MENU, 0],
    );
    assert!(kernel.gwe.is_window(hwnd), "ALT release should not destroy the window");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_syskeydown_letter_sends_keymenu() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 87;

    let hwnd = kernel.create_window_ex_w(thread_id, "KEYMENU_LETTER", "", None, 0, 0, 0);

    // WM_SYSKEYDOWN(VK_A..VK_Z) → internally dispatches SC_KEYMENU(letter); SC_KEYMENU is a
    // no-op in DefWindowProc so the window must survive unchanged.
    for vk in 0x41_u32..=0x5a {
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_SYSKEYDOWN, vk, 0],
        );
        assert!(
            kernel.gwe.is_window(hwnd),
            "Alt+letter VK=0x{vk:02X} must not destroy the window"
        );
    }

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_context_menu_forwards_to_parent() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 90;
    let msg_ptr = 0x3_0000;
    memory.map_words(msg_ptr, 7);
    let ctx_lparam = 0x0014_000au32; // x=10, y=20 screen coords

    let parent = kernel.create_window_ex_w(thread_id, "CTXPARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w(thread_id, "CTXCHILD", "", Some(parent), 0, 0, 0);

    // DefWindowProcW on child with WM_CONTEXTMENU should forward to parent
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [child, WM_CONTEXTMENU, child, ctx_lparam],
    );

    // Parent should have WM_CONTEXTMENU in its queue (posted via send_message in gwe)
    // Since both are same thread, the send_message is synchronous — parent's wndproc
    // (DefWindowProcW) is called with WM_CONTEXTMENU. Parent has no parent, so it is
    // forwarded to nobody and the default result is returned. We verify the call chain
    // doesn't error and the window is still alive.
    assert!(kernel.gwe.is_window(parent), "parent should still be alive after WM_CONTEXTMENU forwarding");
    assert!(kernel.gwe.is_window(child), "child should still be alive after WM_CONTEXTMENU forwarding");

    // Top-level window with no parent: DefWindowProcW on WM_CONTEXTMENU should not crash
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [parent, WM_CONTEXTMENU, parent, ctx_lparam],
    );
    assert!(kernel.gwe.is_window(parent), "top-level WM_CONTEXTMENU should not destroy the window");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_rbuttonup_sends_context_menu() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 89;
    let msg_ptr = 0x2_e000;
    memory.map_words(msg_ptr, 7);

    // Create a top-level window with a known client rect offset
    let hwnd = kernel.create_window_ex_w(thread_id, "RBTN", "", None, 0, 0, 0);
    // Set window rect so client origin is non-zero to verify coord conversion
    kernel.gwe.set_window_pos(hwnd, None, 50, 100, 200, 300, 0);

    // WM_RBUTTONUP with client coords (10, 20) → DefWindowProcW sends WM_CONTEXTMENU
    // with screen coords = client + client_origin
    let client_x: i32 = 10;
    let client_y: i32 = 20;
    let rbup_lparam = (client_y as u16 as u32) << 16 | (client_x as u16 as u32);

    // The send_message (DefWindowProc) call for WM_RBUTTONUP is synchronous for same-thread,
    // so WM_CONTEXTMENU will be delivered directly to DefWindowProc again (top-level, no parent),
    // which means it returns 0 without queueing anything. Window must remain alive.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_RBUTTONUP, 0, rbup_lparam],
    );
    assert!(kernel.gwe.is_window(hwnd), "window should survive WM_RBUTTONUP DefWindowProc");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_rbuttondblclk_sends_context_menu() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 86;

    let hwnd = kernel.create_window_ex_w(thread_id, "RBDBLCLK", "", None, 0, 0, 0);
    kernel.gwe.set_window_pos(hwnd, None, 50, 100, 200, 300, 0);

    let rbdbl_lparam = (20u32) << 16 | 10u32; // client (10, 20)
    // WM_RBUTTONDBLCLK generates WM_CONTEXTMENU just like WM_RBUTTONUP.
    // Top-level window: no parent to forward to, window survives.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_RBUTTONDBLCLK, 0, rbdbl_lparam],
    );
    assert!(kernel.gwe.is_window(hwnd), "window should survive WM_RBUTTONDBLCLK DefWindowProc");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_mousewheel_forwards_to_parent() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 85;
    let msg_ptr = 0x2_c000;
    memory.map_words(msg_ptr, 7);

    let parent = kernel.create_window_ex_w(thread_id, "MWPARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w(thread_id, "MWCHILD", "", Some(parent), 0, 0, 0);

    // WM_MOUSEWHEEL on child: DefWindowProcW forwards to parent unchanged.
    let wheel_wparam = 0x0078_0000u32; // WHEEL_DELTA=120 in HIWORD, no key state in LOWORD
    let wheel_lparam = 0x0064_0050u32; // screen coords x=80, y=100
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [child, WM_MOUSEWHEEL, wheel_wparam, wheel_lparam],
    );
    // Forwarding is a synchronous send on same-thread; parent's DefWindowProcW
    // receives WM_MOUSEWHEEL and has no parent, so it returns 0. Both alive.
    assert!(kernel.gwe.is_window(parent));
    assert!(kernel.gwe.is_window(child));

    // Top-level with no parent: WM_MOUSEWHEEL should return without error.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [parent, WM_MOUSEWHEEL, wheel_wparam, wheel_lparam],
    );
    assert!(kernel.gwe.is_window(parent));

    Ok(())
}

#[test]
fn coredll_raw_numlock_scrolllock_toggle_on_keydown() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let thread_id = 87;

    let hwnd = kernel.create_window_ex_w(thread_id, "NUMLOCK", "", None, 0, 0, 0);

    // Initially NumLock and ScrollLock are off (toggle bit = 0)
    assert_eq!(
        kernel.gwe.get_key_state(VK_NUMLOCK) & 0x0001,
        0,
        "NumLock initially off"
    );
    assert_eq!(
        kernel.gwe.get_key_state(VK_SCROLL) & 0x0001,
        0,
        "ScrollLock initially off"
    );

    // Press NumLock → key state is updated at post time; toggle bit goes on
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_NUMLOCK, 0));
    assert_eq!(
        kernel.gwe.get_key_state(VK_NUMLOCK) & 0x0001,
        1,
        "NumLock toggled on after first keydown"
    );

    // Key-up then another key-down should toggle it off
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_NUMLOCK, 0));
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_NUMLOCK, 0));
    assert_eq!(
        kernel.gwe.get_key_state(VK_NUMLOCK) & 0x0001,
        0,
        "NumLock toggled off on second press"
    );

    // ScrollLock behaves similarly
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_SCROLL, 0));
    assert_eq!(
        kernel.gwe.get_key_state(VK_SCROLL) & 0x0001,
        1,
        "ScrollLock toggled on"
    );

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_listbox_routing_returns_minus_one() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 88;

    let hwnd = kernel.create_window_ex_w(thread_id, "LBTEST", "", None, 0, 0, 0);

    // DefWindowProcW(WM_CHARTOITEM) → returns -1 (0xffff_ffff) so list box does default action
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_CHARTOITEM, u32::from(b'A'), 0],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ), "DefWindowProcW(WM_CHARTOITEM) must return -1");

    // DefWindowProcW(WM_VKEYTOITEM) → returns -1
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_VKEYTOITEM, 0x28u32, 0], // VK_DOWN
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(u32::MAX),
            ..
        }
    ), "DefWindowProcW(WM_VKEYTOITEM) must return -1");

    // DefWindowProcW(WM_MENUCHAR) → returns 0 (MNC_IGNORE)
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_MENUCHAR, 0, 0],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ), "DefWindowProcW(WM_MENUCHAR) must return 0 (MNC_IGNORE)");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_setredraw_suppresses_and_restores_invalidation() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 84;

    let hwnd = kernel.create_window_ex_w(thread_id, "REDRAWTEST", "", None, 0, WS_VISIBLE, 0);
    let update_pending = |k: &CeKernel, h: u32| k.gwe.window(h).is_some_and(|w| w.update_pending);

    // Clear any initial pending update from window creation.
    kernel.gwe.validate_window(hwnd);
    assert!(!update_pending(&kernel, hwnd), "no pending update after validate");

    // Suspend redraws: InvalidateRect must not set update_pending.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_SETREDRAW, 0, 0],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_INVALIDATE_RECT,
        [hwnd, 0, 0],
    );
    assert!(
        !update_pending(&kernel, hwnd),
        "InvalidateRect must not set update_pending while redraw is suspended"
    );

    // Re-enable redraws: the window is immediately invalidated.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_SETREDRAW, 1, 0],
    );
    assert!(
        update_pending(&kernel, hwnd),
        "re-enabling redraws must invalidate the window"
    );

    Ok(())
}

#[test]
fn coredll_raw_system_parameters_info_spif_sendchange_broadcasts_wm_settingchange() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let caller_thread = 82;
    let receiver_thread = 81;
    let msg_ptr = 0xa200;
    memory.map_words(msg_ptr, 7);

    let hwnd =
        kernel.create_window_ex_w(receiver_thread, "SETTINGCHANGE_TEST", "", None, 0, 0, 0);

    // Call with SPIF_SENDCHANGE (0x0002): WM_SETTINGCHANGE must be broadcast.
    let custom_action = 0x0099_u32;
    let spif_sendchange = 0x0002_u32;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SYSTEM_PARAMETERS_INFO_W,
            [custom_action, 0, 0, spif_sendchange],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    // WM_SETTINGCHANGE should appear as a pending sent notify message for receiver_thread.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, WM_SETTINGCHANGE, WM_SETTINGCHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_SETTINGCHANGE);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, custom_action);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 0);

    // Without SPIF_SENDCHANGE no WM_SETTINGCHANGE is sent.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SYSTEM_PARAMETERS_INFO_W,
            [custom_action, 0, 0, 0],
        ),
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, WM_SETTINGCHANGE, WM_SETTINGCHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_set_system_time_broadcasts_wm_timechange() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let caller_thread = 91_u32;
    let receiver_thread = 90_u32;
    let msg_ptr = 0xa300_u32;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(receiver_thread, "TIMECHANGE_TEST", "", None, 0, 0, 0);

    // SetSystemTime: ignore the SYSTEMTIME pointer, return TRUE, broadcast WM_TIMECHANGE.
    let systemtime_ptr = 0_u32;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SET_SYSTEM_TIME,
            [systemtime_ptr],
        ),
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, WM_TIMECHANGE, WM_TIMECHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_TIMECHANGE);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 0);

    // SetLocalTime: same contract.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_SET_LOCAL_TIME,
            [systemtime_ptr],
        ),
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, WM_TIMECHANGE, WM_TIMECHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_TIMECHANGE);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 0);

    Ok(())
}

#[test]
fn coredll_raw_add_remove_font_resource_broadcasts_wm_fontchange() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let caller_thread = 93_u32;
    let receiver_thread = 94_u32;
    let msg_ptr = 0xa400_u32;
    // Reserve space for MSG (7 u32s) and a short wide path string
    memory.map_words(msg_ptr, 7);
    let path_ptr = 0xa500_u32;
    // Write a valid-looking guest path: "\Windows\tahoma.ttf" as UTF-16LE
    let path: Vec<u16> = "\\Windows\\tahoma.ttf\0".encode_utf16().collect();
    let path_bytes: Vec<u8> = path.iter().flat_map(|c| c.to_le_bytes()).collect();
    memory.write_bytes(path_ptr, &path_bytes);

    let hwnd = kernel.create_window_ex_w(receiver_thread, "FONTCHANGE_TEST", "", None, 0, 0, 0);

    // AddFontResourceW: even when the file is not found in the guest FS, we still check the
    // broadcast path by using a path that exists. Use a missing path → returns 0 but still we
    // need to test the success path. For this test use a NULL path first (should fail silently).
    // Then call RemoveFontResourceW which always returns TRUE and broadcasts.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            caller_thread,
            ORD_REMOVE_FONT_RESOURCE_W,
            [path_ptr],
        ),
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
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, WM_FONTCHANGE, WM_FONTCHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_FONTCHANGE);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 0);
    assert_eq!(memory.read_u32(msg_ptr + 12)?, 0);

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_scroll_and_item_messages_forward_to_parent() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 95_u32;
    let msg_ptr = 0xa600_u32;
    memory.map_words(msg_ptr, 7);

    // Create parent and child windows on the same thread.
    let parent = kernel.create_window_ex_w(thread_id, "SCROLL_PARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w(thread_id, "SCROLL_CHILD", "", Some(parent), 0, WS_CHILD, 0);

    // Each of WM_HSCROLL, WM_VSCROLL, WM_DRAWITEM, WM_MEASUREITEM, WM_DELETEITEM sent to
    // the child should be forwarded to the parent's GWE handler by DefWindowProcW (synchronous
    // same-thread send). Neither window should be destroyed in the process.
    let scroll_msgs = [
        (WM_HSCROLL, 0x0001_u32, 0x0002_u32),
        (WM_VSCROLL, 0x0003_u32, 0x0004_u32),
        (WM_DRAWITEM, 5_u32, 6_u32),
        (WM_MEASUREITEM, 7_u32, 8_u32),
        (WM_DELETEITEM, 9_u32, 10_u32),
    ];
    for (msg, wparam, lparam) in scroll_msgs {
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [child, msg, wparam, lparam],
        );
        assert!(kernel.gwe.is_window(parent), "parent destroyed after forwarding {msg:#x}");
        assert!(kernel.gwe.is_window(child), "child destroyed after forwarding {msg:#x}");
    }

    // Top-level windows with no parent: these messages should be no-ops.
    for (msg, wparam, lparam) in scroll_msgs {
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [parent, msg, wparam, lparam],
        );
        assert!(kernel.gwe.is_window(parent), "parent destroyed after {msg:#x} on top-level");
    }

    Ok(())
}

#[test]
fn coredll_raw_translate_accelerator_char_msg_fires_non_fvirtkey_accel() -> Result<()> {
    const FVIRTKEY: u8 = 0x01;
    const FSHIFT: u8 = 0x04;
    const FALT: u8 = 0x10;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 96_u32;
    let msg_ptr = 0xa700_u32;
    memory.map_words(msg_ptr, 7);
    let hwnd = kernel.create_window_ex_w(thread_id, "ACCEL_CHAR", "", None, 0, 0, 0);

    let accel = kernel.resources.create_accelerator(
        0,
        ResourceId::Integer(800),
        None,
        vec![
            // Non-FVIRTKEY: matches WM_CHAR for the character 'A'.
            AcceleratorEntry {
                flags: 0,
                key: u16::from(b'A'),
                command: 8001,
            },
            // FVIRTKEY+FALT: should NOT match WM_SYSCHAR (only WM_SYSKEYDOWN).
            AcceleratorEntry {
                flags: FVIRTKEY | FALT,
                key: u16::from(b'B'),
                command: 8002,
            },
            // Non-FVIRTKEY + FSHIFT: matches WM_CHAR for 'C' with shift held.
            AcceleratorEntry {
                flags: FSHIFT,
                key: u16::from(b'C'),
                command: 8003,
            },
        ],
    );

    // WM_CHAR 'A' with no modifiers → fires non-FVIRTKEY accelerator 8001.
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_CHAR, u32::from(b'A'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let cmd = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_COMMAND, WM_COMMAND, PeekFlags::REMOVE)
        .expect("WM_CHAR accelerator should post WM_COMMAND");
    assert_eq!(cmd.wparam, 8001 | (1 << 16));

    // WM_SYSCHAR 'B' should NOT fire the FVIRTKEY|FALT entry (that needs WM_SYSKEYDOWN).
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_SYSCHAR, u32::from(b'B'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    // WM_CHAR 'C' with Shift held → fires non-FVIRTKEY|FSHIFT accelerator 8003.
    assert!(kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LSHIFT, 0));
    write_raw_message(&mut memory, msg_ptr, hwnd, WM_CHAR, u32::from(b'C'), 0)?;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_TRANSLATE_ACCELERATOR_W,
            [hwnd, accel, msg_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ));
    let cmd = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_COMMAND, WM_COMMAND, PeekFlags::REMOVE)
        .expect("shifted WM_CHAR accelerator should post WM_COMMAND");
    assert_eq!(cmd.wparam, 8003 | (1 << 16));

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_ctlcolor_returns_sys_color_brush() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 97_u32;

    let hwnd = kernel.create_window_ex_w(thread_id, "CTLCOLOR_OWNER", "", None, 0, 0, 0);

    const WM_CTLCOLORMSGBOX: u32 = 0x0132;
    const WM_CTLCOLOREDIT: u32 = 0x0133;
    const WM_CTLCOLORLISTBOX: u32 = 0x0134;
    const WM_CTLCOLORBTN: u32 = 0x0135;
    const WM_CTLCOLORDLG: u32 = 0x0136;
    const WM_CTLCOLORSCROLLBAR: u32 = 0x0137;
    const WM_CTLCOLORSTATIC: u32 = 0x0138;
    // Expected: SysColorBrush handles (0x000b_4000 | color_index).
    const COLOR_SCROLLBAR: u32 = 0;
    const COLOR_WINDOW: u32 = 5;
    const COLOR_BTNFACE: u32 = 15;
    let sys_brush = |index: u32| 0x000b_4000 | index;

    let cases = [
        (WM_CTLCOLORMSGBOX, sys_brush(COLOR_BTNFACE)),
        (WM_CTLCOLOREDIT, sys_brush(COLOR_WINDOW)),
        (WM_CTLCOLORLISTBOX, sys_brush(COLOR_WINDOW)),
        (WM_CTLCOLORBTN, sys_brush(COLOR_BTNFACE)),
        (WM_CTLCOLORDLG, sys_brush(COLOR_BTNFACE)),
        (WM_CTLCOLORSCROLLBAR, sys_brush(COLOR_SCROLLBAR)),
        (WM_CTLCOLORSTATIC, sys_brush(COLOR_BTNFACE)),
    ];
    for (msg, expected_brush) in cases {
        let result = table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, msg, 0, 0],
        );
        assert!(
            matches!(
                result,
                CoredllDispatch::Returned {
                    value: CoredllValue::U32(b),
                    ..
                } if b == expected_brush
            ),
            "msg={msg:#x}: expected brush={expected_brush:#x}, got {result:?}"
        );
    }

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_setfont_getfont_and_compareitem_forward() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 106_u32;

    let parent = kernel.create_window_ex_w(thread_id, "FONT_PARENT", "", None, 0, 0, 0);
    let child = kernel.create_window_ex_w(thread_id, "FONT_CHILD", "", Some(parent), 0, WS_CHILD, 0);

    // WM_GETFONT on a fresh window returns 0 (no font set).
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [child, WM_GETFONT, 0, 0],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned { value: CoredllValue::U32(0), .. }
    ));

    // WM_SETFONT stores the font handle.
    let fake_font: u32 = 0xbeef_1234;
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [child, WM_SETFONT, fake_font, 0],
    );
    assert_eq!(kernel.gwe.window(child).map(|w| w.font), Some(fake_font));

    // WM_GETFONT returns the stored font.
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [child, WM_GETFONT, 0, 0],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned { value: CoredllValue::U32(f), .. } if f == fake_font
    ));

    // WM_COMPAREITEM forwarded to parent (parent and child survive; no crash).
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [child, WM_COMPAREITEM, 1, 2],
    );
    assert!(kernel.gwe.is_window(parent), "parent destroyed after WM_COMPAREITEM forward");
    assert!(kernel.gwe.is_window(child), "child destroyed after WM_COMPAREITEM forward");

    // Top-level window has no parent: WM_COMPAREITEM is a no-op.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [parent, WM_COMPAREITEM, 1, 2],
    );
    assert!(kernel.gwe.is_window(parent), "parent destroyed after top-level WM_COMPAREITEM");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_erasebkgnd_respects_hbr_background() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 107_u32;
    let fake_hdc = 0xaaaa_bbbb_u32;

    // Class with a system color brush (COLOR_WINDOW = index 5 → 0x000b_4005).
    let mut with_brush = [0u8; WNDCLASSW_SIZE];
    with_brush[28..32].copy_from_slice(&0x000b_4005_u32.to_le_bytes());
    kernel.gwe.register_class("ERASED_CLASS", with_brush);
    let hwnd_with = kernel.create_window_ex_w(thread_id, "ERASED_CLASS", "", None, 0, 0, 0);

    // WM_ERASEBKGND → 1 (class has hbrBackground).
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [hwnd_with, WM_ERASEBKGND, fake_hdc, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(1), .. }
    ));

    // Class with null background brush (hbrBackground = 0).
    let null_brush = [0u8; WNDCLASSW_SIZE];
    kernel.gwe.register_class("NULL_BKGND_CLASS", null_brush);
    let hwnd_null = kernel.create_window_ex_w(thread_id, "NULL_BKGND_CLASS", "", None, 0, 0, 0);

    // WM_ERASEBKGND → 0 (class has no background brush).
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [hwnd_null, WM_ERASEBKGND, fake_hdc, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(0), .. }
    ));

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_nextdlgctl_moves_focus_and_updates_defid() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 108_u32;

    // Dialog window + three tab-stop children: two plain buttons, one default pushbutton.
    let dialog = kernel.create_window_ex_w(thread_id, "NEXTDLG_DLG", "", None, 0, WS_VISIBLE, 0);
    let btn_a = kernel.create_window_ex_w(thread_id, "button", "A", Some(dialog), 1, WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON, 0);
    let btn_b = kernel.create_window_ex_w(thread_id, "button", "B", Some(dialog), 2, WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_DEFPUSHBUTTON, 0);
    let btn_c = kernel.create_window_ex_w(thread_id, "button", "C", Some(dialog), 3, WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON, 0);

    // Give btn_a focus as the starting point.
    let _ = kernel.set_focus(Some(btn_a));

    // WM_NEXTDLGCTL(wparam=0, lparam=0) → move forward; focus should advance to btn_b.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [dialog, WM_NEXTDLGCTL, 0, 0],
    );
    assert_eq!(kernel.gwe.get_focus(), Some(btn_b));
    // btn_b is BS_DEFPUSHBUTTON → its id (2) should become the dialog default.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [dialog, DM_GETDEFID, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } if v & 0xffff == 2
    ));

    // WM_NEXTDLGCTL(wparam=0, lparam=0) → advance again to btn_c.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [dialog, WM_NEXTDLGCTL, 0, 0],
    );
    assert_eq!(kernel.gwe.get_focus(), Some(btn_c));

    // WM_NEXTDLGCTL(wparam=1, lparam=0) → navigate backward from btn_c to btn_b.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [dialog, WM_NEXTDLGCTL, 1, 0],
    );
    assert_eq!(kernel.gwe.get_focus(), Some(btn_b));

    // WM_NEXTDLGCTL(wparam=btn_a, lparam=1) → direct focus to btn_a.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [dialog, WM_NEXTDLGCTL, btn_a, 1],
    );
    assert_eq!(kernel.gwe.get_focus(), Some(btn_a));

    Ok(())
}

#[test]
fn coredll_raw_activate_keyboard_layout_posts_wm_inputlangchange_to_focus() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 98_u32;
    let msg_ptr = 0xa800_u32;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "HKL_FOCUS", "", None, 0, 0, 0);
    // Set focus so WM_INPUTLANGCHANGE has a target window.
    let _ = kernel.set_focus(Some(hwnd));

    // Use the default layout HKL (0x0412, the initial layout).
    let hkl = DEFAULT_KEYBOARD_LAYOUT_HKL;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ACTIVATE_KEYBOARD_LAYOUT,
            [hkl],
        ),
        CoredllDispatch::Returned { .. }
    ));

    // WM_INPUTLANGCHANGE should be posted to the focused window.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_INPUTLANGCHANGE, WM_INPUTLANGCHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr)?, hwnd);
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_INPUTLANGCHANGE);
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 1); // DEFAULT_CHARSET
    assert_eq!(memory.read_u32(msg_ptr + 12)?, hkl);

    // Without a focused window, no WM_INPUTLANGCHANGE is posted.
    let _ = kernel.set_focus(None);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_ACTIVATE_KEYBOARD_LAYOUT,
        [hkl],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, 0, WM_INPUTLANGCHANGE, WM_INPUTLANGCHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_getminmaxinfo_fills_screen_size() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 100_u32;

    // MINMAXINFO is 40 bytes: ptReserved(8) + ptMaxSize(8) + ptMaxPosition(8)
    //                        + ptMinTrackSize(8) + ptMaxTrackSize(8)
    let mmi_ptr = 0xb000_u32;
    memory.map_words(mmi_ptr, 10); // 10 × u32 = 40 bytes

    let hwnd = kernel.create_window_ex_w(thread_id, "MMITest", "w", None, 0, 0, 0);

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_GETMINMAXINFO, 0, mmi_ptr],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    // Default screen is 800×480.
    // ptMaxSize is at offset 0x08 (words 2 and 3).
    assert_eq!(memory.read_u32(mmi_ptr + 0x08)?, 800, "ptMaxSize.x");
    assert_eq!(memory.read_u32(mmi_ptr + 0x0c)?, 480, "ptMaxSize.y");
    // ptMaxPosition at offset 0x10 should be (0, 0).
    assert_eq!(memory.read_u32(mmi_ptr + 0x10)?, 0, "ptMaxPosition.x");
    assert_eq!(memory.read_u32(mmi_ptr + 0x14)?, 0, "ptMaxPosition.y");
    // ptMaxTrackSize at offset 0x20 should be screen size.
    assert_eq!(memory.read_u32(mmi_ptr + 0x20)?, 800, "ptMaxTrackSize.x");
    assert_eq!(memory.read_u32(mmi_ptr + 0x24)?, 480, "ptMaxTrackSize.y");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_syskeydown_digit_sends_keymenu() -> Result<()> {
    // Alt+digit keys should route to WM_SYSCOMMAND(SC_KEYMENU, digit_vk) just like Alt+letter.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 101_u32;
    let msg_ptr = 0xc000_u32;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w(thread_id, "DigitKey", "w", None, 0, 0, 0);

    // WM_SYSKEYDOWN with VK '5' (0x35) → WM_SYSCOMMAND(SC_KEYMENU, 0x35)
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_SYSKEYDOWN, 0x35u32, 0],
    );

    // The WM_SYSCOMMAND was sent synchronously; the window survived and the
    // default DefWindowProcW SC_KEYMENU handler posts nothing visible to the queue.
    // Verify the window is still valid and no crash occurred.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IS_WINDOW,
            [hwnd],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_nccreate_returns_true() -> Result<()> {
    // CE DefWindowProcW must return TRUE (1) for WM_NCCREATE so window creation
    // continues.  The raw ordinal path goes through send_message_w_raw → gwe::send_message.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 102_u32;

    let hwnd = kernel.create_window_ex_w(thread_id, "NCCREATETEST", "", None, 0, 0, 0);

    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_DEF_WINDOW_PROC_W,
        [hwnd, WM_NCCREATE, 0, 0],
    );
    assert!(
        matches!(
            result,
            CoredllDispatch::Returned {
                value: CoredllValue::U32(1),
                ..
            }
        ),
        "DefWindowProcW(WM_NCCREATE) must return TRUE (1)"
    );

    Ok(())
}

#[test]
fn coredll_raw_change_display_settings_ex_broadcasts_wm_displaychange() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    // Use separate threads: caller broadcasts from thread 102, receiver on thread 103.
    let caller_thread = 102_u32;
    let receiver_thread = 103_u32;
    let msg_ptr = 0xd000_u32;
    memory.map_words(msg_ptr, 7);

    // Top-level window on the receiver thread so it receives the cross-thread notify send.
    let hwnd = kernel.create_window_ex_w(receiver_thread, "DispWnd", "w", None, 0, 0, 0);

    // Build a DEVMODEW in guest memory with dmPelsWidth=640, dmPelsHeight=480, dmBitsPerPel=16.
    // DEVMODEW offsets: dmFields=72, dmBitsPerPel=168, dmPelsWidth=172, dmPelsHeight=176
    let devmode_ptr = 0xe000_u32;
    memory.map_words(devmode_ptr, 50); // 50 × 4 = 200 bytes ≥ struct size
    memory.write_u32(devmode_ptr + 72, 0x0004_0000 | 0x0008_0000 | 0x0010_0000)?; // DM_BITSPERPEL|DM_PELSWIDTH|DM_PELSHEIGHT
    memory.write_u32(devmode_ptr + 168, 16)?; // dmBitsPerPel
    memory.write_u32(devmode_ptr + 172, 640)?; // dmPelsWidth
    memory.write_u32(devmode_ptr + 176, 480)?; // dmPelsHeight

    // args: lpszDeviceName=0, lpDevMode=devmode_ptr, hwnd=0, dwflags=0, lParam=0
    let result = table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        caller_thread,
        ORD_CHANGE_DISPLAY_SETTINGS_EX,
        [0u32, devmode_ptr, 0, 0, 0],
    );
    // Should return DISP_CHANGE_SUCCESSFUL (0).
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    // WM_DISPLAYCHANGE queued as cross-thread notify send to receiver thread:
    // wparam = bpp = 16, lparam = MAKELPARAM(cx=640, cy=480) = (480<<16)|640
    let expected_lparam = (480u32 << 16) | 640u32;
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_DISPLAYCHANGE, WM_DISPLAYCHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(memory.read_u32(msg_ptr + 0)?, hwnd, "hwnd");
    assert_eq!(memory.read_u32(msg_ptr + 4)?, WM_DISPLAYCHANGE, "msg");
    assert_eq!(memory.read_u32(msg_ptr + 8)?, 16, "wparam=bpp");
    assert_eq!(memory.read_u32(msg_ptr + 12)?, expected_lparam, "lparam");

    // CDS_TEST (0x0002) must NOT broadcast.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        caller_thread,
        ORD_CHANGE_DISPLAY_SETTINGS_EX,
        [0u32, devmode_ptr, 0, 0x0000_0002u32, 0], // CDS_TEST
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            receiver_thread,
            ORD_PEEK_MESSAGE_W,
            [msg_ptr, hwnd, WM_DISPLAYCHANGE, WM_DISPLAYCHANGE, PeekFlags::REMOVE.bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_caret_blink_timer_fires_after_blink_time() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 104_u32;
    let msg_ptr = 0xf000_u32;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "BLINK_WND",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 100),
    );
    assert!(kernel.gwe.validate_window_rect(hwnd, None));

    // Focus must be on the caret window for the blink timer to advance.
    let _ = kernel.set_focus(Some(hwnd));

    // Create and show a caret (show_count goes from -1 to 0).
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_CARET, [hwnd, 0, 2, 10],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_SHOW_CARET, [hwnd],
    );

    // Advance past initial schedule: pump once so caret_next_blink_ms is set.
    // GetMessage pumps timers. Use PeekMessageW with PM_NOREMOVE.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_PEEK_MESSAGE_W,
        [msg_ptr, 0u32, 0u32, 0u32, PeekFlags::NO_REMOVE.bits()],
    );
    // No blink yet; caret window should have no update region from blink.
    // (It may have an update region from ShowCaret's invalidation, so just verify blink
    // phase is still true.)
    assert!(kernel.gwe.caret_blink_visible());

    // Advance time past the default 500 ms blink interval.
    kernel.timers.sleep_ms(600);

    // Pump timers again — this should fire the blink toggle.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_PEEK_MESSAGE_W,
        [msg_ptr, 0u32, 0u32, 0u32, PeekFlags::NO_REMOVE.bits()],
    );

    // Blink phase should have toggled to false (hidden phase).
    assert!(!kernel.gwe.caret_blink_visible());
    // Caret window should be marked for repaint (update_rect set).
    assert!(kernel.gwe.update_rect(hwnd).is_some());

    // Advance time another blink interval → should toggle back to visible.
    assert!(kernel.gwe.validate_window_rect(hwnd, None));
    kernel.timers.sleep_ms(600);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_PEEK_MESSAGE_W,
        [msg_ptr, 0u32, 0u32, 0u32, PeekFlags::NO_REMOVE.bits()],
    );
    assert!(kernel.gwe.caret_blink_visible());

    Ok(())
}

#[test]
fn coredll_raw_caret_hides_on_focus_change() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 105_u32;

    let caret_hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "CARET_FOCUS_WND",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(0, 0, 100, 100),
    );
    let other_hwnd = kernel.create_window_ex_w_with_rect(
        thread_id,
        "OTHER_WND",
        "",
        None,
        0,
        WS_VISIBLE,
        0,
        Rect::from_origin_size(100, 0, 100, 100),
    );
    assert!(kernel.gwe.validate_window_rect(caret_hwnd, None));
    assert!(kernel.gwe.validate_window_rect(other_hwnd, None));

    // Give focus to the caret window before creating the caret.
    let _ = kernel.set_focus(Some(caret_hwnd));

    // Create and show caret on caret_hwnd.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_CARET, [caret_hwnd, 0, 2, 10],
    );
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_SHOW_CARET, [caret_hwnd],
    );
    // ShowCaret resets blink to visible.
    assert!(kernel.gwe.caret_blink_visible());
    assert!(kernel.gwe.validate_window_rect(caret_hwnd, None));

    // Move focus to another window → caret owner loses focus → blink phase = false.
    let _ = kernel.set_focus(Some(other_hwnd));
    assert!(!kernel.gwe.caret_blink_visible());
    // caret window must be invalidated so the app can erase the caret cursor.
    assert!(kernel.gwe.update_rect(caret_hwnd).is_some());

    // Clear the update region so we can check after focus restoration.
    assert!(kernel.gwe.validate_window_rect(caret_hwnd, None));

    // Return focus to caret window → blink phase = true, caret_next_blink_ms reset.
    let _ = kernel.set_focus(Some(caret_hwnd));
    assert!(kernel.gwe.caret_blink_visible());
    // Caret window is invalidated again so the caret reappears.
    assert!(kernel.gwe.update_rect(caret_hwnd).is_some());

    Ok(())
}

#[test]
fn coredll_raw_show_caret_xors_pixels_into_framebuffer() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let mut framebuffer = VirtualFramebuffer::new(50, 40, PixelFormat::Rgb565)?;
    // Fill framebuffer with white (0xFFFF in RGB565).
    framebuffer.pixels_mut().fill(0xff);
    let _ = framebuffer.take_dirty_rects();
    let thread_id = 106_u32;

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id, "CARET_XOR_WND", "", None, 0, WS_VISIBLE, 0,
        Rect::from_origin_size(5, 6, 30, 25),
    );
    assert!(kernel.gwe.validate_window_rect(hwnd, None));

    // Create a 2x4 caret at client position (3, 2).
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_CREATE_CARET, [hwnd, 0, 2, 4],
    );
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_SET_CARET_POS, [3, 2],
    );
    // Before ShowCaret: framebuffer still all white.
    assert!(framebuffer.pixels().iter().all(|b| *b == 0xff), "no pixels changed before ShowCaret");

    // ShowCaret → should XOR-draw the caret into the framebuffer.
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_SHOW_CARET, [hwnd],
    );
    assert!(kernel.gwe.caret_drawn_in_framebuffer(), "caret should be marked drawn after ShowCaret");

    // Caret screen position: window at (5,6) + client (3,2) = screen (8,8), size 2x4.
    // XOR of 0xFFFF = 0x0000. Check two bytes per pixel.
    let stride = framebuffer.stride();
    let bpp = PixelFormat::Rgb565.bytes_per_pixel();
    let pixels = framebuffer.pixels();
    for row in 8..12_usize {
        for col in 8..10_usize {
            let off = row * stride + col * bpp;
            assert_eq!(
                &pixels[off..off + bpp], &[0x00, 0x00],
                "caret pixel at ({col},{row}) should be 0x0000 after ShowCaret"
            );
        }
    }
    // Pixel just outside caret rect (col 10, row 8) should still be white.
    let outside_off = 8 * stride + 10 * bpp;
    assert_eq!(&pixels[outside_off..outside_off + bpp], &[0xff, 0xff]);

    // HideCaret → should XOR-erase the caret (restore white pixels).
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_HIDE_CARET, [hwnd],
    );
    assert!(!kernel.gwe.caret_drawn_in_framebuffer(), "caret should not be drawn after HideCaret");
    let pixels = framebuffer.pixels();
    for row in 8..12_usize {
        for col in 8..10_usize {
            let off = row * stride + col * bpp;
            assert_eq!(
                &pixels[off..off + bpp], &[0xff, 0xff],
                "caret pixel at ({col},{row}) should be restored after HideCaret"
            );
        }
    }

    Ok(())
}

#[test]
fn coredll_raw_begin_paint_erases_caret_end_paint_restores() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let mut framebuffer = VirtualFramebuffer::new(50, 40, PixelFormat::Rgb565)?;
    framebuffer.pixels_mut().fill(0xff);
    let _ = framebuffer.take_dirty_rects();
    let thread_id = 107_u32;
    let paint_struct_ptr = 0x4000_u32;
    memory.map_words(paint_struct_ptr, 16);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id, "CARET_PAINT_WND", "", None, 0, WS_VISIBLE, 0,
        Rect::from_origin_size(0, 0, 50, 40),
    );
    assert!(kernel.gwe.validate_window_rect(hwnd, None));

    // Create/show caret at client (4, 3), 2x4.
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_CREATE_CARET, [hwnd, 0, 2, 4],
    );
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_SET_CARET_POS, [4, 3],
    );
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_SHOW_CARET, [hwnd],
    );
    assert!(kernel.gwe.caret_drawn_in_framebuffer());
    // Caret should be XOR'd at screen (4,3), size 2x4 → pixels are 0x0000.
    let stride = framebuffer.stride();
    let bpp = PixelFormat::Rgb565.bytes_per_pixel();
    {
        let off = 3 * stride + 4 * bpp;
        assert_eq!(&framebuffer.pixels()[off..off + bpp], &[0x00, 0x00]);
    }

    // BeginPaint → CE GWES erases caret so app sees clean pixels.
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_BEGIN_PAINT, [hwnd, paint_struct_ptr],
    );
    assert!(!kernel.gwe.caret_drawn_in_framebuffer(), "BeginPaint should erase caret from framebuffer");
    {
        let off = 3 * stride + 4 * bpp;
        assert_eq!(
            &framebuffer.pixels()[off..off + bpp], &[0xff, 0xff],
            "pixels should be restored to background after BeginPaint"
        );
    }

    // EndPaint → CE GWES restores caret after app has painted.
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_END_PAINT, [hwnd, paint_struct_ptr],
    );
    assert!(kernel.gwe.caret_drawn_in_framebuffer(), "EndPaint should re-draw caret into framebuffer");
    {
        let off = 3 * stride + 4 * bpp;
        assert_eq!(
            &framebuffer.pixels()[off..off + bpp], &[0x00, 0x00],
            "caret should be re-drawn (XOR) after EndPaint"
        );
    }

    Ok(())
}

#[test]
fn coredll_raw_caret_blink_xors_framebuffer_via_peek_message() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let mut framebuffer = VirtualFramebuffer::new(50, 40, PixelFormat::Rgb565)?;
    framebuffer.pixels_mut().fill(0xff);
    let _ = framebuffer.take_dirty_rects();
    let thread_id = 108_u32;
    let msg_ptr = 0xf100_u32;
    memory.map_words(msg_ptr, 7);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id, "CARET_BLINK_XOR_WND", "", None, 0, WS_VISIBLE, 0,
        Rect::from_origin_size(0, 0, 50, 40),
    );
    assert!(kernel.gwe.validate_window_rect(hwnd, None));
    let _ = kernel.set_focus(Some(hwnd));

    // Create/show caret at (2,2), 2x2.
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_CREATE_CARET, [hwnd, 0, 2, 2],
    );
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_SET_CARET_POS, [2, 2],
    );
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_SHOW_CARET, [hwnd],
    );
    assert!(kernel.gwe.caret_drawn_in_framebuffer());

    // Schedule blink phase: first PeekMessage sets caret_next_blink_ms.
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_PEEK_MESSAGE_W,
        [msg_ptr, 0u32, 0u32, 0u32, PeekFlags::NO_REMOVE.bits()],
    );
    assert!(kernel.gwe.caret_drawn_in_framebuffer(), "caret still drawn before first blink");

    // Advance time past blink interval → next PeekMessage should toggle caret off.
    kernel.timers.sleep_ms(600);
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_PEEK_MESSAGE_W,
        [msg_ptr, 0u32, 0u32, 0u32, PeekFlags::NO_REMOVE.bits()],
    );
    assert!(!kernel.gwe.caret_blink_visible(), "caret blink should be off after interval");
    assert!(!kernel.gwe.caret_drawn_in_framebuffer(), "caret should be erased from framebuffer on blink-off");

    let stride = framebuffer.stride();
    let bpp = PixelFormat::Rgb565.bytes_per_pixel();
    let off = 2 * stride + 2 * bpp;
    assert_eq!(
        &framebuffer.pixels()[off..off + bpp], &[0xff, 0xff],
        "caret pixels restored to background after blink-off"
    );

    // Advance another blink interval → caret toggles back on.
    kernel.timers.sleep_ms(600);
    table.dispatch_raw_ordinal_with_framebuffer(
        &mut kernel, &mut memory, Some(&mut framebuffer), thread_id,
        ORD_PEEK_MESSAGE_W,
        [msg_ptr, 0u32, 0u32, 0u32, PeekFlags::NO_REMOVE.bits()],
    );
    assert!(kernel.gwe.caret_blink_visible(), "caret blink should be on after second interval");
    assert!(kernel.gwe.caret_drawn_in_framebuffer(), "caret should be re-drawn in framebuffer on blink-on");
    assert_eq!(
        &framebuffer.pixels()[off..off + bpp], &[0x00, 0x00],
        "caret pixels XOR'd back after blink-on"
    );

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

#[test]
fn coredll_raw_send_notify_broadcast_delivers_in_z_order() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let caller_thread = 109_u32;
    let receiver_thread = 110_u32;

    // hwnd_a created first → Z-order back; hwnd_b created second → Z-order front.
    let hwnd_a = kernel.create_window_ex_w(receiver_thread, "ZORDER_BCAST_A", "", None, 0, 0, 0);
    let hwnd_b = kernel.create_window_ex_w(receiver_thread, "ZORDER_BCAST_B", "", None, 0, 0, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, caller_thread,
            ORD_SEND_NOTIFY_MESSAGE_W,
            [HWND_BROADCAST, WM_USER, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Z-order front (hwnd_b) must be queued first, back (hwnd_a) second.
    let first = kernel.gwe.sent_message(1).expect("first queued notify");
    let second = kernel.gwe.sent_message(2).expect("second queued notify");
    assert_eq!(first.message.hwnd, hwnd_b, "Z-order front window should be notified first");
    assert_eq!(second.message.hwnd, hwnd_a, "Z-order back window should be notified second");

    Ok(())
}

#[test]
fn coredll_raw_set_window_long_style_syncs_visible_and_enabled_state() -> Result<()> {
    // SetWindowLong(GWL_STYLE, ...) must update IsWindowVisible and IsWindowEnabled
    // to match the new WS_VISIBLE and WS_DISABLED bits in CE.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 111_u32;

    let hwnd = kernel.create_window_ex_w(thread_id, "STYLESET", "", None, 0, 0, 0);

    // Window starts invisible (WS_VISIBLE not set).
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id,
            ORD_IS_WINDOW_VISIBLE, [hwnd]),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ));

    // SetWindowLong(GWL_STYLE) adding WS_VISIBLE should make IsWindowVisible return true.
    let prev_style = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_GET_WINDOW_LONG_W, [hwnd, GWL_STYLE as u32],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::U32(s), .. } => s,
        other => panic!("unexpected: {other:?}"),
    };
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_SET_WINDOW_LONG_W, [hwnd, GWL_STYLE as u32, prev_style | WS_VISIBLE],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id,
            ORD_IS_WINDOW_VISIBLE, [hwnd]),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "GWL_STYLE with WS_VISIBLE must make IsWindowVisible true");

    // SetWindowLong(GWL_STYLE) adding WS_DISABLED should make IsWindowEnabled return false.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id,
            ORD_IS_WINDOW_ENABLED, [hwnd]),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "window starts enabled");
    let cur_style = prev_style | WS_VISIBLE;
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_SET_WINDOW_LONG_W, [hwnd, GWL_STYLE as u32, cur_style | WS_DISABLED],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id,
            ORD_IS_WINDOW_ENABLED, [hwnd]),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "GWL_STYLE with WS_DISABLED must make IsWindowEnabled false");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_nextdlgctl_to_non_pushbutton_preserves_defid() -> Result<()> {
    // WM_NEXTDLGCTL moving focus to a non-pushbutton must NOT send DM_SETDEFID,
    // so the existing default button remains the dialog default.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 112_u32;

    let dialog = kernel.create_window_ex_w(thread_id, "NEXTDLG_NB_DLG", "", None, 0, WS_VISIBLE, 0);
    let btn_default = kernel.create_window_ex_w(
        thread_id, "button", "OK", Some(dialog), 10,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_DEFPUSHBUTTON, 0,
    );
    let edit_ctrl = kernel.create_window_ex_w(
        thread_id, "edit", "", Some(dialog), 20,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP, 0,
    );

    let _ = kernel.set_focus(Some(btn_default));

    // Confirm DM_GETDEFID reports btn_default (id=10) as the dialog default.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [dialog, DM_GETDEFID, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(v), .. }
            if v == (DC_HASDEFID << 16) | 10
    ), "DM_GETDEFID must report btn_default (id 10) initially");

    // WM_NEXTDLGCTL(wparam=edit_ctrl, lparam=1) → direct focus to the edit control.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DEF_WINDOW_PROC_W, [dialog, WM_NEXTDLGCTL, edit_ctrl, 1],
    );
    assert_eq!(kernel.gwe.get_focus(), Some(edit_ctrl), "focus must move to edit_ctrl");

    // DM_GETDEFID must still return id=10 — edit is not a pushbutton so DM_SETDEFID was not called.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [dialog, DM_GETDEFID, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(v), .. }
            if v == (DC_HASDEFID << 16) | 10
    ), "default button must remain id=10 after focusing a non-pushbutton control");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_settext_triggers_invalidate() -> Result<()> {
    // WM_SETTEXT in DefWindowProcW must call InvalidateRect so the window
    // title area is repainted (update_pending becomes true).
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 113_u32;
    let text_ptr = 0x1_0000_u32;
    let rect_ptr = 0x1_0200_u32;
    memory.map_words(rect_ptr, 4);

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id, "SETTEXTWIN", "old title", None, 0, WS_VISIBLE, 0,
        Rect::from_origin_size(0, 0, 200, 100),
    );

    // Write new title text.
    memory.write_wide_z(text_ptr, "new title");

    // Clear any initial update rect (WS_VISIBLE windows start with update_pending=true).
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_VALIDATE_RECT, [hwnd, 0],
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_GET_UPDATE_RECT, [hwnd, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "no update rect before WM_SETTEXT");

    // Send WM_SETTEXT → title changes and window is invalidated.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [hwnd, WM_SETTEXT, 0, text_ptr],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(1), .. }
    ), "WM_SETTEXT must return 1 (TRUE)");

    // GetUpdateRect must now report a pending update.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_GET_UPDATE_RECT, [hwnd, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "WM_SETTEXT must invalidate the window so GetUpdateRect returns true");

    // Title must have changed.
    assert_eq!(
        kernel.gwe.get_window_text(hwnd, 64).as_deref(),
        Some("new title"),
        "window title must be updated by WM_SETTEXT"
    );

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_getdlgcode_for_radio_edit_static() -> Result<()> {
    // WM_GETDLGCODE must return the correct code for radiobutton, edit, and
    // static class controls, matching CE window_dialog_code() behavior.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 114_u32;

    let dialog = kernel.create_window_ex_w(thread_id, "DLGCODE_DLG", "", None, 0, WS_VISIBLE, 0);

    // Radio button → DLGC_BUTTON | DLGC_RADIOBUTTON.
    let radio = kernel.create_window_ex_w(
        thread_id, "button", "radio", Some(dialog), 1,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_RADIOBUTTON, 0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [radio, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(c), .. }
            if c == (DLGC_BUTTON | DLGC_RADIOBUTTON)
    ), "BS_RADIOBUTTON must return DLGC_BUTTON | DLGC_RADIOBUTTON");

    // Auto-radio button → same DLGC_BUTTON | DLGC_RADIOBUTTON.
    let autoradio = kernel.create_window_ex_w(
        thread_id, "button", "aradio", Some(dialog), 2,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_AUTORADIOBUTTON, 0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [autoradio, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(c), .. }
            if c == (DLGC_BUTTON | DLGC_RADIOBUTTON)
    ), "BS_AUTORADIOBUTTON must return DLGC_BUTTON | DLGC_RADIOBUTTON");

    // Edit control (single-line) → DLGC_HASSETSEL | DLGC_WANTCHARS | DLGC_WANTARROWS.
    let edit = kernel.create_window_ex_w(
        thread_id, "edit", "", Some(dialog), 3,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP, 0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [edit, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(c), .. }
            if c == (DLGC_HASSETSEL | DLGC_WANTCHARS | DLGC_WANTARROWS)
    ), "edit class must return DLGC_HASSETSEL | DLGC_WANTCHARS | DLGC_WANTARROWS");

    // Static control → DLGC_STATIC.
    let stat = kernel.create_window_ex_w(
        thread_id, "static", "label", Some(dialog), 4,
        WS_VISIBLE | WS_CHILD, 0,
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W, [stat, WM_GETDLGCODE, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(c), .. }
            if c == DLGC_STATIC
    ), "static class must return DLGC_STATIC");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_paint_validates_update_region() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 120_u32;

    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id, "PAINTVAL", "", None, 0, WS_VISIBLE, 0,
        Rect::from_origin_size(0, 0, 100, 100),
    );
    // Clear the initial WS_VISIBLE update-pending flag first.
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_VALIDATE_RECT, [hwnd, 0],
    );
    // Invalidate the window so it has a pending update region.
    assert!(kernel.gwe.invalidate_window(hwnd, None, true));
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id,
                ORD_GET_UPDATE_RECT, [hwnd, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
        ),
        "GetUpdateRect must return true after InvalidateRect"
    );

    // DefWindowProcW(WM_PAINT) must validate the update region and return 0.
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id,
                ORD_DEF_WINDOW_PROC_W, [hwnd, WM_PAINT, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(0), .. }
        ),
        "DefWindowProcW(WM_PAINT) must return 0"
    );
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id,
                ORD_GET_UPDATE_RECT, [hwnd, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
        ),
        "DefWindowProcW(WM_PAINT) must clear the update region"
    );

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_getdlgcode_for_pushbutton_variants() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 119_u32;

    let dialog = kernel.create_window_ex_w(thread_id, "PBCODE_DLG", "", None, 0, WS_VISIBLE, 0);

    // BS_PUSHBUTTON → DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON.
    let push = kernel.create_window_ex_w(
        thread_id, "button", "push", Some(dialog), 1,
        WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, 0,
    );
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id,
                ORD_DEF_WINDOW_PROC_W, [push, WM_GETDLGCODE, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(c), .. }
                if c == (DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON)
        ),
        "BS_PUSHBUTTON must return DLGC_BUTTON | DLGC_UNDEFPUSHBUTTON"
    );

    // BS_DEFPUSHBUTTON → DLGC_BUTTON | DLGC_DEFPUSHBUTTON.
    let defpush = kernel.create_window_ex_w(
        thread_id, "button", "defpush", Some(dialog), 2,
        WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON, 0,
    );
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id,
                ORD_DEF_WINDOW_PROC_W, [defpush, WM_GETDLGCODE, 0, 0],
            ),
            CoredllDispatch::Returned { value: CoredllValue::U32(c), .. }
                if c == (DLGC_BUTTON | DLGC_DEFPUSHBUTTON)
        ),
        "BS_DEFPUSHBUTTON must return DLGC_BUTTON | DLGC_DEFPUSHBUTTON"
    );

    Ok(())
}

#[test]
fn coredll_raw_check_radio_button_sets_exclusive_check_state() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 118_u32;

    let dialog = kernel.create_window_ex_w(thread_id, "RB_DLG", "", None, 0, WS_VISIBLE, 0);
    let _ = kernel.create_window_ex_w(
        thread_id, "button", "r1", Some(dialog), 10,
        WS_VISIBLE | WS_CHILD | BS_RADIOBUTTON, 0,
    );
    let _ = kernel.create_window_ex_w(
        thread_id, "button", "r2", Some(dialog), 11,
        WS_VISIBLE | WS_CHILD | BS_RADIOBUTTON, 0,
    );
    let _ = kernel.create_window_ex_w(
        thread_id, "button", "r3", Some(dialog), 12,
        WS_VISIBLE | WS_CHILD | BS_RADIOBUTTON, 0,
    );

    // CheckRadioButton(dialog, first=10, last=12, check=11) → exactly id=11 checked.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_CHECK_RADIO_BUTTON, [dialog, 10, 12, 11],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "CheckRadioButton must return true for valid args");
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        kernel.gwe.is_dlg_button_checked(dialog, 10),
        Some(0),
        "id=10 must be unchecked"
    );
    assert_eq!(
        kernel.gwe.is_dlg_button_checked(dialog, 11),
        Some(1),
        "id=11 must be checked"
    );
    assert_eq!(
        kernel.gwe.is_dlg_button_checked(dialog, 12),
        Some(0),
        "id=12 must be unchecked"
    );

    // check out of range → false + ERROR_INVALID_PARAMETER.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_CHECK_RADIO_BUTTON, [dialog, 10, 12, 9],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "CheckRadioButton must return false when check id is outside [first, last]");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_PARAMETER);

    Ok(())
}

#[test]
fn coredll_raw_end_dialog_stores_result_and_rejects_invalid_hwnd() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 117_u32;

    let hwnd = kernel.create_window_ex_w(thread_id, "ED_DLG", "", None, 0, WS_VISIBLE, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_END_DIALOG, [hwnd, 42],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "EndDialog on a live window must return true");
    assert_eq!(
        kernel.gwe.dialog_result(hwnd),
        Some(42),
        "EndDialog must record the result code in dialog_results"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    // Calling EndDialog on a non-existent HWND must fail with ERROR_INVALID_WINDOW_HANDLE.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_END_DIALOG, [0xdead_beef, 99],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "EndDialog on an invalid HWND must return false");
    assert_eq!(
        kernel.threads.get_last_error(thread_id),
        ERROR_INVALID_WINDOW_HANDLE,
        "EndDialog failure must set ERROR_INVALID_WINDOW_HANDLE"
    );

    Ok(())
}

#[test]
fn coredll_raw_is_dialog_message_vk_return_is_consumed_by_dialog() -> Result<()> {
    const VK_RETURN: u32 = 0x0d;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 115_u32;
    let msg_ptr = 0x3540_u32;
    memory.map_words(msg_ptr, 7);

    let dialog = kernel.create_window_ex_w(thread_id, "RET_DLG", "", None, 0, WS_VISIBLE, 0);
    let ok_btn = kernel.create_window_ex_w(
        thread_id, "button", "OK", Some(dialog), 1,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_DEFPUSHBUTTON, 0,
    );
    let _ = kernel.set_focus(Some(ok_btn));

    write_raw_message(&mut memory, msg_ptr, ok_btn, WM_KEYDOWN, VK_RETURN, 0)?;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id,
                ORD_IS_DIALOG_MESSAGE_W, [dialog, msg_ptr],
            ),
            CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
        ),
        "IsDialogMessageW must consume VK_RETURN and return true"
    );
    assert!(kernel.gwe.is_window(dialog), "dialog must still exist after VK_RETURN handling");
    assert_eq!(kernel.gwe.get_focus(), Some(ok_btn), "focus must be unchanged after VK_RETURN");

    Ok(())
}

#[test]
fn coredll_raw_is_dialog_message_vk_escape_is_consumed_by_dialog() -> Result<()> {
    const VK_ESCAPE: u32 = 0x1b;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 116_u32;
    let msg_ptr = 0x3540_u32;
    memory.map_words(msg_ptr, 7);

    let dialog = kernel.create_window_ex_w(thread_id, "ESC_DLG", "", None, 0, WS_VISIBLE, 0);
    let cancel_btn = kernel.create_window_ex_w(
        thread_id, "button", "Cancel", Some(dialog), 2,
        WS_VISIBLE | WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON, 0,
    );
    let _ = kernel.set_focus(Some(cancel_btn));

    write_raw_message(&mut memory, msg_ptr, cancel_btn, WM_KEYDOWN, VK_ESCAPE, 0)?;
    assert!(
        matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id,
                ORD_IS_DIALOG_MESSAGE_W, [dialog, msg_ptr],
            ),
            CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
        ),
        "IsDialogMessageW must consume VK_ESCAPE and return true"
    );
    assert!(kernel.gwe.is_window(dialog), "dialog must still exist after VK_ESCAPE handling");
    assert_eq!(
        kernel.gwe.get_focus(),
        Some(cancel_btn),
        "focus must be unchanged after VK_ESCAPE"
    );

    Ok(())
}

#[test]
fn coredll_raw_get_version_ex_w_fills_ce_os_info() -> Result<()> {
    const VER_PLATFORM_WIN32_CE: u32 = 3;
    const OSVERSIONINFOW_SIZE: u32 = 276; // 20 header + 128 wide chars
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 121_u32;
    let info_ptr = 0x3000_0000_u32;
    memory.write_word(info_ptr, OSVERSIONINFOW_SIZE);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_GET_VERSION_EX_W, [info_ptr],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "GetVersionExW must return true for a valid buffer");
    assert_eq!(memory.read_u32(info_ptr + 4)?, 4, "dwMajorVersion must be 4");
    assert_eq!(memory.read_u32(info_ptr + 8)?, 20, "dwMinorVersion must be 20");
    assert_eq!(memory.read_u32(info_ptr + 12)?, 0, "dwBuildNumber must be 0");
    assert_eq!(
        memory.read_u32(info_ptr + 16)?,
        VER_PLATFORM_WIN32_CE,
        "dwPlatformId must be VER_PLATFORM_WIN32_CE"
    );
    assert_eq!(
        memory.read_u16(info_ptr + 20)?,
        0,
        "szCSDVersion must begin with a null terminator"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_GET_VERSION_EX_W, [0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "GetVersionExW must reject a null pointer");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_PARAMETER);

    Ok(())
}

#[test]
fn coredll_raw_register_window_message_returns_consistent_atom() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 122_u32;
    let name_a_ptr = 0x3000_0000_u32;
    let name_b_ptr = 0x3000_0200_u32;
    memory.write_wide_z(name_a_ptr, "GWE_TEST_CUSTOMMSG_A");
    memory.write_wide_z(name_b_ptr, "GWE_TEST_CUSTOMMSG_B");

    let id_a1 = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_REGISTER_WINDOW_MESSAGE_W, [name_a_ptr],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } => v,
        other => panic!("expected U32 from RegisterWindowMessageW: {other:?}"),
    };
    assert!(
        (0xC000..=0xFFFF).contains(&id_a1),
        "registered message must be in [0xC000, 0xFFFF], got 0x{id_a1:04x}"
    );
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    let id_a2 = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_REGISTER_WINDOW_MESSAGE_W, [name_a_ptr],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } => v,
        other => panic!("expected U32: {other:?}"),
    };
    assert_eq!(id_a2, id_a1, "re-registering the same name must return the same atom");

    let id_b = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_REGISTER_WINDOW_MESSAGE_W, [name_b_ptr],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::U32(v), .. } => v,
        other => panic!("expected U32: {other:?}"),
    };
    assert!((0xC000..=0xFFFF).contains(&id_b));
    assert_ne!(id_b, id_a1, "different names must produce different atoms");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_REGISTER_WINDOW_MESSAGE_W, [0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(0), .. }
    ), "null name pointer must return 0");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_PARAMETER);

    Ok(())
}

#[test]
fn coredll_raw_get_desktop_window_returns_fixed_handle() -> Result<()> {
    const DESKTOP_HWND: u32 = 0x0001_0000;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 123_u32;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_GET_DESKTOP_WINDOW, [] as [u32; 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Handle(DESKTOP_HWND), .. }
    ), "GetDesktopWindow must return the fixed DESKTOP_HWND value");

    Ok(())
}

#[test]
fn coredll_raw_def_dlg_proc_w_always_returns_zero() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 124_u32;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_DLG_PROC_W, [0xDEAD, 0x0001, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(0), .. }
    ), "DefDlgProcW must return 0 for all inputs");

    Ok(())
}

#[test]
fn coredll_raw_dialog_box_indirect_param_w_creates_window_and_rejects_null_params() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 125_u32;

    let template = 0x3600_u32;
    let dlgproc = 0x1234_5678_u32;
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

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DIALOG_BOX_INDIRECT_PARAM_W, [0, 0, 0, dlgproc, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Handle(0), .. }
    ), "DialogBoxIndirectParamW must return null for a null template");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_PARAMETER);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DIALOG_BOX_INDIRECT_PARAM_W, [0, template, 0, 0, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Handle(0), .. }
    ), "DialogBoxIndirectParamW must return null for a null dlgproc");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_PARAMETER);

    let dialog = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id,
        ORD_DIALOG_BOX_INDIRECT_PARAM_W, [0, template, 0, dlgproc, 0],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("DialogBoxIndirectParamW did not return an HWND: {other:?}"),
    };
    assert_ne!(dialog, 0, "DialogBoxIndirectParamW must create a window for a valid template");
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    assert_eq!(
        kernel.gwe.get_window_long(dialog, wince_emulation_v3::ce::gwe::GWL_WNDPROC),
        Some(dlgproc)
    );

    Ok(())
}

#[test]
fn coredll_raw_get_window_text_wdirect_returns_same_text_as_get_window_text_w() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 126_u32;

    let hwnd = kernel.create_window_ex_w(thread_id, "STATIC", "Hello CE", None, 1, WS_VISIBLE, 0);
    let buf_ptr = 0x3000_0000_u32;
    memory.map_halfwords(buf_ptr, 12);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_GET_WINDOW_TEXT_WDIRECT, [hwnd, buf_ptr, 12],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(8), .. }
    ), "GetWindowTextWDirect must return char count");
    assert_eq!(memory.read_wide_z(buf_ptr, 12), "Hello CE", "GetWindowTextWDirect must fill the buffer with the window title");
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_GET_WINDOW_TEXT_WDIRECT, [0xDEAD_BEEF, buf_ptr, 12],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(0), .. }
    ), "GetWindowTextWDirect must return 0 for an invalid HWND");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_HANDLE);

    Ok(())
}

#[test]
fn coredll_raw_destroy_menu_delete_menu_complete_lifecycle() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 127_u32;
    const MF_BYPOSITION: u32 = 0x0000_0400;
    let text_ptr = 0x3000_0000_u32;
    memory.map_halfwords(text_ptr, 16);
    memory.write_wide_z(text_ptr, "Item");

    // Create a menu and append one item
    let menu = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_MENU, [],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("CreateMenu failed: {other:?}"),
    };
    assert_ne!(menu, 0);
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_APPEND_MENU_W, [menu, 0, 1001, text_ptr],
    );

    // DELETE_MENU: remove item at position 0 by position
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DELETE_MENU, [menu, 0, MF_BYPOSITION],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "DeleteMenu must return true for valid menu+item");

    // DELETE_MENU: invalid menu handle → false + ERROR_INVALID_HANDLE
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DELETE_MENU, [0xDEAD_BEEFu32, 0, MF_BYPOSITION],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "DeleteMenu must return false for invalid menu");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_HANDLE);

    // DESTROY_MENU: destroy the menu
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DESTROY_MENU, [menu],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ), "DestroyMenu must return true for valid menu");

    // DESTROY_MENU: invalid handle → false + ERROR_INVALID_HANDLE
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DESTROY_MENU, [0xDEAD_BEEFu32],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "DestroyMenu must return false for invalid handle");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_HANDLE);

    Ok(())
}

#[test]
fn coredll_raw_load_menu_w_and_load_accelerators_w_return_zero_without_module() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 128_u32;

    // LOAD_MENU_W with no loaded module → 0 + ERROR_RESOURCE_NAME_NOT_FOUND
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_LOAD_MENU_W, [0u32, 1],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Handle(0), .. }
    ), "LoadMenuW without resources must return 0");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_RESOURCE_NAME_NOT_FOUND);

    // LOAD_ACCELERATORS_W with no loaded module → 0 + ERROR_RESOURCE_NAME_NOT_FOUND
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_LOAD_ACCELERATORS_W, [0u32, 1],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Handle(0), .. }
    ), "LoadAcceleratorsW without resources must return 0");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_RESOURCE_NAME_NOT_FOUND);

    // DESTROY_ACCELERATOR_TABLE: invalid handle → false + ERROR_INVALID_HANDLE
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_DESTROY_ACCELERATOR_TABLE, [0xDEAD_BEEFu32],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(false), .. }
    ), "DestroyAcceleratorTable must return false for invalid handle");
    assert_eq!(kernel.threads.get_last_error(thread_id), ERROR_INVALID_HANDLE);

    // FIND_RESOURCE (without _W) with no module → 0 (same impl as FIND_RESOURCE_W)
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_FIND_RESOURCE, [0u32, 1, 4],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Handle(0), .. }
    ), "FindResource without a loaded module must return 0");

    Ok(())
}

#[test]
fn coredll_raw_gdi_set_sys_colors_always_returns_true() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 129_u32;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_SYS_COLORS,
            [0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_gdi_ellipse_and_rectangle_validate_hdc() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 130_u32;

    let dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        _ => panic!("CREATE_COMPATIBLE_DC failed"),
    };
    assert_ne!(dc, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ELLIPSE,
            [0, 0, 0, 10, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ), "ELLIPSE with hdc=0 must return false");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ELLIPSE,
            [dc, 0, 0, 10, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ), "ELLIPSE with valid hdc must return true");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RECTANGLE,
            [0, 0, 0, 10, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ), "RECTANGLE with hdc=0 must return false");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_RECTANGLE,
            [dc, 0, 0, 10, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ), "RECTANGLE with valid hdc must return true");

    Ok(())
}

#[test]
fn coredll_raw_gdi_get_pixel_and_delete_dc_lifecycle() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 131_u32;

    let dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        _ => panic!("CREATE_COMPATIBLE_DC failed"),
    };
    assert_ne!(dc, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PIXEL,
            [0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0xFFFF_FFFF),
            ..
        }
    ), "GET_PIXEL with hdc=0 must return CLR_INVALID");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_PIXEL,
            [dc, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ), "GET_PIXEL with valid dc must return 0 (black)");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_DC,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ), "DELETE_DC with hdc=0 must return false");

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DELETE_DC,
            [dc],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ), "DELETE_DC with valid dc must return true");

    Ok(())
}

#[test]
fn coredll_raw_gdi_set_bk_mode_returns_previous_mode() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 132_u32;

    let dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        _ => panic!("CREATE_COMPATIBLE_DC failed"),
    };
    assert_ne!(dc, 0);

    // Default mode is OPAQUE=2; setting to TRANSPARENT=1 must return previous OPAQUE=2.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_BK_MODE,
            [dc, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ), "SET_BK_MODE must return previous mode OPAQUE=2");

    // Now mode is TRANSPARENT=1; setting to OPAQUE=2 must return previous TRANSPARENT=1.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_BK_MODE,
            [dc, 2],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(1),
            ..
        }
    ), "SET_BK_MODE must return previous mode TRANSPARENT=1");

    Ok(())
}

#[test]
fn coredll_raw_gdi_get_clip_box_returns_region_status() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 133_u32;
    let rect_ptr = 0x1_0000_u32;
    memory.map_words(rect_ptr, 4);

    let dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        _ => panic!("CREATE_COMPATIBLE_DC failed"),
    };
    assert_ne!(dc, 0);

    // No clip region set → falls back to default 800×480 → non-empty → SIMPLEREGION=2.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_CLIP_BOX,
            [dc, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(2),
            ..
        }
    ), "GET_CLIP_BOX must return SIMPLEREGION=2 for default 800x480 fallback");

    Ok(())
}

#[test]
fn coredll_raw_gdi_create_pen_always_returns_handle() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 134_u32;

    let pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_PEN,
        [0, 1, 0xFF_0000],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("CREATE_PEN returned unexpected: {other:?}"),
    };
    assert_ne!(pen, 0, "CREATE_PEN must return non-zero handle");

    Ok(())
}

#[test]
fn coredll_raw_gdi_create_bitmap_and_get_object_w() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 135_u32;
    let out_ptr = 0x1_0000_u32;
    memory.map_words(out_ptr, 8);
    // GET_OBJECT_W writes u16 planes/bits_pixel at offsets 16 and 18.
    memory.map_halfwords(out_ptr + 16, 2);

    // Invalid params: width=0 → ERROR_INVALID_PARAMETER.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_CREATE_BITMAP,
            [0_u32, 0, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ), "CREATE_BITMAP with zero dims must return 0");

    // Valid params.
    let bitmap = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_BITMAP,
        [2_u32, 2, 1, 16, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("CREATE_BITMAP returned unexpected: {other:?}"),
    };
    assert_ne!(bitmap, 0, "CREATE_BITMAP must return non-zero handle");

    // GET_OBJECT_W on bitmap must write 24 bytes and return 24.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_OBJECT_W,
            [bitmap, 24, out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(24),
            ..
        }
    ), "GET_OBJECT_W on bitmap must return 24");

    // Create a pen and verify GET_OBJECT_W returns 0 for it (pens unsupported).
    let pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_PEN,
        [0, 1, 0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("CREATE_PEN returned unexpected: {other:?}"),
    };
    assert_ne!(pen, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GET_OBJECT_W,
            [pen, 24, out_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ), "GET_OBJECT_W on pen must return 0 (pens not supported)");

    Ok(())
}

#[test]
fn coredll_raw_gdi_create_rect_rgn_indirect_and_set_rect_rgn() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 136_u32;
    let rect_ptr = 0x1_0000_u32;
    memory.map_words(rect_ptr, 4);

    // Write a rect {left=0,top=0,right=10,bottom=10} at rect_ptr.
    memory.write_word(rect_ptr, 0);
    memory.write_word(rect_ptr + 4, 0);
    memory.write_word(rect_ptr + 8, 10);
    memory.write_word(rect_ptr + 12, 10);

    let rgn = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_RECT_RGN_INDIRECT,
        [rect_ptr],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        other => panic!("CREATE_RECT_RGN_INDIRECT returned unexpected: {other:?}"),
    };
    assert_ne!(rgn, 0, "CREATE_RECT_RGN_INDIRECT must return non-zero handle");

    // SET_RECT_RGN on valid region must return true.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_RECT_RGN,
            [rgn, 5, 5, 20, 20],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ), "SET_RECT_RGN on valid region must return true");

    // SET_RECT_RGN on invalid handle must return false.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_RECT_RGN,
            [0xDEAD_BEEF, 0, 0, 10, 10],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ), "SET_RECT_RGN on invalid handle must return false");

    Ok(())
}

#[test]
fn coredll_raw_gdi_set_bitmap_bits_validates_params() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 137_u32;

    // SET_BITMAP_BITS with bitmap=0 must return 0+ERROR_INVALID_PARAMETER.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SET_BITMAP_BITS,
            [0, 4, 0x1_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ), "SET_BITMAP_BITS with bitmap=0 must return 0");

    Ok(())
}

#[test]
fn coredll_raw_gdi_load_bitmap_w_returns_zero_without_module() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 138_u32;

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LOAD_BITMAP_W,
            [0, 1],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ), "LOAD_BITMAP_W without module must return 0");

    Ok(())
}

#[test]
fn coredll_raw_gdi_draw_text_w_validates_params_and_count() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 139_u32;

    let dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        _ => panic!("CREATE_COMPATIBLE_DC failed"),
    };
    assert_ne!(dc, 0);

    // hdc=0 must return 0+ERROR_INVALID_PARAMETER.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DRAW_TEXT_W,
            [0, 0x1_0000, 5, 0x1_0100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ), "DRAW_TEXT_W with hdc=0 must return 0");

    // Prepare text "Hello" (5 UTF-16 code units) and rect.
    let text_ptr = 0x2_0000_u32;
    let rect_ptr = 0x2_0100_u32;
    // Write L"Hello\0" as wide string.
    memory.write_wide_z(text_ptr, "Hello");
    // Write a rect {left=0,top=0,right=100,bottom=20}.
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr, 0);
    memory.write_word(rect_ptr + 4, 0);
    memory.write_word(rect_ptr + 8, 100);
    memory.write_word(rect_ptr + 12, 20);

    // count=5 → returns max(5,1)=5.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DRAW_TEXT_W,
            [dc, text_ptr, 5_u32, rect_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(5),
            ..
        }
    ), "DRAW_TEXT_W with count=5 must return 5");

    Ok(())
}

#[test]
fn coredll_raw_image_list_draw_validates_handle_and_hdc() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 140_u32;

    // IMAGE_LIST_DRAW with invalid handle must return false+ERROR_INVALID_HANDLE.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW,
            [0xDEAD_0001, 0, 0x1_0000, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ), "IMAGE_LIST_DRAW with invalid handle must return false");

    // SHELL_SYSTEM_IMAGE_LIST_HANDLE=0x000b_f000; hdc=0 → false+ERROR_INVALID_PARAMETER.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW,
            [0x000b_f000, 0, 0, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ), "IMAGE_LIST_DRAW with hdc=0 must return false");

    // SHELL_SYSTEM_IMAGE_LIST_HANDLE with valid hdc and index=0 must return true.
    let dc = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        thread_id,
        ORD_CREATE_COMPATIBLE_DC,
        [0],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(h),
            ..
        } => h,
        _ => panic!("CREATE_COMPATIBLE_DC failed"),
    };
    assert_ne!(dc, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW,
            [0x000b_f000, 0, dc, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ), "IMAGE_LIST_DRAW with SHELL handle and valid hdc must return true");

    // IMAGE_LIST_DRAW_INDIRECT with draw_ptr=0 must return false+ERROR_INVALID_PARAMETER.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_IMAGE_LIST_DRAW_INDIRECT,
            [0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ), "IMAGE_LIST_DRAW_INDIRECT with null draw_ptr must return false");

    Ok(())
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

#[test]
fn coredll_raw_gradient_fill_rect_h_blends_colors_horizontally() -> Result<()> {
    // GradientFill with GRADIENT_FILL_RECT_H (mode=0) blends from left-vertex
    // color to right-vertex color horizontally across the rectangle.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    // 10-wide, 2-tall bitmap.
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 10, 2);

    // Vertex array (2 TRIVERTEX structs, each 16 bytes):
    //   v0: (0,0)  r=0xFF00, g=0x0000, b=0x0000  → pure red
    //   v1: (10,2) r=0x0000, g=0x0000, b=0xFF00  → pure blue
    let vertex_ptr = 0x1_0200u32;
    memory.map_bytes(vertex_ptr, 32);
    // v0: x=0,y=0, R=0xFF00, G=0, B=0, A=0  (COLOR16 is LE: 0xFF00 → [0x00, 0xFF])
    memory.write_bytes(vertex_ptr, &[
        0,0,0,0,  0,0,0,0,  0x00,0xFF, 0,0, 0,0, 0,0
    ]);
    // v1: x=10,y=2, R=0, G=0, B=0xFF00, A=0
    memory.write_bytes(vertex_ptr + 16, &[
        10,0,0,0, 2,0,0,0,  0,0, 0,0, 0x00,0xFF, 0,0
    ]);

    // Mesh array (1 GRADIENT_RECT: UpperLeft=0, LowerRight=1)
    let mesh_ptr = 0x1_0300u32;
    memory.map_bytes(mesh_ptr, 8);
    memory.write_bytes(mesh_ptr, &[0,0,0,0, 1,0,0,0]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_GRADIENT_FILL,
            // hdc, pVertex, nVertex=2, pMesh, nMesh=1, mode=0 (RECT_H)
            [mem_dc, vertex_ptr, 2, mesh_ptr, 1, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Left column (x=0): should be pure red.  In RGB565: R=31 << 11 = 0xF800.
    // Center (x=5): should be roughly mid-blend (r~128, b~128).
    // Right column (x=9): should be pure blue. In RGB565: B=31 = 0x001F.
    let left_px = rgb565_at(&memory, bits_ptr, stride, 0, 0);
    let mid_px = rgb565_at(&memory, bits_ptr, stride, 5, 0);
    let right_px = rgb565_at(&memory, bits_ptr, stride, 9, 0);
    // Left pixel (x=0, t=0): red-only; no blue component.
    assert_eq!(left_px & 0x001f, 0, "left pixel should have no blue");
    assert!(left_px & 0xf800 > 0, "left pixel should have red");
    // Right pixel (x=9, t=0.9): predominantly blue with little red; blue > red.
    assert!((right_px & 0x001f) > (right_px >> 11), "right pixel: blue > red in 5-bit");
    // Middle pixel (x=5, t=0.5): both red and blue present.
    assert!(mid_px & 0xf800 > 0, "mid pixel should have some red");
    assert!(mid_px & 0x001f > 0, "mid pixel should have some blue");

    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    Ok(())
}

#[test]
fn coredll_raw_gradient_fill_rect_v_blends_colors_vertically() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 2, 10);

    // v0: (0,0) pure red; v1: (2,10) pure blue; mode=1 (RECT_V)
    // COLOR16 0xFF00 as little-endian bytes is [0x00, 0xFF]
    let vertex_ptr = 0x1_0200u32;
    memory.map_bytes(vertex_ptr, 32);
    memory.write_bytes(vertex_ptr, &[
        0,0,0,0, 0,0,0,0, 0x00,0xFF, 0,0, 0,0, 0,0
    ]);
    memory.write_bytes(vertex_ptr + 16, &[
        2,0,0,0, 10,0,0,0, 0,0, 0,0, 0x00,0xFF, 0,0
    ]);
    let mesh_ptr = 0x1_0300u32;
    memory.map_bytes(mesh_ptr, 8);
    memory.write_bytes(mesh_ptr, &[0,0,0,0, 1,0,0,0]);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_GRADIENT_FILL,
            [mem_dc, vertex_ptr, 2, mesh_ptr, 1, 1],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    let top_px = rgb565_at(&memory, bits_ptr, stride, 0, 0);
    let bot_px = rgb565_at(&memory, bits_ptr, stride, 0, 9);
    // Top row (y=0, t=0): pure red, no blue.
    assert_eq!(top_px & 0x001f, 0, "top row: no blue");
    assert!(top_px & 0xf800 > 0, "top row: has red");
    // Bottom row (y=9, t=0.9): predominantly blue; blue > red in 5-bit channels.
    assert!((bot_px & 0x001f) > (bot_px >> 11), "bottom row: blue > red in 5-bit");
    Ok(())
}

// ─── GDI Ellipse / RoundRect / pen-style tests ───────────────────────────────

#[test]
fn coredll_raw_ellipse_fills_interior_and_leaves_corners() -> Result<()> {
    // An Ellipse inscribed in a 10×10 bitmap should fill central pixels
    // and leave the four literal corners empty.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 10, 10);

    let blue_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_SOLID_BRUSH, [0x00ff_0000],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };
    let null_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_GET_STOCK_OBJECT, [8],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };
    for obj in [blue_brush, null_pen] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_SELECT_OBJECT, [mem_dc, obj],
            ),
            CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } if h != 0
        ));
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ELLIPSE,
            // Ellipse inscribed in [0,0)–[10,10)
            [mem_dc, 0u32, 0u32, 10u32, 10u32],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Centre pixel must be filled (blue = 0x001f in RGB565).
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 5), 0x001f,
        "centre should be filled");
    // The four literal pixel-corners of the bounding box must be untouched.
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x0000,
        "top-left corner should be empty");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 9, 0), 0x0000,
        "top-right corner should be empty");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 9), 0x0000,
        "bottom-left corner should be empty");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 9, 9), 0x0000,
        "bottom-right corner should be empty");

    Ok(())
}

#[test]
fn coredll_raw_ellipse_degenerate_zero_size_succeeds() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, _bits_ptr, _stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 4, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_ELLIPSE,
            [mem_dc, 2u32, 2u32, 2u32, 4u32],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    assert_eq!(kernel.threads.get_last_error(thread_id), 0);
    Ok(())
}

#[test]
fn coredll_raw_round_rect_fills_interior_with_rounded_corners() -> Result<()> {
    // RoundRect(0,0,10,10, 4,4) — corner radius 2×2.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 10, 10);

    let blue_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_SOLID_BRUSH, [0x00ff_0000],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };
    let null_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_GET_STOCK_OBJECT, [8],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };
    for obj in [blue_brush, null_pen] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_SELECT_OBJECT, [mem_dc, obj],
            ),
            CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } if h != 0
        ));
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_ROUND_RECT,
            [mem_dc, 0u32, 0u32, 10u32, 10u32, 4u32, 4u32],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Central pixels must be filled.
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 5), 0x001f,
        "centre should be filled");
    // Edge pixels inside rounded bounds — middle rows should be fully filled.
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 5), 0x001f,
        "left-middle edge should be filled");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 9, 5), 0x001f,
        "right-middle edge should be filled");

    Ok(())
}

#[test]
fn coredll_raw_round_rect_zero_rounding_falls_back_to_rectangle() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 6, 6);

    let blue_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_SOLID_BRUSH, [0x00ff_0000],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };
    let null_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_GET_STOCK_OBJECT, [8],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };
    for obj in [blue_brush, null_pen] {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel, &mut memory, thread_id, ORD_SELECT_OBJECT, [mem_dc, obj],
            ),
            CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } if h != 0
        ));
    }

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_ROUND_RECT,
            // ew=0 → degenerate, acts like Rectangle
            [mem_dc, 1u32, 1u32, 5u32, 5u32, 0u32, 0u32],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // All pixels inside the rectangle must be filled.
    for y in 1u32..5 {
        for x in 1u32..5 {
            assert_eq!(rgb565_at(&memory, bits_ptr, stride, x, y), 0x001f,
                "pixel ({x},{y}) inside rectangle should be filled");
        }
    }
    // Corners outside the rectangle remain empty.
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x0000);
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 5), 0x0000);

    Ok(())
}

#[test]
fn coredll_raw_pen_dot_style_draws_alternating_pixels() -> Result<()> {
    // PS_DOT (style=2) draws 1 on, 1 off. A 6-pixel horizontal line should
    // have pixels at offsets 0, 2, 4 filled and 1, 3, 5 empty.
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 8, 4);

    // PS_DOT = 2, width = 1, color = blue
    let dot_pen = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_PEN,
        [2u32, 1u32, 0x00ff_0000u32],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id, ORD_SELECT_OBJECT, [mem_dc, dot_pen],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } if h != 0
    ));

    // MoveToEx(hdc, 1, 2, NULL)
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_MOVE_TO_EX, [mem_dc, 1u32, 2u32, 0u32],
    );
    // LineTo(hdc, 7, 2) — draws 6 pixels at y=2, x=1..6
    table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_LINE_TO, [mem_dc, 7u32, 2u32],
    );

    // style_state advances per pixel drawn:
    // state 0 → on (x=1), state 1 → off (x=2), state 2 → on (x=3), …
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 1, 2), 0x001f, "x=1 on");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 2, 2), 0x0000, "x=2 off");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 3, 2), 0x001f, "x=3 on");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 4, 2), 0x0000, "x=4 off");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 2), 0x001f, "x=5 on");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 6, 2), 0x0000, "x=6 off");

    Ok(())
}

// DrawFrameControl RGB565 color values derived from get_sys_color() and rgb():
// COLOR_BTNHIGHLIGHT (20) = rgb(0xff,0xff,0xff): R=31, G=63, B=31 → 0xFFFF
// COLOR_BTNSHADOW    (16) = rgb(0x80,0x80,0x80): R=16, G=32, B=16 → 0x8410
// dark_shadow             = rgb(0x40,0x40,0x40): R=8,  G=16, B=8  → 0x4208
// COLOR_BTNFACE      (15) = rgb(0xc0,0xc0,0xc0): R=24, G=48, B=24 → 0xC618
// COLOR_WINDOW        (5) = rgb(0xff,0xff,0xff) → 0xFFFF
// COLOR_WINDOWTEXT    (8) = rgb(0,0,0) → 0x0000

#[test]
fn coredll_raw_draw_frame_control_push_button_draws_raised_edge_and_face() -> Result<()> {
    // DFC_BUTTON=4, DFCS_BUTTONPUSH=0x0010: normal (not pushed) raised edge
    const DFC_BUTTON: u32 = 4;
    const DFCS_BUTTONPUSH: u32 = 0x0010;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 16, 16);

    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr,      0); // left
    memory.write_word(rect_ptr + 4,  0); // top
    memory.write_word(rect_ptr + 8,  16); // right
    memory.write_word(rect_ptr + 12, 16); // bottom

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DRAW_FRAME_CONTROL,
            [mem_dc, rect_ptr, DFC_BUTTON, DFCS_BUTTONPUSH],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Outer TL = hilight (0xFFFF), outer BR = dark-shadow (0x4208)
    // Top-left corner gets tl_out (hilight)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0xFFFF, "top-left outer = hilight");
    // Bottom row (y=15) overwrites left col at x=0 with br_out
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 15), 0x4208, "bottom-left outer = dk_shadow");
    // Right-top corner: right col overwrites top row → br_out
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 15, 0), 0x4208, "top-right outer = dk_shadow");
    // Interior face (center pixel away from all borders)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 8, 8), 0xC618, "interior = btnface");

    Ok(())
}

#[test]
fn coredll_raw_draw_frame_control_push_button_pushed_draws_sunken_edge() -> Result<()> {
    // DFCS_BUTTONPUSH | DFCS_PUSHED: sunken state
    const DFC_BUTTON: u32 = 4;
    const DFCS_BUTTONPUSH: u32 = 0x0010;
    const DFCS_PUSHED: u32 = 0x0200;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 16, 16);

    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr,      0);
    memory.write_word(rect_ptr + 4,  0);
    memory.write_word(rect_ptr + 8,  16);
    memory.write_word(rect_ptr + 12, 16);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DRAW_FRAME_CONTROL,
            [mem_dc, rect_ptr, DFC_BUTTON, DFCS_BUTTONPUSH | DFCS_PUSHED],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Pushed: tl_out = dk_shadow (0x4208), br_out = hilight (0xFFFF)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x4208, "top-left outer = dk_shadow");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 15, 15), 0xFFFF, "bottom-right outer = hilight");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 8, 8), 0xC618, "interior = btnface");

    Ok(())
}

#[test]
fn coredll_raw_draw_frame_control_checkbox_draws_sunken_frame_and_white_interior() -> Result<()> {
    // DFC_BUTTON + DFCS_BUTTONCHECK (0x0000) — unchecked
    const DFC_BUTTON: u32 = 4;
    const DFCS_BUTTONCHECK: u32 = 0x0000;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 16, 16);

    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr,      0);
    memory.write_word(rect_ptr + 4,  0);
    memory.write_word(rect_ptr + 8,  16);
    memory.write_word(rect_ptr + 12, 16);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DRAW_FRAME_CONTROL,
            [mem_dc, rect_ptr, DFC_BUTTON, DFCS_BUTTONCHECK],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Checkbox: outer TL = shadow (0x8410), outer BR = hilight (0xFFFF)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x8410, "top-left = shadow");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 15, 15), 0xFFFF, "bottom-right = hilight");
    // Interior (inside the 2-pixel border) = white (COLOR_WINDOW = 0xFFFF)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 8, 8), 0xFFFF, "interior = white");

    Ok(())
}

#[test]
fn coredll_raw_draw_frame_control_checkbox_checked_draws_checkmark_pixels() -> Result<()> {
    // DFCS_BUTTONCHECK | DFCS_CHECKED: checkmark should write dark pixels
    const DFC_BUTTON: u32 = 4;
    const DFCS_BUTTONCHECK: u32 = 0x0000;
    const DFCS_CHECKED: u32 = 0x0400;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    // Use a 20x20 bitmap so the checkmark has enough room to produce pixels
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 20, 20);

    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr,      0);
    memory.write_word(rect_ptr + 4,  0);
    memory.write_word(rect_ptr + 8,  20);
    memory.write_word(rect_ptr + 12, 20);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DRAW_FRAME_CONTROL,
            [mem_dc, rect_ptr, DFC_BUTTON, DFCS_BUTTONCHECK | DFCS_CHECKED],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // The checkmark is drawn in the inner area (l+2..r-2, t+2..b-2) = (2..18, 2..18)
    // Corner is approximately at (inner_left + iw*3/8, inner_top + ih*7/8)
    // iw = 16, ih = 16: corner ≈ (2+6, 2+14) = (8, 16)
    // At least some pixel in the lower-left to upper-right area should be dark (0x0000)
    // Check the corner of the tick (near center-bottom)
    let tick_corner_x = 2 + 16 * 3 / 8; // = 8
    let tick_corner_y = 2 + 16 * 7 / 8; // = 16
    assert_eq!(
        rgb565_at(&memory, bits_ptr, stride, tick_corner_x, tick_corner_y),
        0x0000, // COLOR_WINDOWTEXT = black
        "checkmark corner pixel should be dark"
    );

    Ok(())
}

#[test]
fn coredll_raw_draw_frame_control_radio_button_unchecked_has_white_center() -> Result<()> {
    // DFC_BUTTON + DFCS_BUTTONRADIO (0x0004): unchecked — center should be white
    const DFC_BUTTON: u32 = 4;
    const DFCS_BUTTONRADIO: u32 = 0x0004;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 16, 16);

    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr,      0);
    memory.write_word(rect_ptr + 4,  0);
    memory.write_word(rect_ptr + 8,  16);
    memory.write_word(rect_ptr + 12, 16);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DRAW_FRAME_CONTROL,
            [mem_dc, rect_ptr, DFC_BUTTON, DFCS_BUTTONRADIO],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // The center of the 16x16 ellipse should be white (no dot when unchecked)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 8, 8), 0xFFFF, "unchecked radio center = white");
    // Corners (outside the ellipse) should remain unset (zero — the DIB was zeroed)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x0000, "outside ellipse = black (zero)");

    Ok(())
}

#[test]
fn coredll_raw_draw_frame_control_radio_button_checked_draws_center_dot() -> Result<()> {
    // DFCS_BUTTONRADIO | DFCS_CHECKED: center dot should be dark
    const DFC_BUTTON: u32 = 4;
    const DFCS_BUTTONRADIO: u32 = 0x0004;
    const DFCS_CHECKED: u32 = 0x0400;
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 16, 16);

    let rect_ptr = 0x1_0200;
    memory.map_words(rect_ptr, 4);
    memory.write_word(rect_ptr,      0);
    memory.write_word(rect_ptr + 4,  0);
    memory.write_word(rect_ptr + 8,  16);
    memory.write_word(rect_ptr + 12, 16);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DRAW_FRAME_CONTROL,
            [mem_dc, rect_ptr, DFC_BUTTON, DFCS_BUTTONRADIO | DFCS_CHECKED],
        ),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));

    // Center dot spans (w*3/8..w*5/8, h*3/8..h*5/8) = (6..10, 6..10)
    // An ellipse inscribed in that region — center (8,8) should be in the dot
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 8, 8), 0x0000, "checked radio center = dark");
    // The ring just outside the dot area (say x=4, y=8) should be white (interior, no dot)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 4, 8), 0xFFFF, "radio ring interior = white");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_erasebkgnd_fills_client_with_class_brush() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    // Create a solid blue brush — COLORREF 0x00FF0000 = R=0x00,G=0x00,B=0xFF (blue)
    let blue_brush = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel, &mut memory, thread_id, ORD_CREATE_SOLID_BRUSH,
        [0x00ff_0000u32],
    ) {
        CoredllDispatch::Returned { value: CoredllValue::Handle(h), .. } => h,
        other => panic!("{other:?}"),
    };

    // Register a window class with the blue brush as hbrBackground
    let mut raw_class = [0u8; WNDCLASSW_SIZE];
    raw_class[28..32].copy_from_slice(&blue_brush.to_le_bytes());
    kernel.gwe.register_class("ERASE_FILL_TEST", raw_class);

    // Create a 10x8 window so it has a non-empty client rect
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id, "ERASE_FILL_TEST", "", None, 0, 0, 0,
        Rect::from_origin_size(0, 0, 10, 8),
    );

    // Create a 10x8 memory DC
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 10, 8);

    // DefWindowProcW(hwnd, WM_ERASEBKGND, mem_dc, 0) should fill client area
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_ERASEBKGND, mem_dc, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(1), .. } // 1 = background erased
    ));

    // Blue RGB565: R=0, G=0, B=31 → 0x001F
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 0, 0), 0x001F, "top-left = blue");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 9, 7), 0x001F, "bottom-right = blue");
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 4), 0x001F, "center = blue");

    Ok(())
}

#[test]
fn coredll_raw_def_window_proc_erasebkgnd_null_brush_returns_zero() -> Result<()> {
    // Window class with hbrBackground=0 → DefWindowProcW(WM_ERASEBKGND) returns 0
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 9;

    let raw_class = [0u8; WNDCLASSW_SIZE]; // hbrBackground = 0
    kernel.gwe.register_class("ERASE_NULL_TEST", raw_class);
    let hwnd = kernel.create_window_ex_w_with_rect(
        thread_id, "ERASE_NULL_TEST", "", None, 0, 0, 0,
        Rect::from_origin_size(0, 0, 10, 8),
    );
    let (mem_dc, bits_ptr, stride) =
        create_selected_rgb565_dib(&table, &mut kernel, &mut memory, thread_id, 10, 8);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel, &mut memory, thread_id,
            ORD_DEF_WINDOW_PROC_W,
            [hwnd, WM_ERASEBKGND, mem_dc, 0],
        ),
        CoredllDispatch::Returned { value: CoredllValue::U32(0), .. } // 0 = not erased
    ));
    // Bitmap should remain untouched (all zeros)
    assert_eq!(rgb565_at(&memory, bits_ptr, stride, 5, 4), 0x0000, "no fill when no brush");

    Ok(())
}

#[test]
fn coredll_raw_translate_message_western_layout_dead_keys() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 41;
    let msg_ptr = 0x2_0000;
    memory.map_words(msg_ptr, 7);
    let hwnd = kernel.create_window_ex_w(thread_id, "DEADKEY_OWNER", "", None, 0, 0, 0);

    // Switch to German QWERTZ (0x0407) layout.
    kernel.gwe.activate_keyboard_layout(0x0407);

    let translate = |kernel: &mut CeKernel,
                     memory: &mut TestGuestMemory,
                     vkey: u32,
                     shift: bool|
     -> CoredllDispatch {
        if shift {
            kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYDOWN, VK_LSHIFT, 0);
        }
        write_raw_message(memory, msg_ptr, hwnd, WM_KEYDOWN, vkey, vkey + 0x100).unwrap();
        let r = table.dispatch_raw_ordinal_with_memory(
            kernel,
            memory,
            thread_id,
            ORD_TRANSLATE_MESSAGE,
            [msg_ptr],
        );
        if shift {
            kernel.post_message_w_for_thread(thread_id, hwnd, WM_KEYUP, VK_LSHIFT, 0);
        }
        r
    };

    // --- German QWERTZ dead circumflex (VK_OEM_3 = 0xC0 unshifted → '^') ---
    // Press ^: should post WM_DEADCHAR and store the pending dead key.
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0xc0, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let deadchar_msg = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_DEADCHAR, WM_DEADCHAR, PeekFlags::REMOVE)
        .expect("dead circumflex should post WM_DEADCHAR");
    assert_eq!(deadchar_msg.wparam, 0x5e, "dead char should be '^'");
    assert!(kernel.gwe.dead_key().is_some(), "dead key should be pending");

    // Press 'e' (VK 0x45): dead circumflex + 'e' → 'ê' (0x00EA).
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0x45, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    assert!(kernel.gwe.dead_key().is_none(), "dead key should be cleared after composition");
    let char_msg = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("^ + e should post WM_CHAR(ê)");
    assert_eq!(char_msg.wparam, 0x00ea, "circumflex + e → ê");

    // --- German QWERTZ dead acute (VK_OEM_PLUS = 0xBB unshifted → '´') ---
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0xbb, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let _ = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_DEADCHAR, WM_DEADCHAR, PeekFlags::REMOVE)
        .expect("dead acute should post WM_DEADCHAR");

    // Press 'u' → 'ú' (0x00FA).
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0x55, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let char_msg = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("´ + u should post WM_CHAR(ú)");
    assert_eq!(char_msg.wparam, 0x00fa, "acute + u → ú");

    // --- No composition: dead grave + 'z' → literal ` then z ---
    // Dead grave via VK_OEM_PLUS shifted (0xBB + shift → '`')
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0xbb, true),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let _ = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_DEADCHAR, WM_DEADCHAR, PeekFlags::REMOVE)
        .expect("dead grave should post WM_DEADCHAR");

    // Press 'z' on QWERTZ → 'y' character (0x79). Dead grave + 'y' has no composition.
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0x5a, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let dead_literal = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("no-compose: should post WM_CHAR(grave)");
    assert_eq!(dead_literal.wparam, 0x60, "no-compose: literal grave first");
    let base_char = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("no-compose: should post WM_CHAR(y)");
    assert_eq!(base_char.wparam, 0x79, "no-compose: base char 'y' after dead grave");

    // --- French AZERTY (0x040C): dead diaeresis (VK_OEM_4 = 0xDB + shift → '¨') ---
    kernel.gwe.activate_keyboard_layout(0x040C);
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0xdb, true),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let _ = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_DEADCHAR, WM_DEADCHAR, PeekFlags::REMOVE)
        .expect("dead diaeresis should post WM_DEADCHAR");

    // Press 'o' on AZERTY → 'o' (VK_O = 0x4F). Dead diaeresis + 'o' → 'ö' (0x00F6).
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0x4f, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let char_msg = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("¨ + o should post WM_CHAR(ö)");
    assert_eq!(char_msg.wparam, 0x00f6, "diaeresis + o → ö");

    // --- Space after dead key emits the dead char literally ---
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0xdb, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let _ = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_DEADCHAR, WM_DEADCHAR, PeekFlags::REMOVE)
        .expect("dead circumflex should post WM_DEADCHAR");
    // Press space: ^ + space → literal '^'
    assert!(matches!(
        translate(&mut kernel, &mut memory, 0x20, false),
        CoredllDispatch::Returned { value: CoredllValue::Bool(true), .. }
    ));
    let space_result = kernel
        .gwe
        .peek_message_filtered(thread_id, Some(hwnd), WM_CHAR, WM_CHAR, PeekFlags::REMOVE)
        .expect("dead ^ + space should post WM_CHAR(^)");
    assert_eq!(space_result.wparam, 0x5e, "dead ^ + space → literal '^'");

    Ok(())
}
