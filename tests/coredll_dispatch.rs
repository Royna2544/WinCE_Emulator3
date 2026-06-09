use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        cemath::{CeMathBinaryF32, CeMathBinaryF64, CeMathCall, CeMathUnaryF64, CeMathValue},
        coredll::{
            CoredllCall, CoredllDispatch, CoredllExportTable, CoredllImplementationStatus,
            CoredllRawContext, CoredllStubAuditClassification, CoredllStubPolicy, CoredllSubsystem,
            CoredllValue, EventModifyAction,
        },
        coredll_ordinals::{
            ORD_ATOF, ORD_CLOSE_HANDLE, ORD_CREATE_APISET, ORD_CREATE_FILE_W,
            ORD_CREATE_WINDOW_EX_W, ORD_D_TO_LL, ORD_D_TO_ULL,
            ORD_DISPATCH_MESSAGE_W, ORD_DPCMP, ORD_DPMUL, ORD_DPTOFP, ORD_DPTOLI, ORD_EVENT_MODIFY,
            ORD_F_TO_LL, ORD_F_TO_ULL, ORD_FMODF, ORD_FPCMP, ORD_FPMUL, ORD_FPTODP, ORD_FPTOUL,
            ORD_GED, ORD_GES, ORD_GET_MESSAGE_W,
            ORD_GET_SYSTEM_TIME_AS_FILE_TIME, ORD_HYPOT,
            ORD_INITIALIZE_CRITICAL_SECTION, ORD_ISWCTYPE, ORD_LITODP, ORD_LITOFP, ORD_LL_DIV,
            ORD_LOAD_LIBRARY_W, ORD_LONGJMP, ORD_LTD, ORD_NES, ORD_POST_MESSAGE_W, ORD_POW,
            ORD_REG_OPEN_KEY_EX_W, ORD_REGISTER_GESTURE, ORD_SETJMP, ORD_SHELL_EXECUTE_EX,
            ORD_SLEEP, ORD_SQRT, ORD_ULTODP, ORD_WAIT_FOR_SINGLE_OBJECT,
            ORD_WRITE_FILE, current_static_export_count, lookup,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        gwe::WM_USER,
        kernel::CeKernel,
        thread::ERROR_NOT_SUPPORTED,
        timer::WAIT_OBJECT_0,
    },
    config::RuntimeConfig,
};

mod support;
use support::{TestGuestMemory, unique_test_root};

#[test]
fn coredll_table_reads_full_static_rust_ordinals() -> Result<()> {
    let table = CoredllExportTable::default();

    assert!(table.export_count() >= current_static_export_count());
    assert_eq!(
        table.resolve_name("CreateFileW").unwrap().ordinal,
        ORD_CREATE_FILE_W
    );
    assert_eq!(
        table.resolve_name("RegOpenKeyExW").unwrap().ordinal,
        ORD_REG_OPEN_KEY_EX_W
    );
    assert_eq!(
        table.resolve_name("GetMessageW").unwrap().ordinal,
        ORD_GET_MESSAGE_W
    );
    assert_eq!(
        table.resolve_name("DispatchMessageW").unwrap().ordinal,
        ORD_DISPATCH_MESSAGE_W
    );
    assert_eq!(table.resolve_name("sqrt").unwrap().ordinal, ORD_SQRT);
    assert_eq!(table.resolve_name("_hypot").unwrap().ordinal, ORD_HYPOT);
    assert_eq!(table.resolve_name("__ll_div").unwrap().ordinal, ORD_LL_DIV);
    assert_eq!(table.resolve_name("longjmp").unwrap().ordinal, ORD_LONGJMP);
    assert_eq!(table.resolve_name("_setjmp").unwrap().ordinal, ORD_SETJMP);
    assert_eq!(
        table.resolve_name("RegisterGesture").unwrap().ordinal,
        ORD_REGISTER_GESTURE
    );
    assert_eq!(
        CoredllExportTable::resolve_static_ordinal(ORD_CREATE_FILE_W)
            .unwrap()
            .name,
        "CreateFileW"
    );
    assert_eq!(ORD_CREATE_APISET, 559);
    assert_eq!(
        table.resolve_name("CreateAPISet").unwrap().ordinal,
        ORD_CREATE_APISET
    );
    assert!(table.resolve_name("ADBSetAccountProperties").is_none());
    assert!(table.resolve_ordinal(1943).is_none());
    assert!(CoredllExportTable::resolve_static_ordinal(1943).is_none());

    Ok(())
}

