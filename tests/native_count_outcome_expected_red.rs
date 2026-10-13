//! E28 O-P1 expected-red controls for the missing independent native count stop.
//! These use a real MIPS PE through the public CPU runner. The debug-string
//! assertion is temporary until `CountLimit` exists as a typed outcome.

#![cfg(feature = "unicorn")]

use std::path::PathBuf;

use wince_emulation_v3::{
    Result,
    ce::{framebuffer::VirtualFramebuffer, kernel::CeKernel},
    config::RuntimeConfig,
    emulator::cpu::{UnicornMips, UnicornRunLimits},
    pe::PeImage,
};

const IMAGE_BASE: u32 = 0x0040_0000;
const ENTRY: u32 = IMAGE_BASE + 0x1000;

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn tiny_counted_mips_pe() -> Vec<u8> {
    let mut bytes = vec![0; 0x400];
    bytes[0..2].copy_from_slice(b"MZ");
    put_u32(&mut bytes, 0x3c, 0x80);
    bytes[0x80..0x84].copy_from_slice(b"PE\0\0");

    let coff = 0x84;
    put_u16(&mut bytes, coff, 0x0166); // MIPS R4000
    put_u16(&mut bytes, coff + 2, 1); // one section
    put_u16(&mut bytes, coff + 16, 0x00e0);
    put_u16(&mut bytes, coff + 18, 0x0102);

    let optional = 0x98;
    put_u16(&mut bytes, optional, 0x010b); // PE32
    bytes[optional + 2] = 8;
    put_u32(&mut bytes, optional + 4, 0x200);
    put_u32(&mut bytes, optional + 16, 0x1000); // entry RVA
    put_u32(&mut bytes, optional + 20, 0x1000);
    put_u32(&mut bytes, optional + 24, 0x1000);
    put_u32(&mut bytes, optional + 28, IMAGE_BASE);
    put_u32(&mut bytes, optional + 32, 0x1000);
    put_u32(&mut bytes, optional + 36, 0x200);
    put_u16(&mut bytes, optional + 40, 4);
    put_u16(&mut bytes, optional + 48, 4);
    put_u32(&mut bytes, optional + 56, 0x2000);
    put_u32(&mut bytes, optional + 60, 0x200);
    put_u16(&mut bytes, optional + 68, 9);
    put_u32(&mut bytes, optional + 72, 0x10000);
    put_u32(&mut bytes, optional + 76, 0x1000);
    put_u32(&mut bytes, optional + 80, 0x10000);
    put_u32(&mut bytes, optional + 84, 0x1000);
    put_u32(&mut bytes, optional + 92, 16);

    let section = 0x178;
    bytes[section..section + 8].copy_from_slice(b".text\0\0\0");
    put_u32(&mut bytes, section + 8, 0x100);
    put_u32(&mut bytes, section + 12, 0x1000);
    put_u32(&mut bytes, section + 16, 0x200);
    put_u32(&mut bytes, section + 20, 0x200);
    put_u32(&mut bytes, section + 36, 0x6000_0020);

    // Three observable straight-line guest instructions, then a terminal spin.
    for (index, word) in [
        0x2410_0001, // addiu s0,zero,1
        0x2610_0001, // addiu s0,s0,1
        0x2610_0001, // addiu s0,s0,1
        0x1000_ffff, // beq zero,zero,-1
        0,           // delay-slot nop
    ]
    .into_iter()
    .enumerate()
    {
        put_u32(&mut bytes, 0x200 + index * 4, word);
    }
    bytes
}

fn run_guest_count_limit(count: usize) -> Result<UnicornMips> {
    let name = format!("e28-native-count-{count}.exe");
    let bytes = tiny_counted_mips_pe();
    let image = PeImage::parse_bytes(name.clone(), &bytes)?;
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    kernel.set_process_module_base(image.image_base());
    kernel.set_process_module_path(format!("\\Test\\{name}"));
    kernel.set_process_module_host_path(PathBuf::from(&name));

    let mut cpu = UnicornMips::new()?;
    cpu.load_pe_image(&image)?;
    let mut framebuffer = VirtualFramebuffer::default_primary()?;
    cpu.run_until_import_trap_with_framebuffer_limits(
        &mut kernel,
        &mut framebuffer,
        UnicornRunLimits {
            instruction_limit: count,
            wall_clock_limit_ms: 500, // safety bound; assertions reject its expiry
            stop_pc: None,
            live_pump: false,
        },
    )?;
    Ok(cpu)
}

#[test]
#[ignore = "expected red: E28 O-P1 lacks an independent native count-stop witness"]
fn native_one_instruction_limit_has_typed_count_outcome() -> Result<()> {
    let cpu = run_guest_count_limit(1)?;
    let snapshot = cpu
        .last_debug_snapshot()
        .expect("real native MIPS snapshot");
    assert_eq!(cpu.saved_context_pc(), Some(ENTRY + 4));
    assert_eq!(snapshot.pc, ENTRY + 4);
    assert!(snapshot.pc_stop.is_none());
    assert!(snapshot.memory_fault.is_none());
    assert!(snapshot.host_wall_clock_stop.is_none());
    assert_eq!(
        format!("{:?}", cpu.last_native_slice_outcome()),
        "CountLimit",
        "a positive configured count alone is not a count-stop witness",
    );
    Ok(())
}

#[test]
#[ignore = "expected red: E28 O-P1 lacks an independent native count-stop witness"]
fn native_two_instruction_limit_has_typed_count_outcome() -> Result<()> {
    let cpu = run_guest_count_limit(2)?;
    let snapshot = cpu
        .last_debug_snapshot()
        .expect("real native MIPS snapshot");
    assert_eq!(cpu.saved_context_pc(), Some(ENTRY + 8));
    assert_eq!(snapshot.pc, ENTRY + 8);
    assert!(snapshot.pc_stop.is_none());
    assert!(snapshot.memory_fault.is_none());
    assert!(snapshot.host_wall_clock_stop.is_none());
    assert_eq!(
        format!("{:?}", cpu.last_native_slice_outcome()),
        "CountLimit",
        "the typed outcome must be tied to this finite native attempt",
    );
    Ok(())
}
