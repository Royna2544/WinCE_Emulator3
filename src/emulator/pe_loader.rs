// PE image loading: maps the main EXE and its DLLs, assigns import traps, patches trampolines.
// The free functions here return a `PeLoadResult` that callers apply to their backend state.

use std::collections::HashMap;
use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

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
    pe::{PeImage, SectionHeader},
};

// ── Structs shared between loader and runtime ────────────────────────────────

/// Host-RW, page-aligned final image bytes. Rust never exposes a mutable slice
/// after construction; Unicorn is given this pointer with guest R/RX access.
#[derive(Debug)]
pub(crate) struct SealedImageBytes {
    ptr: NonNull<u8>,
    len: usize,
    generation: u64,
}

impl SealedImageBytes {
    fn new(bytes: &[u8], generation: u64) -> Result<Self> {
        if bytes.is_empty() || bytes.len() & (IMAGE_PAGE_SIZE as usize - 1) != 0 {
            return Err(Error::InvalidArgument(
                "sealed image run must be non-empty and page-sized".to_owned(),
            ));
        }
        let layout = std::alloc::Layout::from_size_align(bytes.len(), IMAGE_PAGE_SIZE as usize)
            .map_err(|_| Error::InvalidArgument("sealed image allocation overflow".to_owned()))?;
        // SAFETY: `layout` has nonzero size and valid 4-KiB alignment. The
        // allocation remains owned until all Rc pins have been released.
        let ptr = unsafe { std::alloc::alloc(layout) };
        let ptr = NonNull::new(ptr)
            .ok_or_else(|| Error::OutOfStructures("sealed image allocation failed".to_owned()))?;
        // SAFETY: the destination is newly allocated for exactly bytes.len()
        // bytes and does not overlap the final image source.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), bytes.len()) };
        Ok(Self {
            ptr,
            len: bytes.len(),
            generation,
        })
    }

    pub(crate) fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

impl Deref for SealedImageBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        // SAFETY: the allocation is initialized in `new`, stays alive while
        // any Rc exists, and no Rust mutable reference is ever exposed.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for SealedImageBytes {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::from_size_align(self.len, IMAGE_PAGE_SIZE as usize)
            .expect("sealed image layout was checked at construction");
        // SAFETY: this is the same allocation and layout created in `new`.
        // Native owners must release their Rc pins before dropping this owner.
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), layout) };
    }
}

#[derive(Debug, Clone)]
pub(crate) enum BlobBytes {
    Owned(Vec<u8>),
    Sealed(Rc<SealedImageBytes>),
}

impl BlobBytes {
    /// Mutable image access is explicit; a sealed run has no Rust write API.
    pub(crate) fn as_mut_owned(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            Self::Owned(bytes) => Some(bytes),
            Self::Sealed(_) => None,
        }
    }

    pub(crate) fn sealed_pin(&self) -> Option<Rc<SealedImageBytes>> {
        match self {
            Self::Sealed(bytes) => Some(Rc::clone(bytes)),
            Self::Owned(_) => None,
        }
    }
}

impl From<Vec<u8>> for BlobBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Owned(bytes)
    }
}

impl Deref for BlobBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Sealed(bytes) => bytes,
        }
    }
}

impl PartialEq for BlobBytes {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl Eq for BlobBytes {}

impl PartialEq<Vec<u8>> for BlobBytes {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.deref() == other.as_slice()
    }
}