#[test]
fn coredll_ordinal_lookup_covers_static_sdk_and_supplemental_layers() {
    assert_eq!(lookup(ORD_SLEEP).unwrap().name, "Sleep");
    assert_eq!(lookup(ORD_LTD).unwrap().name, "__ltd");
    assert_eq!(
        lookup(ORD_GET_SYSTEM_TIME_AS_FILE_TIME).unwrap().name,
        "GetSystemTimeAsFileTime"
    );
    assert!(lookup(1943).is_none());
}

#[test]
fn cemath_evaluates_crt_and_mips_helper_calls() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let kernel = CeKernel::boot(config);

    assert_eq!(
        kernel.math.eval(CeMathCall::UnaryF64 {
            op: CeMathUnaryF64::Sqrt,
            value: 144.0,
        }),
        CeMathValue::F64(12.0)
    );
    assert_eq!(
        kernel.math.eval(CeMathCall::BinaryF64 {
            op: CeMathBinaryF64::Fmod,
            lhs: 17.5,
            rhs: 5.0,
        }),
        CeMathValue::F64(2.5)
    );
    assert_eq!(
        kernel.math.eval(CeMathCall::BinaryF32 {
            op: CeMathBinaryF32::Fmod,
            lhs: 17.5,
            rhs: 5.0,
        }),
        CeMathValue::F32(2.5)
    );
    assert_eq!(
        kernel.math.eval(CeMathCall::LlDiv { lhs: -21, rhs: 2 }),
        CeMathValue::I64(-10)
    );
    assert_eq!(
        kernel.math.eval(CeMathCall::UllRem { lhs: 22, rhs: 5 }),
        CeMathValue::U64(2)
    );

    Ok(())
}

