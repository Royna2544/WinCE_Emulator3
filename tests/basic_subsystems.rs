use std::{collections::BTreeMap, fs, path::PathBuf};
use wince_emulation_v3::{
    Error, Result,
    ce::{
        audio::{MMSYSERR_NOERROR, WaveBuffer, WaveFormat, WaveOutState},
        cemath::{CeMathBinaryF64, CeMathCall, CeMathUnaryF64, CeMathValue},
        com::{REGDB_E_CLASSNOTREG, S_FALSE, S_OK},
        coredll::{
            CoredllCall, CoredllDispatch, CoredllExportTable, CoredllGuestMemory,
            CoredllImplementationStatus, CoredllStubPolicy, CoredllSubsystem, CoredllValue,
            EventModifyAction,
        },
        coredll_ordinals::{
            ORD_CLIENT_TO_SCREEN, ORD_CLOSE_HANDLE, ORD_CREATE_FILE_W, ORD_CREATE_WINDOW_EX_W,
            ORD_DESTROY_WINDOW, ORD_DISPATCH_MESSAGE_W, ORD_ENABLE_WINDOW, ORD_EVENT_MODIFY,
            ORD_FIND_RESOURCE_W, ORD_GET_CLASS_NAME_W, ORD_GET_CLIENT_RECT, ORD_GET_FOCUS,
            ORD_GET_LAST_ERROR, ORD_GET_MESSAGE_W, ORD_GET_PARENT, ORD_GET_PROCESS_HEAP,
            ORD_GET_TICK_COUNT, ORD_GET_WINDOW_LONG_W, ORD_GET_WINDOW_RECT,
            ORD_GET_WINDOW_TEXT_LENGTH_W, ORD_GET_WINDOW_TEXT_W, ORD_HEAP_ALLOC, ORD_HEAP_CREATE,
            ORD_HEAP_DESTROY, ORD_HEAP_FREE, ORD_HEAP_SIZE, ORD_INITIALIZE_CRITICAL_SECTION,
            ORD_INTERLOCKED_COMPARE_EXCHANGE, ORD_INTERLOCKED_EXCHANGE_ADD,
            ORD_INTERLOCKED_INCREMENT, ORD_IS_WINDOW, ORD_IS_WINDOW_ENABLED, ORD_IS_WINDOW_VISIBLE,
            ORD_LEAVE_CRITICAL_SECTION, ORD_LL_DIV, ORD_LOAD_RESOURCE, ORD_LOAD_STRING_W,
            ORD_LOCAL_ALLOC, ORD_LOCAL_FREE, ORD_LOCAL_RE_ALLOC, ORD_LOCAL_SIZE,
            ORD_MAP_WINDOW_POINTS, ORD_MOVE_WINDOW, ORD_PEEK_MESSAGE_W, ORD_POST_MESSAGE_W,
            ORD_POW, ORD_READ_FILE, ORD_REG_OPEN_KEY_EX_W, ORD_SCREEN_TO_CLIENT, ORD_SET_FOCUS,
            ORD_SET_LAST_ERROR, ORD_SET_WINDOW_LONG_W, ORD_SET_WINDOW_POS, ORD_SET_WINDOW_TEXT_W,
            ORD_SHOW_WINDOW, ORD_SIZEOF_RESOURCE, ORD_SLEEP, ORD_SQRT, ORD_TLS_GET_VALUE,
            ORD_TLS_SET_VALUE, ORD_TRY_ENTER_CRITICAL_SECTION, ORD_USER_CALL_WINDOW_PROC,
            ORD_VIRTUAL_ALLOC, ORD_VIRTUAL_FREE, ORD_WAIT_FOR_SINGLE_OBJECT, ORD_WRITE_FILE,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE, OPEN_EXISTING},
        gwe::{GWL_USERDATA, WM_CREATE, WM_QUIT, WM_TIMER, WM_USER},
        kernel::{CeKernel, MessagePumpResult},
        memory::{HEAP_NO_SERIALIZE, HEAP_ZERO_MEMORY, LMEM_ZEROINIT, MEM_COMMIT, MEM_RELEASE},
        object::{EventObject, KernelObject},
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE, REG_SZ},
        remote::{WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP},
        resource::ResourceId,
        thread::ERROR_INVALID_PARAMETER,
        timer::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    emulator::{
        memory::{MemoryMap, MemoryPerms},
        unicorn::UnicornMips,
    },
};

