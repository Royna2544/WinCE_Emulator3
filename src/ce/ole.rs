use crate::ce::{
    coredll::CoredllGuestMemory,
    kernel::CeKernel,
    memory::{HEAP_ZERO_MEMORY, PROCESS_HEAP_HANDLE},
    thread::{ERROR_INVALID_PARAMETER, ERROR_NOT_ENOUGH_MEMORY, ERROR_SUCCESS},
};

pub const S_OK: u32 = 0;
pub const E_INVALIDARG: u32 = 0x8007_0057;
pub const E_OUTOFMEMORY: u32 = 0x8007_000e;
pub const CO_E_CLASSSTRING: u32 = 0x8004_01f3;

pub fn string_from_clsid_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    clsid_ptr: u32,
    out_ptr: u32,
) -> u32 {
    if clsid_ptr == 0 || out_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return E_INVALIDARG;
    }

    let mut clsid = [0; 16];
    if memory.read_bytes(clsid_ptr, &mut clsid).is_err() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return E_INVALIDARG;
    }

    let string = format_guid(&clsid);
    let wide = wide_nul_bytes(&string);
    let Some(buffer_ptr) =
        kernel
            .memory
            .heap_alloc(PROCESS_HEAP_HANDLE, HEAP_ZERO_MEMORY, wide.len() as u32)
    else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_NOT_ENOUGH_MEMORY);
        return E_OUTOFMEMORY;
    };

    if memory.write_bytes(buffer_ptr, &wide).is_err()
        || memory.write_u32(out_ptr, buffer_ptr).is_err()
    {
        let _ = kernel.memory.heap_free(PROCESS_HEAP_HANDLE, 0, buffer_ptr);
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return E_INVALIDARG;
    }

    kernel.threads.set_last_error(thread_id, ERROR_SUCCESS);
    S_OK
}

pub fn co_task_mem_free_raw(kernel: &mut CeKernel, ptr: u32) {
    if ptr != 0 {
        let _ = kernel.memory.heap_free(PROCESS_HEAP_HANDLE, 0, ptr);
    }
}

pub fn clsid_from_string_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    string_ptr: u32,
    clsid_out: u32,
) -> u32 {
    if string_ptr == 0 || clsid_out == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return E_INVALIDARG;
    }
    let Some(text) = read_wide_z(memory, string_ptr, 64) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return E_INVALIDARG;
    };
    let Some(clsid) = parse_guid_string(&text) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return CO_E_CLASSSTRING;
    };
    if memory.write_bytes(clsid_out, &clsid).is_err() {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return E_INVALIDARG;
    }
    kernel.threads.set_last_error(thread_id, ERROR_SUCCESS);
    S_OK
}

fn format_guid(clsid: &[u8; 16]) -> String {
    let data1 = u32::from_le_bytes(clsid[0..4].try_into().unwrap());
    let data2 = u16::from_le_bytes(clsid[4..6].try_into().unwrap());
    let data3 = u16::from_le_bytes(clsid[6..8].try_into().unwrap());
    format!(
        "{{{data1:08x}-{data2:04x}-{data3:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
        clsid[8], clsid[9], clsid[10], clsid[11], clsid[12], clsid[13], clsid[14], clsid[15]
    )
}

