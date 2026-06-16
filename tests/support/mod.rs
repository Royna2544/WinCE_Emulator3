#![allow(dead_code)]

use std::{collections::BTreeMap, path::PathBuf};

use wince_emulation_v3::{Error, Result, ce::coredll::CoredllGuestMemory};

const TEST_HEAP_RANGE_START: u32 = 0x3000_0000;
const TEST_HEAP_RANGE_END: u32 = 0x5000_0000;

#[derive(Debug, Default)]
pub struct TestGuestMemory {
    bytes: BTreeMap<u32, u8>,
    words: BTreeMap<u32, u32>,
    halfwords: BTreeMap<u32, u16>,
}

impl TestGuestMemory {
    pub fn map_bytes(&mut self, base: u32, bytes: u32) {
        for index in 0..bytes {
            self.bytes.insert(base.wrapping_add(index), 0);
        }
    }

    pub fn map_words(&mut self, base: u32, words: u32) {
        for index in 0..words {
            self.write_word(base.wrapping_add(index * 4), 0);
        }
    }

    pub fn map_halfwords(&mut self, base: u32, halfwords: u32) {
        for index in 0..halfwords {
            self.halfwords.insert(base.wrapping_add(index * 2), 0);
        }
    }

    pub fn write_word(&mut self, addr: u32, value: u32) {
        self.words.insert(addr, value);
    }

    pub fn write_halfword(&mut self, addr: u32, value: u16) {
        self.halfwords.insert(addr, value);
    }

    pub fn write_bytes(&mut self, addr: u32, bytes: &[u8]) {
        for (index, byte) in bytes.iter().copied().enumerate() {
            self.bytes.insert(addr + index as u32, byte);
        }
    }

    pub fn fill(&mut self, addr: u32, value: u8, len: usize) {
        for index in 0..len {
            self.bytes.insert(addr + index as u32, value);
        }
    }

    pub fn read_bytes(&self, addr: u32, len: usize) -> Vec<u8> {
        (0..len)
            .map(|index| self.bytes.get(&(addr + index as u32)).copied().unwrap_or(0))
            .collect()
    }

    pub fn read_i32(&self, addr: u32) -> Result<i32> {
        Ok(self.read_u32(addr)? as i32)
    }

    pub fn write_point(&mut self, addr: u32, x: i32, y: i32) {
        self.write_word(addr, x as u32);
        self.write_word(addr + 4, y as u32);
    }

    pub fn write_wide_z(&mut self, addr: u32, text: &str) {
        for (index, unit) in text.encode_utf16().chain(std::iter::once(0)).enumerate() {
            self.halfwords.insert(addr + (index as u32) * 2, unit);
        }
    }

    pub fn write_wave_format_pcm(&mut self, addr: u32, channels: u16, samples_per_sec: u32) {
        let block_align = channels * 2;
        self.write_halfword(addr, 1);
        self.write_halfword(addr + 2, channels);
        self.write_word(addr + 4, samples_per_sec);
        self.write_word(addr + 8, samples_per_sec * u32::from(block_align));
        self.write_halfword(addr + 12, block_align);
        self.write_halfword(addr + 14, 16);
        self.write_halfword(addr + 16, 0);
    }

    pub fn read_wide_z(&self, addr: u32, max_chars: usize) -> String {
        let mut units = Vec::new();
        for index in 0..max_chars {
            let unit = self
                .halfwords
                .get(&(addr + (index as u32) * 2))
                .copied()
                .unwrap_or(0);
            if unit == 0 {
                break;
            }
            units.push(unit);
        }
        String::from_utf16_lossy(&units)
    }
}

impl CoredllGuestMemory for TestGuestMemory {
    fn read_u8(&self, addr: u32) -> Result<u8> {
        self.bytes
            .get(&addr)
            .copied()
            .or_else(|| is_test_heap_address(addr).then_some(0))
            .ok_or_else(|| Error::Backend(format!("unmapped test byte 0x{addr:08x}")))
    }

    fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        if let Some(byte) = self.bytes.get_mut(&addr) {
            *byte = value;
            Ok(())
        } else if is_test_heap_address(addr) {
            self.bytes.insert(addr, value);
            Ok(())
        } else {
            Err(Error::Backend(format!("unmapped test byte 0x{addr:08x}")))
        }
    }

    fn read_u32(&self, addr: u32) -> Result<u32> {
        self.words
            .get(&addr)
            .copied()
            .ok_or_else(|| Error::Backend(format!("unmapped test word 0x{addr:08x}")))
    }

    fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        if let Some(word) = self.words.get_mut(&addr) {
            *word = value;
            Ok(())
        } else if is_test_heap_address(addr) {
            self.words.insert(addr, value);
            Ok(())
        } else {
            Err(Error::Backend(format!("unmapped test word 0x{addr:08x}")))
        }
    }

    fn read_u16(&self, addr: u32) -> Result<u16> {
        if let Some(halfword) = self.halfwords.get(&addr).copied() {
            Ok(halfword)
        } else if is_test_heap_address(addr) {
            let lo = self.bytes.get(&addr).copied().unwrap_or(0);
            let hi = self.bytes.get(&addr.wrapping_add(1)).copied().unwrap_or(0);
            Ok(u16::from_le_bytes([lo, hi]))
        } else {
            Err(Error::Backend(format!(
                "unmapped test halfword 0x{addr:08x}"
            )))
        }
    }

    fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        if let Some(halfword) = self.halfwords.get_mut(&addr) {
            *halfword = value;
            let [lo, hi] = value.to_le_bytes();
            self.bytes.insert(addr, lo);
            self.bytes.insert(addr.wrapping_add(1), hi);
            Ok(())
        } else if is_test_heap_address(addr) {
            self.halfwords.insert(addr, value);
            let [lo, hi] = value.to_le_bytes();
            self.bytes.insert(addr, lo);
            self.bytes.insert(addr.wrapping_add(1), hi);
            Ok(())
        } else {
            Err(Error::Backend(format!(
                "unmapped test halfword 0x{addr:08x}"
            )))
        }
    }

    fn ensure_mapped(&mut self, addr: u32, len: u32) -> Result<()> {
        self.map_bytes(addr, len);
        Ok(())
    }
}

fn is_test_heap_address(addr: u32) -> bool {
    (TEST_HEAP_RANGE_START..TEST_HEAP_RANGE_END).contains(&addr)
}

pub fn unique_test_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wince_emulation_v3_{name}_{}", std::process::id()))
}
