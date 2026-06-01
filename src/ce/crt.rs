use crate::ce::{
    coredll::{CoredllGuestMemory, read_guest_bytes, write_guest_bytes, write_guest_u16},
    kernel::CeKernel,
    thread::ERROR_INVALID_PARAMETER,
};

pub(crate) fn wcsrchr_raw<M: CoredllGuestMemory>(memory: &M, string: u32, needle: u32) -> u32 {
    if string == 0 {
        return 0;
    }
    let needle = needle as u16;
    let mut last_match = 0;
    for index in 0..0x8000u32 {
        let addr = string.wrapping_add(index * 2);
        let Ok(unit) = memory.read_u16(addr) else {
            return 0;
        };
        if unit == needle {
            last_match = addr;
        }
        if unit == 0 {
            break;
        }
    }
    last_match
}

pub(crate) fn wcsdup_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    string: u32,
) -> u32 {
    if string == 0 {
        return 0;
    }
    let mut units = Vec::new();
    for index in 0..0x8000u32 {
        let addr = string.wrapping_add(index * 2);
        let Ok(unit) = memory.read_u16(addr) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        units.push(unit);
        if unit == 0 {
            break;
        }
    }
    if !units.ends_with(&[0]) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let bytes = u32::try_from(units.len())
        .ok()
        .and_then(|chars| chars.checked_mul(2))
        .unwrap_or(0);
    let ptr = malloc_raw(kernel, thread_id, bytes);
    if ptr == 0 {
        return 0;
    }
    for (index, unit) in units.iter().copied().enumerate() {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            ptr.wrapping_add(index as u32 * 2),
            unit,
        ) {
            free_raw(kernel, ptr);
            return 0;
        }
    }
    ptr
}

pub(crate) fn malloc_raw(kernel: &mut CeKernel, thread_id: u32, bytes: u32) -> u32 {
    match kernel
        .memory
        .heap_alloc(kernel.memory.get_process_heap(), 0, bytes)
    {
        Some(ptr) => ptr,
        None => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

pub(crate) fn free_raw(kernel: &mut CeKernel, ptr: u32) {
    if ptr != 0 {
        let _ = kernel
            .memory
            .heap_free(kernel.memory.get_process_heap(), 0, ptr);
    }
}

pub(crate) fn memcpy_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    src: u32,
    len: u32,
) -> u32 {
    if len == 0 {
        return dest;
    }
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, src, len) else {
        return 0;
    };
    if write_guest_bytes(kernel, memory, thread_id, dest, &bytes) {
        dest
    } else {
        0
    }
}

pub(crate) fn memset_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    value: u32,
    len: u32,
) -> u32 {
    if len == 0 {
        return dest;
    }
    let bytes = vec![value as u8; len as usize];
    if write_guest_bytes(kernel, memory, thread_id, dest, &bytes) {
        dest
    } else {
        0
    }
}

pub(crate) fn printf_family_raw(_kernel: &mut CeKernel, _thread_id: u32) -> u32 {
    0
}
