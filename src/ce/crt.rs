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
    if memory.fill_bytes(dest, value as u8, len).is_ok() {
        dest
    } else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        0
    }
}

pub(crate) fn printf_family_raw(_kernel: &mut CeKernel, _thread_id: u32) -> u32 {
    0
}

pub(crate) fn wsprintf_w_raw<M: CoredllGuestMemory>(
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
    let Some(format) = read_wide_z(memory, format, 4096) else {
        kernel
            .threads
            .set_last_error(thread_id, ERROR_INVALID_PARAMETER);
        return 0;
    };
    let text = format_wide_printf(memory, &format, args);
    for (index, unit) in text.encode_utf16().chain(std::iter::once(0)).enumerate() {
        let addr = dest.wrapping_add(index as u32 * 2);
        if !write_guest_u16(kernel, memory, thread_id, addr, unit) {
            return 0;
        }
    }
    text.encode_utf16().count() as u32
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

fn format_wide_printf<M: CoredllGuestMemory>(memory: &M, format: &str, args: &[u32]) -> String {
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

        while matches!(
            chars.peek(),
            Some('0'..='9' | '-' | '+' | '#' | ' ' | '.' | '*')
        ) {
            if chars.next() == Some('*') {
                arg_index = arg_index.saturating_add(1);
            }
        }

        let mut long_count = 0usize;
        while matches!(chars.peek(), Some('h' | 'l' | 'L' | 'w')) {
            if chars.next() == Some('l') {
                long_count += 1;
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
            'x' => output.push_str(&format!("{value:x}")),
            'X' => output.push_str(&format!("{value:X}")),
            'u' => output.push_str(&value.to_string()),
            'd' | 'i' => output.push_str(&(value as i32).to_string()),
            'c' | 'C' => {
                if let Some(ch) = char::from_u32(value & 0xffff) {
                    output.push(ch);
                }
            }
            's' | 'S' => {
                if value != 0 {
                    if let Some(text) = read_wide_z(memory, value, 4096) {
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
