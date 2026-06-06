//! Virtual NANDUUID stream-device handler.
//!
//! How it works:
//! `D:\INAVI_Emulator\DUMPPLZ\Windows\NANDUUID.dll` exposes `UID1:` through
//! `UID_IOControl`. The driver maps a NAND MMIO page during init and handles a
//! small vendor-private IOCTL set for UUID reads/writes, MICOM/touchpad lock
//! controls, and a CPU-load control word. This module keeps those decoded
//! numeric contracts and returns stable state through the CE stream-device
//! boundary.
//!
//! Scope:
//! - Preserve decoded IOCTL numbers from `NANDUUID.dll`.
//! - Return deterministic UUID values instead of the old synthetic byte echo.
//! - Track lock/reset/load state locally so repeated calls are coherent.
//! - Do not model NAND flash timing, physical ID probing, or PIC/I2C side
//!   effects beyond accepting the same control contracts.

use std::collections::BTreeMap;

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_NAND_UPD_WRITE_UUID_BY_SECTORNUM: u32 = 0xa000_002c;
pub const IOCTL_NAND_UPD_READ_UUID: u32 = 0xa000_00cc;
pub const IOCTL_NAND_UPD_READ_UUID_BY_SECTORNUM: u32 = 0xa000_00d0;
pub const IOCTL_MICOM_RESET: u32 = 0x0023_d021;
pub const IOCTL_TOUCHPAD_LOCK: u32 = 0x0022_1011;
pub const IOCTL_TOUCHPAD_UNLOCK: u32 = 0x0022_1015;
pub const IOCTL_MICOM_LOCK: u32 = 0x0022_1019;
pub const IOCTL_MICOM_UNLOCK: u32 = 0x0022_101d;
pub const IOCTL_NAND_CPU_LOAD_CONTROL: u32 = 0x0022_1025;
pub const IOCTL_NAND_CPU_LOAD_CONTROL_ALT: u32 = 0x0023_c041;

const DEFAULT_UUID: u32 = 20_111_201;

#[derive(Debug, Clone)]
pub struct NandUuid {
    uuid: u32,
    sector_uuids: BTreeMap<u32, u32>,
    cpu_load_control: u32,
    touchpad_locked: bool,
    micom_locked: bool,
    micom_reset_count: u32,
}

impl NandUuid {
    pub fn new() -> Self {
        Self {
            uuid: DEFAULT_UUID,
            sector_uuids: BTreeMap::new(),
            cpu_load_control: 0,
            touchpad_locked: false,
            micom_locked: false,
            micom_reset_count: 0,
        }
    }

    pub fn device_io_control(
        &mut self,
        ioctl_code: u32,
        input: &[u8],
        output_capacity: u32,
    ) -> DeviceIoControlResult {
        match ioctl_code {
            IOCTL_NAND_UPD_READ_UUID => self.read_uuid(output_capacity),
            IOCTL_NAND_UPD_READ_UUID_BY_SECTORNUM => {
                self.read_uuid_by_sector(input, output_capacity)
            }
            IOCTL_NAND_UPD_WRITE_UUID_BY_SECTORNUM => self.write_uuid_by_sector(input),
            IOCTL_MICOM_RESET => {
                self.micom_reset_count = self.micom_reset_count.saturating_add(1);
                success(Vec::new())
            }
            IOCTL_TOUCHPAD_LOCK => {
                self.touchpad_locked = true;
                success(Vec::new())
            }
            IOCTL_TOUCHPAD_UNLOCK => {
                self.touchpad_locked = false;
                success(Vec::new())
            }
            IOCTL_MICOM_LOCK => {
                self.micom_locked = true;
                success(Vec::new())
            }
            IOCTL_MICOM_UNLOCK => {
                self.micom_locked = false;
                success(Vec::new())
            }
            IOCTL_NAND_CPU_LOAD_CONTROL | IOCTL_NAND_CPU_LOAD_CONTROL_ALT => {
                let Some(value) = read_u32_le(input, 0) else {
                    return failure();
                };
                self.cpu_load_control = value;
                success(Vec::new())
            }
            _ => failure(),
        }
    }

    fn read_uuid(&self, output_capacity: u32) -> DeviceIoControlResult {
        write_u32_output(output_capacity, self.uuid)
    }

    fn read_uuid_by_sector(&self, input: &[u8], output_capacity: u32) -> DeviceIoControlResult {
        let Some(sector) = read_u32_le(input, 0) else {
            return failure();
        };
        let value = self
            .sector_uuids
            .get(&sector)
            .copied()
            .unwrap_or_else(|| self.sector_default_uuid(sector));
        write_u32_output(output_capacity, value)
    }

    fn write_uuid_by_sector(&mut self, input: &[u8]) -> DeviceIoControlResult {
        let Some(sector) = read_u32_le(input, 0) else {
            return failure();
        };
        let value = read_u32_le(input, 4).unwrap_or(self.uuid);
        self.sector_uuids.insert(sector, value);
        success(Vec::new())
    }

    fn sector_default_uuid(&self, sector: u32) -> u32 {
        self.uuid
            .wrapping_add(sector.rotate_left(5))
            .wrapping_rem(100_000_000)
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let slice = bytes.get(offset..end)?;
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

fn write_u32_output(output_capacity: u32, value: u32) -> DeviceIoControlResult {
    if output_capacity < 4 {
        return failure();
    }
    success(value.to_le_bytes().to_vec())
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