#[test]
fn coredll_raw_dispatch_routes_mips_soft_float_compare_helpers() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 1;

    assert_eq!(table.resolve_name("__nes").unwrap().ordinal, ORD_NES);
    assert_eq!(table.resolve_name("__ltd").unwrap().ordinal, ORD_LTD);
    assert_eq!(table.resolve_name("__dpmul").unwrap().ordinal, ORD_DPMUL);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NES,
            [3.5_f32.to_bits(), 3.5_f32.to_bits()],
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
            ORD_FMODF,
            [17.5_f32.to_bits(), 5.0_f32.to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(value)),
            ..
        } if value.to_bits() == 2.5_f32.to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NES,
            [3.5_f32.to_bits(), 4.0_f32.to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_GES,
            [0x3d8f_5c29, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FPCMP,
            [1.0_f32.to_bits(), 2.0_f32.to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::Cmp(-1)),
            ..
        }
    ));

    let lhs = 1.25_f64.to_bits();
    let rhs = 2.5_f64.to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LTD,
            [
                lhs as u32,
                (lhs >> 32) as u32,
                rhs as u32,
                (rhs >> 32) as u32
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
            ORD_GED,
            [
                lhs as u32,
                (lhs >> 32) as u32,
                rhs as u32,
                (rhs >> 32) as u32
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
            ORD_DPCMP,
            [
                lhs as u32,
                (lhs >> 32) as u32,
                rhs as u32,
                (rhs >> 32) as u32
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::Cmp(-1)),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LITOFP,
            [192_000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(value)),
            ..
        } if value.to_bits() == (192_000_f32).to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FPMUL,
            [1.5_f32.to_bits(), (-2.0_f32).to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(value)),
            ..
        } if value.to_bits() == (-3.0_f32).to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FPTOUL,
            [17.75_f32.to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::U32(17)),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LITODP,
            [0xffff_ff9c],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(value)),
            ..
        } if value.to_bits() == (-100.0_f64).to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ULTODP,
            [4_000_000_000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(value)),
            ..
        } if value.to_bits() == (4_000_000_000_f64).to_bits()
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_FPTODP,
            [12.25_f32.to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(value)),
            ..
        } if value.to_bits() == (12.25_f64).to_bits()
    ));
    let dpmul_lhs = (-1.5_f64).to_bits();
    let dpmul_rhs = 2.0_f64.to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPMUL,
            [
                dpmul_lhs as u32,
                (dpmul_lhs >> 32) as u32,
                dpmul_rhs as u32,
                (dpmul_rhs >> 32) as u32,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(value)),
            ..
        } if value.to_bits() == (-3.0_f64).to_bits()
    ));
    let dptofp_value = 6.5_f64.to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPTOFP,
            [dptofp_value as u32, (dptofp_value >> 32) as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(value)),
            ..
        } if value.to_bits() == (6.5_f32).to_bits()
    ));
    let dptoli_value = (-42.25_f64).to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_DPTOLI,
            [dptoli_value as u32, (dptoli_value >> 32) as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I32(-42)),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_F_TO_LL,
            [(-42.75_f32).to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(-42)),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_F_TO_ULL,
            [(65_536.75_f32).to_bits()],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::U64(65_536)),
            ..
        }
    ));
    let d_to_ll_value = (-123_456.75_f64).to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_D_TO_LL,
            [d_to_ll_value as u32, (d_to_ll_value >> 32) as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(-123_456)),
            ..
        }
    ));
    let d_to_ull_value = 123_456.75_f64.to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_D_TO_ULL,
            [d_to_ull_value as u32, (d_to_ull_value >> 32) as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::U64(123_456)),
            ..
        }
    ));
    let sqrt_value = 81.0_f64.to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_SQRT,
            [sqrt_value as u32, (sqrt_value >> 32) as u32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(9.0)),
            ..
        }
    ));
    let pow_lhs = 3.0_f64.to_bits();
    let pow_rhs = 4.0_f64.to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_POW,
            [
                pow_lhs as u32,
                (pow_lhs >> 32) as u32,
                pow_rhs as u32,
                (pow_rhs >> 32) as u32,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(81.0)),
            ..
        }
    ));
    let hypot_lhs = 5.0_f64.to_bits();
    let hypot_rhs = 12.0_f64.to_bits();
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_HYPOT,
            [
                hypot_lhs as u32,
                (hypot_lhs >> 32) as u32,
                hypot_rhs as u32,
                (hypot_rhs >> 32) as u32,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(13.0)),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LL_DIV,
            [0x0989_6800, 0, 0x0098_9680, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(16)),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_dispatch_handles_iswctype_masks() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 1;

    assert_eq!(
        table.resolve_name("iswctype").unwrap().ordinal,
        ORD_ISWCTYPE
    );

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ISWCTYPE,
            ['A' as u32, 0x0101],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x0101),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ISWCTYPE,
            ['9' as u32, 0x0084],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x0084),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ISWCTYPE,
            ['한' as u32, 0x0100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0x0100),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ISWCTYPE,
            ['/' as u32, 0x0100],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_dispatcher_routes_ordinals_to_virtual_win32_framework() -> Result<()> {
    let table = CoredllExportTable::default();
    let root = unique_test_root("coredll_dispatcher");
    fs::create_dir_all(&root).unwrap();

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.set_file_root(&root);

    let file = table.dispatch_by_ordinal(
        &mut kernel,
        ORD_CREATE_FILE_W,
        CoredllCall::CreateFileW {
            path: "\\ResidentFlash\\dispatch.bin".to_owned(),
            desired_access: GENERIC_READ | GENERIC_WRITE,
            creation_disposition: CREATE_ALWAYS,
        },
    );
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(file),
        ..
    } = file
    else {
        panic!("CreateFileW did not return a file handle");
    };

    let written = table.dispatch_by_ordinal(
        &mut kernel,
        ORD_WRITE_FILE,
        CoredllCall::WriteFile {
            handle: file,
            data: b"dispatch".to_vec(),
        },
    );
    assert!(matches!(
        written,
        CoredllDispatch::Returned {
            value: CoredllValue::FileIo(_),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_CLOSE_HANDLE,
            CoredllCall::CloseHandle { handle: file },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        fs::read(root.join("ResidentFlash/dispatch.bin")).unwrap(),
        b"dispatch"
    );

    let event = table.dispatch_by_name(
        &mut kernel,
        "CreateEventW",
        CoredllCall::CreateEventW {
            name: Some("dispatch-event".to_owned()),
            manual_reset: false,
            initial_state: false,
        },
    );
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(event),
        ..
    } = event
    else {
        panic!("CreateEventW did not return an event handle");
    };
    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_EVENT_MODIFY,
            CoredllCall::EventModify {
                handle: event,
                action: EventModifyAction::Set,
            },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_WAIT_FOR_SINGLE_OBJECT,
            CoredllCall::WaitForSingleObject {
                handle: event,
                timeout_ms: 0,
                thread_id: 1,
            },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(WAIT_OBJECT_0),
            ..
        }
    ));

    let hwnd = table.dispatch_by_ordinal(
        &mut kernel,
        ORD_CREATE_WINDOW_EX_W,
        CoredllCall::CreateWindowExW {
            thread_id: 1,
            class_name: "STATIC".to_owned(),
            title: "dispatch-window".to_owned(),
            parent: None,
            id: 0,
            style: 0,
            ex_style: 0,
        },
    );
    let CoredllDispatch::Returned {
        value: CoredllValue::Handle(hwnd),
        ..
    } = hwnd
    else {
        panic!("CreateWindowExW did not return hwnd");
    };
    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_POST_MESSAGE_W,
            CoredllCall::PostMessageW {
                hwnd,
                msg: WM_USER + 7,
                wparam: 1,
                lparam: 2,
            },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_GET_MESSAGE_W,
            CoredllCall::GetMessageW { thread_id: 1 },
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::OptionalMessage(Some(_)),
            ..
        }
    ));

    let unimplemented = table.dispatch_untyped_ordinal(ORD_INITIALIZE_CRITICAL_SECTION);
    assert!(matches!(
        unimplemented,
        CoredllDispatch::Stubbed { export, stub }
            if export.name == "InitializeCriticalSection"
                && stub.policy == CoredllStubPolicy::VoidNoOp
                && stub.audit == CoredllStubAuditClassification::SafeNoOp
    ));
    let critical = table.dispatch_untyped_ordinal(ORD_SHELL_EXECUTE_EX);
    assert!(matches!(
        critical,
        CoredllDispatch::Stubbed { export, stub }
            if export.name == "ShellExecuteEx"
                && stub.audit == CoredllStubAuditClassification::MustImplement
    ));

    Ok(())
}