impl PartialEq<BlobBytes> for Vec<u8> {
    fn eq(&self, other: &BlobBytes) -> bool {
        self.as_slice() == other.deref()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MappedBlob {
    pub(crate) name: String,
    pub(crate) base: u32,
    pub(crate) bytes: BlobBytes,
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
    // Per-256MB-region stub windows: a stub in the same region as its origin
    // lets the patcher emit an architecturally self-contained `j stub` at the
    // patched site instead of the hook-interpreted lui/ori sentinel.
    #[cfg(feature = "unicorn")]
    let mut trampoline_base_by_region: HashMap<u32, u32> = HashMap::new();
    #[cfg(feature = "unicorn")]
    {
        let _ =
            patch_mips_unicorn_trampolines(image, image.image_base(), &mut image_occupancy, None)?;
    }
    let image_mapped_size = align_up_4k(image_occupancy.len() as u32)?;
    drop(image_occupancy);
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
                let region = load_base & 0xf000_0000;
                let region_cursor = trampoline_base_by_region.entry(region).or_insert(
                    if region == EXTERNAL_TRAMPOLINE_BASE & 0xf000_0000 {
                        EXTERNAL_TRAMPOLINE_BASE
                    } else {
                        region | 0x0f00_0000
                    },
                );
                let mut trampoline_base = allocate_relocated_dll_base(
                    0x0010_0000,
                    &occupied_image_ranges,
                    region_cursor,
                )?;
                if trampoline_base & 0xf000_0000 != region {
                    // The region-local window is exhausted; fall back to the
                    // legacy external window. Those sites keep the
                    // hook-interpreted sentinel form.
                    trampoline_base = allocate_relocated_dll_base(
                        0x0010_0000,
                        &occupied_image_ranges,
                        &mut next_trampoline_base,
                    )?;
                }
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

    // The final main-image bytes have now passed every loader patcher. The
    // patched branch origins are final executable bytes, not future writes;
    // their stub tails live beyond SizeOfImage. Split only this image; DLLs
    // and runtime import/trap pages remain copy-backed.
    let main_image_runs = final_main_image_runs(image, mapped)?;

    // Memory regions: main image runs, DLLs, trampolines (same address order).
    for (blob, perms) in &main_image_runs {
        result.memory_regions.push((
            blob.base,
            u32::try_from(blob.bytes.len()).map_err(|_| {
                Error::InvalidArgument("main image run exceeds guest address width".to_owned())
            })?,
            *perms,
            "pe-image".to_owned(),
        ));
    }
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
    result
        .mapped_blobs
        .extend(main_image_runs.into_iter().map(|(blob, _)| blob));
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
            bytes: dll_mapped.into(),
        });
    }
    #[cfg(feature = "unicorn")]
    for (name, base, bytes) in trampoline_blobs {
        result.mapped_blobs.push(MappedBlob {
            name,
            base,
            bytes: bytes.into(),
        });
    }
    result.mapped_blobs.push(MappedBlob {
        name: "user-kdata".to_owned(),
        base: USER_KDATA_PAGE_BASE,
        bytes: user_kdata_page().into(),
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

const IMAGE_PAGE_SIZE: u32 = 0x1000;
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
const IMAGE_SCN_MEM_READ: u32 = 0x4000_0000;
const IMAGE_SCN_MEM_WRITE: u32 = 0x8000_0000;

/// A complete image page borrowing the final, already-patched loader bytes.
/// Selection does not allocate or copy a second image buffer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ImmutableImagePage<'a> {
    pub(crate) rva: u32,
    pub(crate) perms: MemoryPerms,
    pub(crate) bytes: &'a [u8],
}

/// Identify pages that may be sealed and mapped as immutable image bytes.
/// All ranges are image-relative, half-open `(rva, size)` pairs. A page must
/// be fully inside exactly one non-writable, readable section, with no header
/// or patch-range overlap. Publication and native mapping happen later.
pub(crate) fn eligible_immutable_image_pages<'a>(
    size_of_image: u32,
    size_of_headers: u32,
    sections: &[SectionHeader],
    patched_image: &'a [u8],
    excluded_rva_ranges: &[(u32, u32)],
) -> Result<Vec<ImmutableImagePage<'a>>> {
    let image_len = usize::try_from(size_of_image)
        .map_err(|_| Error::InvalidArgument("image size does not fit host usize".to_owned()))?;
    if size_of_headers > size_of_image || patched_image.len() < image_len {
        return Err(Error::InvalidArgument(
            "immutable image layout exceeds final mapped bytes".to_owned(),
        ));
    }

    let mut section_spans = Vec::with_capacity(sections.len());
    for section in sections {
        let end = section
            .virtual_address
            .checked_add(section.virtual_size.max(section.size_of_raw_data))
            .ok_or_else(|| Error::InvalidArgument("image section RVA overflow".to_owned()))?;
        if end > size_of_image {
            return Err(Error::InvalidArgument(
                "image section exceeds SizeOfImage".to_owned(),
            ));
        }
        section_spans.push((section.virtual_address, end, section.characteristics));
    }

    let mut excluded_spans = Vec::with_capacity(excluded_rva_ranges.len());
    for &(start, size) in excluded_rva_ranges {
        let end = start
            .checked_add(size)
            .ok_or_else(|| Error::InvalidArgument("image patch RVA overflow".to_owned()))?;
        if end > size_of_image {
            return Err(Error::InvalidArgument(
                "image patch range exceeds SizeOfImage".to_owned(),
            ));
        }
        if size != 0 {
            excluded_spans.push((start, end));
        }
    }

    let mut pages = Vec::new();
    for index in 0..size_of_image / IMAGE_PAGE_SIZE {
        let page_rva = index
            .checked_mul(IMAGE_PAGE_SIZE)
            .ok_or_else(|| Error::InvalidArgument("image page RVA overflow".to_owned()))?;
        let page_end = page_rva
            .checked_add(IMAGE_PAGE_SIZE)
            .ok_or_else(|| Error::InvalidArgument("image page end overflow".to_owned()))?;
        if page_rva < size_of_headers
            || excluded_spans
                .iter()
                .any(|&(start, end)| page_rva < end && start < page_end)
        {
            continue;
        }

        let mut overlapping_sections = 0;
        let mut perms = None;
        for &(start, end, characteristics) in &section_spans {
            if page_rva < end && start < page_end {
                overlapping_sections += 1;
                if overlapping_sections > 1 {
                    perms = None;
                    break;
                }
                if start <= page_rva
                    && page_end <= end
                    && characteristics & IMAGE_SCN_MEM_READ != 0
                    && characteristics & IMAGE_SCN_MEM_WRITE == 0
                {
                    let mut selected = MemoryPerms::READ;
                    if characteristics & IMAGE_SCN_MEM_EXECUTE != 0 {
                        selected |= MemoryPerms::EXEC;
                    }
                    perms = Some(selected);
                }
            }
        }
        if let Some(perms) = perms {
            let start = usize::try_from(page_rva).map_err(|_| {
                Error::InvalidArgument("image page offset does not fit host usize".to_owned())
            })?;
            let end = usize::try_from(page_end).map_err(|_| {
                Error::InvalidArgument("image page end does not fit host usize".to_owned())
            })?;
            let bytes = patched_image.get(start..end).ok_or_else(|| {
                Error::InvalidArgument("immutable image page exceeds final bytes".to_owned())
            })?;
            pages.push(ImmutableImagePage {
                rva: page_rva,
                perms,
                bytes,
            });
        }
    }
    Ok(pages)
}