#[derive(Debug, Default)]
struct TestGuestMemory {
    bytes: BTreeMap<u32, u8>,
    words: BTreeMap<u32, u32>,
    halfwords: BTreeMap<u32, u16>,
}

impl TestGuestMemory {
    fn map_bytes(&mut self, base: u32, bytes: u32) {
        for index in 0..bytes {
            self.bytes.insert(base.wrapping_add(index), 0);
        }
    }

    fn map_words(&mut self, base: u32, words: u32) {
        for index in 0..words {
            self.write_word(base.wrapping_add(index * 4), 0);
        }
    }

    fn map_halfwords(&mut self, base: u32, halfwords: u32) {
        for index in 0..halfwords {
            self.halfwords.insert(base.wrapping_add(index * 2), 0);
        }
    }

    fn write_word(&mut self, addr: u32, value: u32) {
        self.words.insert(addr, value);
    }

    fn write_bytes(&mut self, addr: u32, bytes: &[u8]) {
        for (index, byte) in bytes.iter().copied().enumerate() {
            self.bytes.insert(addr + index as u32, byte);
        }
    }

    fn read_bytes(&self, addr: u32, len: usize) -> Vec<u8> {
        (0..len)
            .map(|index| self.bytes.get(&(addr + index as u32)).copied().unwrap_or(0))
            .collect()
    }

    fn read_i32(&self, addr: u32) -> Result<i32> {
        Ok(self.read_u32(addr)? as i32)
    }

    fn write_point(&mut self, addr: u32, x: i32, y: i32) {
        self.write_word(addr, x as u32);
        self.write_word(addr + 4, y as u32);
    }

    fn write_wide_z(&mut self, addr: u32, text: &str) {
        for (index, unit) in text.encode_utf16().chain(std::iter::once(0)).enumerate() {
            self.halfwords.insert(addr + (index as u32) * 2, unit);
        }
    }

    fn read_wide_z(&self, addr: u32, max_chars: usize) -> String {
        let mut units = Vec::new();
        for index in 0..max_chars {
            let unit = self
                .halfwords
                .get(&(addr + (index as u32) * 2))
                .copied()
                .unwrap_or(0);
            if unit == 0 {
                break;
            }
            units.push(unit);
        }
        String::from_utf16_lossy(&units)
    }
}

impl CoredllGuestMemory for TestGuestMemory {
    fn read_u8(&self, addr: u32) -> Result<u8> {
        self.bytes
            .get(&addr)
            .copied()
            .ok_or_else(|| Error::Backend(format!("unmapped test byte 0x{addr:08x}")))
    }

    fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        if let Some(byte) = self.bytes.get_mut(&addr) {
            *byte = value;
            Ok(())
        } else {
            Err(Error::Backend(format!("unmapped test byte 0x{addr:08x}")))
        }
    }

    fn read_u32(&self, addr: u32) -> Result<u32> {
        self.words
            .get(&addr)
            .copied()
            .ok_or_else(|| Error::Backend(format!("unmapped test word 0x{addr:08x}")))
    }

    fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        if let Some(word) = self.words.get_mut(&addr) {
            *word = value;
            Ok(())
        } else {
            Err(Error::Backend(format!("unmapped test word 0x{addr:08x}")))
        }
    }

    fn read_u16(&self, addr: u32) -> Result<u16> {
        self.halfwords
            .get(&addr)
            .copied()
            .ok_or_else(|| Error::Backend(format!("unmapped test halfword 0x{addr:08x}")))
    }

    fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        if let Some(halfword) = self.halfwords.get_mut(&addr) {
            *halfword = value;
            Ok(())
        } else {
            Err(Error::Backend(format!(
                "unmapped test halfword 0x{addr:08x}"
            )))
        }
    }
}

