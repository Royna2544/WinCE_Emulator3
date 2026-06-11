use wince_emulation_v3::{
    Result,
    ce::{
        audio::{
            HostAudioSink, MMSYSERR_BADDEVICEID, MMSYSERR_INVALHANDLE, MMSYSERR_NOERROR,
            WaveOutCallback,
        },
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{
            ORD_WAVE_OUT_BREAK_LOOP, ORD_WAVE_OUT_CLOSE, ORD_WAVE_OUT_GET_DEV_CAPS,
            ORD_WAVE_OUT_GET_ERROR_TEXT, ORD_WAVE_OUT_GET_ID, ORD_WAVE_OUT_GET_NUM_DEVS,
            ORD_WAVE_OUT_GET_PITCH, ORD_WAVE_OUT_GET_PLAYBACK_RATE, ORD_WAVE_OUT_GET_POSITION,
            ORD_WAVE_OUT_GET_VOLUME, ORD_WAVE_OUT_MESSAGE, ORD_WAVE_OUT_OPEN, ORD_WAVE_OUT_PAUSE,
            ORD_WAVE_OUT_PREPARE_HEADER, ORD_WAVE_OUT_RESET, ORD_WAVE_OUT_RESTART,
            ORD_WAVE_OUT_SET_PITCH, ORD_WAVE_OUT_SET_PLAYBACK_RATE, ORD_WAVE_OUT_SET_VOLUME,
            ORD_WAVE_OUT_UNPREPARE_HEADER, ORD_WAVE_OUT_WRITE,
        },
        kernel::CeKernel,
    },
    config::RuntimeConfig,
};

mod support;
use support::TestGuestMemory;

#[test]
fn coredll_raw_waveout_ordinals_use_unplugged_audio_adapter() -> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut host_sink = HostAudioSink::named_unplugged("host-test", 4);
    host_sink.connect();
    assert!(kernel.audio.register_sink(host_sink));
    let mut memory = TestGuestMemory::default();
    let thread_id = 12;
    let handle_ptr = 0x1_5000;
    let format_ptr = 0x1_5100;
    let header_ptr = 0x1_5200;
    let volume_ptr = 0x1_5300;
    let time_ptr = 0x1_5400;

    memory.map_words(handle_ptr, 1);
    memory.write_wave_format_pcm(format_ptr, 2, 44_100);
    memory.map_words(header_ptr, 8);
    memory.write_bytes(0x2_0000, &[0x10; 2048]);
    memory.write_word(header_ptr, 0x2_0000);
    memory.write_word(header_ptr + 4, 2048);
    memory.write_word(header_ptr + 16, 0);
    memory.map_words(volume_ptr, 1);
    memory.map_words(time_ptr, 2);
    memory.write_word(time_ptr, 4);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_NUM_DEVS,
            [],
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
            ORD_WAVE_OUT_OPEN,
            [handle_ptr, u32::MAX, format_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    let wave = memory.read_u32(handle_ptr)?;
    assert!(wave != 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_PREPARE_HEADER,
            [wave, header_ptr, 32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(header_ptr + 16)? & 0x2, 0x2);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_WRITE,
            [wave, header_ptr, 32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(header_ptr + 16)? & 0x10, 0x10);
    assert_eq!(kernel.audio.output(wave).unwrap().submitted_bytes, 2048);
    assert_eq!(kernel.audio.queued_sink_chunk_count("host-test"), Some(1));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_SET_VOLUME,
            [wave, 0x8000_8000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_VOLUME,
            [wave, volume_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(volume_ptr)?, 0x8000_8000);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_SET_PLAYBACK_RATE,
            [wave, 0x0002_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_PLAYBACK_RATE,
            [wave, volume_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(volume_ptr)?, 0x0002_0000);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_POSITION,
            [wave, time_ptr, 8],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(time_ptr + 4)?, 2048);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_PAUSE,
            [wave],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_RESTART,
            [wave],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_UNPREPARE_HEADER,
            [wave, header_ptr, 32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(header_ptr + 16)? & 0x2, 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_CLOSE,
            [wave],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));

    Ok(())
}

#[test]
fn coredll_raw_waveout_function_callback_is_not_event_handle() -> Result<()> {
    const CALLBACK_FUNCTION: u32 = 0x0003_0000;

    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    let thread_id = 12;
    let handle_ptr = 0x1_5000;
    let format_ptr = 0x1_5100;
    let header_ptr = 0x1_5200;
    let callback_ptr = 0x1d_6608;

    memory.map_words(handle_ptr, 1);
    memory.write_wave_format_pcm(format_ptr, 2, 44_100);
    memory.map_words(header_ptr, 8);
    memory.write_word(header_ptr, 0x2_0000);
    memory.write_word(header_ptr + 4, 2048);
    memory.write_word(header_ptr + 16, 0);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_OPEN,
            [
                handle_ptr,
                u32::MAX,
                format_ptr,
                callback_ptr,
                0,
                CALLBACK_FUNCTION,
            ],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    let wave = memory.read_u32(handle_ptr)?;
    assert_eq!(
        kernel.audio.output(wave).unwrap().callback,
        Some(WaveOutCallback::Function {
            callback: callback_ptr,
            instance: 0,
        })
    );
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_WRITE,
            [wave, header_ptr, 32],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(header_ptr + 16)? & 0x10, 0x10);
    Ok(())
}

