use crate::ce::{
    coredll::{CoredllGuestMemory, read_guest_bytes, write_guest_bytes, write_guest_u16},
    file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE, OPEN_ALWAYS, OPEN_EXISTING},
    kernel::CeKernel,
    thread::{ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER},
};

const CRT_ACP_CODE_PAGE: u32 = 949;

pub(crate) fn wcsrchr_raw<M: CoredllGuestMemory>(memory: &M, string: u32, needle: u32) -> u32 {
    if string == 0 {
        return 0;
    }
    let needle = needle as u16;
    let mut last_match = 0;
    for index in 0..0x8000u32 {
        let addr = string.wrapping_add(index * 2);
        let Ok(unit) = memory.read_u16(addr) else {
            return last_match;
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

pub(crate) fn wcsstr_raw<M: CoredllGuestMemory>(memory: &M, haystack: u32, needle: u32) -> u32 {
    if haystack == 0 || needle == 0 {
        return 0;
    }
    let Ok(first_needle) = memory.read_u16(needle) else {
        return 0;
    };
    if first_needle == 0 {
        return haystack;
    }

    for hay_index in 0..0x8000u32 {
        let candidate = haystack.wrapping_add(hay_index * 2);
        let Ok(hay_unit) = memory.read_u16(candidate) else {
            return 0;
        };
        if hay_unit == 0 {
            return 0;
        }
        if hay_unit != first_needle {
            continue;
        }

        for match_index in 1..0x8000u32.saturating_sub(hay_index) {
            let Ok(needle_unit) = memory.read_u16(needle.wrapping_add(match_index * 2)) else {
                return 0;
            };
            if needle_unit == 0 {
                return candidate;
            }
            let Ok(hay_unit) = memory.read_u16(candidate.wrapping_add(match_index * 2)) else {
                return 0;
            };
            if hay_unit != needle_unit {
                break;
            }
            if hay_unit == 0 {
                return 0;
            }
        }
    }
    0
}

pub(crate) fn wcslen_raw<M: CoredllGuestMemory>(memory: &M, string: u32) -> u32 {
    if string == 0 {
        return 0;
    }
    for index in 0..0x8000u32 {
        let addr = string.wrapping_add(index * 2);
        let Ok(unit) = memory.read_u16(addr) else {
            return index;
        };
        if unit == 0 {
            return index;
        }
    }
    0x8000
}

pub(crate) fn wcspbrk_raw<M: CoredllGuestMemory>(memory: &M, string: u32, accept: u32) -> u32 {
    if string == 0 || accept == 0 {
        return 0;
    }

    for string_index in 0..0x8000u32 {
        let candidate = string.wrapping_add(string_index * 2);
        let Ok(candidate_unit) = memory.read_u16(candidate) else {
            return 0;
        };
        if candidate_unit == 0 {
            return 0;
        }

        for accept_index in 0..0x8000u32 {
            let Ok(accept_unit) = memory.read_u16(accept.wrapping_add(accept_index * 2)) else {
                return 0;
            };
            if accept_unit == 0 {
                break;
            }
            if accept_unit == candidate_unit {
                return candidate;
            }
        }
    }
    0
}

pub(crate) fn wcschr_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    string: u32,
    needle: u32,
) -> u32 {
    if string == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let needle = needle as u16;
    for index in 0..0x8000u32 {
        let addr = string.wrapping_add(index * 2);
        let Ok(unit) = memory.read_u16(addr) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        if unit == needle {
            kernel.threads.set_last_error(thread_id, 0);
            return addr;
        }
        if unit == 0 {
            kernel.threads.set_last_error(thread_id, 0);
            return 0;
        }
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
    0
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

pub(crate) fn wcsnicmp_raw<M: CoredllGuestMemory>(
    memory: &M,
    left: u32,
    right: u32,
    count: u32,
) -> i32 {
    if count == 0 {
        return 0;
    }
    if left == 0 || right == 0 {
        return if left == right {
            0
        } else if left == 0 {
            -1
        } else {
            1
        };
    }
    for index in 0..count.min(0x8000) {
        let left_addr = left.wrapping_add(index * 2);
        let right_addr = right.wrapping_add(index * 2);
        let Ok(left_unit) = memory.read_u16(left_addr) else {
            return -1;
        };
        let Ok(right_unit) = memory.read_u16(right_addr) else {
            return 1;
        };
        let left_folded = fold_ascii_wide(left_unit);
        let right_folded = fold_ascii_wide(right_unit);
        if left_folded != right_folded {
            return i32::from(left_folded).saturating_sub(i32::from(right_folded));
        }
        if left_unit == 0 {
            return 0;
        }
    }
    0
}

pub(crate) fn wcsncmp_raw<M: CoredllGuestMemory>(
    memory: &M,
    left: u32,
    right: u32,
    count: u32,
) -> i32 {
    if count == 0 {
        return 0;
    }
    if left == 0 || right == 0 {
        return if left == right {
            0
        } else if left == 0 {
            -1
        } else {
            1
        };
    }
    for index in 0..count.min(0x8000) {
        let left_addr = left.wrapping_add(index * 2);
        let right_addr = right.wrapping_add(index * 2);
        let Ok(left_unit) = memory.read_u16(left_addr) else {
            return -1;
        };
        let Ok(right_unit) = memory.read_u16(right_addr) else {
            return 1;
        };
        if left_unit != right_unit {
            return i32::from(left_unit).saturating_sub(i32::from(right_unit));
        }
        if left_unit == 0 {
            return 0;
        }
    }
    0
}

pub(crate) fn wcsicmp_raw<M: CoredllGuestMemory>(memory: &M, left: u32, right: u32) -> i32 {
    if left == 0 || right == 0 {
        return if left == right {
            0
        } else if left == 0 {
            -1
        } else {
            1
        };
    }
    for index in 0..0x8000u32 {
        let left_addr = left.wrapping_add(index * 2);
        let right_addr = right.wrapping_add(index * 2);
        let Ok(left_unit) = memory.read_u16(left_addr) else {
            return -1;
        };
        let Ok(right_unit) = memory.read_u16(right_addr) else {
            return 1;
        };
        let left_folded = fold_ascii_wide(left_unit);
        let right_folded = fold_ascii_wide(right_unit);
        if left_folded != right_folded {
            return i32::from(left_folded).saturating_sub(i32::from(right_folded));
        }
        if left_unit == 0 {
            return 0;
        }
    }
    0
}

pub(crate) fn wtol_raw<M: CoredllGuestMemory>(memory: &M, text_ptr: u32) -> i32 {
    let Some(text) = read_wide_z(memory, text_ptr, 128) else {
        return 0;
    };
    parse_decimal_prefix(text.trim_start())
}

pub(crate) fn wcstoul_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    text_ptr: u32,
    end_ptr: u32,
    base: u32,
) -> u32 {
    if text_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(text) = read_wide_z(memory, text_ptr, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let parsed = parse_unsigned_prefix(&text, base);
    if end_ptr != 0
        && memory
            .write_u32(
                end_ptr,
                text_ptr.wrapping_add((parsed.consumed as u32).saturating_mul(2)),
            )
            .is_err()
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    parsed.value
}

pub(crate) fn wcsncpy_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    src: u32,
    count: u32,
) -> u32 {
    if count == 0 {
        return dest;
    }
    if dest == 0 || src == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }

    let units = ce_wcsncpy_units(count);
    let src = wcsncpy_source(memory, src);
    let mut padding = false;
    for index in 0..units {
        let unit = if padding {
            0
        } else {
            let Ok(unit) = memory.read_u16(src.wrapping_add(index * 2)) else {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
                return 0;
            };
            if unit == 0 {
                padding = true;
            }
            unit
        };
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            dest.wrapping_add(index * 2),
            unit,
        ) {
            return 0;
        }
    }
    dest
}

pub(crate) fn wcscpy_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    src: u32,
) -> u32 {
    if dest == 0 || src == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let mut units = Vec::new();
    for index in 0..0x8000u32 {
        let Ok(unit) = memory.read_u16(src.wrapping_add(index * 2)) else {
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
    for (index, unit) in units.into_iter().enumerate() {
        if !write_guest_u16(
            kernel,
            memory,
            thread_id,
            dest.wrapping_add((index as u32) * 2),
            unit,
        ) {
            return 0;
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    dest
}

fn wcsncpy_source<M: CoredllGuestMemory>(memory: &M, src: u32) -> u32 {
    let Ok(pointer) = memory.read_u32(src) else {
        return src;
    };
    if pointer <= 0xffff || memory.read_u16(src).is_ok_and(is_plain_wide_start) {
        return src;
    }
    if memory.read_u16(pointer).is_ok_and(is_plain_wide_start) {
        pointer
    } else {
        src
    }
}

fn is_plain_wide_start(unit: u16) -> bool {
    unit == b'\\' as u16
        || unit == b'/' as u16
        || (b'A' as u16..=b'Z' as u16).contains(&unit)
        || (b'a' as u16..=b'z' as u16).contains(&unit)
}

fn ce_wcsncpy_units(count: u32) -> u32 {
    if count % 2 == 0 { count / 2 } else { count }
}

fn fold_ascii_wide(unit: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&unit) {
        unit + 0x20
    } else {
        unit
    }
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

pub(crate) fn msize_raw(kernel: &mut CeKernel, thread_id: u32, ptr: u32) -> u32 {
    match kernel.memory.allocation(ptr) {
        Some(allocation) if allocation.heap == kernel.memory.get_process_heap() => {
            kernel.threads.set_last_error(thread_id, 0);
            allocation.actual_size
        }
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            0
        }
    }
}

pub(crate) fn realloc_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    ptr: u32,
    bytes: u32,
) -> u32 {
    if ptr == 0 {
        return malloc_raw(kernel, thread_id, bytes);
    }
    if bytes == 0 {
        free_raw(kernel, ptr);
        return 0;
    }
    match kernel
        .memory
        .heap_re_alloc_detail(kernel.memory.get_process_heap(), 0, ptr, bytes)
    {
        Some(result) => {
            if result.moved {
                let copy_len = result.old_actual_size.min(result.new_actual_size);
                if copy_len != 0 {
                    let Some(old_bytes) =
                        read_guest_bytes(kernel, memory, thread_id, result.old_ptr, copy_len)
                    else {
                        return 0;
                    };
                    if !write_guest_bytes(kernel, memory, thread_id, result.ptr, &old_bytes) {
                        return 0;
                    }
                }
            }
            kernel.threads.set_last_error(thread_id, 0);
            result.ptr
        }
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
    if copy_guest_bytes_chunked(memory, dest, src, len).is_ok() {
        dest
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        0
    }
}

fn copy_guest_bytes_chunked<M: CoredllGuestMemory>(
    memory: &mut M,
    dest: u32,
    src: u32,
    len: u32,
) -> std::result::Result<(), ()> {
    const CHUNK: u32 = 64 * 1024;
    let mut buffer = vec![0; CHUNK as usize];
    let overlaps_backward = dest > src && dest < src.wrapping_add(len);
    if overlaps_backward {
        let mut remaining = len;
        while remaining != 0 {
            let count = remaining.min(CHUNK);
            remaining = remaining.wrapping_sub(count);
            let count_usize = count as usize;
            memory
                .read_bytes(src.wrapping_add(remaining), &mut buffer[..count_usize])
                .map_err(|_| ())?;
            memory
                .write_bytes(dest.wrapping_add(remaining), &buffer[..count_usize])
                .map_err(|_| ())?;
        }
    } else {
        let mut offset = 0u32;
        while offset < len {
            let count = (len - offset).min(CHUNK);
            let count_usize = count as usize;
            memory
                .read_bytes(src.wrapping_add(offset), &mut buffer[..count_usize])
                .map_err(|_| ())?;
            memory
                .write_bytes(dest.wrapping_add(offset), &buffer[..count_usize])
                .map_err(|_| ())?;
            offset = offset.wrapping_add(count);
        }
    }
    Ok(())
}

pub(crate) fn strcpy_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    src: u32,
) -> u32 {
    if dest == 0 || src == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let mut bytes = Vec::new();
    for index in 0..0x8000u32 {
        let Ok(byte) = memory.read_u8(src.wrapping_add(index)) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        bytes.push(byte);
        if byte == 0 {
            break;
        }
    }
    if !bytes.ends_with(&[0]) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if write_guest_bytes(kernel, memory, thread_id, dest, &bytes) {
        kernel.threads.set_last_error(thread_id, 0);
        dest
    } else {
        0
    }
}

pub(crate) fn strcat_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    src: u32,
) -> u32 {
    if dest == 0 || src == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }

    let mut dest_len = None;
    for index in 0..0x8000u32 {
        let Ok(byte) = memory.read_u8(dest.wrapping_add(index)) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        if byte == 0 {
            dest_len = Some(index);
            break;
        }
    }
    let Some(dest_len) = dest_len else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };

    let mut bytes = Vec::new();
    for index in 0..0x8000u32 {
        let Ok(byte) = memory.read_u8(src.wrapping_add(index)) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        bytes.push(byte);
        if byte == 0 {
            break;
        }
    }
    if !bytes.ends_with(&[0]) {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }

    if write_guest_bytes(
        kernel,
        memory,
        thread_id,
        dest.wrapping_add(dest_len),
        &bytes,
    ) {
        kernel.threads.set_last_error(thread_id, 0);
        dest
    } else {
        0
    }
}

