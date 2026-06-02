use std::collections::BTreeMap;

use crate::{
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        kernel::CeKernel,
    },
    error::{Error, Result},
    pe::{ImportBy, ImportDescriptor, PeImage},
};

pub const IMPORT_TRAP_BASE: u32 = 0x7fff_0000;
pub const IMPORT_TRAP_STRIDE: u32 = 0x10;
pub const IMPORT_TRAP_PAGE_SIZE: u32 = 0x0001_0000;
pub const DYNAMIC_COREDLL_PROC_TRAP_BASE: u32 = IMPORT_TRAP_BASE + 0x5000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportTrap {
    pub address: u32,
    pub module_kind: ImportModuleKind,
    pub module_name: String,
    pub ordinal: Option<u32>,
    pub name: Option<String>,
    pub iat_va: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportModuleKind {
    Coredll,
    Mfc,
    CommonControls,
    Winsock,
    Ole,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportTrapTable {
    traps: BTreeMap<u32, ImportTrap>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExternalImportTable {
    modules: BTreeMap<String, ExternalImportModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalImportModule {
    pub module_name: String,
    pub image_base: u32,
    by_ordinal: BTreeMap<u32, u32>,
    by_name: BTreeMap<String, u32>,
}

impl ImportTrapTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.traps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.traps.is_empty()
    }

    pub fn traps(&self) -> impl Iterator<Item = &ImportTrap> {
        self.traps.values()
    }

    pub fn trap_at(&self, address: u32) -> Option<&ImportTrap> {
        self.traps.get(&address)
    }

    pub fn merge(&mut self, other: Self) {
        self.traps.extend(other.traps);
    }

    pub fn dispatch_trap<M: CoredllGuestMemory>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        thread_id: u32,
        address: u32,
        args: Vec<u32>,
    ) -> Option<u32> {
        let trap = self
            .trap_at(address)
            .cloned()
            .or_else(|| dynamic_coredll_proc_trap(address))?;
        Some(match trap.module_kind {
            ImportModuleKind::Coredll => {
                let Some(ordinal) = trap.ordinal else {
                    return Some(0);
                };
                let table = CoredllExportTable::default();
                dispatch_return_to_u32(
                    table
                        .dispatch_raw_ordinal_with_memory(kernel, memory, thread_id, ordinal, args),
                )
            }
            ImportModuleKind::CommonControls
            | ImportModuleKind::Winsock
            | ImportModuleKind::Ole => dispatch_external_stub_to_u32(&trap, memory, &args),
            ImportModuleKind::Mfc => return None,
        })
    }

    fn insert(&mut self, trap: ImportTrap) {
        self.traps.insert(trap.address, trap);
    }
}

pub fn dynamic_coredll_proc_address(ordinal: u32) -> Option<u32> {
    let address =
        DYNAMIC_COREDLL_PROC_TRAP_BASE.checked_add(ordinal.checked_mul(IMPORT_TRAP_STRIDE)?)?;
    (address < IMPORT_TRAP_BASE.saturating_add(IMPORT_TRAP_PAGE_SIZE)).then_some(address)
}

pub fn dynamic_coredll_proc_trap(address: u32) -> Option<ImportTrap> {
    if address < DYNAMIC_COREDLL_PROC_TRAP_BASE
        || address >= IMPORT_TRAP_BASE.saturating_add(IMPORT_TRAP_PAGE_SIZE)
    {
        return None;
    }
    let offset = address - DYNAMIC_COREDLL_PROC_TRAP_BASE;
    if offset % IMPORT_TRAP_STRIDE != 0 {
        return None;
    }
    let ordinal = offset / IMPORT_TRAP_STRIDE;
    let export = crate::ce::coredll::CoredllExportTable::resolve_static_ordinal(ordinal)?;
    Some(ImportTrap {
        address,
        module_kind: ImportModuleKind::Coredll,
        module_name: "COREDLL.dll".to_owned(),
        ordinal: Some(ordinal),
        name: Some(export.name),
        iat_va: 0,
    })
}

pub fn patch_pe_coredll_imports(
    image: &PeImage,
    mapped: &mut [u8],
    exports: &CoredllExportTable,
    trap_base: u32,
) -> Result<ImportTrapTable> {
    patch_supported_imports_with_external(
        mapped,
        image.image_base(),
        &image.imports,
        exports,
        trap_base,
        &ExternalImportTable::default(),
    )
}

pub fn patch_pe_imports(
    image: &PeImage,
    mapped: &mut [u8],
    exports: &CoredllExportTable,
    trap_base: u32,
    external: &ExternalImportTable,
) -> Result<ImportTrapTable> {
    patch_supported_imports_with_external(
        mapped,
        image.image_base(),
        &image.imports,
        exports,
        trap_base,
        external,
    )
}

pub fn patch_coredll_imports(
    mapped: &mut [u8],
    image_base: u32,
    imports: &[ImportDescriptor],
    exports: &CoredllExportTable,
    trap_base: u32,
) -> Result<ImportTrapTable> {
    patch_supported_imports_with_external(
        mapped,
        image_base,
        imports,
        exports,
        trap_base,
        &ExternalImportTable::default(),
    )
}

pub fn patch_supported_imports(
    mapped: &mut [u8],
    image_base: u32,
    imports: &[ImportDescriptor],
    exports: &CoredllExportTable,
    trap_base: u32,
) -> Result<ImportTrapTable> {
    patch_supported_imports_with_external(
        mapped,
        image_base,
        imports,
        exports,
        trap_base,
        &ExternalImportTable::default(),
    )
}

pub fn patch_supported_imports_with_external(
    mapped: &mut [u8],
    image_base: u32,
    imports: &[ImportDescriptor],
    exports: &CoredllExportTable,
    trap_base: u32,
    external: &ExternalImportTable,
) -> Result<ImportTrapTable> {
    let mut table = ImportTrapTable::new();
    let mut next_address = trap_base;

    for descriptor in imports {
        let Some(module_kind) = classify_import_module(&descriptor.module_name) else {
            continue;
        };
        for thunk in &descriptor.imports {
            if module_kind != ImportModuleKind::Coredll {
                if let Some(address) = external.resolve(&descriptor.module_name, &thunk.import) {
                    write_mapped_u32(mapped, thunk.iat_rva, address)?;
                    continue;
                }
                if module_kind == ImportModuleKind::Mfc {
                    continue;
                }
            }

            let (ordinal, name) = match &thunk.import {
                ImportBy::Ordinal(ordinal) => (Some(u32::from(*ordinal)), None),
                ImportBy::Name { name, .. } => {
                    let ordinal = (module_kind == ImportModuleKind::Coredll)
                        .then(|| exports.resolve_name(name).map(|export| export.ordinal))
                        .flatten();
                    (ordinal, Some(name.clone()))
                }
            };
            let iat_va = image_base.wrapping_add(thunk.iat_rva);
            write_mapped_u32(mapped, thunk.iat_rva, next_address)?;
            table.insert(ImportTrap {
                address: next_address,
                module_kind,
                module_name: descriptor.module_name.clone(),
                ordinal,
                name,
                iat_va,
            });
            next_address = next_address
                .checked_add(IMPORT_TRAP_STRIDE)
                .ok_or_else(|| Error::InvalidArgument("import trap address overflow".to_owned()))?;
            if next_address >= trap_base.saturating_add(IMPORT_TRAP_PAGE_SIZE) {
                return Err(Error::InvalidArgument(
                    "import trap page is full".to_owned(),
                ));
            }
        }
    }

    Ok(table)
}

impl ExternalImportTable {
    pub fn add_pe_image(&mut self, module_name: &str, image: &PeImage, load_base: u32) {
        let mut module = ExternalImportModule {
            module_name: module_name.to_owned(),
            image_base: load_base,
            by_ordinal: BTreeMap::new(),
            by_name: BTreeMap::new(),
        };
        if let Some(exports) = image.exports.as_ref() {
            for export in &exports.functions {
                if export.rva == 0 || export.forwarder.is_some() {
                    continue;
                }
                let va = load_base.wrapping_add(export.rva);
                module.by_ordinal.insert(export.ordinal, va);
                if let Some(name) = export.name.as_deref() {
                    module.by_name.insert(normalize_symbol(name), va);
                }
            }
        }
        self.modules.insert(normalize_module(module_name), module);
    }

    pub fn resolve(&self, module_name: &str, import: &ImportBy) -> Option<u32> {
        let module = self.modules.get(&normalize_module(module_name))?;
        match import {
            ImportBy::Ordinal(ordinal) => module.by_ordinal.get(&u32::from(*ordinal)).copied(),
            ImportBy::Name { name, .. } => module.by_name.get(&normalize_symbol(name)).copied(),
        }
    }
}

fn dispatch_external_stub_to_u32<M: CoredllGuestMemory>(
    trap: &ImportTrap,
    memory: &mut M,
    args: &[u32],
) -> u32 {
    let name = trap.name.as_deref().unwrap_or("");
    tracing::debug!(
        target: "ce.imports",
        module = trap.module_name.as_str(),
        kind = ?trap.module_kind,
        name,
        ordinal = trap.ordinal,
        a0 = format_args!("0x{:08x}", raw_import_arg(args, 0)),
        a1 = format_args!("0x{:08x}", raw_import_arg(args, 1)),
        a2 = format_args!("0x{:08x}", raw_import_arg(args, 2)),
        a3 = format_args!("0x{:08x}", raw_import_arg(args, 3)),
        "external DLL import stub"
    );
    match trap.module_kind {
        ImportModuleKind::CommonControls => common_controls_stub_return(name),
        ImportModuleKind::Winsock => winsock_stub_return(trap, memory, args),
        ImportModuleKind::Ole => ole_stub_return(name),
        ImportModuleKind::Coredll => 0,
        ImportModuleKind::Mfc => {
            unreachable!("MFC imports must resolve to SDK DLL exports, not emulator stubs")
        }
    }
}

fn raw_import_arg(args: &[u32], index: usize) -> u32 {
    args.get(index).copied().unwrap_or(0)
}

fn common_controls_stub_return(name: &str) -> u32 {
    match normalize_symbol(name).as_str() {
        "initcommoncontrols" => 0,
        "initcommoncontrolsex" => 1,
        _ => 0,
    }
}

fn winsock_stub_return<M: CoredllGuestMemory>(
    trap: &ImportTrap,
    memory: &mut M,
    args: &[u32],
) -> u32 {
    let name = trap.name.as_deref().unwrap_or("");
    match (trap.ordinal, normalize_symbol(name).as_str()) {
        (Some(3), _) | (_, "wsastartup") => {
            wsa_startup(memory, raw_import_arg(args, 0), raw_import_arg(args, 1))
        }
        (Some(1), _) | (_, "wsacleanup") => 0,
        (_, "socket" | "accept") => u32::MAX,
        _ => 0,
    }
}

fn wsa_startup<M: CoredllGuestMemory>(memory: &mut M, requested: u32, data_ptr: u32) -> u32 {
    const WSAVERNOTSUPPORTED: u32 = 10092;
    const WSADESCRIPTION_LEN: usize = 256;
    const WSASYS_STATUS_LEN: usize = 128;
    if data_ptr == 0 {
        return WSAVERNOTSUPPORTED;
    }
    let major = ((requested >> 8) & 0xff).clamp(1, 2) as u16;
    let minor = (requested & 0xff).min(2) as u16;
    let version = (major << 8) | minor;
    if memory.write_u16(data_ptr, version).is_err()
        || memory.write_u16(data_ptr + 2, 0x0202).is_err()
        || write_guest_bytes(
            memory,
            data_ptr + 4,
            b"FakeCE Winsock\0",
            WSADESCRIPTION_LEN + 1,
        )
        .is_err()
        || write_guest_bytes(
            memory,
            data_ptr + 4 + 257,
            b"Running\0",
            WSASYS_STATUS_LEN + 1,
        )
        .is_err()
        || memory.write_u16(data_ptr + 4 + 257 + 129, 0).is_err()
        || memory.write_u16(data_ptr + 4 + 257 + 129 + 2, 0).is_err()
        || memory.write_u32(data_ptr + 4 + 257 + 129 + 4, 0).is_err()
    {
        return WSAVERNOTSUPPORTED;
    }
    0
}

fn write_guest_bytes<M: CoredllGuestMemory>(
    memory: &mut M,
    addr: u32,
    bytes: &[u8],
    capacity: usize,
) -> Result<()> {
    for index in 0..capacity {
        let value = bytes.get(index).copied().unwrap_or(0);
        memory.write_u8(addr + index as u32, value)?;
    }
    Ok(())
}

fn ole_stub_return(name: &str) -> u32 {
    match normalize_symbol(name).as_str() {
        "coinitialize" | "coinitializeex" | "couninitialize" | "oleinitialize" => 0,
        "cocreateinstance" | "cogetclassobject" => 0x8000_4001,
        _ => 0,
    }
}

pub fn import_trap_code_page(table: &ImportTrapTable) -> Vec<u8> {
    let mut page = vec![0; IMPORT_TRAP_PAGE_SIZE as usize];
    for trap in table.traps() {
        let Some(offset) = trap.address.checked_sub(IMPORT_TRAP_BASE) else {
            continue;
        };
        let offset = offset as usize;
        if offset + 8 > page.len() {
            continue;
        }
        page[offset..offset + 4].copy_from_slice(&0x03e0_0008u32.to_le_bytes());
        page[offset + 4..offset + 8].copy_from_slice(&0u32.to_le_bytes());
    }
    let dynamic_start = (DYNAMIC_COREDLL_PROC_TRAP_BASE - IMPORT_TRAP_BASE) as usize;
    for offset in (dynamic_start..page.len().saturating_sub(8)).step_by(IMPORT_TRAP_STRIDE as usize)
    {
        page[offset..offset + 4].copy_from_slice(&0x03e0_0008u32.to_le_bytes());
        page[offset + 4..offset + 8].copy_from_slice(&0u32.to_le_bytes());
    }
    page
}

fn dispatch_return_to_u32(dispatch: CoredllDispatch) -> u32 {
    match dispatch {
        CoredllDispatch::Returned { value, .. } => coredll_value_to_u32(value),
        CoredllDispatch::Stubbed { stub, .. } => stub.return_value,
        CoredllDispatch::UnresolvedOrdinal(_)
        | CoredllDispatch::UnresolvedName(_)
        | CoredllDispatch::Unimplemented { .. }
        | CoredllDispatch::OrdinalMismatch { .. } => 0,
    }
}

fn coredll_value_to_u32(value: CoredllValue) -> u32 {
    match value {
        CoredllValue::Bool(value) => u32::from(value),
        CoredllValue::U32(value) | CoredllValue::Handle(value) | CoredllValue::MmResult(value) => {
            value
        }
        CoredllValue::MmOpen { status, handle } => {
            if status == 0 {
                handle.unwrap_or(0)
            } else {
                status
            }
        }
        CoredllValue::FileIo(value) => u32::from(value.success),
        CoredllValue::DeviceIoControl(value) => u32::from(value.success),
        CoredllValue::RegOpen(value) => value.status,
        CoredllValue::RegQuery(value) => value.status,
        CoredllValue::CeMath(value) => cemath_value_to_u32(value),
        CoredllValue::Bytes(_)
        | CoredllValue::String(_)
        | CoredllValue::OptionalMessage(_)
        | CoredllValue::MessagePump(_) => 0,
    }
}

fn cemath_value_to_u32(value: crate::ce::cemath::CeMathValue) -> u32 {
    match value {
        crate::ce::cemath::CeMathValue::I32(value) | crate::ce::cemath::CeMathValue::Cmp(value) => {
            value as u32
        }
        crate::ce::cemath::CeMathValue::U32(value) => value,
        crate::ce::cemath::CeMathValue::I64(value) => value as u32,
        crate::ce::cemath::CeMathValue::U64(value) => value as u32,
        crate::ce::cemath::CeMathValue::F32(value) => value.to_bits(),
        crate::ce::cemath::CeMathValue::F64(value) => value.to_bits() as u32,
        crate::ce::cemath::CeMathValue::Div { quot, .. } => quot as u32,
        crate::ce::cemath::CeMathValue::Frexp { fraction, .. } => fraction.to_bits() as u32,
        crate::ce::cemath::CeMathValue::Modf { fraction, .. } => fraction.to_bits() as u32,
        crate::ce::cemath::CeMathValue::DivideByZero => 0,
    }
}

fn classify_import_module(module_name: &str) -> Option<ImportModuleKind> {
    let normalized = normalize_module(module_name);
    if normalized == "coredll" {
        Some(ImportModuleKind::Coredll)
    } else if normalized.starts_with("mfc") {
        Some(ImportModuleKind::Mfc)
    } else if normalized == "commctrl" || normalized == "commctrlce" {
        Some(ImportModuleKind::CommonControls)
    } else if normalized == "winsock" || normalized == "ws2" || normalized == "ws2_32" {
        Some(ImportModuleKind::Winsock)
    } else if normalized == "ole32" || normalized == "oleaut32" || normalized == "olece" {
        Some(ImportModuleKind::Ole)
    } else {
        None
    }
}

fn normalize_module(module_name: &str) -> String {
    module_name
        .trim()
        .trim_end_matches('\0')
        .trim_end_matches(".dll")
        .trim_end_matches(".DLL")
        .to_ascii_lowercase()
}

fn normalize_symbol(name: &str) -> String {
    name.trim_start_matches('_')
        .split('@')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn write_mapped_u32(mapped: &mut [u8], rva: u32, value: u32) -> Result<()> {
    let offset = rva as usize;
    let end = offset
        .checked_add(4)
        .ok_or_else(|| Error::InvalidArgument("IAT write overflow".to_owned()))?;
    let slot = mapped.get_mut(offset..end).ok_or_else(|| {
        Error::InvalidArgument(format!("IAT RVA 0x{rva:08x} is outside mapped image"))
    })?;
    slot.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pe::ImportThunk;

    #[test]
    fn patches_coredll_iat_to_trap_addresses() {
        let imports = vec![ImportDescriptor {
            module_name: "COREDLL.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![
                ImportThunk {
                    thunk_rva: 0x2000,
                    iat_rva: 0x3000,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "GetTickCount".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(461),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_coredll_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();

        assert_eq!(table.len(), 2);
        assert_eq!(
            u32::from_le_bytes(mapped[0x3000..0x3004].try_into().unwrap()),
            IMPORT_TRAP_BASE
        );
        assert_eq!(
            u32::from_le_bytes(mapped[0x3004..0x3008].try_into().unwrap()),
            IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE
        );
        assert_eq!(
            table.trap_at(IMPORT_TRAP_BASE).unwrap().ordinal,
            Some(crate::ce::coredll_ordinals::ORD_GET_TICK_COUNT)
        );
    }

    #[test]
    fn trap_page_contains_mips_return_stubs() {
        let mut table = ImportTrapTable::new();
        table.insert(ImportTrap {
            address: IMPORT_TRAP_BASE,
            module_kind: ImportModuleKind::Coredll,
            module_name: "COREDLL.dll".to_owned(),
            ordinal: Some(1),
            name: None,
            iat_va: 0x4000,
        });

        let page = import_trap_code_page(&table);
        assert_eq!(
            u32::from_le_bytes(page[0..4].try_into().unwrap()),
            0x03e0_0008
        );
        assert_eq!(u32::from_le_bytes(page[4..8].try_into().unwrap()), 0);
    }

    #[test]
    fn patches_commctrl_winsock_and_ole_imports_as_supported_traps_without_mfc_stub() {
        let imports = vec![
            ImportDescriptor {
                module_name: "MFC400.DLL".to_owned(),
                original_first_thunk: 0x2000,
                time_date_stamp: 0,
                forwarder_chain: 0,
                name_rva: 0x2040,
                first_thunk: 0x3000,
                imports: vec![ImportThunk {
                    thunk_rva: 0x2000,
                    iat_rva: 0x3000,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "AfxWinInit".to_owned(),
                    },
                }],
            },
            ImportDescriptor {
                module_name: "commctrl.dll".to_owned(),
                original_first_thunk: 0x2010,
                time_date_stamp: 0,
                forwarder_chain: 0,
                name_rva: 0x2050,
                first_thunk: 0x3010,
                imports: vec![ImportThunk {
                    thunk_rva: 0x2010,
                    iat_rva: 0x3010,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "InitCommonControlsEx".to_owned(),
                    },
                }],
            },
            ImportDescriptor {
                module_name: "winsock.dll".to_owned(),
                original_first_thunk: 0x2020,
                time_date_stamp: 0,
                forwarder_chain: 0,
                name_rva: 0x2060,
                first_thunk: 0x3020,
                imports: vec![ImportThunk {
                    thunk_rva: 0x2020,
                    iat_rva: 0x3020,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "WSAStartup".to_owned(),
                    },
                }],
            },
            ImportDescriptor {
                module_name: "ole32.dll".to_owned(),
                original_first_thunk: 0x2030,
                time_date_stamp: 0,
                forwarder_chain: 0,
                name_rva: 0x2070,
                first_thunk: 0x3030,
                imports: vec![ImportThunk {
                    thunk_rva: 0x2030,
                    iat_rva: 0x3030,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "CoInitializeEx".to_owned(),
                    },
                }],
            },
        ];
        let mut mapped = vec![0; 0x4000];
        mapped[0x3000..0x3004].copy_from_slice(&0xfeed_faceu32.to_le_bytes());
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();

        assert_eq!(table.len(), 3);
        assert_eq!(
            u32::from_le_bytes(mapped[0x3000..0x3004].try_into().unwrap()),
            0xfeed_face
        );
        assert_eq!(
            table.trap_at(IMPORT_TRAP_BASE).unwrap().module_kind,
            ImportModuleKind::CommonControls
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE)
                .unwrap()
                .module_kind,
            ImportModuleKind::Winsock
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2)
                .unwrap()
                .module_kind,
            ImportModuleKind::Ole
        );
    }
}