static NEXT_IMAGE_LOAD_GENERATION: AtomicU64 = AtomicU64::new(1);

fn next_image_load_generation() -> Result<u64> {
    NEXT_IMAGE_LOAD_GENERATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .map_err(|_| Error::OutOfStructures("image load generation exhausted".to_owned()))
}

/// Consume the fully patched main image into contiguous, page-aligned runs.
/// IAT thunks remain copy-backed because import binding can change after
/// publication. One-time loader relocations and trampoline-origin rewrites
/// are already final bytes at this boundary and may be sealed.
fn final_main_image_runs(
    image: &PeImage,
    mapped: Vec<u8>,
) -> Result<Vec<(MappedBlob, MemoryPerms)>> {
    let mut excluded = Vec::new();
    for descriptor in &image.imports {
        for thunk in &descriptor.imports {
            excluded.push((thunk.iat_rva, 4));
        }
    }
    let selected = eligible_immutable_image_pages(
        image.optional_header.size_of_image,
        image.optional_header.size_of_headers,
        &image.sections,
        &mapped,
        &excluded,
    )?;
    let eligible = selected
        .into_iter()
        .map(|page| (page.rva, page.perms))
        .collect::<Vec<_>>();
    partition_main_image_runs(&image.path, image.image_base(), mapped, &eligible)
}