pub(crate) fn strupr_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    string: u32,
) -> u32 {
    if string == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    for index in 0..0x8000u32 {
        let addr = string.wrapping_add(index);
        let Ok(byte) = memory.read_u8(addr) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            return 0;
        };
        if byte == 0 {
            kernel.threads.set_last_error(thread_id, 0);
            return string;
        }
        if byte.is_ascii_lowercase()
            && !write_guest_bytes(
                kernel,
                memory,
                thread_id,
                addr,
                &[byte.to_ascii_uppercase()],
            )
        {
            return 0;
        }
    }
    kernel
        .threads
        .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
    0
}

pub(crate) fn strtok_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    string: u32,
    delimiters: u32,
) -> u32 {
    if delimiters == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(delimiters) = read_delimiter_set(memory, delimiters) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let mut cursor = if string != 0 {
        string
    } else {
        kernel.crt_strtok_next(thread_id)
    };
    if cursor == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }

    loop {
        let Ok(byte) = memory.read_u8(cursor) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            kernel.crt_set_strtok_next(thread_id, 0);
            return 0;
        };
        if byte == 0 {
            kernel.crt_set_strtok_next(thread_id, 0);
            kernel.threads.set_last_error(thread_id, 0);
            return 0;
        }
        if !delimiters.contains(&byte) {
            break;
        }
        cursor = cursor.wrapping_add(1);
    }

    let token = cursor;
    loop {
        let Ok(byte) = memory.read_u8(cursor) else {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            kernel.crt_set_strtok_next(thread_id, 0);
            return 0;
        };
        if byte == 0 {
            kernel.crt_set_strtok_next(thread_id, 0);
            kernel.threads.set_last_error(thread_id, 0);
            return token;
        }
        if delimiters.contains(&byte) {
            if !write_guest_bytes(kernel, memory, thread_id, cursor, &[0]) {
                kernel.crt_set_strtok_next(thread_id, 0);
                return 0;
            }
            kernel.crt_set_strtok_next(thread_id, cursor.wrapping_add(1));
            kernel.threads.set_last_error(thread_id, 0);
            return token;
        }
        cursor = cursor.wrapping_add(1);
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
    if memory.fill_bytes(dest, value as u8, len).is_ok() {
        dest
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        0
    }
}

