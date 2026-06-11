//! Generic virtual I2C stream-device handler.
//!
//! How it works:
//! This module models the IOCTL contract exposed by `gio_i2c*.dll` in the
//! dumped CE image. The sensor drivers do not use `ReadFile`/`WriteFile` on
//! `I2C2:` and friends; they issue `DeviceIoControl` requests where the first
//! input byte is the register/address phase and the remaining input bytes are
//! payload. The individual DLLs accept slightly different private IOCTL sets,
//! so the implementation keeps that per-device contract alongside a small
//! register map.
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
    contract: I2cContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum I2cContract {
    WriteRead,
    TransferWriteRead,
    WriteReadOnly,
}

impl I2cBus {
    pub fn new_for_guest(guest_name: &str) -> Self {
        let normalized = guest_name.trim_end_matches(':').to_ascii_uppercase();
        let contract = match normalized.as_str() {
            "I2C3" => I2cContract::TransferWriteRead,
            "I2C4" => I2cContract::WriteReadOnly,
            _ => I2cContract::WriteRead,
        };
        Self::new_with_contract(contract)
    }

    fn new_with_contract(contract: I2cContract) -> Self {
        let mut registers = [0; 256];
        registers[0x2e] = 0x10;
        registers[0x2f] = 0x20;
        registers[0x23] = 0x40;
        Self {
            registers,
            contract,
        }
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
            IOCTL_I2C_WRITE_READ if self.contract.accepts_write_read() => {
                self.write_read_transaction(input, output_capacity)
            }
            IOCTL_I2C_GIO_I2C2_TRANSFER if self.contract.accepts_transfer() => {
                self.write_read_transaction(input, output_capacity)
            }
            _ => failure(),
        }
    }

    fn write_read_transaction(
        &mut self,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        let write = self.write_transaction(input);
        if !write.success {
            write
        } else {
            self.read_transaction(input, output_capacity)
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

impl I2cContract {
    fn accepts_transfer(self) -> bool {
        matches!(self, Self::TransferWriteRead)
    }

    fn accepts_write_read(self) -> bool {
        matches!(self, Self::WriteRead | Self::TransferWriteRead)
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
