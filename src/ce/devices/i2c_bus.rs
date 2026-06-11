//! Generic virtual I2C stream-device handler.
//!
//! How it works:
//! This module models the IOCTL contract exposed by `gio_i2c*.dll` in the
//! dumped CE image. The sensor drivers do not use `ReadFile`/`WriteFile` on
//! `I2C2:` and friends; they issue `DeviceIoControl` requests where the first
//! input byte is the register/address phase and the remaining input bytes are
//! payload. The implementation keeps a small register map so higher-level
//! sensor handlers and direct `I2C*:` callers observe stable state.
//!
//! Scope:
//! - Preserve the private CE IOCTL numbers and buffer shapes.
//! - Return deterministic bytes for reads.
//! - Store write payloads into a virtual register map.
//! - Do not emulate physical bus timing, ACK phases, IRQs, or electrical
//!   behavior. Those belong below the CE stream-device boundary.

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_I2C_GIO_I2C2_TRANSFER: u32 = 0x8000_2001;
pub const IOCTL_I2C_WRITE: u32 = 0x8000_2004;
pub const IOCTL_I2C_READ: u32 = 0x8000_2005;
pub const IOCTL_I2C_WRITE_READ: u32 = 0x8000_2006;

#[derive(Debug, Clone)]
pub struct I2cBus {
    registers: [u8; 256],
}

impl I2cBus {
    pub fn new() -> Self {
        let mut registers = [0; 256];
        registers[0x2e] = 0x10;
        registers[0x2f] = 0x20;
        registers[0x23] = 0x40;
        Self { registers }
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        match ioctl_code {
            IOCTL_I2C_WRITE => self.write_transaction(input),
            IOCTL_I2C_READ => self.read_transaction(input, output_capacity),
            IOCTL_I2C_GIO_I2C2_TRANSFER | IOCTL_I2C_WRITE_READ => {
                let write = self.write_transaction(input);
                if !write.success {
                    write
                } else {
                    self.read_transaction(input, output_capacity)
                }
            }
            _ => failure(),
        }
    }

    fn write_transaction(&mut self, input: &[u8]) -> DeviceIoControlResult {
        if input.len() <= 1 || input[0] == 0 {
            return failure();
        }
        let start = input[0] as usize;
        for (offset, byte) in input[1..].iter().copied().enumerate() {
            self.registers[(start + offset) & 0xff] = byte;
        }
        success(Vec::new())
    }

    fn read_transaction(&self, input: &[u8], output_capacity: u32) -> DeviceIoControlResult {
        if input.is_empty() || output_capacity == 0 {
            return failure();
        }
        let start = input[0] as usize;
        let mut output = Vec::with_capacity(output_capacity as usize);
        for offset in 0..output_capacity as usize {
            output.push(self.registers[(start + offset) & 0xff]);
        }
        success(output)
    }
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