pub(crate) fn memcmp_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    left: u32,
    right: u32,
    len: u32,
) -> i32 {
    if len == 0 {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    let Some(left_bytes) = read_guest_bytes(kernel, memory, thread_id, left, len) else {
        return -1;
    };
    let Some(right_bytes) = read_guest_bytes(kernel, memory, thread_id, right, len) else {
        return 1;
    };
    for (left, right) in left_bytes.iter().zip(right_bytes.iter()) {
        if left != right {
            kernel.threads.set_last_error(thread_id, 0);
            return i32::from(*left) - i32::from(*right);
        }
    }
    kernel.threads.set_last_error(thread_id, 0);
    0
}

pub(crate) fn printf_family_raw(_kernel: &mut CeKernel, _thread_id: u32) -> u32 {
    0
}

pub(crate) fn srand_raw(kernel: &mut CeKernel, thread_id: u32, seed: u32) -> u32 {
    kernel.crt_srand(seed);
    kernel.threads.set_last_error(thread_id, 0);
    0
}

pub(crate) fn rand_raw(kernel: &mut CeKernel, thread_id: u32) -> u32 {
    let value = kernel.crt_rand();
    kernel.threads.set_last_error(thread_id, 0);
    value
}

pub(crate) fn atoi_raw<M: CoredllGuestMemory>(memory: &M, text_ptr: u32) -> i32 {
    let Some(text) = read_narrow_z(memory, text_ptr, 128) else {
        return 0;
    };
    parse_decimal_prefix(text.trim_start())
}

pub(crate) fn atof_raw<M: CoredllGuestMemory>(memory: &M, text_ptr: u32) -> f64 {
    let Some(text) = read_narrow_z(memory, text_ptr, 4096) else {
        return 0.0;
    };
    parse_float_prefix(&text)
}

pub(crate) fn tolower_raw(ch: u32) -> u32 {
    if (b'A' as u32..=b'Z' as u32).contains(&ch) {
        ch + 0x20
    } else {
        ch
    }
}

pub(crate) fn toupper_raw(ch: u32) -> u32 {
    if (b'a' as u32..=b'z' as u32).contains(&ch) {
        ch - 0x20
    } else {
        ch
    }
}

pub(crate) fn strtoul_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    text_ptr: u32,
    end_ptr: u32,
    base: u32,
) -> u32 {
    if text_ptr == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(text) = read_narrow_z(memory, text_ptr, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let parsed = parse_unsigned_prefix(&text, base);
    if end_ptr != 0
        && memory
            .write_u32(end_ptr, text_ptr.wrapping_add(parsed.consumed as u32))
            .is_err()
    {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    kernel.threads.set_last_error(thread_id, 0);
    parsed.value
}

pub(crate) fn fopen_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    path: u32,
    mode: u32,
) -> u32 {
    let Some(path) = read_narrow_z(memory, path, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let mode = read_narrow_z(memory, mode, 32).unwrap_or_else(|| "r".to_owned());
    let (access, disposition) = stdio_open_flags(&mode);
    match kernel.create_file_w(&path, access, disposition) {
        Ok(handle) => {
            if mode.contains('a') {
                let _ = kernel.set_file_pointer(handle, 0, 2);
            }
            kernel.threads.set_last_error(thread_id, 0);
            handle
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            0
        }
    }
}

pub(crate) fn wfopen_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    path: u32,
    mode: u32,
) -> u32 {
    let Some(path) = read_wide_z(memory, path, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let mode = read_wide_z(memory, mode, 32).unwrap_or_else(|| "r".to_owned());
    let (access, disposition) = stdio_open_flags(&mode);
    match kernel.create_file_w(&path, access, disposition) {
        Ok(handle) => {
            if mode.contains('a') {
                let _ = kernel.set_file_pointer(handle, 0, 2);
            }
            kernel.threads.set_last_error(thread_id, 0);
            handle
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_FILE_NOT_FOUND);
            0
        }
    }
}

pub(crate) fn fread_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    size: u32,
    count: u32,
    stream: u32,
) -> u32 {
    if dest == 0 || size == 0 || count == 0 {
        return 0;
    }
    let Some(requested) = size.checked_mul(count) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    match kernel.read_file(stream, requested) {
        Ok(bytes) => {
            if !write_guest_bytes(kernel, memory, thread_id, dest, &bytes) {
                return 0;
            }
            kernel.threads.set_last_error(thread_id, 0);
            (bytes.len() as u32) / size
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            0
        }
    }
}

