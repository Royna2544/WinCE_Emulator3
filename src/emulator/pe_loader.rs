// PE image loading: maps the main EXE and its DLLs, assigns import traps, patches trampolines.
// The free functions here return a `PeLoadResult` that callers apply to their backend state.

use std::collections::HashMap;

#[cfg(feature = "unicorn")]
use crate::emulator::cpu_mips::{MipsTrampolineJump, patch_mips_unicorn_trampolines};
use crate::{
    emulator::{
        imports::{
            DYNAMIC_COREDLL_PROC_TRAP_BASE, ExternalImportTable, IMPORT_TRAP_BASE,
            IMPORT_TRAP_PAGE_SIZE, ImportTrapTable, import_trap_code_page, patch_external_imports,
            patch_pe_coredll_imports, patch_pe_imports,
        },
        memory::MemoryPerms,
    },
    error::{Error, Result},
    pe::PeImage,
};

// ── Structs shared between loader and runtime ────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct MappedBlob {
    pub(crate) name: String,
    pub(crate) base: u32,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "unicorn"), allow(dead_code))]
pub(crate) struct MappedResourceString {
    pub(crate) module: u32,
    pub(crate) id: u32,
    pub(crate) text: String,
    pub(crate) data_ptr: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MappedResource {
    pub(crate) module: u32,
    pub(crate) name: u32,
    pub(crate) name_string: Option<String>,
    pub(crate) kind: u32,
    pub(crate) data_ptr: u32,
    pub(crate) size: u32,
}

// ── Constants ────────────────────────────────────────────────────────────────

pub(crate) const USER_KDATA_PAGE_BASE: u32 = 0x0000_5000;
pub(crate) const USER_KDATA_PAGE_SIZE: u32 = 0x0000_1000;
pub(crate) const USER_KDATA_BASE: u32 = 0x0000_5800;
const USER_KDATA_SYSHANDLE_OFFSET: u32 = 0x0000_0004;
pub(crate) const SYS_HANDLE_CURRENT_THREAD: usize = 1;
pub(crate) const SYS_HANDLE_CURRENT_PROCESS: usize = 2;

pub(crate) const GUEST_STACK_MIN_RESERVE: u32 = 0x0040_0000;
pub(crate) const GUEST_HEAP_ARENA_BASE: u32 = 0x3000_0000;
pub(crate) const GUEST_HEAP_ARENA_SIZE: u32 = 0x0100_0000;
#[cfg(feature = "unicorn")]
pub(crate) const EXTERNAL_TRAMPOLINE_BASE: u32 = 0x7000_0000;

pub(crate) const RESERVED_IMPORT_TRAP_STUB_BYTES: u32 = crate::emulator::imports::IMPORT_TRAP_STRIDE
    * (6 + crate::ce::coredll::FSDMGR_FMD_CALLBACK_TRAP_COUNT);

// ── PeLoadResult ─────────────────────────────────────────────────────────────

/// Everything produced by `load_pe_image_with_dlls` that the backend must apply.
pub(crate) struct PeLoadResult {
    pub(crate) entry: u32,
    pub(crate) entry_image_base: u32,
    pub(crate) stack_top: u32,
    /// Memory regions that must be mapped (PE image, DLLs, trampolines, stack).
    pub(crate) memory_regions: Vec<(u32, u32, MemoryPerms, String)>,
    /// Shared infrastructure regions (import trap page, kdata, heap arena) that
    /// should only be mapped if not already present in the backend's memory map.
    pub(crate) shared_memory_regions: Vec<(u32, u32, MemoryPerms, String)>,
    pub(crate) mapped_blobs: Vec<MappedBlob>,
    pub(crate) import_traps: ImportTrapTable,
    pub(crate) loaded_modules: Vec<crate::emulator::types::LoadedPeModuleInfo>,
    pub(crate) resource_strings: Vec<MappedResourceString>,
    pub(crate) resources: Vec<MappedResource>,
    #[cfg(feature = "unicorn")]
    pub(crate) trampoline_ranges: Vec<(u32, u32)>,
    #[cfg(feature = "unicorn")]
    pub(crate) trampoline_jumps: Vec<MipsTrampolineJump>,
}

// ── Public loader entry points ────────────────────────────────────────────────

pub(crate) fn load_pe_image_with_dlls(image: &PeImage, dlls: &[PeImage]) -> Result<PeLoadResult> {
    let mut result = PeLoadResult {
        entry: 0,
        entry_image_base: 0,
        stack_top: 0,
        memory_regions: Vec::new(),
        shared_memory_regions: Vec::new(),
        mapped_blobs: Vec::new(),
        import_traps: ImportTrapTable::new(),
        loaded_modules: Vec::new(),
        resource_strings: Vec::new(),
        resources: Vec::new(),
        #[cfg(feature = "unicorn")]
        trampoline_ranges: Vec::new(),
        #[cfg(feature = "unicorn")]
        trampoline_jumps: Vec::new(),
    };

    let mut external = ExternalImportTable::default();
    let mut next_dll_base = 0x6000_0000u32;
    let mut next_trap_base = IMPORT_TRAP_BASE;
    #[allow(unused_mut)]
    let mut image_occupancy = image.mapped_image()?;
    #[cfg(feature = "unicorn")]
    let mut trampoline_blobs: Vec<(String, u32, Vec<u8>)> = Vec::new();
    #[cfg(feature = "unicorn")]
    let mut next_trampoline_base = EXTERNAL_TRAMPOLINE_BASE;
    #[cfg(feature = "unicorn")]
    {
        let _ =
            patch_mips_unicorn_trampolines(image, image.image_base(), &mut image_occupancy, None)?;
    }
    let image_mapped_size = align_up_4k(image_occupancy.len() as u32)?;
    let mut occupied_image_ranges = vec![(image.image_base(), image_mapped_size)];

    // Pass 1: process each DLL — compute load base, patch imports, collect traps.
    let mut loaded_dlls: Vec<(String, u32, Vec<u8>)> = Vec::new(); // (path, load_base, mapped)
    for dll in dlls {
        let dll_size = align_up_4k(dll.optional_header.size_of_image)?;
        let mut load_base = choose_dll_load_base(
            dll.image_base(),
            dll_size,
            &occupied_image_ranges,
            &mut next_dll_base,
        )?;
        let (mapped, traps, trampoline_patch, mapped_size) = loop {
            let mut mapped = dll.mapped_image_at(load_base)?;
            let traps = patch_pe_coredll_imports(
                dll,
                &mut mapped,
                &crate::ce::coredll::CoredllExportTable::default(),
                next_trap_base,
            )?;
            #[cfg(feature = "unicorn")]
            let trampoline_patch = {
                let trampoline_base = allocate_relocated_dll_base(
                    0x0010_0000,
                    &occupied_image_ranges,
                    &mut next_trampoline_base,
                )?;
                Some(patch_mips_unicorn_trampolines(
                    dll,
                    load_base,
                    &mut mapped,
                    Some(trampoline_base),
                )?)
            };
            #[cfg(not(feature = "unicorn"))]
            let trampoline_patch: Option<()> = None;
            let mapped_size = align_up_4k(mapped.len() as u32)?;
            if !range_overlaps_any(load_base, mapped_size, &occupied_image_ranges) {
                break (mapped, traps, trampoline_patch, mapped_size);
            }
            load_base = allocate_relocated_dll_base(
                mapped_size,
                &occupied_image_ranges,
                &mut next_dll_base,
            )?;
        };
        occupied_image_ranges.push((load_base, mapped_size));
        next_trap_base = advance_trap_base(next_trap_base, traps.len())?;
        result.import_traps.merge(traps);
        external.add_pe_image(module_file_name(&dll.path), dll, load_base);
        #[cfg(not(feature = "unicorn"))]
        let _ = trampoline_patch;
        #[cfg(feature = "unicorn")]
        if let Some(mut trampoline_patch) = trampoline_patch {
            if let Some(range) = trampoline_patch.range {
                result.trampoline_ranges.push(range);
                if let Some(bytes) = trampoline_patch.external_mapped.take() {
                    occupied_image_ranges.push((range.0, range.1));
                    trampoline_blobs.push((format!("trampoline:{}", dll.path), range.0, bytes));
                }
            }
            result.trampoline_jumps.extend(trampoline_patch.jumps);
        }
        result
            .loaded_modules
            .push(loaded_module_info(dll, load_base)?);
        loaded_dlls.push((dll.path.clone(), load_base, mapped));
    }

    // Pass 2: patch cross-DLL imports now that all load bases are known.
    for (path, _load_base, mapped) in &mut loaded_dlls {
        if let Some(dll) = dlls.iter().find(|dll| dll.path == *path) {
            patch_external_imports(mapped, &dll.imports, &external)?;
        }
    }

    // Main image: patch all imports (CE traps + external DLL refs).
    let mut mapped = image.mapped_image()?;
    let traps = patch_pe_imports(
        image,
        &mut mapped,
        &crate::ce::coredll::CoredllExportTable::default(),
        next_trap_base,
        &external,
    )?;
    #[cfg(feature = "unicorn")]
    {
        let trampoline_patch =
            patch_mips_unicorn_trampolines(image, image.image_base(), &mut mapped, None)?;
        if let Some(range) = trampoline_patch.range {
            result.trampoline_ranges.push(range);
        }
        result.trampoline_jumps.extend(trampoline_patch.jumps);
    }
    result.import_traps.merge(traps);

    // Memory regions: main image, DLLs, trampolines (order matches original).
    result.memory_regions.push((
        image.image_base(),
        align_up_4k(mapped.len() as u32)?,
        MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC,
        "pe-image".to_owned(),
    ));
    for (path, load_base, dll_mapped) in &loaded_dlls {
        result.memory_regions.push((
            *load_base,
            align_up_4k(dll_mapped.len() as u32)?,
            MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC,
            format!("dll:{path}"),
        ));
    }
    #[cfg(feature = "unicorn")]
    for (name, base, bytes) in &trampoline_blobs {
        result.memory_regions.push((
            *base,
            align_up_4k(bytes.len() as u32)?,
            MemoryPerms::READ | MemoryPerms::EXEC,
            name.clone(),
        ));
    }

    // Shared infrastructure regions (skip if already present in backend memory).
    result.shared_memory_regions.push((
        IMPORT_TRAP_BASE,
        IMPORT_TRAP_PAGE_SIZE,
        MemoryPerms::READ | MemoryPerms::EXEC,
        "ce-import-traps".to_owned(),
    ));
    result.shared_memory_regions.push((
        USER_KDATA_PAGE_BASE,
        USER_KDATA_PAGE_SIZE,
        MemoryPerms::READ | MemoryPerms::WRITE,
        "ce-user-kdata".to_owned(),
    ));
    result.shared_memory_regions.push((
        GUEST_HEAP_ARENA_BASE,
        GUEST_HEAP_ARENA_SIZE,
        MemoryPerms::READ | MemoryPerms::WRITE,
        "ce-heap-arena".to_owned(),
    ));

    // Guest stack.
    let stack_size = align_up_4k(
        image
            .optional_header
            .size_of_stack_reserve
            .max(GUEST_STACK_MIN_RESERVE),
    )?;
    let stack_top = IMPORT_TRAP_BASE
        .checked_sub(0x10000)
        .ok_or_else(|| Error::InvalidArgument("guest stack top underflow".to_owned()))?;
    let stack_base = stack_top
        .checked_sub(stack_size)
        .ok_or_else(|| Error::InvalidArgument("guest stack base underflow".to_owned()))?;
    result.memory_regions.push((
        stack_base,
        stack_size,
        MemoryPerms::READ | MemoryPerms::WRITE,
        "guest-stack".to_owned(),
    ));

    result.entry = image.entry_point_va();
    result.entry_image_base = image.image_base();
    result.stack_top = stack_top;

    // Blobs and resources.
    collect_resource_strings(
        image,
        image.image_base(),
        &mut result.resource_strings,
        &mut result.resources,
    )?;
    result.mapped_blobs.push(MappedBlob {
        name: format!("image:{}", image.path),
        base: image.image_base(),
        bytes: mapped,
    });
    for (path, load_base, dll_mapped) in loaded_dlls {
        if let Some(dll) = dlls.iter().find(|dll| dll.path == path) {
            collect_resource_strings(
                dll,
                load_base,
                &mut result.resource_strings,
                &mut result.resources,
            )?;
        }
        result.mapped_blobs.push(MappedBlob {
            name: format!("dll:{path}"),
            base: load_base,
            bytes: dll_mapped,
        });
    }
    #[cfg(feature = "unicorn")]
    for (name, base, bytes) in trampoline_blobs {
        result.mapped_blobs.push(MappedBlob { name, base, bytes });
    }
    result.mapped_blobs.push(MappedBlob {
        name: "user-kdata".to_owned(),
        base: USER_KDATA_PAGE_BASE,
        bytes: user_kdata_page(),
    });
    refresh_import_trap_page_blob(&mut result.mapped_blobs, &result.import_traps);

    Ok(result)
}

// ── Utility functions ─────────────────────────────────────────────────────────

pub(crate) fn align_up_4k(size: u32) -> Result<u32> {
    size.checked_add(0xfff)
        .map(|size| size & !0xfff)
        .ok_or_else(|| Error::InvalidArgument("mapping size overflow".to_owned()))
}

pub(crate) fn ranges_overlap(lhs_base: u32, lhs_size: u32, rhs_base: u32, rhs_size: u32) -> bool {
    let lhs_end = lhs_base.saturating_add(lhs_size);
    let rhs_end = rhs_base.saturating_add(rhs_size);
    lhs_base < rhs_end && rhs_base < lhs_end
}

#[cfg_attr(not(feature = "unicorn"), allow(dead_code))]
pub(crate) fn range_contains(
    outer_base: u32,
    outer_size: u32,
    inner_base: u32,
    inner_size: u32,
) -> bool {
    let outer_end = outer_base.saturating_add(outer_size);
    let inner_end = inner_base.saturating_add(inner_size);
    outer_base <= inner_base && inner_end <= outer_end
}

pub(crate) fn range_overlaps_any(base: u32, size: u32, ranges: &[(u32, u32)]) -> bool {
    ranges
        .iter()
        .any(|(other_base, other_size)| ranges_overlap(base, size, *other_base, *other_size))
}

pub(crate) fn user_kdata_handle_address(index: usize) -> u32 {
    USER_KDATA_BASE + USER_KDATA_SYSHANDLE_OFFSET + index as u32 * 4
}

pub(crate) fn user_kdata_page() -> Vec<u8> {
    let mut page = vec![0; USER_KDATA_PAGE_SIZE as usize];
    write_user_kdata_handle(&mut page, SYS_HANDLE_CURRENT_THREAD, 1);
    write_user_kdata_handle(&mut page, SYS_HANDLE_CURRENT_PROCESS, 1);
    page
}

pub(crate) fn refresh_import_trap_page_blob(
    mapped_blobs: &mut Vec<MappedBlob>,
    traps: &ImportTrapTable,
) {
    let trap_page = import_trap_code_page(traps);
    if let Some(blob) = mapped_blobs
        .iter_mut()
        .find(|blob| blob.name == "ce-import-traps")
    {
        blob.base = IMPORT_TRAP_BASE;
        blob.bytes = trap_page;
    } else {
        mapped_blobs.push(MappedBlob {
            name: "ce-import-traps".to_owned(),
            base: IMPORT_TRAP_BASE,
            bytes: trap_page,
        });
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn write_user_kdata_handle(page: &mut [u8], index: usize, value: u32) {
    let offset = user_kdata_handle_address(index).saturating_sub(USER_KDATA_PAGE_BASE) as usize;
    page[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn module_file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

pub(crate) fn loaded_module_info(
    image: &PeImage,
    load_base: u32,
) -> Result<crate::emulator::types::LoadedPeModuleInfo> {
    let mut exports_by_name = HashMap::new();
    let mut exports_by_ordinal = HashMap::new();
    let mut forwarders_by_name = HashMap::new();
    let mut forwarders_by_ordinal = HashMap::new();
    if let Some(exports) = image.exports.as_ref() {
        for export in &exports.functions {
            if export.rva == 0 {
                continue;
            }
            if let Some(forwarder) = export.forwarder.as_ref() {
                forwarders_by_ordinal.insert(export.ordinal, forwarder.clone());
                if let Some(name) = export.name.as_deref() {
                    forwarders_by_name.insert(
                        crate::ce::kernel::normalize_symbol_name(name),
                        forwarder.clone(),
                    );
                }
            } else {
                let va = load_base.wrapping_add(export.rva);
                exports_by_ordinal.insert(export.ordinal, va);
                if let Some(name) = export.name.as_deref() {
                    exports_by_name.insert(crate::ce::kernel::normalize_symbol_name(name), va);
                }
            }
        }
    }
    Ok(crate::emulator::types::LoadedPeModuleInfo {
        name: module_file_name(&image.path).to_owned(),
        base: load_base,
        guest_path: None,
        host_path: Some(std::path::PathBuf::from(&image.path)),
        image_size: image.optional_header.size_of_image,
        entry_point: load_base.wrapping_add(image.optional_header.address_of_entry_point),
        dependencies: image
            .imports
            .iter()
            .map(|descriptor| descriptor.module_name.clone())
            .collect(),
        tls_callbacks: image.tls_callback_vas(load_base)?,
        load_flags: 0,
        dynamic: false,
        exports_by_name,
        exports_by_ordinal,
        forwarders_by_name,
        forwarders_by_ordinal,
    })
}

pub(crate) fn choose_dll_load_base(
    preferred_base: u32,
    image_size: u32,
    occupied_ranges: &[(u32, u32)],
    next_dll_base: &mut u32,
) -> Result<u32> {
    if !range_overlaps_any(preferred_base, image_size, occupied_ranges) {
        return Ok(preferred_base);
    }
    allocate_relocated_dll_base(image_size, occupied_ranges, next_dll_base)
}

fn allocate_relocated_dll_base(
    image_size: u32,
    occupied_ranges: &[(u32, u32)],
    next_dll_base: &mut u32,
) -> Result<u32> {
    let mut candidate = align_up_4k(*next_dll_base)?;
    while range_overlaps_any(candidate, image_size, occupied_ranges) {
        candidate = candidate
            .checked_add(image_size)
            .and_then(|base| base.checked_add(0x0010_0000))
            .ok_or_else(|| Error::InvalidArgument("DLL load base overflow".to_owned()))?;
        candidate = align_up_4k(candidate)?;
    }
    *next_dll_base = candidate
        .checked_add(image_size)
        .and_then(|base| base.checked_add(0x0010_0000))
        .ok_or_else(|| Error::InvalidArgument("DLL load base overflow".to_owned()))?;
    Ok(candidate)
}

fn advance_trap_base(current: u32, trap_count: usize) -> Result<u32> {
    let bytes = u32::try_from(trap_count)
        .ok()
        .and_then(|count| count.checked_mul(crate::emulator::imports::IMPORT_TRAP_STRIDE))
        .ok_or_else(|| Error::InvalidArgument("import trap count overflow".to_owned()))?;
    let next = current
        .checked_add(bytes)
        .ok_or_else(|| Error::InvalidArgument("import trap base overflow".to_owned()))?;
    if next >= DYNAMIC_COREDLL_PROC_TRAP_BASE.saturating_sub(RESERVED_IMPORT_TRAP_STUB_BYTES) {
        return Err(Error::InvalidArgument(
            "import trap page is full".to_owned(),
        ));
    }
    Ok(next)
}

fn collect_resource_strings(
    image: &PeImage,
    load_base: u32,
    strings: &mut Vec<MappedResourceString>,
    resources: &mut Vec<MappedResource>,
) -> Result<()> {
    for string in image.resource_strings()? {
        strings.push(MappedResourceString {
            module: load_base,
            id: string.id,
            text: string.text,
            data_ptr: Some(load_base.wrapping_add(string.data_rva)),
        });
    }
    for resource in image.resource_data_entries()? {
        resources.push(MappedResource {
            module: load_base,
            name: resource.name,
            name_string: resource.name_string,
            kind: resource.kind,
            data_ptr: load_base.wrapping_add(resource.data_rva),
            size: resource.size,
        });
    }
    Ok(())
}
