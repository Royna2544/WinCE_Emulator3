//! Isolated Windows Unicorn code-cache arena controls (E-5/E-6).
//!
//! The small-arena tests can be run by exact name in fresh test processes:
//! the old native access violation terminated the whole host, not one test.

#![cfg(all(windows, feature = "unicorn"))]

use std::sync::Mutex;

use unicorn_engine::{
    RegisterMIPS, Unicorn,
    unicorn_const::{Arch, ControlType, Mode, Prot, Query, uc_ctl, uc_error},
};

const MIB: u32 = 1024 * 1024;
const CODE_START: u64 = 0x1000;
const CODE_END: u64 = CODE_START + 16;

// include/unicorn/unicorn.h: UC_CTL(type, nr, rw) and
// uc_ctl_{set,get}_tcg_buffer_size. The Rust wrapper has no typed TCG setter.
const TCG_BUFFER_SIZE_TYPE: u32 = 13;
const ONE_ARGUMENT: u32 = 1 << 26;
const IO_WRITE: u32 = 1 << 30;
const IO_READ: u32 = 2 << 30;

// Avoid overlapping native VEH ownership when the default Rust test runner
// runs the active 8 MiB and default controls in parallel.
static ARENA_TEST_LOCK: Mutex<()> = Mutex::new(());

fn arena_control(io: u32) -> ControlType {
    assert_eq!(ControlType::TCG_BUFFER_SIZE.0, TCG_BUFFER_SIZE_TYPE as i32);
    ControlType((TCG_BUFFER_SIZE_TYPE | ONE_ARGUMENT | io) as i32)
}

fn smoke(requested_size: Option<u32>) {
    let _guard = ARENA_TEST_LOCK.lock().unwrap();
    let mut uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 | Mode::LITTLE_ENDIAN).unwrap();

    if let Some(size) = requested_size {
        // Write before any query, mapping, or emulation: the read below calls
        // UC_INIT and freezes the selected native arena size.
        let result = unsafe { uc_ctl(uc.get_handle(), arena_control(IO_WRITE), size) };
        assert_eq!(result, uc_error::OK, "pre-init TCG arena request failed");
    }

    let mut actual_size = 0u32;
    let result = unsafe {
        uc_ctl(
            uc.get_handle(),
            arena_control(IO_READ),
            &mut actual_size as *mut u32,
        )
    };
    assert_eq!(result, uc_error::OK, "native arena size query failed");
    eprintln!("TCG arena requested={requested_size:?} actual={actual_size}");
    if let Some(size) = requested_size {
        assert_eq!(actual_size, size, "native arena adjusted the request");
    } else {
        assert!(actual_size >= MIB, "default native arena is too small");
    }

    // Four real MIPS32 little-endian instructions: s0 = 1, then increment
    // three times. The register result proves translated guest execution.
    const CODE: [u8; 16] = [
        0x01, 0x00, 0x10, 0x24, // addiu s0, zero, 1
        0x01, 0x00, 0x10, 0x26, // addiu s0, s0, 1
        0x01, 0x00, 0x10, 0x26, // addiu s0, s0, 1
        0x01, 0x00, 0x10, 0x26, // addiu s0, s0, 1
    ];
    uc.mem_map(CODE_START, 0x1000, Prot::ALL).unwrap();
    uc.mem_write(CODE_START, &CODE).unwrap();
    uc.reg_write(RegisterMIPS::S0, 0).unwrap();
    uc.emu_start(CODE_START, CODE_END, 500_000, 4).unwrap();

    assert_eq!(uc.reg_read(RegisterMIPS::S0).unwrap(), 4);
    assert_eq!(uc.query(Query::TIMEOUT), Ok(0), "smoke hit host timeout");
}

#[test]
fn one_mib_preinit_arena_runs_mips_smoke() {
    smoke(Some(MIB));
}

#[test]
fn two_mib_preinit_arena_runs_mips_smoke() {
    smoke(Some(2 * MIB));
}

#[test]
fn eight_mib_preinit_arena_runs_mips_smoke() {
    smoke(Some(8 * MIB));
}

#[test]
fn default_arena_runs_mips_smoke() {
    smoke(None);
}