pub(crate) fn fgets_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    count: u32,
    stream: u32,
) -> u32 {
    if dest == 0 || count == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    if count == 1 {
        return if write_guest_bytes(kernel, memory, thread_id, dest, &[0]) {
            kernel.threads.set_last_error(thread_id, 0);
            dest
        } else {
            0
        };
    }
    let mut bytes = Vec::new();
    while bytes.len() < count.saturating_sub(1) as usize {
        match kernel.read_file(stream, 1) {
            Ok(chunk) if chunk.is_empty() => break,
            Ok(chunk) => {
                let byte = chunk[0];
                bytes.push(byte);
                if byte == b'\n' {
                    break;
                }
            }
            Err(_) => {
                kernel
                    .threads
                    .set_last_error(thread_id, ERROR_INVALID_HANDLE);
                return 0;
            }
        }
    }
    if bytes.is_empty() {
        kernel.threads.set_last_error(thread_id, 0);
        return 0;
    }
    bytes.push(0);
    if write_guest_bytes(kernel, memory, thread_id, dest, &bytes) {
        kernel.threads.set_last_error(thread_id, 0);
        dest
    } else {
        0
    }
}

pub(crate) fn fwrite_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &M,
    thread_id: u32,
    src: u32,
    size: u32,
    count: u32,
    stream: u32,
) -> u32 {
    if src == 0 || size == 0 || count == 0 {
        return 0;
    }
    let Some(requested) = size.checked_mul(count) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(bytes) = read_guest_bytes(kernel, memory, thread_id, src, requested) else {
        return 0;
    };
    match kernel.write_file(stream, &bytes) {
        Ok(result) if result.success => {
            kernel.threads.set_last_error(thread_id, 0);
            result.bytes_transferred / size
        }
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            0
        }
    }
}