#[test]
fn boots_and_smokes_basic_ce_subsystems() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    let ident_key = kernel
        .registry
        .reg_open_key_exw(HKEY_LOCAL_MACHINE, Some("Ident"), 0, 0);
    assert_eq!(ident_key.status, ERROR_SUCCESS);
    let ident_key = ident_key.hkey.unwrap();

    let name = kernel
        .registry
        .reg_query_value_exw(ident_key, Some("Name"), Some(128));
    assert_eq!(name.status, ERROR_SUCCESS);
    assert_eq!(name.value_type, Some(REG_SZ));
    assert!(name.required_len > 2);
    assert_eq!(kernel.registry.reg_close_key(ident_key), ERROR_SUCCESS);

    let serial = kernel.devices.open("COM7")?;
    assert_eq!(serial.guest_name, "COM7:");

    let hwnd = kernel.gwe.create_window(42, "SmokeWindow", "smoke");
    assert_eq!(kernel.gwe.get_message(42).unwrap().msg, WM_CREATE);
    let timer_id = kernel.timers.set_timer(Some(hwnd), Some(77), 0, WM_TIMER);
    assert_eq!(timer_id, 77);
    kernel.pump_timers_to_gwe(42);
    let timer_msg = kernel.gwe.get_message(42).unwrap();
    assert_eq!(timer_msg.hwnd, hwnd);
    assert_eq!(timer_msg.msg, WM_TIMER);
    assert_eq!(timer_msg.wparam, 77);

    let wave_id = kernel.audio.open_wave_out(WaveFormat::pcm_16bit(2, 44_100));
    assert!(kernel.audio.write(
        wave_id,
        WaveBuffer {
            guest_ptr: 0x2000,
            len: 512,
        },
    ));
    assert_eq!(
        kernel.audio.output(wave_id).unwrap().state,
        WaveOutState::Playing
    );
    assert_eq!(
        kernel
            .audio
            .complete_next_buffer(wave_id)
            .unwrap()
            .guest_ptr,
        0x2000
    );

    let event_handle = kernel.handles.insert(KernelObject::Event(EventObject {
        name: Some("smoke".to_owned()),
        manual_reset: true,
        signaled: false,
    }));
    assert!(matches!(
        kernel.handles.get(event_handle).unwrap(),
        KernelObject::Event(_)
    ));
    kernel.handles.close(event_handle)?;

    let mut cpu = UnicornMips::new()?;
    cpu.map_region(
        0x1000_0000,
        0x0001_0000,
        MemoryPerms::READ | MemoryPerms::WRITE,
        "smoke-ram",
    )?;
    assert_eq!(cpu.memory().regions().count(), 1);

    Ok(())
}

