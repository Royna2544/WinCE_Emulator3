//! Virtual SMB380/BMA150 accelerometer stream-device handler.
//!
//! How it works:
//! The registry exposes `SMB1:` through `SMB380.dll`, while the app descriptor
//! names a BMA150-family accelerometer. The DLL's `SMB_IOControl` accepts a
//! dense vendor-private IOCTL range starting at `0x01012ee0`; strings in the
//! image identify most entries. This module keeps those numeric contracts and
//! returns stationary, physically plausible readings. Configuration setters
//! update local state, and getters return the stored value.
//!
//! Scope:
//! - Preserve exact decoded IOCTL constants.
//! - Implement data reads, register reads/writes, and common config knobs.
//! - Return success for inert/reset/control requests that the real driver
//!   accepts.
//! - Do not model impact interrupts, waitable interrupt events, or sensor noise
//!   until traces require those behaviors.

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_SMB380_PRIVATE_01012EE0: u32 = 0x0101_2ee0;
pub const IOCTL_SMB380_PRIVATE_01012EE4: u32 = 0x0101_2ee4;
pub const IOCTL_SMB380_SOFT_RESET: u32 = 0x0101_2ee8;
pub const IOCTL_SMB380_UPDATE_IMAGE: u32 = 0x0101_2eec;
pub const IOCTL_SMB380_SET_IMAGE: u32 = 0x0101_2ef0;
pub const IOCTL_SMB380_GET_IMAGE: u32 = 0x0101_2ef4;
pub const IOCTL_SMB380_GET_OFFSET: u32 = 0x0101_2ef8;
pub const IOCTL_SMB380_SET_OFFSET: u32 = 0x0101_2efc;
pub const IOCTL_SMB380_SET_OFFSET_EEPROM: u32 = 0x0101_2f00;
pub const IOCTL_SMB380_SET_EE_W: u32 = 0x0101_2f04;
pub const IOCTL_SMB380_WRITE_EE: u32 = 0x0101_2f08;
pub const IOCTL_SMB380_SELFTEST: u32 = 0x0101_2f0c;
pub const IOCTL_SMB380_SET_RANGE: u32 = 0x0101_2f10;
pub const IOCTL_SMB380_GET_RANGE: u32 = 0x0101_2f14;
pub const IOCTL_SMB380_SET_MODE: u32 = 0x0101_2f18;
pub const IOCTL_SMB380_GET_MODE: u32 = 0x0101_2f1c;
pub const IOCTL_SMB380_SET_BANDWIDTH: u32 = 0x0101_2f20;
pub const IOCTL_SMB380_GET_BANDWIDTH: u32 = 0x0101_2f24;
pub const IOCTL_SMB380_SET_WAKE_UP_PAUSE: u32 = 0x0101_2f28;
pub const IOCTL_SMB380_GET_WAKE_UP_PAUSE: u32 = 0x0101_2f2c;
pub const IOCTL_SMB380_SET_LOW_G_THRESHOLD: u32 = 0x0101_2f30;
pub const IOCTL_SMB380_GET_LOW_G_THRESHOLD: u32 = 0x0101_2f34;
pub const IOCTL_SMB380_SET_LOW_G_COUNTDOWN: u32 = 0x0101_2f38;
pub const IOCTL_SMB380_GET_LOW_G_COUNTDOWN: u32 = 0x0101_2f3c;
pub const IOCTL_SMB380_SET_HIGH_G_COUNTDOWN: u32 = 0x0101_2f40;
pub const IOCTL_SMB380_GET_HIGH_G_COUNTDOWN: u32 = 0x0101_2f44;
pub const IOCTL_SMB380_SET_LOW_G_DURATION: u32 = 0x0101_2f48;
pub const IOCTL_SMB380_GET_LOW_G_DURATION: u32 = 0x0101_2f4c;
pub const IOCTL_SMB380_SET_HIGH_G_THRESHOLD: u32 = 0x0101_2f50;
pub const IOCTL_SMB380_GET_HIGH_G_THRESHOLD: u32 = 0x0101_2f54;
pub const IOCTL_SMB380_SET_HIGH_G_DURATION: u32 = 0x0101_2f58;
pub const IOCTL_SMB380_GET_HIGH_G_DURATION: u32 = 0x0101_2f5c;
pub const IOCTL_SMB380_SET_ANY_MOTION_THRESHOLD: u32 = 0x0101_2f60;
pub const IOCTL_SMB380_GET_ANY_MOTION_THRESHOLD: u32 = 0x0101_2f64;
pub const IOCTL_SMB380_SET_ANY_MOTION_COUNT: u32 = 0x0101_2f68;
pub const IOCTL_SMB380_GET_ANY_MOTION_COUNT: u32 = 0x0101_2f6c;
pub const IOCTL_SMB380_SET_INTERRUPT_MASK: u32 = 0x0101_2f70;
pub const IOCTL_SMB380_GET_INTERRUPT_MASK: u32 = 0x0101_2f74;
pub const IOCTL_SMB380_RESET_INTERRUPT: u32 = 0x0101_2f78;
pub const IOCTL_SMB380_READ_ACCEL_X: u32 = 0x0101_2f7c;
pub const IOCTL_SMB380_READ_ACCEL_Y: u32 = 0x0101_2f80;
pub const IOCTL_SMB380_READ_ACCEL_Z: u32 = 0x0101_2f84;
pub const IOCTL_SMB380_READ_TEMPERATURE: u32 = 0x0101_2f88;
pub const IOCTL_SMB380_READ_ACCEL_XYZT: u32 = 0x0101_2f8c;
pub const IOCTL_SMB380_GET_INTERRUPT_STATUS: u32 = 0x0101_2f90;
pub const IOCTL_SMB380_SET_LOW_G_INT: u32 = 0x0101_2f94;
pub const IOCTL_SMB380_SET_HIGH_G_INT: u32 = 0x0101_2f98;
pub const IOCTL_SMB380_SET_ANY_MOTION_INT: u32 = 0x0101_2f9c;
pub const IOCTL_SMB380_SET_ALERT_INT: u32 = 0x0101_2fa0;
pub const IOCTL_SMB380_SET_ADVANCED_INT: u32 = 0x0101_2fa4;
pub const IOCTL_SMB380_LATCH_INT: u32 = 0x0101_2fa8;
pub const IOCTL_SMB380_SET_NEW_DATA_INT: u32 = 0x0101_2fac;
pub const IOCTL_SMB380_PAUSE: u32 = 0x0101_2fb0;
pub const IOCTL_SMB380_READ_REG: u32 = 0x0101_2fb4;
pub const IOCTL_SMB380_WRITE_REG: u32 = 0x0101_2fb8;
pub const IOCTL_SMB380_WAIT_INTERRUPT: u32 = 0x0101_2fbc;
pub const IOCTL_SMB380_GET_LOW_G_HYST: u32 = 0x0101_2fc0;
pub const IOCTL_SMB380_SET_LOW_G_HYST: u32 = 0x0101_2fc4;
pub const IOCTL_SMB380_GET_HIGH_G_HYST: u32 = 0x0101_2fc8;
pub const IOCTL_SMB380_SET_HIGH_G_HYST: u32 = 0x0101_2fcc;

