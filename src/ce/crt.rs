use crate::ce::{
    coredll::{CoredllGuestMemory, read_guest_bytes, write_guest_bytes, write_guest_u16},
    file::{CREATE_ALWAYS, GENERIC_READ, GENERIC_WRITE, OPEN_ALWAYS, OPEN_EXISTING},
    kernel::CeKernel,
    thread::{ERROR_FILE_NOT_FOUND, ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER},
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
    let mut bytes = text.into_bytes();
    let len = bytes.len() as u32;
    bytes.push(0);
    if write_guest_bytes(kernel, memory, thread_id, dest, &bytes) {
        len
    } else {
        0
    }
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
            return Some(String::from_utf8_lossy(&bytes).into_owned());
        }
        bytes.push(byte);
    }
    None
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

        while matches!(
            chars.peek(),
            Some('0'..='9' | '-' | '+' | '#' | ' ' | '.' | '*')
        ) {
            if chars.next() == Some('*') {
                arg_index = arg_index.saturating_add(1);
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
