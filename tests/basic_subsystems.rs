use std::{fs, path::PathBuf};
use wince_emulation_v3::{
    Result,
    ce::{
        audio::{MMSYSERR_NOERROR, WaveBuffer, WaveFormat, WaveOutState},
        coredll::{
            CoredllCall, CoredllDispatch, CoredllExportTable, CoredllValue,
            DEFAULT_CORE_COMMON_DEF, EventModifyAction,
        },
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        gwe::{GWL_USERDATA, WM_CREATE, WM_QUIT, WM_TIMER, WM_USER},
        kernel::{CeKernel, MessagePumpResult},
        object::{EventObject, KernelObject},
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE, REG_SZ},
        timer::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    emulator::{
        memory::{MemoryMap, MemoryPerms},
        unicorn::UnicornMips,
    },
};

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
fn coredll_table_reads_full_core_common_def_ordinals() -> Result<()> {
    let table = CoredllExportTable::from_core_common_def_path(DEFAULT_CORE_COMMON_DEF)?;

    assert_eq!(table.export_count(), 1698);
    assert_eq!(table.resolve_name("CreateFileW").unwrap().ordinal, 168);
    assert_eq!(table.resolve_name("RegOpenKeyExW").unwrap().ordinal, 461);
    assert_eq!(table.resolve_name("waveOutOpen").unwrap().ordinal, 399);
    assert_eq!(table.resolve_name("GetMessageW").unwrap().ordinal, 861);
    assert_eq!(table.resolve_name("DispatchMessageW").unwrap().ordinal, 859);
    assert_eq!(table.resolve_ordinal(168).unwrap().name, "CreateFileW");
    assert_eq!(table.exports_by_ordinal(2867).len(), 1);
    assert_eq!(
        table.resolve_ordinal(2867).unwrap().name,
        "UserCallWindowProc"
    );

    Ok(())
}

#[test]
fn coredll_dispatcher_routes_ordinals_to_virtual_win32_framework() -> Result<()> {
    let table = CoredllExportTable::from_core_common_def_path(DEFAULT_CORE_COMMON_DEF)?;
    let root = unique_test_root("coredll_dispatcher");
    fs::create_dir_all(&root).unwrap();

    let config = RuntimeConfig::load("regs.json", "serial_devices.json")?;
    let mut kernel = CeKernel::boot(config);
    kernel.set_file_root(&root);

    let file = table.dispatch_by_ordinal(
        &mut kernel,
        168,
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
        171,
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
        table.dispatch_by_ordinal(&mut kernel, 553, CoredllCall::CloseHandle { handle: file }),
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
            494,
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
            497,
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
        246,
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
            865,
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
        table.dispatch_by_ordinal(&mut kernel, 861, CoredllCall::GetMessageW { thread_id: 1 }),
        CoredllDispatch::Returned {
            value: CoredllValue::OptionalMessage(Some(_)),
            ..
        }
    ));

    let unimplemented = table.dispatch_untyped_ordinal(2);
    assert!(matches!(
        unimplemented,
        CoredllDispatch::Unimplemented { export } if export.name == "InitializeCriticalSection"
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

fn unique_test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wince_emulation_v3_{name}_{}", std::process::id()))
}