#[test]
fn coredll_raw_waveout_break_loop_message_get_dev_caps_get_error_text_get_id_reset_pitch()
-> Result<()> {
    let table = CoredllExportTable::default();
    let config = RuntimeConfig::load_default()?;
    let mut kernel = CeKernel::boot(config);
    let mut host_sink = HostAudioSink::named_unplugged("host-test2", 4);
    host_sink.connect();
    assert!(kernel.audio.register_sink(host_sink));
    let mut memory = TestGuestMemory::default();
    let thread_id = 13_u32;
    let handle_ptr = 0x1_6000_u32;
    let format_ptr = 0x1_6100_u32;
    let id_ptr = 0x3000_0000_u32;
    let pitch_ptr = 0x3000_0010_u32;
    let caps_ptr = 0x3000_0100_u32;
    let error_buf = 0x3000_0400_u32;

    memory.map_words(handle_ptr, 1);
    memory.write_wave_format_pcm(format_ptr, 1, 22_050);

    // Open a waveout to get a valid handle.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_OPEN,
            [handle_ptr, u32::MAX, format_ptr, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    let wave = memory.read_u32(handle_ptr)?;
    assert_ne!(wave, 0);

    // BREAK_LOOP always returns MMSYSERR_NOERROR.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_BREAK_LOOP,
            [wave],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));

    // MESSAGE always returns U32(0).
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_MESSAGE,
            [wave, 0, 0, 0],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));

    // GET_DEV_CAPS with valid device_id=0 → MMSYSERR_NOERROR.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_DEV_CAPS,
            [0, caps_ptr, 84],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));

    // GET_DEV_CAPS with bad device_id → MMSYSERR_BADDEVICEID.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_DEV_CAPS,
            [99, caps_ptr, 84],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_BADDEVICEID),
            ..
        }
    ));

    // GET_ERROR_TEXT for MMSYSERR_NOERROR → writes "No error", returns MMSYSERR_NOERROR.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_ERROR_TEXT,
            [0, error_buf, 64],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));

    // GET_ERROR_TEXT for MMSYSERR_INVALHANDLE(5) → writes "Invalid handle".
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_ERROR_TEXT,
            [MMSYSERR_INVALHANDLE as u32, error_buf, 64],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));

    // GET_ID on valid handle → writes device index 0, returns MMSYSERR_NOERROR.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_ID,
            [wave, id_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(id_ptr)?, 0);

    // GET_ID on invalid handle → MMSYSERR_INVALHANDLE.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_ID,
            [0xDEAD_BEEF, id_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_INVALHANDLE),
            ..
        }
    ));

    // SET_PITCH to 2× (0x0002_0000) → MMSYSERR_NOERROR.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_SET_PITCH,
            [wave, 0x0002_0000],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));

    // GET_PITCH → reads back the pitch we just set.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_GET_PITCH,
            [wave, pitch_ptr],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(memory.read_u32(pitch_ptr)?, 0x0002_0000);

    // RESET on valid handle → MMSYSERR_NOERROR, clears submitted bytes.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_RESET,
            [wave],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_NOERROR),
            ..
        }
    ));
    assert_eq!(kernel.audio.output(wave).unwrap().submitted_bytes, 0);

    // RESET on invalid handle → MMSYSERR_INVALHANDLE.
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            thread_id,
            ORD_WAVE_OUT_RESET,
            [0xDEAD_BEEF],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::MmResult(MMSYSERR_INVALHANDLE),
            ..
        }
    ));

    Ok(())
}