fn partition_main_image_runs(
    path: &str,
    base: u32,
    mut mapped: Vec<u8>,
    eligible: &[(u32, MemoryPerms)],
) -> Result<Vec<(MappedBlob, MemoryPerms)>> {
    let mapped_len = u32::try_from(mapped.len())
        .map_err(|_| Error::InvalidArgument("main image exceeds guest address width".to_owned()))?;
    let aligned_len = align_up_4k(mapped_len)? as usize;
    mapped.resize(aligned_len, 0);
    let mut page_perms = vec![None; aligned_len / IMAGE_PAGE_SIZE as usize];
    for &(rva, perms) in eligible {
        let page_index = usize::try_from(rva / IMAGE_PAGE_SIZE).map_err(|_| {
            Error::InvalidArgument("main image page index exceeds host usize".to_owned())
        })?;
        if rva & (IMAGE_PAGE_SIZE - 1) != 0
            || perms.contains(MemoryPerms::WRITE)
            || !perms.contains(MemoryPerms::READ)
            || page_perms.get(page_index) != Some(&None)
        {
            return Err(Error::InvalidArgument(
                "invalid or duplicate immutable main image page".to_owned(),
            ));
        }
        page_perms[page_index] = Some(perms);
    }

    let generation = next_image_load_generation()?;
    let mut bytes = mapped.into_iter();
    let mut runs = Vec::new();
    let mut page_index = 0;
    while page_index < page_perms.len() {
        let start_page = page_index;
        let sealed_perms = page_perms[page_index];
        page_index += 1;
        while page_index < page_perms.len() && page_perms[page_index] == sealed_perms {
            page_index += 1;
        }
        let start = start_page * IMAGE_PAGE_SIZE as usize;
        let size = (page_index - start_page) * IMAGE_PAGE_SIZE as usize;
        let run_base =
            base.checked_add(u32::try_from(start).map_err(|_| {
                Error::InvalidArgument("main image run offset overflow".to_owned())
            })?)
            .ok_or_else(|| Error::InvalidArgument("main image run address overflow".to_owned()))?;
        let (bytes, perms) = match sealed_perms {
            Some(perms) => {
                let source = bytes.as_slice().get(..size).ok_or_else(|| {
                    Error::InvalidArgument("main image run lost final bytes".to_owned())
                })?;
                let sealed = Rc::new(SealedImageBytes::new(source, generation)?);
                // Vec::IntoIter::nth advances over this run without creating
                // a second temporary Vec of the same sealed bytes.
                bytes.nth(size - 1).ok_or_else(|| {
                    Error::InvalidArgument("main image run lost final bytes".to_owned())
                })?;
                (BlobBytes::Sealed(sealed), perms)
            }
            None => {
                let run_bytes: Vec<u8> = bytes.by_ref().take(size).collect();
                if run_bytes.len() != size {
                    return Err(Error::InvalidArgument(
                        "main image run lost final bytes".to_owned(),
                    ));
                }
                (
                    BlobBytes::Owned(run_bytes),
                    MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC,
                )
            }
        };
        runs.push((
            MappedBlob {
                name: format!("image:{path}"),
                base: run_base,
                bytes,
            },
            perms,
        ));
    }
    Ok(runs)
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
        blob.bytes = trap_page.into();
    } else {
        mapped_blobs.push(MappedBlob {
            name: "ce-import-traps".to_owned(),
            base: IMPORT_TRAP_BASE,
            bytes: trap_page.into(),
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

#[cfg(test)]
mod immutable_image_page_tests {
    use super::*;
    use crate::pe::SectionHeader;
    #[cfg(feature = "unicorn")]
    use crate::pe::{ImportBy, ImportDescriptor, ImportThunk};

    const SCN_EXECUTE: u32 = 0x2000_0000;
    const SCN_READ: u32 = 0x4000_0000;
    const SCN_WRITE: u32 = 0x8000_0000;

    fn section(name: &str, rva: u32, virtual_size: u32, characteristics: u32) -> SectionHeader {
        SectionHeader {
            name: name.to_owned(),
            virtual_size,
            virtual_address: rva,
            size_of_raw_data: 0,
            pointer_to_raw_data: 0,
            pointer_to_relocations: 0,
            pointer_to_linenumbers: 0,
            number_of_relocations: 0,
            number_of_linenumbers: 0,
            characteristics,
        }
    }

    #[test]
    fn immutable_image_page_selects_only_final_patched_readonly_bytes() {
        let mut patched = vec![0x5a; 0x6000];
        patched[0x1037] = 0xe8;
        let sections = [
            section(".text", 0x1000, 0x2000, SCN_READ | SCN_EXECUTE),
            section(".rdata", 0x3000, 0x1000, SCN_READ),
            section(".data", 0x4000, 0x1000, SCN_READ | SCN_WRITE),
        ];
        let pages =
            eligible_immutable_image_pages(0x6000, 0x1000, &sections, &patched, &[(0x2001, 4)])
                .unwrap();

        assert_eq!(
            pages.iter().map(|page| page.rva).collect::<Vec<_>>(),
            [0x1000, 0x3000]
        );
        assert_eq!(pages[0].perms, MemoryPerms::READ | MemoryPerms::EXEC);
        assert_eq!(pages[1].perms, MemoryPerms::READ);
        assert_eq!(pages[0].bytes[0x37], 0xe8);
        assert!(std::ptr::eq(
            pages[0].bytes.as_ptr(),
            patched[0x1000..].as_ptr()
        ));
    }

    #[test]
    fn immutable_image_page_uses_characteristics_not_section_names() {
        let patched = vec![0; 0x5000];
        let sections = [
            section(".text", 0x1000, 0x1000, SCN_READ | SCN_EXECUTE | SCN_WRITE),
            section(".data", 0x2000, 0x1000, SCN_READ | SCN_EXECUTE),
            section("execute-only", 0x3000, 0x1000, SCN_EXECUTE),
        ];
        let pages =
            eligible_immutable_image_pages(0x5000, 0x1000, &sections, &patched, &[]).unwrap();

        assert_eq!(
            pages.iter().map(|page| page.rva).collect::<Vec<_>>(),
            [0x2000]
        );
        assert_eq!(pages[0].perms, MemoryPerms::READ | MemoryPerms::EXEC);
    }

    #[test]
    fn immutable_image_page_rejects_headers_partial_and_mixed_pages() {
        let patched = vec![0; 0x7000];
        let sections = [
            section("header-mixed", 0x1000, 0x1800, SCN_READ),
            section("overlap", 0x2800, 0x1000, SCN_READ),
            section("partial", 0x4000, 0x0800, SCN_READ),
            section("complete", 0x5000, 0x1000, SCN_READ),
        ];
        let pages =
            eligible_immutable_image_pages(0x7000, 0x1800, &sections, &patched, &[]).unwrap();

        assert_eq!(
            pages.iter().map(|page| page.rva).collect::<Vec<_>>(),
            [0x5000]
        );
    }

    #[test]
    fn immutable_image_page_fails_closed_on_overflow_or_out_of_image_input() {
        let patched = vec![0; 0x3000];
        assert!(
            eligible_immutable_image_pages(
                0x3000,
                0x1000,
                &[section("overflow", 0xffff_f000, 0x2000, SCN_READ)],
                &patched,
                &[],
            )
            .is_err()
        );
        assert!(
            eligible_immutable_image_pages(
                0x3000,
                0x1000,
                &[section("out-of-image", 0x2000, 0x2000, SCN_READ)],
                &patched,
                &[],
            )
            .is_err()
        );
        assert!(
            eligible_immutable_image_pages(
                0x3000,
                0x1000,
                &[section("valid", 0x1000, 0x1000, SCN_READ)],
                &patched,
                &[(0xffff_ffff, 2)],
            )
            .is_err()
        );
        assert!(
            eligible_immutable_image_pages(
                0x4000,
                0x1000,
                &[section("valid", 0x1000, 0x1000, SCN_READ)],
                &patched,
                &[],
            )
            .is_err()
        );
    }

    #[test]
    fn immutable_image_page_ignores_loader_trampoline_tail_past_size_of_image() {
        let mut patched_with_tail = vec![0; 0x5000];
        patched_with_tail[0x1004] = 0xa7;
        patched_with_tail[0x4004] = 0xcc;
        let pages = eligible_immutable_image_pages(
            0x3000,
            0x1000,
            &[section("code", 0x1000, 0x1000, SCN_READ | SCN_EXECUTE)],
            &patched_with_tail,
            &[],
        )
        .unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].rva, 0x1000);
        assert_eq!(pages[0].bytes[4], 0xa7);
    }

    #[test]
    fn main_image_runs_share_only_sealed_bytes_and_keep_writable_clones_independent() {
        let mut final_bytes = vec![0; 0x5000];
        for (page, chunk) in final_bytes.chunks_exact_mut(0x1000).enumerate() {
            chunk.fill(page as u8);
        }
        let runs = partition_main_image_runs(
            "synthetic.exe",
            0x0040_0000,
            final_bytes,
            &[
                (0x1000, MemoryPerms::READ | MemoryPerms::EXEC),
                (0x2000, MemoryPerms::READ | MemoryPerms::EXEC),
                (0x3000, MemoryPerms::READ),
            ],
        )
        .unwrap();
        assert_eq!(runs.len(), 4);
        assert_eq!(runs[0].0.base, 0x0040_0000);
        assert_eq!(runs[1].0.base, 0x0040_1000);
        assert_eq!(runs[1].0.bytes.len(), 0x2000);
        assert_eq!(runs[1].1, MemoryPerms::READ | MemoryPerms::EXEC);
        assert_eq!(runs[2].0.base, 0x0040_3000);
        assert_eq!(runs[2].1, MemoryPerms::READ);
        assert_eq!(runs[3].0.base, 0x0040_4000);
        assert_eq!(
            runs[3].1,
            MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC
        );
        assert_eq!(runs[1].0.bytes[0], 1);
        assert_eq!(runs[1].0.bytes[0x1000], 2);
        assert_eq!(runs[3].0.bytes[0], 4);

        let first_pin = runs[1].0.bytes.sealed_pin().unwrap();
        assert_eq!((first_pin.as_ptr() as usize) & 0xfff, 0);
        assert_eq!(first_pin.len(), 0x2000);
        assert!(runs[0].0.bytes.sealed_pin().is_none());
        let mut cloned = runs.clone();
        let second_pin = cloned[1].0.bytes.sealed_pin().unwrap();
        assert!(Rc::ptr_eq(&first_pin, &second_pin));
        assert_eq!(first_pin.generation(), second_pin.generation());
        cloned[0].0.bytes.as_mut_owned().unwrap()[0] = 0xee;
        assert_eq!(runs[0].0.bytes[0], 0);
        assert_eq!(cloned[0].0.bytes[0], 0xee);
        assert!(cloned[1].0.bytes.as_mut_owned().is_none());
    }

    #[cfg(feature = "unicorn")]
    #[test]
    fn loader_keeps_patched_rx_page_copy_backed_when_iat_shares_it() -> Result<()> {
        const ENTRY: u32 = 0x0040_1000;
        const SOURCE_BRANCH: u32 = 0x5060_0003; // beqzl v1,zero,+3
        const PATCHED_JUMP: u32 = 0x0810_0800; // j 0x0040_2000
        const IAT_RVA: u32 = 0x1020;

        fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
            bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }

        fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }

        let mut bytes = vec![0; 0x400];
        bytes[..2].copy_from_slice(b"MZ");
        put_u32(&mut bytes, 0x3c, 0x80);
        bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
        let coff = 0x84;
        put_u16(&mut bytes, coff, 0x0166); // MIPS R4000
        put_u16(&mut bytes, coff + 2, 1); // one section
        put_u16(&mut bytes, coff + 16, 0x00e0);
        put_u16(&mut bytes, coff + 18, 0x0102);
        let optional = 0x98;
        put_u16(&mut bytes, optional, 0x010b); // PE32
        put_u32(&mut bytes, optional + 4, 0x200); // SizeOfCode
        put_u32(&mut bytes, optional + 16, 0x1000); // entry RVA
        put_u32(&mut bytes, optional + 20, 0x1000); // BaseOfCode
        put_u32(&mut bytes, optional + 28, 0x0040_0000); // ImageBase
        put_u32(&mut bytes, optional + 32, 0x1000); // SectionAlignment
        put_u32(&mut bytes, optional + 36, 0x200); // FileAlignment
        put_u32(&mut bytes, optional + 56, 0x2000); // SizeOfImage
        put_u32(&mut bytes, optional + 60, 0x200); // SizeOfHeaders
        put_u16(&mut bytes, optional + 68, 9); // Windows CE subsystem
        put_u32(&mut bytes, optional + 72, 0x10000); // stack reserve
        put_u32(&mut bytes, optional + 76, 0x1000); // stack commit
        put_u32(&mut bytes, optional + 80, 0x10000); // heap reserve
        put_u32(&mut bytes, optional + 84, 0x1000); // heap commit
        put_u32(&mut bytes, optional + 92, 16); // directory count
        let section = 0x178;
        bytes[section..section + 8].copy_from_slice(b".text\0\0\0");
        put_u32(&mut bytes, section + 8, 0x1000); // full RX page
        put_u32(&mut bytes, section + 12, 0x1000);
        put_u32(&mut bytes, section + 16, 0x200);
        put_u32(&mut bytes, section + 20, 0x200);
        put_u32(&mut bytes, section + 36, SCN_READ | SCN_EXECUTE | 0x20);
        put_u32(&mut bytes, 0x200, SOURCE_BRANCH);
        put_u32(&mut bytes, 0x204, 0x2442_0001); // addiu v0,v0,1

        let image = PeImage::parse_bytes("patched-branch-with-iat.exe", &bytes)?;
        let control = load_pe_image_with_dlls(&image, &[])?;
        let sealed_entry = control
            .mapped_blobs
            .iter()
            .find(|blob| blob.base == ENTRY)
            .expect("RX entry page without IAT is mapped");
        assert!(sealed_entry.bytes.sealed_pin().is_some());
        assert_eq!(&sealed_entry.bytes[..4], &PATCHED_JUMP.to_le_bytes());

        // The very same executable page gains one writable import thunk.
        // Both patches run first; page-level IAT exclusion must win at seal time.
        let mut imported = image.clone();
        imported.imports.push(ImportDescriptor {
            module_name: "coredll.dll".to_owned(),
            original_first_thunk: IAT_RVA,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0,
            first_thunk: IAT_RVA,
            imports: vec![ImportThunk {
                thunk_rva: IAT_RVA,
                iat_rva: IAT_RVA,
                import: ImportBy::Name {
                    hint: 0,
                    name: "CreateFileW".to_owned(),
                },
            }],
        });
        let result = load_pe_image_with_dlls(&imported, &[])?;
        assert_eq!(result.import_traps.len(), 1);
        let image_name = format!("image:{}", imported.path);
        let entry_blob = result
            .mapped_blobs
            .iter()
            .find(|blob| {
                blob.name == image_name
                    && blob.base <= ENTRY
                    && u64::from(ENTRY - blob.base) + 0x24 <= blob.bytes.len() as u64
            })
            .expect("patched entry and IAT share one mapped blob");
        assert!(entry_blob.bytes.sealed_pin().is_none());
        let entry_offset = (ENTRY - entry_blob.base) as usize;
        assert_eq!(
            &entry_blob.bytes[entry_offset..entry_offset + 4],
            &PATCHED_JUMP.to_le_bytes()
        );
        assert_eq!(
            &entry_blob.bytes[entry_offset + 4..entry_offset + 8],
            &0u32.to_le_bytes()
        );
        assert_eq!(
            &entry_blob.bytes[entry_offset + 0x20..entry_offset + 0x24],
            &IMPORT_TRAP_BASE.to_le_bytes()
        );
        assert!(
            result
                .trampoline_jumps
                .iter()
                .any(|jump| jump.origin == ENTRY && jump.stub == 0x0040_2000)
        );
        Ok(())
    }

    /// Read-only, opt-in coverage diagnostic for a real PE. This calls the
    /// production loader but never constructs a CPU or executes guest code.
    #[cfg(feature = "unicorn")]
    #[test]
    #[ignore = "opt-in read-only real EXE loader coverage"]
    fn real_main_image_sealed_page_coverage() -> Result<()> {
        use crate::emulator::dll_search::{
            emulator_provided_import_module, normalize_module_name, resolve_dll_path,
        };
        use std::{
            collections::{BTreeMap, BTreeSet},
            path::PathBuf,
        };

        let exe_path =
            PathBuf::from(std::env::var_os("WINCE_COVERAGE_EXE").ok_or_else(|| {
                Error::InvalidArgument("WINCE_COVERAGE_EXE is required".to_owned())
            })?);
        let dll_dir =
            PathBuf::from(std::env::var_os("WINCE_COVERAGE_DLL_DIR").ok_or_else(|| {
                Error::InvalidArgument("WINCE_COVERAGE_DLL_DIR is required".to_owned())
            })?);
        let image = PeImage::inspect(&exe_path)?;
        let search_dirs = [dll_dir];
        let mut loaded = Vec::new();
        let mut seen = BTreeSet::new();
        for descriptor in &image.imports {
            let normalized = normalize_module_name(&descriptor.module_name);
            if emulator_provided_import_module(&normalized) || !seen.insert(normalized) {
                continue;
            }
            let path =
                resolve_dll_path(&descriptor.module_name, None, &search_dirs).ok_or_else(|| {
                    Error::MissingImportDll {
                        dll: descriptor.module_name.clone(),
                    }
                })?;
            loaded.push(PeImage::inspect(path)?);
        }
        if seen.insert(normalize_module_name("commctrl.dll"))
            && let Some(path) = resolve_dll_path("commctrl.dll", None, &search_dirs)
        {
            loaded.push(PeImage::inspect(path)?);
        }
        let parent = exe_path.parent().ok_or_else(|| {
            Error::InvalidArgument("coverage EXE has no parent directory".to_owned())
        })?;
        let mut siblings = std::fs::read_dir(parent)
            .map_err(|source| Error::Read {
                path: parent.to_path_buf(),
                source,
            })?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|source| Error::Read {
                        path: parent.to_path_buf(),
                        source,
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        siblings.sort();
        for path in siblings {
            if !path.is_file()
                || !path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
            {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    Error::InvalidArgument("coverage DLL filename is not UTF-8".to_owned())
                })?;
            let normalized = normalize_module_name(name);
            if emulator_provided_import_module(&normalized) || !seen.insert(normalized) {
                continue;
            }
            loaded.push(PeImage::inspect(path)?);
        }

        let image_size = image.optional_header.size_of_image;
        if image_size & (IMAGE_PAGE_SIZE - 1) != 0 {
            return Err(Error::InvalidArgument(
                "coverage SizeOfImage is not page-aligned".to_owned(),
            ));
        }
        let total_pages = image_size / IMAGE_PAGE_SIZE;
        let original = image.mapped_image()?;
        let candidate_pages = eligible_immutable_image_pages(
            image_size,
            image.optional_header.size_of_headers,
            &image.sections,
            &original,
            &[],
        )?;
        let non_writable_pages = candidate_pages.len();
        let result = load_pe_image_with_dlls(&image, &loaded)?;
        let main_name = format!("image:{}", image.path);
        let main_blobs: Vec<_> = result
            .mapped_blobs
            .iter()
            .filter(|blob| blob.name == main_name)
            .collect();
        let sealed_pages: usize = result
            .mapped_blobs
            .iter()
            .filter(|blob| blob.name == main_name && blob.bytes.sealed_pin().is_some())
            .map(|blob| blob.bytes.len() / IMAGE_PAGE_SIZE as usize)
            .sum();
        let excluded_ranges: Vec<(u32, u32)> = image
            .imports
            .iter()
            .flat_map(|descriptor| descriptor.imports.iter().map(|thunk| (thunk.iat_rva, 4)))
            .collect();
        let mut sealed_changed_pages = 0usize;
        let mut sealed_unchanged_pages = 0usize;
        let mut excluded_changed_pages = 0usize;
        let mut excluded_unchanged_pages = 0usize;
        for page in candidate_pages {
            let address = image
                .image_base()
                .checked_add(page.rva)
                .ok_or_else(|| Error::Backend("coverage page VA overflow".to_owned()))?;
            let blob = main_blobs
                .iter()
                .find(|blob| {
                    address >= blob.base
                        && (address - blob.base) as usize + IMAGE_PAGE_SIZE as usize
                            <= blob.bytes.len()
                })
                .ok_or_else(|| Error::Backend("coverage page has no main blob".to_owned()))?;
            let offset = (address - blob.base) as usize;
            let changed = &blob.bytes[offset..offset + IMAGE_PAGE_SIZE as usize] != page.bytes;
            let excluded = excluded_ranges.iter().any(|&(start, size)| {
                page.rva < start.saturating_add(size) && start < page.rva + IMAGE_PAGE_SIZE
            });
            match (blob.bytes.sealed_pin().is_some(), changed, excluded) {
                (true, true, false) => sealed_changed_pages += 1,
                (true, false, false) => sealed_unchanged_pages += 1,
                (false, true, true) => excluded_changed_pages += 1,
                (false, false, true) => excluded_unchanged_pages += 1,
                _ => {
                    return Err(Error::Backend(format!(
                        "coverage page at RVA 0x{:08x} violates final-byte selection",
                        page.rva
                    )));
                }
            }
        }
        if sealed_pages > non_writable_pages || sealed_pages > total_pages as usize {
            return Err(Error::Backend(
                "sealed coverage exceeds a denominator".to_owned(),
            ));
        }
        if sealed_changed_pages
            + sealed_unchanged_pages
            + excluded_changed_pages
            + excluded_unchanged_pages
            != non_writable_pages
            || sealed_pages != sealed_changed_pages + sealed_unchanged_pages
        {
            return Err(Error::Backend(
                "sealed coverage breakdown mismatch".to_owned(),
            ));
        }
        for dll in &loaded {
            eprintln!("real_main_image_coverage_dll path={}", dll.path);
        }
        eprintln!(
            "real_main_image_coverage exe={} dlls={} image_bytes={} total_pages={} non_writable_section_pages={} sealed_changed_pages={} sealed_unchanged_pages={} iat_excluded_changed_pages={} iat_excluded_unchanged_pages={} sealed_pages={} sealed_mib={:.6}",
            exe_path.display(),
            loaded.len(),
            image_size,
            total_pages,
            non_writable_pages,
            sealed_changed_pages,
            sealed_unchanged_pages,
            excluded_changed_pages,
            excluded_unchanged_pages,
            sealed_pages,
            sealed_pages as f64 * IMAGE_PAGE_SIZE as f64 / (1024.0 * 1024.0),
        );
        // This creates only the production owner-side memory/blob plan. It
        // does not construct a native CPU or execute any guest instruction.
        let mut cpu = crate::emulator::unicorn::UnicornMips::new()?;
        cpu.load_pe_image_with_dlls(&image, &loaded)?;
        let owner_blobs = cpu.mapped_blob_ranges();
        let mut families: BTreeMap<String, (usize, u64, u64)> = BTreeMap::new();
        for owner_blob in &owner_blobs {
            let bytes = u64::from(owner_blob.size);
            let sealed = result.mapped_blobs.iter().any(|blob| {
                blob.name == owner_blob.name
                    && blob.base == owner_blob.base
                    && blob.bytes.len() == bytes as usize
                    && blob.bytes.sealed_pin().is_some()
            });
            let entry = families.entry(owner_blob.name.clone()).or_default();
            entry.0 += 1;
            entry.1 += bytes;
            if sealed {
                entry.2 += bytes;
            }
        }
        let (mut blob_count, mut blob_bytes, mut sealed_bytes) = (0usize, 0u64, 0u64);
        for (name, (count, bytes, sealed)) in families {
            eprintln!(
                "real_owner_blob_family name={name:?} blobs={count} bytes={bytes} mib={:.6} sealed_bytes={sealed}",
                bytes as f64 / (1024.0 * 1024.0),
            );
            blob_count += count;
            blob_bytes += bytes;
            sealed_bytes += sealed;
        }
        eprintln!(
            "real_owner_blob_total blobs={blob_count} bytes={blob_bytes} mib={:.6} sealed_bytes={sealed_bytes}",
            blob_bytes as f64 / (1024.0 * 1024.0),
        );
        Ok(())
    }
}