#[test]
fn coredll_table_reads_full_static_rust_ordinals() -> Result<()> {
    let table = CoredllExportTable::default();

    assert_eq!(table.export_count(), 1752);
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
fn coredll_raw_ordinals_execute_kernel_thread_time_and_sync_semantics() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
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
    assert!(matches!(
        table
            .dispatch_raw_ordinal_with_memory(&mut kernel, &mut memory, thread_id, ORD_SLEEP, [0],),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    let event = kernel.create_event_w(Some("raw-event".to_owned()), false, false);
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

    Ok(())
}

#[test]
fn coredll_raw_memory_and_file_ordinals_use_virtual_ce_heap_and_guest_buffers() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    let root = unique_test_root("raw_memory_file");
    fs::create_dir_all(&root).unwrap();
    kernel.set_file_root(&root);
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
    memory.write_wide_z(class_ptr, "CHILD");
    memory.write_wide_z(title_ptr, "child");
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
    assert_eq!(kernel.gwe.window(child).unwrap().class_name, "CHILD");

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
            value: CoredllValue::U32(1),
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
    assert_eq!(memory.read_wide_z(class_buffer, 32), "CHILD");
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

    let resource = kernel.resources.register(
        0x4000,
        ResourceId::Integer(10),
        ResourceId::Integer(6),
        0x5000,
        32,
    );
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
fn com_subsystem_tracks_apartments_and_registered_class_creation() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);

    assert_eq!(kernel.com.co_initialize_ex(17, 0), S_OK);
    assert_eq!(kernel.com.co_initialize_ex(17, 0), S_FALSE);
    assert_eq!(
        kernel.com.co_create_instance(0x7000, 0x7010),
        Err(REGDB_E_CLASSNOTREG)
    );
    kernel.com.register_class(0x7000, 0x99);
    let object = kernel.com.co_create_instance(0x7000, 0x7010).unwrap();
    assert_eq!(kernel.com.object(object).unwrap().clsid_ptr, 0x7000);
    kernel.com.co_uninitialize(17);
    kernel.com.co_uninitialize(17);

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

#[test]
fn memory_map_rejects_overlaps() -> Result<()> {
    let mut map = MemoryMap::default();
    map.map(0x2000_0000, 0x0001_0000, MemoryPerms::READ, "first")?;

    let overlap = map.map(0x2000_8000, 0x0001_0000, MemoryPerms::READ, "overlap");
    assert!(overlap.is_err());

    Ok(())
}

#[test]
fn virtual_win32_api_smoke_covers_file_device_sync_gwe_and_audio() -> Result<()> {
    let root = unique_test_root("virtual_win32_api_smoke");
    fs::create_dir_all(&root).unwrap();

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.set_file_root(&root);

    let file = kernel.create_file_w(
        "\\ResidentFlash\\test.bin",
        GENERIC_READ | GENERIC_WRITE,
        CREATE_ALWAYS,
    )?;
    let written = kernel.write_file(file, b"abc")?;
    assert!(written.success);
    assert_eq!(written.bytes_transferred, 3);
    assert!(kernel.close_handle(file)?);
    assert_eq!(
        fs::read(root.join("ResidentFlash/test.bin")).unwrap(),
        b"abc"
    );

    let com = kernel.create_file_w("COM3:", GENERIC_READ | GENERIC_WRITE, CREATE_ALWAYS)?;
    let serial_write = kernel.write_file(com, b"$PMTK")?;
    assert_eq!(serial_write.bytes_transferred, 5);
    let serial_ioctl = kernel.device_io_control(com, 0x1234, &[], 16)?;
    assert!(!serial_ioctl.success);
    assert!(kernel.close_handle(com)?);

    let uid = kernel.create_file_w("UID1:", GENERIC_READ, CREATE_ALWAYS)?;
    let uid_ioctl = kernel.device_io_control(uid, 0x2222, &[], 16)?;
    assert!(uid_ioctl.success);
    assert!(uid_ioctl.bytes_returned > 0);
    assert!(kernel.close_handle(uid)?);

    let auto_event = kernel.create_event_w(Some("auto".to_owned()), false, true);
    assert_eq!(
        kernel.wait_for_single_object(auto_event, 0, 7),
        WAIT_OBJECT_0
    );
    assert_eq!(
        kernel.wait_for_single_object(auto_event, 0, 7),
        WAIT_TIMEOUT
    );
    assert!(kernel.set_event(auto_event));
    assert_eq!(
        kernel.wait_for_single_object(auto_event, 0, 7),
        WAIT_OBJECT_0
    );

    let mutex = kernel.create_mutex_w(Some("mx".to_owned()), None);
    assert_eq!(kernel.wait_for_single_object(mutex, 0, 7), WAIT_OBJECT_0);
    assert!(kernel.release_mutex(mutex, 7));

    let hwnd = kernel.create_window_ex_w(7, "STATIC", "old", None, 100, 0x4000_0000, 0);
    assert_eq!(
        kernel.message_pump_step(7),
        MessagePumpResult::Dispatched(0)
    );
    assert!(kernel.gwe.set_window_text(hwnd, "new title"));
    assert_eq!(
        kernel.gwe.get_window_text(hwnd, 32).as_deref(),
        Some("new title")
    );
    assert_eq!(
        kernel.gwe.set_window_long(hwnd, GWL_USERDATA, 0xbeef),
        Some(0)
    );
    assert_eq!(kernel.gwe.get_window_long(hwnd, GWL_USERDATA), Some(0xbeef));
    assert!(kernel.post_message_w(hwnd, WM_USER + 1, 11, 22));
    assert_eq!(
        kernel.message_pump_step(7),
        MessagePumpResult::Dispatched(0)
    );
    kernel.gwe.post_quit_message(7, 99, 0);
    assert_eq!(kernel.message_pump_step(7), MessagePumpResult::Quit(99));
    assert!(
        kernel
            .gwe
            .peek_message_filtered(
                7,
                None,
                WM_QUIT,
                WM_QUIT,
                wince_emulation_v3::ce::gwe::PeekFlags::NO_REMOVE
            )
            .is_none()
    );

    let wave = kernel
        .wave_out_open(WaveFormat::pcm_16bit(2, 44_100))
        .unwrap();
    assert_eq!(
        kernel.wave_out_write(
            wave,
            WaveBuffer {
                guest_ptr: 0x3000,
                len: 1024,
            },
        ),
        MMSYSERR_NOERROR
    );
    assert_eq!(kernel.audio.pause(wave), MMSYSERR_NOERROR);
    assert_eq!(
        kernel.audio.output(wave).unwrap().state,
        WaveOutState::Paused
    );
    assert_eq!(kernel.audio.restart(wave), MMSYSERR_NOERROR);

    Ok(())
}

#[test]
fn remote_server_api_state_queues_input_serial_audio_and_status() -> Result<()> {
    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.remote.set_framebuffer_size(800, 480);

    let hwnd = kernel.create_window_ex_w(99, "REMOTE", "remote", None, 1, 0, 0);
    assert_eq!(
        kernel.message_pump_step(99),
        MessagePumpResult::Dispatched(0)
    );

    let touch = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "touch",
        "phase": "tap",
        "x": 12,
        "y": 34
    }));
    assert_eq!(touch["ok"], true);

    let key = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "key",
        "phase": "down",
        "vk": 0x26
    }));
    assert_eq!(key["ok"], true);

    assert_eq!(kernel.drain_remote_input_to_gwe(99, hwnd), 3);
    let down = kernel.gwe.get_message(99).unwrap();
    assert_eq!(down.msg, WM_LBUTTONDOWN);
    assert_eq!(down.lparam & 0xffff, 12);
    assert_eq!((down.lparam >> 16) & 0xffff, 34);
    assert_eq!(kernel.gwe.get_message(99).unwrap().msg, WM_LBUTTONUP);
    let key_down = kernel.gwe.get_message(99).unwrap();
    assert_eq!(key_down.msg, WM_KEYDOWN);
    assert_eq!(key_down.wparam, 0x26);

    let nmea = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "nmea",
        "sentences": ["$GPRMC,remote*00"]
    }));
    assert_eq!(nmea["accepted"], 1);
    assert_eq!(kernel.read_remote_serial_bytes(64), b"$GPRMC,remote*00\r\n");

    let location = kernel.dispatch_remote_control_message(&serde_json::json!({
        "type": "location",
        "lat": 37.5,
        "lon": 127.25,
        "timestampMs": 0
    }));
    assert_eq!(location["sentencesGenerated"], 3);
    assert!(kernel.remote.serial_byte_count() > 0);

    assert_eq!(
        kernel.dispatch_remote_control_message(&serde_json::json!({"type": "pause"}))["paused"],
        true
    );
    let status = kernel.remote_status_json();
    assert_eq!(status["paused"], true);
    assert_eq!(status["guest_width"], 800);
    assert!(
        status["gps_target"]
            .as_str()
            .is_some_and(|target| !target.is_empty())
    );

    kernel.remote.register_audio_client(1000);
    assert_eq!(kernel.remote.audio_client_count(), 1);
    assert_eq!(
        kernel.remote.publish_audio_chunk(vec![1, 2, 3, 4], 20),
        Some(1)
    );
    assert_eq!(kernel.remote.take_audio_chunks(1)[0].pts_ms, 1000);

    Ok(())
}

fn unique_test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wince_emulation_v3_{name}_{}", std::process::id()))
}