#[derive(Debug, Clone)]
pub struct Accelerometer {
    registers: [u8; 256],
    range: u8,
    mode: u8,
    bandwidth: u8,
    wake_up_pause: u8,
    low_g_threshold: u8,
    low_g_countdown: u8,
    high_g_countdown: u8,
    low_g_duration: u8,
    high_g_threshold: u8,
    high_g_duration: u8,
    any_motion_threshold: u8,
    any_motion_count: u8,
    interrupt_mask: u8,
    low_g_hyst: u8,
    high_g_hyst: u8,
    x: i16,
    y: i16,
    z: i16,
    temperature: u8,
}

impl Accelerometer {
    pub fn new() -> Self {
        let mut registers = [0; 256];
        registers[0x00] = 0x02;
        registers[0x01] = 0x00;
        registers[0x02..0x08].copy_from_slice(&[0, 0, 0, 0, 0x40, 0x00]);
        Self {
            registers,
            range: 0,
            mode: 0,
            bandwidth: 2,
            wake_up_pause: 0,
            low_g_threshold: 0,
            low_g_countdown: 0,
            high_g_countdown: 0,
            low_g_duration: 0,
            high_g_threshold: 0,
            high_g_duration: 0,
            any_motion_threshold: 0,
            any_motion_count: 0,
            interrupt_mask: 0,
            low_g_hyst: 0,
            high_g_hyst: 0,
            x: 0,
            y: 0,
            z: 256,
            temperature: 25,
        }
    }