pub(crate) fn fseek_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    stream: u32,
    offset: u32,
    origin: u32,
) -> u32 {
    match kernel.set_file_pointer(stream, i64::from(offset as i32), origin) {
        Ok(_) => {
            kernel.threads.set_last_error(thread_id, 0);
            0
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
            u32::MAX
        }
    }
}

pub(crate) fn ftell_raw(kernel: &mut CeKernel, thread_id: u32, stream: u32) -> u32 {
    match kernel.file_position(stream) {
        Ok(position) => {
            kernel.threads.set_last_error(thread_id, 0);
            position as u32
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
    }
}

pub(crate) fn fclose_raw(kernel: &mut CeKernel, thread_id: u32, stream: u32) -> u32 {
    match kernel.close_handle(stream) {
        Ok(true) => {
            kernel.threads.set_last_error(thread_id, 0);
            0
        }
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
    }
}

pub(crate) fn fflush_raw(kernel: &mut CeKernel, thread_id: u32, stream: u32) -> u32 {
    match kernel.flush_file_buffers(stream) {
        Ok(true) => {
            kernel.threads.set_last_error(thread_id, 0);
            0
        }
        _ => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            u32::MAX
        }
    }
}

pub(crate) fn feof_raw(kernel: &mut CeKernel, thread_id: u32, stream: u32) -> u32 {
    match kernel.file_is_eof(stream) {
        Ok(eof) => {
            kernel.threads.set_last_error(thread_id, 0);
            u32::from(eof)
        }
        Err(_) => {
            kernel
                .threads
                .set_last_error(thread_id, ERROR_INVALID_HANDLE);
            0
        }
    }
}

pub(crate) fn ferror_raw(kernel: &mut CeKernel, thread_id: u32, stream: u32) -> u32 {
    if kernel.file_position(stream).is_ok() {
        kernel.threads.set_last_error(thread_id, 0);
        0
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_HANDLE);
        1
    }
}

fn stdio_open_flags(mode: &str) -> (u32, u32) {
    let write = mode.contains('w');
    let append = mode.contains('a');
    let update = mode.contains('+');
    let access = if write || append || update {
        GENERIC_READ | GENERIC_WRITE
    } else {
        GENERIC_READ
    };
    let disposition = if write {
        CREATE_ALWAYS
    } else if append {
        OPEN_ALWAYS
    } else {
        OPEN_EXISTING
    };
    (access, disposition)
}

pub(crate) fn sprintf_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    format: u32,
    args: &[u32],
) -> u32 {
    if dest == 0 || format == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(format) = read_narrow_z(memory, format, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let text = format_wide_printf(memory, &format, args, WideStringMode::NarrowDefault);
    let Some(mut bytes) = encode_narrow_acp(&text) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let len = bytes.len() as u32;
    bytes.push(0);
    if write_guest_bytes(kernel, memory, thread_id, dest, &bytes) {
        len
    } else {
        0
    }
}

pub(crate) fn snprintf_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    count: u32,
    format: u32,
    args: &[u32],
) -> u32 {
    if dest == 0 || format == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    }
    let Some(format) = read_narrow_z(memory, format, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    };
    let text = format_wide_printf(memory, &format, args, WideStringMode::NarrowDefault);
    write_bounded_narrow(kernel, memory, thread_id, dest, count, &text)
}

pub(crate) fn vsnprintf_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    count: u32,
    format: u32,
    va_list: u32,
) -> u32 {
    if va_list == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    }
    let args = read_va_list_words(memory, va_list, 64);
    snprintf_raw(kernel, memory, thread_id, dest, count, format, &args)
}

pub(crate) fn vsprintf_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    format: u32,
    va_list: u32,
) -> u32 {
    if va_list == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let args = read_va_list_words(memory, va_list, 64);
    sprintf_raw(kernel, memory, thread_id, dest, format, &args)
}

pub(crate) fn wsprintf_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    format: u32,
    args: &[u32],
) -> u32 {
    wsprintf_w_raw_with_mode(
        kernel,
        memory,
        thread_id,
        dest,
        format,
        args,
        WideStringMode::WideDefault,
    )
}

pub(crate) fn swprintf_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    format: u32,
    args: &[u32],
) -> u32 {
    wsprintf_w_raw_with_mode(
        kernel,
        memory,
        thread_id,
        dest,
        format,
        args,
        WideStringMode::WideDefault,
    )
}

