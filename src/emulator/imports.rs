use std::collections::BTreeMap;

use crate::{
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        kernel::CeKernel,
        ole,
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
    Winsock,
    Ole,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportTrapTable {
    traps: BTreeMap<u32, ImportTrap>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportTrapReturn {
    pub v0: u32,
    pub v1: Option<u32>,
}

impl ImportTrapReturn {
    fn v0(value: u32) -> Self {
        Self {
            v0: value,
            v1: None,
        }
    }

    fn v0_v1(v0: u32, v1: u32) -> Self {
        Self { v0, v1: Some(v1) }
    }
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

    pub fn dispatch_trap<M: CoredllGuestMemory, I: AsRef<[u32]>>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        thread_id: u32,
        address: u32,
        args: I,
    ) -> Option<u32> {
        self.dispatch_trap_registers(kernel, memory, thread_id, address, args)
            .map(|result| result.v0)
    }

    pub fn dispatch_trap_with_framebuffer<M: CoredllGuestMemory, I: AsRef<[u32]>>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        framebuffer: Option<&mut dyn crate::ce::framebuffer::Framebuffer>,
        thread_id: u32,
        address: u32,
        args: I,
    ) -> Option<u32> {
        self.dispatch_trap_registers_with_framebuffer(
            kernel,
            memory,
            framebuffer,
            thread_id,
            address,
            args,
        )
        .map(|result| result.v0)
    }

    pub fn dispatch_trap_registers<M: CoredllGuestMemory, I: AsRef<[u32]>>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        thread_id: u32,
        address: u32,
        args: I,
    ) -> Option<ImportTrapReturn> {
        self.dispatch_trap_registers_with_framebuffer(
            kernel, memory, None, thread_id, address, args,
        )
    }

    pub fn dispatch_trap_registers_with_framebuffer<M: CoredllGuestMemory, I: AsRef<[u32]>>(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        framebuffer: Option<&mut dyn crate::ce::framebuffer::Framebuffer>,
        thread_id: u32,
        address: u32,
        args: I,
    ) -> Option<ImportTrapReturn> {
        let args = args.as_ref();
        let dynamic_trap;
        let trap = if let Some(trap) = self.trap_at(address) {
            trap
        } else {
            dynamic_trap = dynamic_coredll_proc_trap(address)?;
            &dynamic_trap
        };
        Some(match trap.module_kind {
            ImportModuleKind::Coredll => {
                let Some(ordinal) = trap.ordinal else {
                    return None;
                };
                dispatch_return_to_registers(
                    CoredllExportTable::static_ordinals()
                        .dispatch_raw_ordinal_with_framebuffer_args(
                            kernel,
                            memory,
                            framebuffer,
                            thread_id,
                            ordinal,
                            args,
                        ),
                )?
            }
            ImportModuleKind::Winsock | ImportModuleKind::Ole => ImportTrapReturn::v0(
                dispatch_external_stub_to_u32(kernel, trap, memory, thread_id, args),
            ),
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
        for thunk in &descriptor.imports {
            if let Some(address) = external.resolve(&descriptor.module_name, &thunk.import) {
                write_mapped_u32(mapped, thunk.iat_rva, address)?;
                continue;
            }

            let Some(module_kind) = classify_import_module(&descriptor.module_name) else {
                continue;
            };
            if module_kind != ImportModuleKind::Coredll {
                if module_kind == ImportModuleKind::Mfc {
                    continue;
                }
            }

            let (ordinal, name) = match &thunk.import {
                ImportBy::Ordinal(ordinal) => (
                    Some(normalize_coredll_import_ordinal(u32::from(*ordinal))),
                    None,
                ),
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

pub fn patch_external_imports(
    mapped: &mut [u8],
    imports: &[ImportDescriptor],
    external: &ExternalImportTable,
) -> Result<()> {
    for descriptor in imports {
        for thunk in &descriptor.imports {
            if let Some(address) = external.resolve(&descriptor.module_name, &thunk.import) {
                write_mapped_u32(mapped, thunk.iat_rva, address)?;
            }
        }
    }
    Ok(())
}

fn normalize_coredll_import_ordinal(ordinal: u32) -> u32 {
    if crate::ce::coredll_ordinals::SDK_ORDINALS
        .iter()
        .any(|export| export.ordinal == ordinal)
    {
        return ordinal;
    }
    if CoredllExportTable::resolve_static_ordinal(ordinal).is_some() {
        return ordinal;
    }
    if let Some(export) = crate::ce::coredll_ordinals::lookup_export_index(ordinal) {
        return export.ordinal;
    }
    ordinal
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
    kernel: &mut CeKernel,
    trap: &ImportTrap,
    memory: &mut M,
    thread_id: u32,
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
        ImportModuleKind::Winsock => winsock_stub_return(trap, memory, args),
        ImportModuleKind::Ole => ole_stub_return(kernel, memory, thread_id, name, args),
        ImportModuleKind::Coredll => 0,
        ImportModuleKind::Mfc => {
            unreachable!("MFC imports must resolve to SDK DLL exports, not emulator stubs")
        }
    }
}

fn raw_import_arg(args: &[u32], index: usize) -> u32 {
    args.get(index).copied().unwrap_or(0)
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

fn ole_stub_return<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    name: &str,
    args: &[u32],
) -> u32 {
    match normalize_symbol(name).as_str() {
        "coinitialize" | "coinitializeex" | "couninitialize" | "oleinitialize" => 0,
        "stringfromclsid" => ole::string_from_clsid_raw(
            kernel,
            memory,
            thread_id,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
        ),
        "clsidfromstring" => ole::clsid_from_string_raw(
            kernel,
            memory,
            thread_id,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
        ),
        "cotaskmemfree" => {
            ole::co_task_mem_free_raw(kernel, raw_import_arg(args, 0));
            0
        }
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

fn dispatch_return_to_registers(dispatch: CoredllDispatch) -> Option<ImportTrapReturn> {
    match dispatch {
        CoredllDispatch::Returned { value, .. } => Some(coredll_value_to_registers(value)),
        CoredllDispatch::Stubbed { .. } => None,
        CoredllDispatch::UnresolvedOrdinal(_)
        | CoredllDispatch::UnresolvedName(_)
        | CoredllDispatch::Unimplemented { .. }
        | CoredllDispatch::OrdinalMismatch { .. } => None,
    }
}

fn coredll_value_to_registers(value: CoredllValue) -> ImportTrapReturn {
    match value {
        CoredllValue::Bool(value) => ImportTrapReturn::v0(u32::from(value)),
        CoredllValue::U32(value) | CoredllValue::Handle(value) | CoredllValue::MmResult(value) => {
            ImportTrapReturn::v0(value)
        }
        CoredllValue::MmOpen { status, handle } => {
            if status == 0 {
                ImportTrapReturn::v0(handle.unwrap_or(0))
            } else {
                ImportTrapReturn::v0(status)
            }
        }
        CoredllValue::FileIo(value) => ImportTrapReturn::v0(u32::from(value.success)),
        CoredllValue::DeviceIoControl(value) => ImportTrapReturn::v0(u32::from(value.success)),
        CoredllValue::RegOpen(value) => ImportTrapReturn::v0(value.status),
        CoredllValue::RegQuery(value) => ImportTrapReturn::v0(value.status),
        CoredllValue::CeMath(value) => cemath_value_to_registers(value),
        CoredllValue::Bytes(_)
        | CoredllValue::String(_)
        | CoredllValue::OptionalMessage(_)
        | CoredllValue::MessagePump(_) => ImportTrapReturn::v0(0),
    }
}

fn cemath_value_to_registers(value: crate::ce::cemath::CeMathValue) -> ImportTrapReturn {
    match value {
        crate::ce::cemath::CeMathValue::I32(value) | crate::ce::cemath::CeMathValue::Cmp(value) => {
            ImportTrapReturn::v0(value as u32)
        }
        crate::ce::cemath::CeMathValue::U32(value) => ImportTrapReturn::v0(value),
        crate::ce::cemath::CeMathValue::I64(value) => {
            let bits = value as u64;
            ImportTrapReturn::v0_v1(bits as u32, (bits >> 32) as u32)
        }
        crate::ce::cemath::CeMathValue::U64(value) => {
            ImportTrapReturn::v0_v1(value as u32, (value >> 32) as u32)
        }
        crate::ce::cemath::CeMathValue::F32(value) => ImportTrapReturn::v0(value.to_bits()),
        crate::ce::cemath::CeMathValue::F64(value) => {
            let bits = value.to_bits();
            ImportTrapReturn::v0_v1(bits as u32, (bits >> 32) as u32)
        }
        crate::ce::cemath::CeMathValue::Div { quot, .. } => ImportTrapReturn::v0(quot as u32),
        crate::ce::cemath::CeMathValue::Frexp { fraction, .. } => {
            let bits = fraction.to_bits();
            ImportTrapReturn::v0_v1(bits as u32, (bits >> 32) as u32)
        }
        crate::ce::cemath::CeMathValue::Modf { fraction, .. } => {
            let bits = fraction.to_bits();
            ImportTrapReturn::v0_v1(bits as u32, (bits >> 32) as u32)
        }
        crate::ce::cemath::CeMathValue::DivideByZero => ImportTrapReturn::v0(0),
    }
}

fn classify_import_module(module_name: &str) -> Option<ImportModuleKind> {
    let normalized = normalize_module(module_name);
    if normalized == "coredll" {
        Some(ImportModuleKind::Coredll)
    } else if normalized.starts_with("mfc") {
        Some(ImportModuleKind::Mfc)
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
    use crate::{
        ce::{coredll::CoredllGuestMemory, kernel::CeKernel},
        config::RuntimeConfig,
    };

    #[derive(Default)]
    struct TestMemory;

    impl CoredllGuestMemory for TestMemory {
        fn read_u8(&self, _addr: u32) -> Result<u8> {
            Err(Error::Backend("unexpected read_u8".to_owned()))
        }

        fn write_u8(&mut self, _addr: u32, _value: u8) -> Result<()> {
            Err(Error::Backend("unexpected write_u8".to_owned()))
        }

        fn read_u32(&self, _addr: u32) -> Result<u32> {
            Err(Error::Backend("unexpected read_u32".to_owned()))
        }

        fn write_u32(&mut self, _addr: u32, _value: u32) -> Result<()> {
            Err(Error::Backend("unexpected write_u32".to_owned()))
        }

        fn read_u16(&self, _addr: u32) -> Result<u16> {
            Err(Error::Backend("unexpected read_u16".to_owned()))
        }

        fn write_u16(&mut self, _addr: u32, _value: u16) -> Result<()> {
            Err(Error::Backend("unexpected write_u16".to_owned()))
        }
    }

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
    fn normalizes_coredll_export_index_ordinals_to_current_map_exports() {
        let imports = vec![ImportDescriptor {
            module_name: "COREDLL.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![ImportThunk {
                thunk_rva: 0x2000,
                iat_rva: 0x3000,
                import: ImportBy::Ordinal(1576),
            }],
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

        let trap = table.trap_at(IMPORT_TRAP_BASE).unwrap();
        assert_eq!(
            trap.ordinal,
            Some(crate::ce::coredll_ordinals::ORD_GET_PALETTE_ENTRIES)
        );
        assert_eq!(trap.name, None);
    }

    #[test]
    fn keeps_checked_coredll_crt_ordinals_before_export_index_fallback() {
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
                    import: ImportBy::Ordinal(crate::ce::coredll_ordinals::ORD_MEMSET as u16),
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(crate::ce::coredll_ordinals::ORD_SWPRINTF as u16),
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

        assert_eq!(
            table.trap_at(IMPORT_TRAP_BASE).unwrap().ordinal,
            Some(crate::ce::coredll_ordinals::ORD_MEMSET)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE)
                .unwrap()
                .ordinal,
            Some(crate::ce::coredll_ordinals::ORD_SWPRINTF)
        );
    }

    #[test]
    fn unresolved_coredll_name_trap_does_not_fake_zero_return() {
        let imports = vec![ImportDescriptor {
            module_name: "COREDLL.dll".to_owned(),
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
                    name: "NotARealCeApi".to_owned(),
                },
            }],
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

        let trap = table.trap_at(IMPORT_TRAP_BASE).unwrap();
        assert_eq!(trap.module_kind, ImportModuleKind::Coredll);
        assert_eq!(trap.ordinal, None);
        assert_eq!(trap.name.as_deref(), Some("NotARealCeApi"));

        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory;
        assert_eq!(
            table.dispatch_trap(&mut kernel, &mut memory, 1, IMPORT_TRAP_BASE, vec![]),
            None
        );
    }

    #[test]
    fn coredll_import_trap_exposes_64_bit_return_registers() {
        let mut table = ImportTrapTable::new();
        table.insert(ImportTrap {
            address: IMPORT_TRAP_BASE,
            module_kind: ImportModuleKind::Coredll,
            module_name: "COREDLL.dll".to_owned(),
            ordinal: Some(crate::ce::coredll_ordinals::ORD_LL_DIV),
            name: Some("__ll_div".to_owned()),
            iat_va: 0x4000,
        });

        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory;
        assert_eq!(
            table.dispatch_trap_registers(
                &mut kernel,
                &mut memory,
                1,
                IMPORT_TRAP_BASE,
                vec![0x0989_6800, 0, 0x0098_9680, 0],
            ),
            Some(ImportTrapReturn {
                v0: 16,
                v1: Some(0)
            })
        );
        assert_eq!(
            table.dispatch_trap_registers(
                &mut kernel,
                &mut memory,
                1,
                IMPORT_TRAP_BASE,
                vec![(-21_i64) as u32, u32::MAX, 2, 0],
            ),
            Some(ImportTrapReturn {
                v0: (-10_i64) as u32,
                v1: Some(u32::MAX)
            })
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
    fn patches_loaded_commctrl_exports_from_external_table_without_stub_trap() {
        let imports = vec![ImportDescriptor {
            module_name: "commctrl.dll".to_owned(),
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
                        name: "InitCommonControlsEx".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(17),
                },
            ],
        }];
        let mut module = ExternalImportModule {
            module_name: "commctrl.dll".to_owned(),
            image_base: 0x6200_0000,
            by_ordinal: BTreeMap::new(),
            by_name: BTreeMap::new(),
        };
        module
            .by_name
            .insert("initcommoncontrolsex".to_owned(), 0x6200_1234);
        module.by_ordinal.insert(17, 0x6200_5678);
        let mut external = ExternalImportTable::default();
        external.modules.insert("commctrl".to_owned(), module);

        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports_with_external(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
            &external,
        )
        .unwrap();

        assert!(table.is_empty());
        assert_eq!(
            u32::from_le_bytes(mapped[0x3000..0x3004].try_into().unwrap()),
            0x6200_1234
        );
        assert_eq!(
            u32::from_le_bytes(mapped[0x3004..0x3008].try_into().unwrap()),
            0x6200_5678
        );
    }

    #[test]
    fn second_external_pass_patches_late_loaded_ordinal_imports() {
        let imports = vec![ImportDescriptor {
            module_name: "commctrl.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![ImportThunk {
                thunk_rva: 0x2000,
                iat_rva: 0x3000,
                import: ImportBy::Ordinal(2),
            }],
        }];
        let mut module = ExternalImportModule {
            module_name: "commctrl.dll".to_owned(),
            image_base: 0x4015_0000,
            by_ordinal: BTreeMap::new(),
            by_name: BTreeMap::new(),
        };
        module.by_ordinal.insert(2, 0x4017_9d5c);
        let mut external = ExternalImportTable::default();
        external.modules.insert("commctrl".to_owned(), module);

        let mut mapped = vec![0; 0x4000];
        mapped[0x3000..0x3004].copy_from_slice(&0x8000_0002u32.to_le_bytes());

        patch_external_imports(&mut mapped, &imports, &external).unwrap();

        assert_eq!(
            u32::from_le_bytes(mapped[0x3000..0x3004].try_into().unwrap()),
            0x4017_9d5c
        );
    }

    #[test]
    fn patches_winsock_and_ole_imports_as_supported_traps_without_mfc_or_commctrl_stub() {
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
        mapped[0x3010..0x3014].copy_from_slice(&0xc001_cafeu32.to_le_bytes());
        let table = patch_supported_imports(
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
            0xfeed_face
        );
        assert_eq!(
            u32::from_le_bytes(mapped[0x3010..0x3014].try_into().unwrap()),
            0xc001_cafe
        );
        assert_eq!(
            table.trap_at(IMPORT_TRAP_BASE).unwrap().module_kind,
            ImportModuleKind::Winsock
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE)
                .unwrap()
                .module_kind,
            ImportModuleKind::Ole
        );
    }
}