#[test]
fn coredll_raw_dispatch_routes_atof_as_double_return() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 1;
    let text = 0x2000;

    memory.map_bytes(text, 32);
    memory.write_bytes(text, b" \t-12.5e+1px\0");

    assert_eq!(table.resolve_name("atof").unwrap().ordinal, ORD_ATOF);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_ATOF,
            [text],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(value)),
            ..
        } if value.to_bits() == (-125.0_f64).to_bits()
    ));

    Ok(())
}

#[test]
fn raw_stub_audit_keeps_import_trap_context() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();

    let dispatch = table.dispatch_raw_ordinal_with_framebuffer_and_context(
        &mut kernel,
        &mut memory,
        None,
        CoredllRawContext {
            thread_id: 7,
            caller_pc: Some(0x0040_1234),
            trap_pc: Some(0x7fff_4000),
            caller_module: Some("dll:mfcce400.dll".to_owned()),
        },
        31, // "memchr" — CRT export without ORD_* constant; always stubbed
        [0, 0, 0, 0, 0],
    );

    assert!(
        matches!(
            dispatch,
            CoredllDispatch::Stubbed {
                ref export,
                ref stub,
            } if export.name == "memchr"
                && stub.audit == CoredllStubAuditClassification::SafeFailure
                && stub.context == Some(CoredllRawContext {
                    thread_id: 7,
                    caller_pc: Some(0x0040_1234),
                    trap_pc: Some(0x7fff_4000),
                    caller_module: Some("dll:mfcce400.dll".to_owned()),
                })
                && stub.last_error == None
                && stub.return_value == 0
        ),
        "{dispatch:?}"
    );
    assert_eq!(kernel.threads.get_last_error(7), 0);

    let load_stub = table.dispatch_untyped_ordinal(ORD_LOAD_LIBRARY_W);
    assert!(
        matches!(
            load_stub,
            CoredllDispatch::Stubbed {
                ref export,
                ref stub,
            } if export.name == "LoadLibraryW"
                && stub.audit == CoredllStubAuditClassification::MustImplement
                && stub.policy == CoredllStubPolicy::NullPointer
                && stub.last_error == Some(ERROR_NOT_SUPPORTED)
                && stub.return_value == 0
        ),
        "{load_stub:?}"
    );

    Ok(())
}