fn wsprintf_w_raw_with_mode<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    format: u32,
    args: &[u32],
    string_mode: WideStringMode,
) -> u32 {
    if dest == 0 || format == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let Some(format) = read_wide_z(memory, format, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let text = format_wide_printf(memory, &format, args, string_mode);
    for (index, unit) in text.encode_utf16().chain(std::iter::once(0)).enumerate() {
        let addr = dest.wrapping_add(index as u32 * 2);
        if !write_guest_u16(kernel, memory, thread_id, addr, unit) {
            return 0;
        }
    }
    text.encode_utf16().count() as u32
}

pub(crate) fn snwprintf_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    count: u32,
    format: u32,
    args: &[u32],
) -> u32 {
    if dest == 0 || format == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    }
    let Some(format) = read_wide_z(memory, format, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    };
    let text = format_wide_printf(memory, &format, args, WideStringMode::WideDefault);
    write_bounded_wide(kernel, memory, thread_id, dest, count, &text)
}

pub(crate) fn vswprintf_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    format: u32,
    va_list: u32,
) -> u32 {
    if va_list == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let args = read_va_list_words(memory, va_list, 64);
    wsprintf_w_raw_with_mode(
        kernel,
        memory,
        thread_id,
        dest,
        format,
        &args,
        WideStringMode::WideDefault,
    )
}

pub(crate) fn wvsprintf_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    format: u32,
    va_list: u32,
) -> u32 {
    if va_list == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    }
    let args = read_va_list_words(memory, va_list, 64);
    wsprintf_w_raw_with_mode(
        kernel,
        memory,
        thread_id,
        dest,
        format,
        &args,
        WideStringMode::WideDefault,
    )
}

pub(crate) fn vsnwprintf_w_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    count: u32,
    format: u32,
    va_list: u32,
) -> u32 {
    if va_list == 0 {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    }
    let args = read_va_list_words(memory, va_list, 64);
    snwprintf_w_raw(kernel, memory, thread_id, dest, count, format, &args)
}

fn read_va_list_words<M: CoredllGuestMemory>(memory: &M, va_list: u32, max_words: u32) -> Vec<u32> {
    let mut args = Vec::new();
    for index in 0..max_words {
        let addr = va_list.wrapping_add(index * 4);
        let Ok(value) = memory.read_u32(addr) else {
            break;
        };
        args.push(value);
    }
    args
}

fn read_wide_z<M: CoredllGuestMemory>(memory: &M, ptr: u32, max_chars: usize) -> Option<String> {
    let mut units = Vec::new();
    for index in 0..max_chars {
        let unit = memory.read_u16(ptr.wrapping_add(index as u32 * 2)).ok()?;
        if unit == 0 {
            return String::from_utf16(&units).ok();
        }
        units.push(unit);
    }
    None
}

fn read_narrow_z<M: CoredllGuestMemory>(memory: &M, ptr: u32, max_chars: usize) -> Option<String> {
    let mut bytes = Vec::new();
    for index in 0..max_chars {
        let byte = memory.read_u8(ptr.wrapping_add(index as u32)).ok()?;
        if byte == 0 {
            return decode_narrow_acp(&bytes);
        }
        bytes.push(byte);
    }
    None
}

#[cfg(windows)]
fn decode_narrow_acp(bytes: &[u8]) -> Option<String> {
    use windows::Win32::Globalization::{MULTI_BYTE_TO_WIDE_CHAR_FLAGS, MultiByteToWideChar};

    if bytes.is_empty() {
        return Some(String::new());
    }
    let needed = unsafe {
        MultiByteToWideChar(
            CRT_ACP_CODE_PAGE,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            bytes,
            None,
        )
    };
    if needed <= 0 {
        return None;
    }
    let mut units = vec![0; needed as usize];
    let written = unsafe {
        MultiByteToWideChar(
            CRT_ACP_CODE_PAGE,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            bytes,
            Some(&mut units),
        )
    };
    if written <= 0 {
        return None;
    }
    units.truncate(written as usize);
    String::from_utf16(&units).ok()
}

#[cfg(not(windows))]
fn decode_narrow_acp(bytes: &[u8]) -> Option<String> {
    Some(
        bytes
            .iter()
            .copied()
            .map(|byte| {
                if byte <= 0x7f {
                    char::from(byte)
                } else {
                    char::REPLACEMENT_CHARACTER
                }
            })
            .collect(),
    )
}

#[cfg(windows)]
fn encode_narrow_acp(text: &str) -> Option<Vec<u8>> {
    use windows::{
        Win32::{Foundation::BOOL, Globalization::WideCharToMultiByte},
        core::PCSTR,
    };

    if text.is_empty() {
        return Some(Vec::new());
    }
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut used_default = BOOL(0);
    let needed = unsafe {
        WideCharToMultiByte(
            CRT_ACP_CODE_PAGE,
            0,
            &units,
            None,
            PCSTR::null(),
            Some(&mut used_default),
        )
    };
    if needed <= 0 {
        return None;
    }
    let mut bytes = vec![0; needed as usize];
    let written = unsafe {
        WideCharToMultiByte(
            CRT_ACP_CODE_PAGE,
            0,
            &units,
            Some(&mut bytes),
            PCSTR::null(),
            Some(&mut used_default),
        )
    };
    if written <= 0 {
        return None;
    }
    bytes.truncate(written as usize);
    Some(bytes)
}

