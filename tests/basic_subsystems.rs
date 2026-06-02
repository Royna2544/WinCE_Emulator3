use std::fs;
use wince_emulation_v3::{
    Result,
    ce::{
        audio::{MMSYSERR_NOERROR, WaveBuffer, WaveFormat, WaveOutState},
        com::{REGDB_E_CLASSNOTREG, S_FALSE, S_OK},
        file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE},
        gwe::{GWL_USERDATA, WM_QUIT, WM_TIMER, WM_USER},
        kernel::{CeKernel, MessagePumpResult},
        object::{EventObject, KernelObject},
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE, REG_SZ},
        remote::{WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP},
        timer::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    },
    config::RuntimeConfig,
    emulator::{
        memory::{MemoryMap, MemoryPerms},
        unicorn::UnicornMips,
    },
};

mod support;
use support::unique_test_root;

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
    assert!(kernel.gwe.get_message(42).is_none());
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
    assert_eq!(kernel.message_pump_step(7), MessagePumpResult::Idle);
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
    assert_eq!(kernel.message_pump_step(99), MessagePumpResult::Idle);

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

    let first_audio_client = kernel.remote.register_audio_client(1000);
    assert_eq!(kernel.remote.audio_client_count(), 1);
    assert_eq!(
        kernel.remote.publish_audio_chunk(vec![1, 2, 3, 4], 20),
        Some(1)
    );
    assert!(kernel.remote.audio_client_needs_flush(first_audio_client));
    let first_chunks = kernel
        .remote
        .take_audio_chunks_for_client(first_audio_client, 1);
    assert_eq!(first_chunks[0].pts_ms, 1000);
    assert!(first_chunks[0].flush);

    assert_eq!(
        kernel
            .remote
            .publish_audio_chunk(vec![10, 11, 12, 13, 14, 15, 16, 17], 40),
        Some(2)
    );
    let late_audio_client = kernel.remote.register_audio_client(1030);
    let late_chunks = kernel
        .remote
        .take_audio_chunks_for_client(late_audio_client, 1);
    assert_eq!(late_chunks[0].sequence, 2);
    assert_eq!(late_chunks[0].pts_ms, 1030);
    assert_eq!(late_chunks[0].duration_ms, 30);
    assert_eq!(late_chunks[0].payload, vec![12, 13, 14, 15, 16, 17]);
    assert!(late_chunks[0].flush);

    Ok(())
}