    pub fn set_axes(&mut self, x: i16, y: i16, z: i16) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.registers[0x02..0x04].copy_from_slice(&x.to_le_bytes());
        self.registers[0x04..0x06].copy_from_slice(&y.to_le_bytes());
        self.registers[0x06..0x08].copy_from_slice(&z.to_le_bytes());
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        match ioctl_code {
            IOCTL_SMB380_PRIVATE_01012EE0
            | IOCTL_SMB380_PRIVATE_01012EE4
            | IOCTL_SMB380_SOFT_RESET
            | IOCTL_SMB380_UPDATE_IMAGE
            | IOCTL_SMB380_SET_OFFSET_EEPROM
            | IOCTL_SMB380_SET_EE_W
            | IOCTL_SMB380_WRITE_EE
            | IOCTL_SMB380_SELFTEST
            | IOCTL_SMB380_RESET_INTERRUPT
            | IOCTL_SMB380_WAIT_INTERRUPT => success(Vec::new()),

            IOCTL_SMB380_SET_IMAGE => self.copy_input_to_registers(input),
            IOCTL_SMB380_GET_IMAGE => self.read_registers_from(0, output_capacity),
            IOCTL_SMB380_GET_OFFSET => self.get_fixed(&[0, 0, 0]),
            IOCTL_SMB380_SET_OFFSET => self.accept_len(input, 3),

            IOCTL_SMB380_SET_RANGE => self.set_u8(input, |this, value| this.range = value),
            IOCTL_SMB380_GET_RANGE => self.get_u8(output_capacity, self.range),
            IOCTL_SMB380_SET_MODE => self.set_u8(input, |this, value| this.mode = value),
            IOCTL_SMB380_GET_MODE => self.get_u8(output_capacity, self.mode),
            IOCTL_SMB380_SET_BANDWIDTH => self.set_u8(input, |this, value| this.bandwidth = value),
            IOCTL_SMB380_GET_BANDWIDTH => self.get_u8(output_capacity, self.bandwidth),
            IOCTL_SMB380_SET_WAKE_UP_PAUSE => {
                self.set_u8(input, |this, value| this.wake_up_pause = value)
            }
            IOCTL_SMB380_GET_WAKE_UP_PAUSE => self.get_u8(output_capacity, self.wake_up_pause),
            IOCTL_SMB380_SET_LOW_G_THRESHOLD => {
                self.set_u8(input, |this, value| this.low_g_threshold = value)
            }
            IOCTL_SMB380_GET_LOW_G_THRESHOLD => self.get_u8(output_capacity, self.low_g_threshold),
            IOCTL_SMB380_SET_LOW_G_COUNTDOWN => {
                self.set_u8(input, |this, value| this.low_g_countdown = value)
            }
            IOCTL_SMB380_GET_LOW_G_COUNTDOWN => self.get_u8(output_capacity, self.low_g_countdown),
            IOCTL_SMB380_SET_HIGH_G_COUNTDOWN => {
                self.set_u8(input, |this, value| this.high_g_countdown = value)
            }
            IOCTL_SMB380_GET_HIGH_G_COUNTDOWN => {
                self.get_u8(output_capacity, self.high_g_countdown)
            }
            IOCTL_SMB380_SET_LOW_G_DURATION => {
                self.set_u8(input, |this, value| this.low_g_duration = value)
            }
            IOCTL_SMB380_GET_LOW_G_DURATION => self.get_u8(output_capacity, self.low_g_duration),
            IOCTL_SMB380_SET_HIGH_G_THRESHOLD => {
                self.set_u8(input, |this, value| this.high_g_threshold = value)
            }
            IOCTL_SMB380_GET_HIGH_G_THRESHOLD => {
                self.get_u8(output_capacity, self.high_g_threshold)
            }
            IOCTL_SMB380_SET_HIGH_G_DURATION => {
                self.set_u8(input, |this, value| this.high_g_duration = value)
            }
            IOCTL_SMB380_GET_HIGH_G_DURATION => self.get_u8(output_capacity, self.high_g_duration),
            IOCTL_SMB380_SET_ANY_MOTION_THRESHOLD => {
                self.set_u8(input, |this, value| this.any_motion_threshold = value)
            }
            IOCTL_SMB380_GET_ANY_MOTION_THRESHOLD => {
                self.get_u8(output_capacity, self.any_motion_threshold)
            }
            IOCTL_SMB380_SET_ANY_MOTION_COUNT => {
                self.set_u8(input, |this, value| this.any_motion_count = value)
            }
            IOCTL_SMB380_GET_ANY_MOTION_COUNT => {
                self.get_u8(output_capacity, self.any_motion_count)
            }
            IOCTL_SMB380_SET_INTERRUPT_MASK => {
                self.set_u8(input, |this, value| this.interrupt_mask = value)
            }
            IOCTL_SMB380_GET_INTERRUPT_MASK => self.get_u8(output_capacity, self.interrupt_mask),
            IOCTL_SMB380_SET_LOW_G_INT
            | IOCTL_SMB380_SET_HIGH_G_INT
            | IOCTL_SMB380_SET_ANY_MOTION_INT
            | IOCTL_SMB380_SET_ALERT_INT
            | IOCTL_SMB380_SET_ADVANCED_INT
            | IOCTL_SMB380_LATCH_INT
            | IOCTL_SMB380_SET_NEW_DATA_INT => self.accept_len(input, 1),
            IOCTL_SMB380_PAUSE => self.accept_len(input, 4),
            IOCTL_SMB380_GET_LOW_G_HYST => self.get_u8(output_capacity, self.low_g_hyst),
            IOCTL_SMB380_SET_LOW_G_HYST => {
                self.set_u8(input, |this, value| this.low_g_hyst = value)
            }
            IOCTL_SMB380_GET_HIGH_G_HYST => self.get_u8(output_capacity, self.high_g_hyst),
            IOCTL_SMB380_SET_HIGH_G_HYST => {
                self.set_u8(input, |this, value| this.high_g_hyst = value)
            }

            IOCTL_SMB380_READ_ACCEL_X => self.read_i16(output_capacity, self.x),
            IOCTL_SMB380_READ_ACCEL_Y => self.read_i16(output_capacity, self.y),
            IOCTL_SMB380_READ_ACCEL_Z => self.read_i16(output_capacity, self.z),
            IOCTL_SMB380_READ_TEMPERATURE => self.get_u8(output_capacity, self.temperature),
            IOCTL_SMB380_READ_ACCEL_XYZT => self.read_xyzt(output_capacity),
            IOCTL_SMB380_GET_INTERRUPT_STATUS => self.get_u8(output_capacity, 0),
            IOCTL_SMB380_READ_REG => self.read_reg(input, output_capacity),
            IOCTL_SMB380_WRITE_REG => self.write_reg(input),
            _ => failure(),
        }
    }

    fn set_u8(
        &mut self,
        input: &[u8],
        setter: impl FnOnce(&mut Self, u8),
    ) -> DeviceIoControlResult {
        let Some(value) = input.first().copied() else {
            return failure();
        };
        setter(self, value);
        success(Vec::new())
    }

    fn get_u8(&self, output_capacity: u32, value: u8) -> DeviceIoControlResult {
        if output_capacity < 1 {
            return failure();
        }
        success(vec![value])
    }

    fn read_i16(&self, output_capacity: u32, value: i16) -> DeviceIoControlResult {
        if output_capacity < 2 {
            return failure();
        }
        success(value.to_le_bytes().to_vec())
    }

    fn read_xyzt(&self, output_capacity: u32) -> DeviceIoControlResult {
        if output_capacity < 6 {
            return failure();
        }
        let mut output = Vec::with_capacity(6);
        output.extend_from_slice(&self.x.to_le_bytes());
        output.extend_from_slice(&self.y.to_le_bytes());
        output.extend_from_slice(&self.z.to_le_bytes());
        success(output)
    }

    fn read_reg(&self, input: &[u8], output_capacity: u32) -> DeviceIoControlResult {
        if input.len() != 2 || output_capacity != u32::from(input[1]) || output_capacity == 0 {
            return failure();
        }
        self.read_registers_from(input[0], output_capacity)
    }

    fn write_reg(&mut self, input: &[u8]) -> DeviceIoControlResult {
        if input.len() != 8 {
            return failure();
        }
        self.registers[input[0] as usize] = input[1];
        success(Vec::new())
    }

    fn copy_input_to_registers(&mut self, input: &[u8]) -> DeviceIoControlResult {
        for (index, byte) in input.iter().copied().enumerate().take(self.registers.len()) {
            self.registers[index] = byte;
        }
        success(Vec::new())
    }

    fn read_registers_from(&self, start: u8, output_capacity: u32) -> DeviceIoControlResult {
        let mut output = Vec::with_capacity(output_capacity as usize);
        for offset in 0..output_capacity as usize {
            output.push(self.registers[(start as usize + offset) & 0xff]);
        }
        success(output)
    }

    fn get_fixed(&self, bytes: &[u8]) -> DeviceIoControlResult {
        success(bytes.to_vec())
    }

    fn accept_len(&self, input: &[u8], len: usize) -> DeviceIoControlResult {
        if input.len() == len {
            success(Vec::new())
        } else {
            failure()
        }
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