#[cfg(not(windows))]
fn encode_narrow_acp(text: &str) -> Option<Vec<u8>> {
    Some(
        text.encode_utf16()
            .map(|unit| if unit <= 0x7f { unit as u8 } else { b'?' })
            .collect(),
    )
}

fn write_bounded_narrow<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    count: u32,
    text: &str,
) -> u32 {
    let Some(bytes) = encode_narrow_acp(text) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return u32::MAX;
    };
    if count == 0 {
        return if bytes.is_empty() { 0 } else { u32::MAX };
    }
    let count = count as usize;
    if bytes.len() < count {
        let mut out = Vec::with_capacity(bytes.len() + 1);
        out.extend_from_slice(&bytes);
        out.push(0);
        if write_guest_bytes(kernel, memory, thread_id, dest, &out) {
            bytes.len() as u32
        } else {
            u32::MAX
        }
    } else {
        let truncated = &bytes[..count];
        if write_guest_bytes(kernel, memory, thread_id, dest, truncated) {
            u32::MAX
        } else {
            u32::MAX
        }
    }
}

fn write_bounded_wide<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    dest: u32,
    count: u32,
    text: &str,
) -> u32 {
    let units: Vec<u16> = text.encode_utf16().collect();
    if count == 0 {
        return if units.is_empty() { 0 } else { u32::MAX };
    }
    let count = count as usize;
    let write_units: Vec<u16> = if units.len() < count {
        units.iter().copied().chain(std::iter::once(0)).collect()
    } else {
        units.iter().copied().take(count).collect()
    };
    for (index, unit) in write_units.into_iter().enumerate() {
        let addr = dest.wrapping_add(index as u32 * 2);
        if !write_guest_u16(kernel, memory, thread_id, addr, unit) {
            return u32::MAX;
        }
    }
    if units.len() < count {
        units.len() as u32
    } else {
        u32::MAX
    }
}

fn read_delimiter_set<M: CoredllGuestMemory>(memory: &M, ptr: u32) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    for index in 0..256u32 {
        let byte = memory.read_u8(ptr.wrapping_add(index)).ok()?;
        if byte == 0 {
            return Some(bytes);
        }
        bytes.push(byte);
    }
    None
}