fn parse_guid_string(text: &str) -> Option<[u8; 16]> {
    let trimmed = text.trim();
    let body = trimmed.strip_prefix('{')?.strip_suffix('}')?;
    let mut parts = body.split('-');
    let data1 = u32::from_str_radix(parts.next()?, 16).ok()?;
    let data2 = u16::from_str_radix(parts.next()?, 16).ok()?;
    let data3 = u16::from_str_radix(parts.next()?, 16).ok()?;
    let data4a = parts.next()?;
    let data4b = parts.next()?;
    if parts.next().is_some() || body.len() != 36 || data4a.len() != 4 || data4b.len() != 12 {
        return None;
    }
    let mut out = [0; 16];
    out[0..4].copy_from_slice(&data1.to_le_bytes());
    out[4..6].copy_from_slice(&data2.to_le_bytes());
    out[6..8].copy_from_slice(&data3.to_le_bytes());
    for index in 0..2 {
        out[8 + index] = u8::from_str_radix(&data4a[index * 2..index * 2 + 2], 16).ok()?;
    }
    for index in 0..6 {
        out[10 + index] = u8::from_str_radix(&data4b[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn read_wide_z<M: CoredllGuestMemory>(memory: &M, ptr: u32, max_chars: u32) -> Option<String> {
    let mut units = Vec::new();
    for index in 0..max_chars {
        let unit = memory.read_u16(ptr.wrapping_add(index * 2)).ok()?;
        if unit == 0 {
            return String::from_utf16(&units).ok();
        }
        units.push(unit);
    }
    None
}

fn wide_nul_bytes(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{config::RuntimeConfig, error::Result};

    #[derive(Default)]
    struct TestMemory {
        bytes: BTreeMap<u32, u8>,
    }

    impl TestMemory {
        fn write_seed(&mut self, addr: u32, bytes: &[u8]) {
            for (offset, byte) in bytes.iter().copied().enumerate() {
                self.bytes.insert(addr + offset as u32, byte);
            }
        }

        fn read_wide_string(&self, addr: u32) -> String {
            let mut units = Vec::new();
            for offset in (0..512).step_by(2) {
                let lo = self.bytes.get(&(addr + offset)).copied().unwrap_or(0);
                let hi = self.bytes.get(&(addr + offset + 1)).copied().unwrap_or(0);
                let unit = u16::from_le_bytes([lo, hi]);
                if unit == 0 {
                    break;
                }
                units.push(unit);
            }
            String::from_utf16(&units).unwrap()
        }
    }

    impl CoredllGuestMemory for TestMemory {
        fn read_u8(&self, addr: u32) -> Result<u8> {
            self.bytes
                .get(&addr)
                .copied()
                .ok_or_else(|| crate::error::Error::Backend(format!("unmapped read 0x{addr:08x}")))
        }

        fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
            self.bytes.insert(addr, value);
            Ok(())
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            let mut bytes = [0; 4];
            self.read_bytes(addr, &mut bytes)?;
            Ok(u32::from_le_bytes(bytes))
        }

        fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
            self.write_bytes(addr, &value.to_le_bytes())
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            let mut bytes = [0; 2];
            self.read_bytes(addr, &mut bytes)?;
            Ok(u16::from_le_bytes(bytes))
        }

        fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
            self.write_bytes(addr, &value.to_le_bytes())
        }
    }

    #[test]
    fn string_from_clsid_allocates_cotaskmem_wide_guid() {
        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory::default();
        memory.write_seed(
            0x1000,
            &[
                0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
                0xcd, 0xef,
            ],
        );

        assert_eq!(
            string_from_clsid_raw(&mut kernel, &mut memory, 1, 0x1000, 0x2000),
            S_OK
        );
        let allocated = memory.read_u32(0x2000).unwrap();
        assert_ne!(allocated, 0);
        assert_eq!(
            memory.read_wide_string(allocated),
            "{01234567-89ab-cdef-0123-456789abcdef}"
        );
        assert_eq!(
            kernel.memory.heap_size(PROCESS_HEAP_HANDLE, 0, allocated),
            Some(78)
        );

        co_task_mem_free_raw(&mut kernel, allocated);
        assert_eq!(
            kernel.memory.heap_size(PROCESS_HEAP_HANDLE, 0, allocated),
            None
        );
    }

    #[test]
    fn string_from_clsid_rejects_null_output_pointer() {
        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory::default();

        assert_eq!(
            string_from_clsid_raw(&mut kernel, &mut memory, 1, 0x1000, 0),
            E_INVALIDARG
        );
        assert_eq!(kernel.threads.get_last_error(1), ERROR_INVALID_PARAMETER);
    }

    #[test]
    fn clsid_from_string_parses_wide_guid_into_ce_layout() {
        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory::default();
        let text = "{01234567-89ab-cdef-0123-456789abcdef}";
        let bytes = wide_nul_bytes(text);
        memory.write_seed(0x1000, &bytes);

        assert_eq!(
            clsid_from_string_raw(&mut kernel, &mut memory, 1, 0x1000, 0x2000),
            S_OK
        );
        let mut parsed = [0; 16];
        memory.read_bytes(0x2000, &mut parsed).unwrap();
        assert_eq!(
            parsed,
            [
                0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
                0xcd, 0xef,
            ]
        );
    }
}
