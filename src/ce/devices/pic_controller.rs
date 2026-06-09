//! Virtual PIC controller stream-device handler.
//!
//! How it works:
//! `D:\INAVI_Emulator\DUMPPLZ\Windows\pic_drv.dll` exposes `PIC1:` through
//! `PIC_IOControl`. The dumped driver opens companion devices such as
//! `BTN1:` and `I2C2:`, then translates private IOCTLs into short MCU command
//! frames. This module preserves the decoded IOCTL contracts and returns
//! deterministic acknowledgements at the CE stream-device boundary.
//!
//! Scope:
//! - Keep the private `0xd00000xx` PIC IOCTL numbers seen in `PIC_IOControl`.
//! - Accept the MICOM reset controls that `NANDUUID.dll` forwards to `PIC1:`.
//! - Track EEPROM/display/power state in-process for stable follow-up calls.
//! - Do not emulate MCU firmware timing, interrupts, button events, or the
//!   physical I2C transport until traces require those layers.

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_DEVICE_PIC_READ_VERSION: u32 = 0xd000_0004;
pub const IOCTL_DEVICE_PIC_I2C_INIT: u32 = 0xd000_0008;
pub const IOCTL_DEVICE_PIC_I2C_READ_PIC: u32 = 0xd000_000c;
pub const IOCTL_DEVICE_PIC_I2C_RESET: u32 = 0xd000_0010;
pub const IOCTL_DEVICE_PIC_I2C_SLEEP_OK: u32 = 0xd000_0014;
pub const IOCTL_DEVICE_PIC_I2C_PWR_ON_OK: u32 = 0xd000_0018;
pub const IOCTL_DEVICE_PIC_I2C_SET_EEPROM_COMMAND_WRITE: u32 = 0xd000_001c;
pub const IOCTL_DEVICE_PIC_I2C_SET_EEPROM_COMMAND_READ: u32 = 0xd000_0020;
pub const IOCTL_DEVICE_PIC_I2C_DISPLAY_STATE: u32 = 0xd000_002c;
pub const IOCTL_DEVICE_PIC_I2C_OS_UPGRADE: u32 = 0xd000_003c;
pub const IOCTL_DEVICE_PIC_I2C_PWR_LED: u32 = 0xd000_0040;

#[allow(dead_code)]
pub const IOCTL_NANDUUID_MICOM_RESET_STAGE: u32 = IOCTL_DEVICE_PIC_I2C_PWR_ON_OK;
pub const IOCTL_NANDUUID_MICOM_RESET_ACK: u32 = 0xa007_0014;

#[derive(Debug, Clone)]
pub struct PicController {
    version: u8,
    eeprom: [u8; 256],
    display_state: u8,
    power_led: u8,
    os_upgrade_state: u8,
    reset_count: u32,
    sleep_ok_count: u32,
    power_on_ok_count: u32,
}

impl PicController {
    pub fn new() -> Self {
        let mut eeprom = [0; 256];
        eeprom[0x00] = 0x42;
        eeprom[0x10] = 0x11;
        Self {
            version: 0x42,
            eeprom,
            display_state: 1,
            power_led: 1,
            os_upgrade_state: 0,
            reset_count: 0,
            sleep_ok_count: 0,
            power_on_ok_count: 0,
        }
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        match ioctl_code {
            IOCTL_DEVICE_PIC_READ_VERSION => self.output_optional_u8(output_capacity, self.version),
            IOCTL_DEVICE_PIC_I2C_INIT | IOCTL_DEVICE_PIC_I2C_READ_PIC => success(Vec::new()),
            IOCTL_DEVICE_PIC_I2C_RESET => {
                self.reset_count = self.reset_count.saturating_add(1);
                success(Vec::new())
            }
            IOCTL_NANDUUID_MICOM_RESET_ACK => success(Vec::new()),
            IOCTL_DEVICE_PIC_I2C_SLEEP_OK => {
                self.sleep_ok_count = self.sleep_ok_count.saturating_add(1);
                success(Vec::new())
            }
            IOCTL_DEVICE_PIC_I2C_PWR_ON_OK => {
                self.power_on_ok_count = self.power_on_ok_count.saturating_add(1);
                success(Vec::new())
            }
            IOCTL_DEVICE_PIC_I2C_SET_EEPROM_COMMAND_WRITE => {
                self.write_eeprom_command(input, output_capacity)
            }
            IOCTL_DEVICE_PIC_I2C_SET_EEPROM_COMMAND_READ => {
                self.read_eeprom_command(input, output_capacity)
            }
            IOCTL_DEVICE_PIC_I2C_DISPLAY_STATE => {
                self.set_u8_state(input, |this, value| this.display_state = value)
            }
            IOCTL_DEVICE_PIC_I2C_PWR_LED => {
                self.set_u8_state(input, |this, value| this.power_led = value)
            }
            IOCTL_DEVICE_PIC_I2C_OS_UPGRADE => {
                self.set_u8_state(input, |this, value| this.os_upgrade_state = value)
            }
            _ => failure(),
        }
    }

    fn write_eeprom_command(
        &mut self,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        let Some(address) = input.first().copied() else {
            return failure();
        };
        if let Some(value) = input.get(1).copied() {
            self.eeprom[address as usize] = value;
        }
        self.output_optional_u8(output_capacity, 0)
    }

    fn read_eeprom_command(&self, input: &[u8], output_capacity: u32) -> DeviceIoControlResult {
        let Some(address) = input.first().copied() else {
            return failure();
        };
        if output_capacity == 0 {
            return failure();
        }
        success(vec![self.eeprom[address as usize]])
    }

    fn set_u8_state(
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

    fn output_optional_u8(&self, output_capacity: u32, value: u8) -> DeviceIoControlResult {
        if output_capacity == 0 {
            success(Vec::new())
        } else {
            success(vec![value])
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
