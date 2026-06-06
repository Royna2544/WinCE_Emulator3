//! Virtual YAS526 magnetometer/e-compass stream-device handler.
//!
//! How it works:
//! The registry exposes `MFS1:` backed by `YAS526B.dll`, while the app's
//! ResidentFlash sensor descriptor names `YAS526C`. The driver is a CE stream
//! device whose public surface is a very small set of private IOCTLs. Its read
//! and write paths talk to I2C registers `0x2e` and `0x2f`; the first input
//! byte selects the register bank and the third byte is used as a read count.
//! This module keeps a deterministic register map and returns compass-like
//! bytes without inventing app-specific state.
//!
//! Scope:
//! - Preserve the observed private IOCTL numbers.
//! - Support register write/read calls with the same 4-byte input contract.
//! - Treat the accepted control codes as successful no-ops until traces prove
//!   a stronger behavior is needed.
//! - Do not implement sensor calibration math or magnetic field physics here.

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_MFS_WRITE_REGISTERS: u32 = 0xb000_0000;
pub const IOCTL_MFS_READ_REGISTERS: u32 = 0xb000_0004;
pub const IOCTL_MFS_CONTROL_08: u32 = 0xb000_0008;
pub const IOCTL_MFS_CONTROL_0C: u32 = 0xb000_000c;
pub const IOCTL_MFS_CONTROL_10: u32 = 0xb000_0010;

#[derive(Debug, Clone)]
pub struct Magnetometer {
    bank_2e: [u8; 256],
    bank_2f: [u8; 256],
}

impl Magnetometer {
    pub fn new() -> Self {
        let mut bank_2e = [0; 256];
        let mut bank_2f = [0; 256];
        bank_2e[..8].copy_from_slice(&[0x10, 0x00, 0xf0, 0xff, 0x40, 0x00, 0x00, 0x00]);
        bank_2f[..8].copy_from_slice(&[0x20, 0x00, 0x08, 0x00, 0xf8, 0xff, 0x00, 0x00]);
        Self { bank_2e, bank_2f }
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        match ioctl_code {
            IOCTL_MFS_WRITE_REGISTERS => self.write_registers(input),
            IOCTL_MFS_READ_REGISTERS => self.read_registers(input, output_capacity),
            IOCTL_MFS_CONTROL_08 | IOCTL_MFS_CONTROL_0C | IOCTL_MFS_CONTROL_10 => {
                success(Vec::new())
            }
            _ => failure(),
        }
    }

    fn write_registers(&mut self, input: &[u8]) -> DeviceIoControlResult {
        if input.len() != 4 || !valid_selector(input[1]) {
            return failure();
        }
        let bank = self.bank_mut(input[0]);
        let offset = input[1] as usize;
        bank[offset] = input[3];
        success(Vec::new())
    }

    fn read_registers(&self, input: &[u8], output_capacity: u32) -> DeviceIoControlResult {
        if input.len() != 4 || output_capacity == 0 || !valid_selector(input[1]) {
            return failure();
        }
        let requested = input[2] as usize;
        if requested == 0 || requested > output_capacity as usize {
            return failure();
        }
        let bank = self.bank(input[0]);
        let offset = input[1] as usize;
        let output = (0..requested)
            .map(|idx| bank[(offset + idx) & 0xff])
            .collect();
        success(output)
    }

    fn bank(&self, selector: u8) -> &[u8; 256] {
        if selector == 1 {
            &self.bank_2f
        } else {
            &self.bank_2e
        }
    }

    fn bank_mut(&mut self, selector: u8) -> &mut [u8; 256] {
        if selector == 1 {
            &mut self.bank_2f
        } else {
            &mut self.bank_2e
        }
    }
}

fn valid_selector(value: u8) -> bool {
    matches!(value, 0x00 | 0x40 | 0x80 | 0xc0)
}

fn success(output: Vec<u8>) -> DeviceIoControlResult {
    DeviceIoControlResult {
        success: true,
        bytes_returned: output.len() as u32,
        output,
    }
}

fn failure() -> DeviceIoControlResult {
    DeviceIoControlResult {
        success: false,
        bytes_returned: 0,
        output: Vec::new(),
    }
}
