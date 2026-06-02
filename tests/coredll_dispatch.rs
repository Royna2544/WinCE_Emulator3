use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        cemath::{CeMathBinaryF64, CeMathCall, CeMathUnaryF64, CeMathValue},
        coredll::{
            CoredllCall, CoredllDispatch, CoredllExportTable, CoredllImplementationStatus,
            CoredllStubPolicy, CoredllSubsystem, CoredllValue, EventModifyAction,
        },
        coredll_ordinals::{
            COREDLL_EXPORTS, ORD_CLOSE_HANDLE, ORD_CREATE_FILE_W, ORD_CREATE_WINDOW_EX_W,
            ORD_DISPATCH_MESSAGE_W, ORD_EVENT_MODIFY, ORD_GET_MESSAGE_W,
            ORD_INITIALIZE_CRITICAL_SECTION, ORD_LITOFP, ORD_LL_DIV, ORD_LONGJMP, ORD_LTD, ORD_NES,
            ORD_POST_MESSAGE_W, ORD_POW, ORD_REG_OPEN_KEY_EX_W, ORD_SETJMP, ORD_SQRT,
            ORD_USER_CALL_WINDOW_PROC, ORD_WAIT_FOR_SINGLE_OBJECT, ORD_WRITE_FILE, SDK_ORDINALS,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        gwe::WM_USER,
        kernel::CeKernel,
        timer::WAIT_OBJECT_0,
    },
    config::RuntimeConfig,
};

mod support;
use support::{TestGuestMemory, unique_test_root};

#[test]
fn coredll_table_reads_full_static_rust_ordinals() -> Result<()> {
    let table = CoredllExportTable::default();

    assert_eq!(
        table.export_count(),
        COREDLL_EXPORTS.len() + SDK_ORDINALS.len()
    );
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
    assert_eq!(table.resolve_name("__ll_div").unwrap().ordinal, ORD_LL_DIV);
    assert_eq!(table.resolve_name("longjmp").unwrap().ordinal, ORD_LONGJMP);
    assert_eq!(table.resolve_name("_setjmp").unwrap().ordinal, ORD_SETJMP);
    assert_eq!(
        CoredllExportTable::resolve_static_ordinal(ORD_CREATE_FILE_W)
            .unwrap()
            .name,
        "CreateFileW"
    );
    assert_eq!(table.exports_by_ordinal(ORD_USER_CALL_WINDOW_PROC).len(), 1);
    assert_eq!(
        table
            .resolve_ordinal(ORD_USER_CALL_WINDOW_PROC)
            .unwrap()
            .name,
        "UserCallWindowProc"
    );

    Ok(())
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

    memory.map_words(0x1000, 1);
    memory.map_words(0x2000, 1);
    memory.write_word(0x1000, 3.5_f32.to_bits());
    memory.write_word(0x2000, 3.5_f32.to_bits());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NES,
            [0x1000, 0x2000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(false),
            ..
        }
    ));
    memory.write_word(0x2000, 4.0_f32.to_bits());
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_NES,
            [0x1000, 0x2000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));

    let lhs = 1.25_f64.to_bits();
    let rhs = 2.5_f64.to_bits();
    memory.map_words(0x3000, 2);
    memory.map_words(0x4000, 2);
    memory.write_word(0x3000, lhs as u32);
    memory.write_word(0x3004, (lhs >> 32) as u32);
    memory.write_word(0x4000, rhs as u32);
    memory.write_word(0x4004, (rhs >> 32) as u32);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_LTD,
            [0x3000, 0x4000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
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
    ));

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

    let mut covered = 0;
    for ordinal in table.ordinals() {
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

    assert!(covered >= 1_700);

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
