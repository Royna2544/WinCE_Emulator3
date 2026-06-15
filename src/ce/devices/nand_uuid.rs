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

use std::{collections::BTreeMap, fs};

use crate::ce::devices::DeviceIoControlResult;

pub const IOCTL_NAND_UPD_WRITE_UUID_BY_SECTORNUM: u32 = 0xa001_002c;
pub const IOCTL_NAND_UPD_READ_UUID: u32 = 0xa001_00cc;
pub const IOCTL_NAND_UPD_READ_UUID_BY_SECTORNUM: u32 = 0xa001_00d0;
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
    seed_bytes: Vec<u8>,
    sector_uuids: BTreeMap<u32, [u8; 16]>,
    cpu_load_control: u32,
    touchpad_locked: bool,
    micom_locked: bool,
    micom_reset_count: u32,
}

impl NandUuid {
    pub fn new() -> Self {
        Self {
            uuid: DEFAULT_UUID,
            seed_bytes: Vec::new(),
            sector_uuids: BTreeMap::new(),
            cpu_load_control: 0,
            touchpad_locked: false,
            micom_locked: false,
            micom_reset_count: 0,
        }
    }

    pub fn new_from_host(host: Option<&str>) -> Self {
        let Some(host) = host else {
            return Self::new();
        };
        let Ok(text) = fs::read_to_string(host) else {
            return Self::new();
        };
        Self::from_seed(text.trim().as_bytes())
    }

    pub fn from_seed(seed: &[u8]) -> Self {
        let mut device = Self::new();
        let trimmed = trim_ascii(seed);
        if trimmed.is_empty() {
            return device;
        }
        device.uuid = decimal_prefix_uuid(trimmed).unwrap_or(DEFAULT_UUID);
        device.seed_bytes = trimmed.to_vec();
        device
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
        let bytes = self
            .sector_uuids
            .get(&sector)
            .copied()
            .unwrap_or_else(|| self.sector_default_uuid_bytes(sector));
        if output_capacity >= 16 {
            success(bytes.to_vec())
        } else {
            write_u32_output(
                output_capacity,
                u32::from_le_bytes(bytes[0..4].try_into().expect("uuid word")),
            )
        }
    }

    fn write_uuid_by_sector(&mut self, input: &[u8]) -> DeviceIoControlResult {
        let Some(sector) = read_u32_le(input, 0) else {
            return failure();
        };
        let value = if let Some(bytes) = input.get(4..20) {
            bytes.try_into().expect("sector uuid bytes")
        } else {
            self.sector_default_uuid_bytes(sector)
        };
        self.sector_uuids.insert(sector, value);
        success(Vec::new())
    }

    fn sector_default_uuid_bytes(&self, sector: u32) -> [u8; 16] {
        if !self.seed_bytes.is_empty() {
            return self.seed_sector_bytes(sector);
        }
        let first_word = self
            .uuid
            .wrapping_add(sector.rotate_left(5))
            .wrapping_rem(100_000_000);
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&first_word.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.uuid.rotate_left(7).wrapping_add(sector).to_le_bytes());
        bytes[8..12].copy_from_slice(
            &sector
                .rotate_left(13)
                .wrapping_add(0x5549_4431)
                .to_le_bytes(),
        );
        bytes[12..16].copy_from_slice(
            &self
                .uuid
                .wrapping_mul(33)
                .wrapping_add(sector.rotate_right(3))
                .to_le_bytes(),
        );
        bytes
    }

    fn seed_sector_bytes(&self, sector: u32) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        let copy_len = self.seed_bytes.len().min(bytes.len());
        bytes[..copy_len].copy_from_slice(&self.seed_bytes[..copy_len]);
        let sector_bytes = sector.to_le_bytes();
        for (index, byte) in bytes.iter_mut().enumerate().skip(copy_len) {
            *byte = sector_bytes[index % sector_bytes.len()]
                .wrapping_add((index as u8).wrapping_mul(17))
                .wrapping_add(self.seed_bytes[index % self.seed_bytes.len()]);
        }
        bytes
    }
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(start);
    &bytes[start..end]
}

fn decimal_prefix_uuid(bytes: &[u8]) -> Option<u32> {
    let digits: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|byte| byte.is_ascii_digit())
        .take(10)
        .collect();
    if digits.is_empty() {
        return None;
    }
    std::str::from_utf8(&digits).ok()?.parse::<u32>().ok()
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
