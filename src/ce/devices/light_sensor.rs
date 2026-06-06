//! Virtual light sensor stream-device handler.
//!
//! How it works:
//! The dumped `light_sensor_drv.dll` exports a CE stream device named `LSD1:`.
//! Its useful behavior is in `LSD_IOControl`; `ReadFile`, `WriteFile`, and
//! `Seek` are inert stubs. The driver opens an I2C bus and uses private IOCTLs
//! to configure/read a light sensor register. This module keeps that contract
//! at the `DeviceIoControl` boundary and returns a stable ambient-light value.
//!
//! Scope:
//! - `0xd2000004` stores the first input byte as the sensor control value,
//!   equivalent to the driver writing register `0x23` through I2C.
//! - `0xd2000008` returns a little-endian 16-bit light value.
//! - `0xd2000014` and `0xd2000018` toggle the virtual sensing path.
//! - No real backlight policy, sensing thread, or hardware interrupt is modeled.

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_LSD_SET_CONTROL: u32 = 0xd200_0004;
pub const IOCTL_LSD_READ_LUX: u32 = 0xd200_0008;
pub const IOCTL_LSD_START_SENSING: u32 = 0xd200_0014;
pub const IOCTL_LSD_STOP_SENSING: u32 = 0xd200_0018;

#[derive(Debug, Clone)]
pub struct LightSensor {
    control: u8,
    lux: u16,
    sensing: bool,
}

impl LightSensor {
    pub fn new() -> Self {
        Self {
            control: 0,
            lux: 0x0120,
            sensing: false,
        }
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        match ioctl_code {
            IOCTL_LSD_SET_CONTROL => {
                let Some(control) = input.first().copied() else {
                    return failure();
                };
                self.control = control;
                success(Vec::new())
            }
            IOCTL_LSD_READ_LUX => {
                if output_capacity < 2 {
                    return failure();
                }
                success(self.lux.to_le_bytes().to_vec())
            }
            IOCTL_LSD_START_SENSING => {
                self.sensing = true;
                success(Vec::new())
            }
            IOCTL_LSD_STOP_SENSING => {
                self.sensing = false;
                success(Vec::new())
            }
            _ => failure(),
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