#[test]
fn coredll_raw_dispatch_has_defined_path_for_every_parsed_ordinal() -> Result<()> {
    let table = CoredllExportTable::default();
    let plan = table.ordinal_plan();
    assert_eq!(plan.len(), table.export_count());
    assert!(plan.iter().any(|item| {
        item.subsystem == CoredllSubsystem::Registry
            && item.status == CoredllImplementationStatus::Implemented
            && item.export.name == "RegOpenKeyExW"
    }));
    assert!(plan.iter().any(|item| {
        item.subsystem == CoredllSubsystem::GweWindow
            && item.status == CoredllImplementationStatus::Implemented
            && item.export.name == "CreateWindowExW"
    }));
    assert!(plan.iter().any(|item| {
        item.subsystem == CoredllSubsystem::Memory
            && item.status == CoredllImplementationStatus::Implemented
            && item.export.name == "LocalAlloc"
    }));

    let ordinals: Vec<_> = table.ordinals().collect();
    let mut covered = 0;
    for ordinal in ordinals.iter().copied() {
        match table.dispatch_raw_ordinal(ordinal, [0x1111_0000, 0x2222_0000]) {
            CoredllDispatch::Stubbed { stub, .. } => {
                assert_eq!(stub.args, vec![0x1111_0000, 0x2222_0000]);
                assert!(matches!(
                    stub.subsystem,
                    CoredllSubsystem::KernelSync
                        | CoredllSubsystem::ThreadProcess
                        | CoredllSubsystem::Memory
                        | CoredllSubsystem::FileSystem
                        | CoredllSubsystem::DeviceIo
                        | CoredllSubsystem::Registry
                        | CoredllSubsystem::GweWindow
                        | CoredllSubsystem::GweMessage
                        | CoredllSubsystem::GdiGraphics
                        | CoredllSubsystem::Multimedia
                        | CoredllSubsystem::LocaleString
                        | CoredllSubsystem::Time
                        | CoredllSubsystem::Crypto
                        | CoredllSubsystem::Comm
                        | CoredllSubsystem::Storage
                        | CoredllSubsystem::MsgQueue
                        | CoredllSubsystem::Power
                        | CoredllSubsystem::Services
                        | CoredllSubsystem::Telephony
                        | CoredllSubsystem::Security
                        | CoredllSubsystem::Debug
                        | CoredllSubsystem::InputIme
                        | CoredllSubsystem::ShellUi
                        | CoredllSubsystem::Bluetooth
                        | CoredllSubsystem::EventLog
                        | CoredllSubsystem::Credential
                        | CoredllSubsystem::MathCrt
                        | CoredllSubsystem::KernelPrivate
                ));
                covered += 1;
            }
            other => panic!("ordinal {ordinal} did not have raw dispatch coverage: {other:?}"),
        }
    }

    assert_eq!(covered, ordinals.len());

    Ok(())
}

#[test]
fn coredll_dispatcher_routes_cemath_ordinals() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_SQRT,
            CoredllCall::CeMath(CeMathCall::UnaryF64 {
                op: CeMathUnaryF64::Sqrt,
                value: 81.0,
            }),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(9.0)),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_POW,
            CoredllCall::CeMath(CeMathCall::BinaryF64 {
                op: CeMathBinaryF64::Pow,
                lhs: 3.0,
                rhs: 4.0,
            }),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(81.0)),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_HYPOT,
            CoredllCall::CeMath(CeMathCall::BinaryF64 {
                op: CeMathBinaryF64::Hypot,
                lhs: 5.0,
                rhs: 12.0,
            }),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F64(13.0)),
            ..
        }
    ));

    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_FMODF,
            CoredllCall::CeMath(CeMathCall::BinaryF32 {
                op: CeMathBinaryF32::Fmod,
                lhs: 17.5,
                rhs: 5.0,
            }),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::F32(value)),
            ..
        } if value.to_bits() == 2.5_f32.to_bits()
    ));

    assert!(matches!(
        table.dispatch_by_ordinal(
            &mut kernel,
            ORD_LL_DIV,
            CoredllCall::CeMath(CeMathCall::LlDiv { lhs: -21, rhs: 2 }),
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::CeMath(CeMathValue::I64(-10)),
            ..
        }
    ));

    Ok(())
}