fn parse_decimal_prefix(text: &str) -> i32 {
    let mut chars = text.chars();
    let mut sign = 1_i64;
    let mut value = 0_i64;
    match chars.clone().next() {
        Some('-') => {
            sign = -1;
            chars.next();
        }
        Some('+') => {
            chars.next();
        }
        _ => {}
    }
    let mut saw_digit = false;
    for ch in chars {
        let Some(digit) = ch.to_digit(10) else {
            break;
        };
        saw_digit = true;
        value = value.saturating_mul(10).saturating_add(i64::from(digit));
    }
    if !saw_digit {
        return 0;
    }
    value
        .saturating_mul(sign)
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn parse_float_prefix(text: &str) -> f64 {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    let start = index;
    if matches!(bytes.get(index), Some(b'+' | b'-')) {
        index += 1;
    }

    let integer_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }
    let integer_digits = index.saturating_sub(integer_start);

    let mut fraction_digits = 0usize;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        fraction_digits = index.saturating_sub(fraction_start);
    }

    if integer_digits == 0 && fraction_digits == 0 {
        return 0.0;
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_marker = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == exponent_start {
            index = exponent_marker;
        }
    }

    text.get(start..index)
        .and_then(|prefix| prefix.parse::<f64>().ok())
        .unwrap_or(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedUnsigned {
    value: u32,
    consumed: usize,
}

fn parse_unsigned_prefix(text: &str, requested_base: u32) -> ParsedUnsigned {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    let number_start = index;
    let negative = match bytes.get(index) {
        Some(b'-') => {
            index += 1;
            true
        }
        Some(b'+') => {
            index += 1;
            false
        }
        _ => false,
    };

    let mut base = requested_base;
    if base != 0 && !(2..=36).contains(&base) {
        return ParsedUnsigned {
            value: 0,
            consumed: number_start,
        };
    }
    if base == 0 {
        if bytes.get(index) == Some(&b'0')
            && matches!(bytes.get(index + 1), Some(b'x' | b'X'))
            && bytes
                .get(index + 2)
                .and_then(|byte| (*byte as char).to_digit(16))
                .is_some()
        {
            base = 16;
            index += 2;
        } else if bytes.get(index) == Some(&b'0') {
            base = 8;
        } else {
            base = 10;
        }
    } else if base == 16
        && bytes.get(index) == Some(&b'0')
        && matches!(bytes.get(index + 1), Some(b'x' | b'X'))
        && bytes
            .get(index + 2)
            .and_then(|byte| (*byte as char).to_digit(16))
            .is_some()
    {
        index += 2;
    }

    let digits_start = index;
    let mut value = 0u64;
    while let Some(digit) = bytes
        .get(index)
        .and_then(|byte| (*byte as char).to_digit(base))
    {
        value = value
            .saturating_mul(u64::from(base))
            .saturating_add(u64::from(digit))
            .min(u64::from(u32::MAX));
        index += 1;
    }

    if index == digits_start {
        return ParsedUnsigned {
            value: 0,
            consumed: number_start,
        };
    }
    let value = value as u32;
    ParsedUnsigned {
        value: if negative {
            value.wrapping_neg()
        } else {
            value
        },
        consumed: index,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WideStringMode {
    WideDefault,
    NarrowDefault,
}

fn format_wide_printf<M: CoredllGuestMemory>(
    memory: &M,
    format: &str,
    args: &[u32],
    string_mode: WideStringMode,
) -> String {
    let mut output = String::new();
    let mut chars = format.chars().peekable();
    let mut arg_index = 0usize;
    while let Some(ch) = chars.next() {
        if ch != '%' {
            output.push(ch);
            continue;
        }
        if chars.peek() == Some(&'%') {
            chars.next();
            output.push('%');
            continue;
        }

        let mut zero_pad = false;
        let mut left_align = false;
        while matches!(chars.peek(), Some('0' | '-' | '+' | '#' | ' ')) {
            match chars.next() {
                Some('0') => zero_pad = true,
                Some('-') => left_align = true,
                _ => {}
            }
        }
        let mut width = 0usize;
        if chars.peek() == Some(&'*') {
            chars.next();
            let raw_width = args.get(arg_index).copied().unwrap_or(0) as i32;
            arg_index = arg_index.saturating_add(1);
            if raw_width < 0 {
                left_align = true;
                width = raw_width.unsigned_abs() as usize;
            } else {
                width = raw_width as usize;
            }
        } else {
            while let Some(digit) = chars.peek().and_then(|ch| ch.to_digit(10)) {
                chars.next();
                width = width.saturating_mul(10).saturating_add(digit as usize);
            }
        }
        if chars.peek() == Some(&'.') {
            chars.next();
            if chars.peek() == Some(&'*') {
                chars.next();
                arg_index = arg_index.saturating_add(1);
            } else {
                while chars.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                    chars.next();
                }
            }
        }

        let mut long_count = 0usize;
        let mut short_count = 0usize;
        while matches!(chars.peek(), Some('h' | 'l' | 'L' | 'w')) {
            match chars.next() {
                Some('l') | Some('w') | Some('L') => long_count += 1,
                Some('h') => short_count += 1,
                _ => {}
            }
        }
        if chars.peek() == Some(&'I') {
            chars.next();
            if chars.peek() == Some(&'3') {
                chars.next();
                if chars.peek() == Some(&'2') {
                    chars.next();
                }
            } else if chars.peek() == Some(&'6') {
                chars.next();
                if chars.peek() == Some(&'4') {
                    chars.next();
                }
            }
        }

        let Some(spec) = chars.next() else {
            output.push('%');
            break;
        };
        let value = args.get(arg_index).copied().unwrap_or(0);
        arg_index = arg_index.saturating_add(1);
        match spec {
            'p' => output.push_str(&format!("{value:08x}")),
            'x' => push_padded(
                &mut output,
                format!("{value:x}"),
                width,
                zero_pad,
                left_align,
            ),
            'X' => push_padded(
                &mut output,
                format!("{value:X}"),
                width,
                zero_pad,
                left_align,
            ),
            'u' => push_padded(&mut output, value.to_string(), width, zero_pad, left_align),
            'd' | 'i' => push_padded(
                &mut output,
                (value as i32).to_string(),
                width,
                zero_pad,
                left_align,
            ),
            'c' | 'C' => {
                if let Some(ch) = char::from_u32(value & 0xffff) {
                    output.push(ch);
                }
            }
            's' | 'S' => {
                if value != 0 {
                    let read_wide = if spec == 'S' {
                        string_mode == WideStringMode::NarrowDefault
                    } else if short_count != 0 {
                        false
                    } else if long_count != 0 {
                        true
                    } else {
                        string_mode == WideStringMode::WideDefault
                    };
                    let text = if read_wide {
                        read_wide_z(memory, value, 4096)
                    } else {
                        read_narrow_z(memory, value, 4096)
                    };
                    if let Some(text) = text {
                        output.push_str(&text);
                    }
                } else {
                    output.push_str("(null)");
                }
            }
            _ => {
                output.push('%');
                for _ in 0..long_count {
                    output.push('l');
                }
                output.push(spec);
            }
        }
    }
    output
}

fn push_padded(output: &mut String, text: String, width: usize, zero_pad: bool, left_align: bool) {
    let len = text.chars().count();
    if width <= len {
        output.push_str(&text);
        return;
    }

    let pad_len = width - len;
    if left_align {
        output.push_str(&text);
        output.extend(std::iter::repeat_n(' ', pad_len));
    } else if zero_pad {
        if let Some(sign @ ('-' | '+')) = text.chars().next() {
            output.push(sign);
            output.extend(std::iter::repeat_n('0', pad_len));
            output.push_str(&text[sign.len_utf8()..]);
        } else {
            output.extend(std::iter::repeat_n('0', pad_len));
            output.push_str(&text);
        }
    } else {
        output.extend(std::iter::repeat_n(' ', pad_len));
        output.push_str(&text);
    }
}

#[cfg(test)]
mod tests {
    use super::ce_wcsncpy_units;

    #[test]
    fn wcsncpy_uses_ce_byte_counts() {
        assert_eq!(ce_wcsncpy_units(1), 1);
        assert_eq!(ce_wcsncpy_units(2), 1);
        assert_eq!(ce_wcsncpy_units(34), 17);
        assert_eq!(ce_wcsncpy_units(260), 130);
    }
}
