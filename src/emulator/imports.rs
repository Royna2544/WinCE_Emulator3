use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ce::{
        coredll::{
            CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllRawContext,
            CoredllValue,
        },
        kernel::CeKernel,
        ole,
    },
    error::{Error, Result},
    pe::{ImportBy, ImportDescriptor, PeImage},
    winsock,
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
    Fsdmgr,
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
    by_ordinal: BTreeMap<u32, ExternalImportTarget>,
    by_name: BTreeMap<String, ExternalImportTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExternalImportTarget {
    Address(u32),
    Forwarder(String),
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

    pub fn next_static_trap_base(&self, reserved_bytes: u32) -> Result<u32> {
        let mut next_address = IMPORT_TRAP_BASE;
        for trap in self.traps.values() {
            if trap.address < IMPORT_TRAP_BASE || trap.address >= DYNAMIC_COREDLL_PROC_TRAP_BASE {
                continue;
            }
            let after_trap = trap
                .address
                .checked_add(IMPORT_TRAP_STRIDE)
                .ok_or_else(|| Error::InvalidArgument("import trap base overflow".to_owned()))?;
            next_address = next_address.max(after_trap);
        }
        if next_address >= DYNAMIC_COREDLL_PROC_TRAP_BASE.saturating_sub(reserved_bytes) {
            return Err(Error::InvalidArgument(
                "import trap page is full".to_owned(),
            ));
        }
        Ok(next_address)
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
        self.dispatch_trap_registers_with_framebuffer_and_context(
            kernel,
            memory,
            framebuffer,
            CoredllRawContext {
                thread_id,
                caller_pc: None,
                trap_pc: Some(address),
                caller_module: None,
                ..Default::default()
            },
            address,
            args,
        )
    }

    pub fn dispatch_trap_registers_with_framebuffer_and_context<
        M: CoredllGuestMemory,
        I: AsRef<[u32]>,
    >(
        &self,
        kernel: &mut CeKernel,
        memory: &mut M,
        framebuffer: Option<&mut dyn crate::ce::framebuffer::Framebuffer>,
        context: CoredllRawContext,
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
                        .dispatch_raw_ordinal_with_framebuffer_args_and_context(
                            kernel,
                            memory,
                            framebuffer,
                            context,
                            ordinal,
                            args,
                        ),
                )?
            }
            ImportModuleKind::Winsock | ImportModuleKind::Ole => ImportTrapReturn::v0(
                dispatch_external_stub_to_u32(kernel, trap, memory, context.thread_id, args),
            ),
            ImportModuleKind::Fsdmgr => {
                ImportTrapReturn::v0(crate::ce::coredll::dispatch_fsdmgr_import_raw(
                    kernel,
                    memory,
                    context.thread_id,
                    trap.ordinal,
                    trap.name.as_deref(),
                    args,
                )?)
            }
            ImportModuleKind::Mfc => return None,
        })
    }

    pub(crate) fn insert(&mut self, trap: ImportTrap) {
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
                if module_kind == ImportModuleKind::Fsdmgr
                    && !is_supported_fsdmgr_import(&thunk.import)
                {
                    continue;
                }
            }

            let (ordinal, name) = match &thunk.import {
                ImportBy::Ordinal(ordinal) => {
                    let ordinal = u32::from(*ordinal);
                    let ordinal = if module_kind == ImportModuleKind::Coredll {
                        normalize_coredll_import_ordinal(ordinal)
                    } else {
                        ordinal
                    };
                    (Some(ordinal), None)
                }
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
        let mut by_ordinal = BTreeMap::new();
        let mut by_name = BTreeMap::new();
        if let Some(exports) = image.exports.as_ref() {
            for export in &exports.functions {
                if export.rva == 0 {
                    continue;
                }
                let target = if let Some(forwarder) = export.forwarder.as_ref() {
                    ExternalImportTarget::Forwarder(forwarder.clone())
                } else {
                    ExternalImportTarget::Address(load_base.wrapping_add(export.rva))
                };
                by_ordinal.insert(export.ordinal, target.clone());
                if let Some(name) = export.name.as_deref() {
                    by_name.insert(normalize_symbol(name), target);
                }
            }
        }
        self.modules.insert(
            normalize_module(module_name),
            ExternalImportModule {
                module_name: module_name.to_owned(),
                image_base: load_base,
                by_ordinal,
                by_name,
            },
        );
    }

    pub fn add_module_exports<N, O>(
        &mut self,
        module_name: &str,
        image_base: u32,
        exports_by_name: N,
        exports_by_ordinal: O,
    ) where
        N: IntoIterator<Item = (String, u32)>,
        O: IntoIterator<Item = (u32, u32)>,
    {
        let mut module = ExternalImportModule {
            module_name: module_name.to_owned(),
            image_base,
            by_ordinal: BTreeMap::new(),
            by_name: BTreeMap::new(),
        };
        for (name, address) in exports_by_name {
            module.by_name.insert(
                normalize_symbol(&name),
                ExternalImportTarget::Address(address),
            );
        }
        for (ordinal, address) in exports_by_ordinal {
            module
                .by_ordinal
                .insert(ordinal, ExternalImportTarget::Address(address));
        }
        self.modules.insert(normalize_module(module_name), module);
    }

    pub fn resolve(&self, module_name: &str, import: &ImportBy) -> Option<u32> {
        self.resolve_inner(module_name, import, &mut BTreeSet::new())
    }

    fn resolve_inner(
        &self,
        module_name: &str,
        import: &ImportBy,
        seen: &mut BTreeSet<String>,
    ) -> Option<u32> {
        let module_key = normalize_module(module_name);
        let import_key = import_identity(import);
        if !seen.insert(format!("{module_key}!{import_key}")) {
            return None;
        }
        let module = self.modules.get(&module_key)?;
        let target = match import {
            ImportBy::Ordinal(ordinal) => module.by_ordinal.get(&u32::from(*ordinal))?,
            ImportBy::Name { name, .. } => module.by_name.get(&normalize_symbol(name))?,
        };
        match target {
            ExternalImportTarget::Address(address) => Some(*address),
            ExternalImportTarget::Forwarder(forwarder) => {
                let (forward_module, forward_import) = parse_forwarder_target(forwarder)?;
                self.resolve_inner(&forward_module, &forward_import, seen)
            }
        }
    }
}

pub fn parse_forwarder_target(forwarder: &str) -> Option<(String, ImportBy)> {
    let (module_name, symbol) = forwarder.rsplit_once('.')?;
    if module_name.is_empty()
        || symbol.is_empty()
        || module_name != module_name.trim()
        || symbol != symbol.trim()
    {
        return None;
    }
    let import = if let Some(ordinal) = symbol.strip_prefix('#') {
        ImportBy::Ordinal(ordinal.parse::<u16>().ok()?)
    } else {
        ImportBy::Name {
            hint: 0,
            name: symbol.to_owned(),
        }
    };
    Some((module_name.to_owned(), import))
}

fn import_identity(import: &ImportBy) -> String {
    match import {
        ImportBy::Ordinal(ordinal) => format!("#{ordinal}"),
        ImportBy::Name { name, .. } => normalize_symbol(name),
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
        ImportModuleKind::Winsock => winsock::dispatch_import(
            kernel,
            thread_id,
            trap.ordinal,
            trap.name.as_deref(),
            memory,
            args,
        ),
        ImportModuleKind::Ole => ole_stub_return(kernel, memory, thread_id, name, args),
        ImportModuleKind::Coredll => 0,
        ImportModuleKind::Fsdmgr => 0,
        ImportModuleKind::Mfc => {
            unreachable!("MFC imports must resolve to SDK DLL exports, not emulator stubs")
        }
    }
}

fn raw_import_arg(args: &[u32], index: usize) -> u32 {
    args.get(index).copied().unwrap_or(0)
}

fn ole_stub_return<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    name: &str,
    args: &[u32],
) -> u32 {
    match normalize_symbol(name).as_str() {
        "coinitialize" | "oleinitialize" => kernel.com.co_initialize_ex(thread_id, 0),
        "coinitializeex" => kernel
            .com
            .co_initialize_ex(thread_id, raw_import_arg(args, 1)),
        "couninitialize" => {
            kernel.com.co_uninitialize(thread_id);
            0
        }
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
        "cocreateinstance" => ole::co_create_instance_raw(
            kernel,
            memory,
            thread_id,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 3),
            raw_import_arg(args, 4),
        ),
        "cogetclassobject" => 0x8000_4001,
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
    } else if normalized == "fsdmgr" {
        Some(ImportModuleKind::Fsdmgr)
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

fn is_supported_fsdmgr_import(import: &ImportBy) -> bool {
    match import {
        ImportBy::Ordinal(ordinal) => {
            matches!(
                u32::from(*ordinal),
                2 | 3
                    | 4
                    | 5
                    | 6
                    | 7
                    | 8
                    | 9
                    | 10
                    | 11
                    | 12
                    | 14
                    | 15
                    | 16
                    | 17
                    | 18
                    | 19
                    | 20
                    | 21
                    | 22
                    | 24
                    | 25
                    | 26
                    | 27
                    | 30
                    | 31
                    | 32
                    | 35
                    | 36
                    | 37
                    | 44
                    | 54..=67
                    | 68..=75 | 80..=82
            )
        }
        ImportBy::Name { name, .. } => crate::ce::coredll::is_fsdmgr_import(name),
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
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use crate::pe::ImportThunk;
    use crate::{
        ce::{
            coredll::CoredllGuestMemory,
            kernel::{CeKernel, DeviceInterfaceAdvertisement},
            registry::RegistryValue,
        },
        config::{MountConfig, RuntimeConfig},
    };

    #[derive(Default)]
    struct TestMemory {
        bytes: BTreeMap<u32, u8>,
        halfwords: BTreeMap<u32, u16>,
        words: BTreeMap<u32, u32>,
    }

    impl TestMemory {
        fn map_bytes(&mut self, addr: u32, bytes: &[u8]) {
            for (offset, byte) in bytes.iter().copied().enumerate() {
                self.bytes.insert(addr + offset as u32, byte);
            }
        }

        fn map_wide_z(&mut self, addr: u32, value: &str) {
            for (index, unit) in value.encode_utf16().chain(std::iter::once(0)).enumerate() {
                self.halfwords.insert(addr + (index as u32 * 2), unit);
            }
        }

        fn map_wide_buffer(&mut self, addr: u32, chars: usize) {
            for index in 0..chars {
                self.halfwords.insert(addr + (index as u32 * 2), 0);
            }
        }

        fn map_word(&mut self, addr: u32, value: u32) {
            self.words.insert(addr, value);
        }

        fn map_halfword(&mut self, addr: u32, value: u16) {
            self.halfwords.insert(addr, value);
        }

        fn word(&self, addr: u32) -> u32 {
            self.words.get(&addr).copied().unwrap_or(0)
        }

        fn read_le_u32_from_bytes(&self, addr: u32) -> u32 {
            let mut bytes = [0u8; 4];
            for (offset, byte) in bytes.iter_mut().enumerate() {
                *byte = self
                    .bytes
                    .get(&(addr + offset as u32))
                    .copied()
                    .unwrap_or(0);
            }
            u32::from_le_bytes(bytes)
        }

        fn read_wide_z(&self, addr: u32, max_chars: usize) -> String {
            let mut units = Vec::new();
            for index in 0..max_chars {
                let unit = self
                    .halfwords
                    .get(&(addr + (index as u32 * 2)))
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

    impl CoredllGuestMemory for TestMemory {
        fn read_u8(&self, addr: u32) -> Result<u8> {
            self.bytes
                .get(&addr)
                .copied()
                .ok_or_else(|| Error::Backend(format!("unexpected read_u8 0x{addr:08x}")))
        }

        fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
            if let Some(byte) = self.bytes.get_mut(&addr) {
                *byte = value;
                Ok(())
            } else {
                Err(Error::Backend(format!("unexpected write_u8 0x{addr:08x}")))
            }
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            self.words
                .get(&addr)
                .copied()
                .ok_or_else(|| Error::Backend(format!("unexpected read_u32 0x{addr:08x}")))
        }

        fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
            if let Some(word) = self.words.get_mut(&addr) {
                *word = value;
                Ok(())
            } else {
                Err(Error::Backend(format!("unexpected write_u32 0x{addr:08x}")))
            }
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            self.halfwords
                .get(&addr)
                .copied()
                .ok_or_else(|| Error::Backend(format!("unexpected read_u16 0x{addr:08x}")))
        }

        fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
            if let Some(halfword) = self.halfwords.get_mut(&addr) {
                *halfword = value;
                Ok(())
            } else {
                Err(Error::Backend(format!("unexpected write_u16 0x{addr:08x}")))
            }
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
    fn patches_supported_fsdmgr_imports_only() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_GetVolumeName".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(37),
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_RegisterVolume".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x200c,
                    iat_rva: 0x300c,
                    import: ImportBy::Ordinal(21),
                },
                ImportThunk {
                    thunk_rva: 0x2010,
                    iat_rva: 0x3010,
                    import: ImportBy::Ordinal(10),
                },
                ImportThunk {
                    thunk_rva: 0x2014,
                    iat_rva: 0x3014,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_FindNextChangeNotification".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2018,
                    iat_rva: 0x3018,
                    import: ImportBy::Ordinal(74),
                },
                ImportThunk {
                    thunk_rva: 0x201c,
                    iat_rva: 0x301c,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "STOREMGR_FsIoControlW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2020,
                    iat_rva: 0x3020,
                    import: ImportBy::Ordinal(44),
                },
                ImportThunk {
                    thunk_rva: 0x2024,
                    iat_rva: 0x3024,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_CreateFileHandle".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2028,
                    iat_rva: 0x3028,
                    import: ImportBy::Ordinal(8),
                },
                ImportThunk {
                    thunk_rva: 0x202c,
                    iat_rva: 0x302c,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_CreateCache".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2030,
                    iat_rva: 0x3030,
                    import: ImportBy::Ordinal(24),
                },
                ImportThunk {
                    thunk_rva: 0x2034,
                    iat_rva: 0x3034,
                    import: ImportBy::Ordinal(12),
                },
                ImportThunk {
                    thunk_rva: 0x2038,
                    iat_rva: 0x3038,
                    import: ImportBy::Ordinal(25),
                },
                ImportThunk {
                    thunk_rva: 0x203c,
                    iat_rva: 0x303c,
                    import: ImportBy::Ordinal(35),
                },
                ImportThunk {
                    thunk_rva: 0x2040,
                    iat_rva: 0x3040,
                    import: ImportBy::Ordinal(26),
                },
                ImportThunk {
                    thunk_rva: 0x2044,
                    iat_rva: 0x3044,
                    import: ImportBy::Ordinal(36),
                },
                ImportThunk {
                    thunk_rva: 0x2048,
                    iat_rva: 0x3048,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_GetDiskInfo".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x204c,
                    iat_rva: 0x304c,
                    import: ImportBy::Ordinal(17),
                },
                ImportThunk {
                    thunk_rva: 0x2050,
                    iat_rva: 0x3050,
                    import: ImportBy::Ordinal(18),
                },
                ImportThunk {
                    thunk_rva: 0x2054,
                    iat_rva: 0x3054,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_GetRegistryString".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2058,
                    iat_rva: 0x3058,
                    import: ImportBy::Ordinal(20),
                },
                ImportThunk {
                    thunk_rva: 0x205c,
                    iat_rva: 0x305c,
                    import: ImportBy::Ordinal(11),
                },
                ImportThunk {
                    thunk_rva: 0x2060,
                    iat_rva: 0x3060,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_FormatVolume".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2064,
                    iat_rva: 0x3064,
                    import: ImportBy::Ordinal(31),
                },
                ImportThunk {
                    thunk_rva: 0x2068,
                    iat_rva: 0x3068,
                    import: ImportBy::Ordinal(80),
                },
                ImportThunk {
                    thunk_rva: 0x206c,
                    iat_rva: 0x306c,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_AsyncExitVolume".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2070,
                    iat_rva: 0x3070,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_ParseSecurityDescriptor".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2074,
                    iat_rva: 0x3074,
                    import: ImportBy::Ordinal(2),
                },
                ImportThunk {
                    thunk_rva: 0x2078,
                    iat_rva: 0x3078,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_UnsupportedStorageApi".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();

        assert_eq!(table.len(), 30);
        for index in 0..30 {
            let iat = 0x3000 + index * 4;
            assert_eq!(
                u32::from_le_bytes(mapped[iat..iat + 4].try_into().unwrap()),
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * index as u32
            );
        }
        assert_eq!(
            u32::from_le_bytes(mapped[0x3078..0x307c].try_into().unwrap()),
            0
        );
        assert_eq!(
            table.trap_at(IMPORT_TRAP_BASE).unwrap().module_kind,
            ImportModuleKind::Fsdmgr
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE)
                .unwrap()
                .ordinal,
            Some(37)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_RegisterVolume")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3)
                .unwrap()
                .ordinal,
            Some(21)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 4)
                .unwrap()
                .ordinal,
            Some(10)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 6)
                .unwrap()
                .ordinal,
            Some(74)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 7)
                .unwrap()
                .name
                .as_deref(),
            Some("STOREMGR_FsIoControlW")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 8)
                .unwrap()
                .ordinal,
            Some(44)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 9)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_CreateFileHandle")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 10)
                .unwrap()
                .ordinal,
            Some(8)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 11)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_CreateCache")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 12)
                .unwrap()
                .ordinal,
            Some(24)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 13)
                .unwrap()
                .ordinal,
            Some(12)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 14)
                .unwrap()
                .ordinal,
            Some(25)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 15)
                .unwrap()
                .ordinal,
            Some(35)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 16)
                .unwrap()
                .ordinal,
            Some(26)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 17)
                .unwrap()
                .ordinal,
            Some(36)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 18)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_GetDiskInfo")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 19)
                .unwrap()
                .ordinal,
            Some(17)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 20)
                .unwrap()
                .ordinal,
            Some(18)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 21)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_GetRegistryString")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 22)
                .unwrap()
                .ordinal,
            Some(20)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 23)
                .unwrap()
                .ordinal,
            Some(11)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 24)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_FormatVolume")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 25)
                .unwrap()
                .ordinal,
            Some(31)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 26)
                .unwrap()
                .ordinal,
            Some(80)
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 27)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_AsyncExitVolume")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 28)
                .unwrap()
                .name
                .as_deref(),
            Some("FSDMGR_ParseSecurityDescriptor")
        );
        assert_eq!(
            table
                .trap_at(IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 29)
                .unwrap()
                .ordinal,
            Some(2)
        );
    }

    #[test]
    fn fsdmgr_advertise_interface_import_publishes_and_removes_device_interface() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_AdvertiseInterface".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(2),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let guid_ptr = 0x1000_0000;
        let empty_name_ptr = 0x1000_0100;
        let name_ptr = 0x1000_0200;
        let class_guid = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xff, 0x10,
        ];
        memory.map_bytes(guid_ptr, &class_guid);
        memory.map_wide_z(empty_name_ptr, "");
        memory.map_wide_z(name_ptr, "\\StoreMgr\\DSK1:");

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [guid_ptr, empty_name_ptr, 1],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert!(
            kernel
                .advertised_device_interfaces()
                .contains(&DeviceInterfaceAdvertisement {
                    class_guid,
                    name: "\\".to_owned(),
                })
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [guid_ptr, name_ptr, 1],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert!(
            kernel
                .advertised_device_interfaces()
                .contains(&DeviceInterfaceAdvertisement {
                    class_guid,
                    name: "\\StoreMgr\\DSK1:".to_owned(),
                })
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [guid_ptr, name_ptr, 0],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert!(
            !kernel
                .advertised_device_interfaces()
                .contains(&DeviceInterfaceAdvertisement {
                    class_guid,
                    name: "\\StoreMgr\\DSK1:".to_owned(),
                })
        );
    }

    #[test]
    fn fsdmgr_notification_import_dispatches_to_raw_reset_path() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                    name: "FSINT_FindNextChangeNotification".to_owned(),
                },
            }],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [0x1234_5678],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_INVALID_HANDLE
        );
    }

    #[test]
    fn fsdmgr_create_file_and_search_handle_return_fsd_context_pointer() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_CreateFileHandle".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(8),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [0x1111_2222, 0x3333_4444, 0x5555_6666],
            ),
            Some(0x5555_6666)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [0x1111_2222, 0x3333_4444, 0],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
    }

    #[test]
    fn fsdmgr_storemgr_fs_io_control_import_dispatches_to_mounted_volume_info() {
        const FSCTL_COPY_EXTERNAL_START: u32 = 0x0009_004c;
        const FSCTL_COPY_EXTERNAL_COMPLETE: u32 = 0x0009_0050;
        const FSCTL_REFRESH_VOLUME: u32 = 0x0009_007c;
        const FSCTL_GET_VOLUME_INFO: u32 = 0x0009_0080;
        const FSCTL_FLUSH_BUFFERS: u32 = 0x0009_0084;
        const FILE_COPY_EXTERNAL_SIZE: u32 = 536;
        const UNKNOWN_FSCTL: u32 = 0x0009_0099;
        const CE_VOLUME_INFO_SIZE: u32 = 144;
        const CE_VOLUME_ATTRIBUTE_REMOVABLE: u32 = 0x0000_0004;
        const ERROR_INVALID_PARAMETER: u32 = 87;
        const ERROR_NOT_SUPPORTED: u32 = 50;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                    name: "STOREMGR_FsIoControlW".to_owned(),
                },
            }],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let root = std::env::temp_dir().join(format!(
            "wince_storemgr_fsctl_import_{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&root);
        kernel.mount_guest_root("\\SDMMC Disk", root.clone());

        let mut memory = TestMemory::default();
        let path = 0x1000_0000;
        let info_level = 0x1000_0100;
        let volume_info = 0x1000_0200;
        let bytes_returned = 0x1000_0400;
        let copy_external = 0x1000_0500;
        let copy_external_out = 0x1000_0800;
        memory.map_wide_z(path, "\\SDMMC Disk");
        memory.map_word(info_level, 0);
        memory.map_bytes(volume_info, &vec![0x5a; CE_VOLUME_INFO_SIZE as usize]);
        memory.map_word(bytes_returned, 0xfeed_cafe);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [
                    77,
                    path,
                    FSCTL_GET_VOLUME_INFO,
                    info_level,
                    4,
                    volume_info,
                    CE_VOLUME_INFO_SIZE,
                    bytes_returned,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned), CE_VOLUME_INFO_SIZE);
        assert_eq!(
            memory.read_le_u32_from_bytes(volume_info + 4) & CE_VOLUME_ATTRIBUTE_REMOVABLE,
            CE_VOLUME_ATTRIBUTE_REMOVABLE
        );

        for ioctl in [FSCTL_REFRESH_VOLUME, FSCTL_FLUSH_BUFFERS] {
            memory.map_word(bytes_returned, 0xfeed_cafe);
            assert_eq!(
                table.dispatch_trap(
                    &mut kernel,
                    &mut memory,
                    11,
                    IMPORT_TRAP_BASE,
                    [77, path, ioctl, 0, 0, 0, 0, bytes_returned, 0],
                ),
                Some(1)
            );
            assert_eq!(kernel.threads.get_last_error(11), 0);
            assert_eq!(memory.word(bytes_returned), 0);
        }

        memory.map_bytes(copy_external, &vec![0x33; FILE_COPY_EXTERNAL_SIZE as usize]);
        memory.map_word(copy_external, FILE_COPY_EXTERNAL_SIZE);
        memory.map_bytes(copy_external_out, &[0xa5, 0xa5, 0xa5, 0xa5]);
        for ioctl in [FSCTL_COPY_EXTERNAL_START, FSCTL_COPY_EXTERNAL_COMPLETE] {
            memory.map_word(bytes_returned, 0xfeed_cafe);
            assert_eq!(
                table.dispatch_trap(
                    &mut kernel,
                    &mut memory,
                    11,
                    IMPORT_TRAP_BASE,
                    [
                        77,
                        path,
                        ioctl,
                        copy_external,
                        FILE_COPY_EXTERNAL_SIZE,
                        copy_external_out,
                        4,
                        bytes_returned,
                        0,
                    ],
                ),
                Some(0)
            );
            assert_eq!(kernel.threads.get_last_error(11), ERROR_NOT_SUPPORTED);
            assert_eq!(memory.bytes.get(&(copy_external + 32)).copied(), Some(0x33));
            assert_eq!(memory.bytes.get(&copy_external_out).copied(), Some(0xa5));
            assert_eq!(memory.word(bytes_returned), 0xfeed_cafe);
        }

        memory.map_word(copy_external, FILE_COPY_EXTERNAL_SIZE - 4);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [
                    77,
                    path,
                    FSCTL_COPY_EXTERNAL_START,
                    copy_external,
                    FILE_COPY_EXTERNAL_SIZE,
                    copy_external_out,
                    4,
                    bytes_returned,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);

        memory.map_word(bytes_returned, 0xfeed_cafe);
        memory.map_bytes(volume_info, &vec![0xa5; CE_VOLUME_INFO_SIZE as usize]);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [
                    77,
                    path,
                    UNKNOWN_FSCTL,
                    0,
                    0,
                    volume_info,
                    CE_VOLUME_INFO_SIZE,
                    bytes_returned,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_NOT_SUPPORTED);
        assert_eq!(memory.word(bytes_returned), 0xfeed_cafe);
        assert_eq!(memory.bytes.get(&volume_info).copied(), Some(0xa5));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn fsdmgr_path_imports_route_mounted_mutations_and_attributes() {
        const FILE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;
        const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![
                ImportThunk {
                    thunk_rva: 0x2000,
                    iat_rva: 0x3000,
                    import: ImportBy::Ordinal(54),
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSINT_CreateDirectoryW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FS_GetFileAttributesW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x200c,
                    iat_rva: 0x300c,
                    import: ImportBy::Ordinal(58),
                },
                ImportThunk {
                    thunk_rva: 0x2010,
                    iat_rva: 0x3010,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FS_MoveFileW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2014,
                    iat_rva: 0x3014,
                    import: ImportBy::Ordinal(59),
                },
                ImportThunk {
                    thunk_rva: 0x2018,
                    iat_rva: 0x3018,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FS_DeleteAndRenameFileW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x201c,
                    iat_rva: 0x301c,
                    import: ImportBy::Ordinal(56),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("wince_fsdmgr_path_imports_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        kernel.mount_guest_root("\\ResidentFlash", root.clone());

        fs::write(root.join("attrs.txt"), b"attrs").unwrap();
        fs::write(root.join("move-src.txt"), b"move").unwrap();
        fs::write(root.join("delete-me.txt"), b"delete").unwrap();
        fs::write(root.join("old.txt"), b"old").unwrap();
        fs::write(root.join("replacement.txt"), b"replacement").unwrap();

        let mut memory = TestMemory::default();
        let ext_dir = 0x1000_0000;
        let int_dir = 0x1000_0100;
        let attrs_file = 0x1000_0200;
        let move_src = 0x1000_0300;
        let move_dst = 0x1000_0400;
        let delete_file = 0x1000_0500;
        let old_file = 0x1000_0600;
        let replacement_file = 0x1000_0700;
        memory.map_wide_z(ext_dir, "\\ResidentFlash\\ext-dir");
        memory.map_wide_z(int_dir, "\\ResidentFlash\\int-dir");
        memory.map_wide_z(attrs_file, "\\ResidentFlash\\attrs.txt");
        memory.map_wide_z(move_src, "\\ResidentFlash\\move-src.txt");
        memory.map_wide_z(move_dst, "\\ResidentFlash\\move-dst.txt");
        memory.map_wide_z(delete_file, "\\ResidentFlash\\delete-me.txt");
        memory.map_wide_z(old_file, "\\ResidentFlash\\old.txt");
        memory.map_wide_z(replacement_file, "\\ResidentFlash\\replacement.txt");

        assert_eq!(
            table.dispatch_trap(&mut kernel, &mut memory, 11, IMPORT_TRAP_BASE, [ext_dir, 0],),
            Some(1)
        );
        assert!(root.join("ext-dir").is_dir());

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [int_dir, 0],
            ),
            Some(1)
        );
        assert!(root.join("int-dir").is_dir());

        let attrs = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [ext_dir],
            )
            .unwrap();
        assert_eq!(attrs & FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_DIRECTORY);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3,
                [attrs_file, FILE_ATTRIBUTE_READONLY],
            ),
            Some(1)
        );
        assert!(
            fs::metadata(root.join("attrs.txt"))
                .unwrap()
                .permissions()
                .readonly()
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 4,
                [move_src, move_dst],
            ),
            Some(1)
        );
        assert!(!root.join("move-src.txt").exists());
        assert_eq!(fs::read(root.join("move-dst.txt")).unwrap(), b"move");

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 5,
                [delete_file],
            ),
            Some(1)
        );
        assert!(!root.join("delete-me.txt").exists());

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 6,
                [old_file, replacement_file],
            ),
            Some(1)
        );
        assert_eq!(fs::read(root.join("old.txt")).unwrap(), b"replacement");
        assert!(!root.join("replacement.txt").exists());

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 7,
                [ext_dir],
            ),
            Some(1)
        );
        assert!(!root.join("ext-dir").exists());

        let mut perms = fs::metadata(root.join("attrs.txt")).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(root.join("attrs.txt"), perms).unwrap();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn fsdmgr_path_query_imports_route_capacity_find_create_and_system_flags() {
        const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0000_0020;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x0000_0004;
        const GENERIC_READ: u32 = 0x8000_0000;
        const GENERIC_WRITE: u32 = 0x4000_0000;
        const CREATE_ALWAYS: u32 = 2;
        const OPEN_EXISTING: u32 = 3;
        const ERROR_DIRECTORY: u32 = 267;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![
                ImportThunk {
                    thunk_rva: 0x2000,
                    iat_rva: 0x3000,
                    import: ImportBy::Ordinal(62),
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_FindFirstFileW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Ordinal(64),
                },
                ImportThunk {
                    thunk_rva: 0x200c,
                    iat_rva: 0x300c,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_CreateFileW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2010,
                    iat_rva: 0x3010,
                    import: ImportBy::Ordinal(66),
                },
                ImportThunk {
                    thunk_rva: 0x2014,
                    iat_rva: 0x3014,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FS_IsSystemFileW".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();

        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "wince_fsdmgr_path_query_imports_{}",
                std::process::id()
            ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("payload.bin"), b"payload").unwrap();
        fs::write(root.join("system.bin"), b"system").unwrap();
        kernel.files.mount(MountConfig {
            name: Some("Resident Flash".to_owned()),
            device_name: None,
            bus_name: None,
            guest_root: r"\ResidentFlash".to_owned(),
            host_root: Some(root.clone()),
            total_mbytes: 128,
            free_mbytes: 64,
            writable: true,
            removable: false,
            system: true,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: Vec::new(),
            registry_subkey: None,
        });

        let mut memory = TestMemory::default();
        let dir_path = 0x1000_0000;
        let payload_path = 0x1000_0100;
        let system_path = 0x1000_0200;
        let pattern_path = 0x1000_0300;
        let created_path = 0x1000_0400;
        let internal_created_path = 0x1000_0500;
        let free_available = 0x1000_1000;
        let total_bytes = 0x1000_1010;
        let total_free = 0x1000_1020;
        let find_data = 0x1000_2000;
        memory.map_wide_z(dir_path, "\\ResidentFlash");
        memory.map_wide_z(payload_path, "\\ResidentFlash\\payload.bin");
        memory.map_wide_z(system_path, "\\ResidentFlash\\system.bin");
        memory.map_wide_z(pattern_path, "\\ResidentFlash\\payload.bin");
        memory.map_wide_z(created_path, "\\ResidentFlash\\created.bin");
        memory.map_wide_z(
            internal_created_path,
            "\\ResidentFlash\\internal-created.bin",
        );
        for ptr in [free_available, total_bytes, total_free] {
            memory.map_word(ptr, 0);
            memory.map_word(ptr + 4, 0);
        }
        for offset in (0..=36).step_by(4) {
            memory.map_word(find_data + offset, 0);
        }
        memory.map_wide_buffer(find_data + 40, 260);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [dir_path, free_available, total_bytes, total_free],
            ),
            Some(1)
        );
        assert_eq!(memory.word(free_available), 64 * 1024 * 1024);
        assert_eq!(memory.word(free_available + 4), 0);
        assert_eq!(memory.word(total_bytes), 128 * 1024 * 1024);
        assert_eq!(memory.word(total_bytes + 4), 0);
        assert_eq!(memory.word(total_free), 64 * 1024 * 1024);
        assert_eq!(memory.word(total_free + 4), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [payload_path, free_available, total_bytes, total_free],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_DIRECTORY);

        let find_handle = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [pattern_path, find_data, 560],
            )
            .unwrap();
        assert_ne!(find_handle, u32::MAX);
        assert_eq!(
            memory.word(find_data),
            FILE_ATTRIBUTE_ARCHIVE | FILE_ATTRIBUTE_SYSTEM
        );
        assert_eq!(memory.word(find_data + 32), 7);
        assert_eq!(memory.read_wide_z(find_data + 40, 260), "payload.bin");

        let internal_find_handle = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [system_path, find_data, 560],
            )
            .unwrap();
        assert_ne!(internal_find_handle, u32::MAX);
        assert_eq!(memory.read_wide_z(find_data + 40, 260), "system.bin");

        let opened = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3,
                [payload_path, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0],
            )
            .unwrap();
        assert_ne!(opened, u32::MAX);

        let created = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 4,
                [
                    internal_created_path,
                    GENERIC_WRITE,
                    0,
                    0,
                    CREATE_ALWAYS,
                    0,
                    0,
                ],
            )
            .unwrap();
        assert_ne!(created, u32::MAX);
        assert!(root.join("internal-created.bin").exists());

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 5,
                [system_path],
            ),
            Some(1)
        );

        let _ = kernel.close_handle(find_handle);
        let _ = kernel.close_handle(internal_find_handle);
        let _ = kernel.close_handle(opened);
        let _ = kernel.close_handle(created);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn fsdmgr_mount_table_imports_query_volume_name_and_flags() {
        const AFS_FLAG_HIDDEN: u32 = 0x0001;
        const AFS_FLAG_SYSTEM: u32 = 0x0020;
        const AFS_FLAG_PERMANENT: u32 = 0x0040;
        const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_GetVolumeName".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(37),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let root =
            std::env::temp_dir().join(format!("wince_fsdmgr_mount_query_{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        kernel.files.mount(MountConfig {
            name: Some("Resident Flash Store".to_owned()),
            device_name: None,
            bus_name: None,
            guest_root: r"\ResidentFlash".to_owned(),
            host_root: Some(root.clone()),
            total_mbytes: 128,
            free_mbytes: 64,
            writable: true,
            removable: false,
            system: true,
            hidden: true,
            interface_classes: Vec::new(),
            registry_roots: Vec::new(),
            registry_subkey: None,
        });
        let volume = kernel
            .create_volume_handle_for_guest_root(r"\ResidentFlash")
            .unwrap();

        let mut memory = TestMemory::default();
        let name_ptr = 0x1000_0000;
        let flags_ptr = 0x1000_0100;
        memory.map_wide_buffer(name_ptr, 32);
        memory.map_word(flags_ptr, 0xfeed_cafe);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [volume, name_ptr, 32],
            ),
            Some("ResidentFlash".encode_utf16().count() as u32)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.read_wide_z(name_ptr, 32), "ResidentFlash");

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [volume, flags_ptr],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            memory.word(flags_ptr),
            AFS_FLAG_HIDDEN | AFS_FLAG_SYSTEM | AFS_FLAG_PERMANENT
        );

        let short_name_ptr = 0x1000_0200;
        memory.map_wide_z(short_name_ptr, "unchanged");
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [
                    volume,
                    short_name_ptr,
                    "ResidentFlash".encode_utf16().count() as u32,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INSUFFICIENT_BUFFER);
        assert_eq!(memory.read_wide_z(short_name_ptr, 16), "unchanged");

        let invalid_flags_ptr = 0x1000_0300;
        memory.map_word(invalid_flags_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [0x1234_5678, invalid_flags_ptr],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_INVALID_PARAMETER
        );
        assert_eq!(memory.word(invalid_flags_ptr), 0xfeed_cafe);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn fsdmgr_register_volume_maps_disk_pointer_to_volume_handle() {
        const ERROR_OUT_OF_STRUCTURES: u32 = 84;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_RegisterVolume".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(21),
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_GetVolumeName".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x200c,
                    iat_rva: 0x300c,
                    import: ImportBy::Ordinal(10),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let disk_ptr = 0x1000_2000;
        let mount_name_ptr = 0x1000_3000;
        let name_out = 0x1000_4000;
        memory.map_wide_z(mount_name_ptr, r"\Fsd Card");
        memory.map_wide_buffer(name_out, 32);

        let volume = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, mount_name_ptr, 0x0bad_f00d],
            )
            .unwrap();
        assert_ne!(volume, 0);
        assert_eq!(kernel.threads.get_last_error(11), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr],
            ),
            Some(volume)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [volume, name_out, 32],
            ),
            Some("Fsd Card".encode_utf16().count() as u32)
        );
        assert_eq!(memory.read_wide_z(name_out, 32), "Fsd Card");

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, mount_name_ptr, 0x0bad_f00d],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_ALREADY_EXISTS
        );

        let second_disk_ptr = disk_ptr + 4;
        let second_volume = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [second_disk_ptr, mount_name_ptr, 0x0bad_f002],
            )
            .unwrap();
        assert_ne!(second_volume, 0);
        assert_ne!(second_volume, volume);
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [second_volume, name_out, 32],
            ),
            Some("Fsd Card2".encode_utf16().count() as u32)
        );
        assert_eq!(memory.read_wide_z(name_out, 32), "Fsd Card2");

        for suffix in 3..=9 {
            let next_volume = table
                .dispatch_trap(
                    &mut kernel,
                    &mut memory,
                    11,
                    IMPORT_TRAP_BASE,
                    [disk_ptr + suffix * 4, mount_name_ptr, 0x0bad_f000 + suffix],
                )
                .unwrap();
            assert_ne!(next_volume, 0, "suffix {suffix} should still register");
            assert_eq!(kernel.threads.get_last_error(11), 0);
        }
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr + 0x40, mount_name_ptr, 0x0bad_f010],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_OUT_OF_STRUCTURES);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3,
                [volume],
            ),
            Some(0)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr],
            ),
            Some(volume)
        );
    }

    #[test]
    fn fsdmgr_null_cache_imports_track_ids_and_nullcache_results() {
        const IOCTL_DISK_DELETE_SECTORS: u32 = 0x0007_1c4c;
        const UNSUPPORTED_IOCTL: u32 = 0xfeed_cafe;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_CreateCache".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(9),
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Ordinal(30),
                },
                ImportThunk {
                    thunk_rva: 0x200c,
                    iat_rva: 0x300c,
                    import: ImportBy::Ordinal(14),
                },
                ImportThunk {
                    thunk_rva: 0x2010,
                    iat_rva: 0x3010,
                    import: ImportBy::Ordinal(32),
                },
                ImportThunk {
                    thunk_rva: 0x2014,
                    iat_rva: 0x3014,
                    import: ImportBy::Ordinal(24),
                },
                ImportThunk {
                    thunk_rva: 0x2018,
                    iat_rva: 0x3018,
                    import: ImportBy::Ordinal(4),
                },
                ImportThunk {
                    thunk_rva: 0x201c,
                    iat_rva: 0x301c,
                    import: ImportBy::Ordinal(5),
                },
                ImportThunk {
                    thunk_rva: 0x2020,
                    iat_rva: 0x3020,
                    import: ImportBy::Ordinal(3),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let disk_ptr = 0x1000_2000;
        let write_sector_ptr = 0x1000_4000;
        let read_sector_ptr = 0x1000_5000;

        let create = IMPORT_TRAP_BASE;
        let delete = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE;
        let resize = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2;
        let flush = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3;
        let sync = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 4;
        let invalidate = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 5;
        let cached_read = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 6;
        let cached_write = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 7;
        let cache_ioctl = IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 8;

        let cache_id = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                create,
                [disk_ptr, 0, 0, 0, 512, 0],
            )
            .unwrap();
        assert_eq!(cache_id, 0);
        assert_eq!(kernel.threads.get_last_error(11), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                create,
                [disk_ptr + 4, 0, 0, 0, 512, 0],
            ),
            Some(1)
        );

        assert_eq!(
            table.dispatch_trap(&mut kernel, &mut memory, 11, delete, [cache_id]),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(&mut kernel, &mut memory, 11, delete, [0xdead_beef]),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_INVALID_PARAMETER
        );

        let cache_id = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                create,
                [disk_ptr, 0, 0, 0, 512, 0],
            )
            .unwrap();
        assert_eq!(cache_id, 0);

        assert_eq!(
            table.dispatch_trap(&mut kernel, &mut memory, 11, resize, [0xdead_beef, 64, 0]),
            Some(0)
        );
        assert_eq!(
            table.dispatch_trap(&mut kernel, &mut memory, 11, sync, [0xdead_beef, 0]),
            Some(0)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                invalidate,
                [0xdead_beef, 0, 0]
            ),
            Some(0)
        );

        assert_eq!(
            table.dispatch_trap(&mut kernel, &mut memory, 11, flush, [cache_id, 0]),
            Some(0)
        );
        let mut sector_bytes = vec![0; 512];
        sector_bytes[..13].copy_from_slice(b"sector-write!");
        memory.map_bytes(write_sector_ptr, &sector_bytes);
        memory.map_bytes(read_sector_ptr, &vec![0xa5; 512]);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                cached_write,
                [cache_id, 7, 1, write_sector_ptr, 0],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                cached_read,
                [cache_id, 7, 1, read_sector_ptr, 0]
            ),
            Some(0)
        );
        let mut read_back = vec![0; 512];
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert_eq!(&read_back[..13], b"sector-write!");
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                cached_write,
                [0xdead_beef, 0, 1, 0, 0],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                cache_ioctl,
                [0xdead_beef, IOCTL_DISK_DELETE_SECTORS, 0, 0, 0, 0, 0],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                cache_ioctl,
                [cache_id, IOCTL_DISK_DELETE_SECTORS, 0, 0, 0, 0, 0],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                cache_ioctl,
                [cache_id, UNSUPPORTED_IOCTL, 0, 0, 0, 0, 0],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_NOT_SUPPORTED
        );
    }

    #[test]
    fn fsdmgr_disk_support_imports_round_trip_sparse_sectors_and_info() {
        const DISK_IOCTL_GETINFO: u32 = 1;
        const DISK_IOCTL_FORMAT_MEDIA: u32 = 6;
        const IOCTL_DISK_FORMAT_VOLUME: u32 = 0x0007_0220;
        const IOCTL_DISK_SCAN_VOLUME: u32 = 0x0007_0224;
        const IOCTL_DISK_DEVICE_INFO: u32 = 0x0007_1800;
        const IOCTL_DISK_GETINFO: u32 = 0x0007_1c00;
        const IOCTL_DISK_SETINFO: u32 = 0x0007_1c04;
        const IOCTL_DISK_INITIALIZED: u32 = 0x0007_1c10;
        const IOCTL_DISK_SET_STANDBY_TIMER: u32 = 0x0007_1c18;
        const IOCTL_DISK_STANDBY_NOW: u32 = 0x0007_1c1c;
        const IOCTL_DISK_GETNAME: u32 = 0x0007_1c20;
        const IOCTL_DISK_GET_STORAGEID: u32 = 0x0007_1c24;
        const IOCTL_DISK_DELETE_CLUSTER: u32 = 0x0007_1c40;
        const IOCTL_DISK_READ_CDROM: u32 = 0x0007_1c44;
        const IOCTL_DISK_WRITE_CDROM: u32 = 0x0007_1c48;
        const IOCTL_DISK_DELETE_SECTORS: u32 = 0x0007_1c4c;
        const IOCTL_DISK_GET_SECTOR_ADDR: u32 = 0x0007_1c50;
        const IOCTL_DISK_FLUSH_CACHE: u32 = 0x0007_1c54;
        const IOCTL_DISK_COPY_EXTERNAL_START: u32 = 0x0007_1c58;
        const IOCTL_DISK_COPY_EXTERNAL_COMPLETE: u32 = 0x0007_1c5c;
        const IOCTL_DISK_GETPMTIMINGS: u32 = 0x0007_1c60;
        const IOCTL_DISK_SECURE_WIPE: u32 = 0x0007_1c64;
        const IOCTL_DISK_SET_SECURE_WIPE_FLAG: u32 = 0x0007_1c80;
        const IOCTL_FMD_SET_XIPMODE: u32 = 0x0007_1f80;
        const IOCTL_FMD_LOCK_BLOCKS: u32 = 0x0007_1f84;
        const IOCTL_FMD_UNLOCK_BLOCKS: u32 = 0x0007_1f88;
        const IOCTL_FMD_GET_INTERFACE: u32 = 0x0007_1f8c;
        const IOCTL_FMD_GET_XIPMODE: u32 = 0x0007_1f90;
        const IOCTL_FMD_READ_RESERVED: u32 = 0x0007_1f94;
        const IOCTL_FMD_WRITE_RESERVED: u32 = 0x0007_1f98;
        const IOCTL_FMD_GET_RESERVED_TABLE: u32 = 0x0007_1f9c;
        const IOCTL_FMD_SET_REGION_TABLE: u32 = 0x0007_1fa0;
        const IOCTL_FMD_SET_SECTORSIZE: u32 = 0x0007_1fa4;
        const IOCTL_FMD_RAW_WRITE_BLOCKS: u32 = 0x0007_1fa8;
        const IOCTL_FMD_GET_RAW_BLOCK_SIZE: u32 = 0x0007_1fac;
        const IOCTL_FMD_GET_INFO: u32 = 0x0007_1fb0;
        const DISK_COPY_EXTERNAL_SIZE: u32 = 552;
        const DISK_POWER_TIMINGS_SIZE: u32 = 68;
        const ERROR_INVALID_PARAMETER: u32 = 87;
        const ERROR_NOT_SUPPORTED: u32 = 50;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![
                ImportThunk {
                    thunk_rva: 0x2000,
                    iat_rva: 0x3000,
                    import: ImportBy::Ordinal(35),
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(25),
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Ordinal(12),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let disk_ptr = 0x1000_6000;
        let write_sector_ptr = 0x1000_7000;
        let read_sector_ptr = 0x1000_8000;
        let disk_info_ptr = 0x1000_9000;
        let bytes_returned_ptr = 0x1000_a000;
        let delete_info_ptr = 0x1000_b000;
        let disk_name_ptr = 0x1000_c000;
        let device_info_ptr = 0x1000_d000;
        let storage_id_ptr = 0x1000_e000;
        let sector_list_ptr = 0x1000_f000;
        let sector_addr_ptr = 0x1001_0000;
        let power_timings_ptr = 0x1001_1000;
        let copy_external_ptr = 0x1001_2000;
        let copy_external_out_ptr = 0x1001_3000;
        let fmd_info_ptr = 0x1001_4000;
        let fmd_reserved_out_ptr = 0x1001_5000;
        let fmd_xip_mode_ptr = 0x1001_6000;
        let fmd_block_lock_ptr = 0x1001_7000;
        let fmd_sector_size_ptr = 0x1001_8000;
        let fmd_raw_write_req_ptr = 0x1001_9000;
        let fmd_raw_write_buffer_ptr = 0x1001_a000;
        let fmd_reserved_req_ptr = 0x1001_b000;
        let fmd_reserved_buffer_ptr = 0x1001_c000;
        let fmd_region_table_ptr = 0x1001_d000;
        let fmd_interface_ptr = 0x1001_e000;

        let mut sector_bytes = vec![0; 512];
        sector_bytes[..17].copy_from_slice(b"direct-disk-write");
        memory.map_bytes(write_sector_ptr, &sector_bytes);
        memory.map_bytes(read_sector_ptr, &vec![0x5a; 512]);
        for offset in (0..24).step_by(4) {
            memory.map_word(disk_info_ptr + offset, 0xfeed_cafe);
        }
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, 11, 1, write_sector_ptr, 512],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 11, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        let mut read_back = vec![0; 512];
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert_eq!(&read_back[..17], b"direct-disk-write");

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 99, 0, 0, 0],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, 99, 0, 0, 0],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [0, 99, 0, 0, 0],
            ),
            Some(ERROR_INVALID_PARAMETER)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [0, 99, 0, 0, 0],
            ),
            Some(ERROR_INVALID_PARAMETER)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    DISK_IOCTL_GETINFO,
                    disk_info_ptr,
                    24,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(disk_info_ptr + 4), 512);
        assert_eq!(memory.word(bytes_returned_ptr), 24);

        for (index, value) in [0x4444, 1024, 4, 8, 16, 0x0000_0008]
            .into_iter()
            .enumerate()
        {
            memory.map_word(disk_info_ptr + (index as u32 * 4), value);
        }
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_SETINFO,
                    disk_info_ptr,
                    24,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        for offset in (0..24).step_by(4) {
            memory.map_word(disk_info_ptr + offset, 0xfeed_cafe);
        }
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    DISK_IOCTL_GETINFO,
                    disk_info_ptr,
                    24,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(disk_info_ptr), 0x4444);
        assert_eq!(memory.word(disk_info_ptr + 4), 1024);
        assert_eq!(memory.word(disk_info_ptr + 8), 4);
        assert_eq!(memory.word(disk_info_ptr + 12), 8);
        assert_eq!(memory.word(disk_info_ptr + 16), 16);
        assert_eq!(memory.word(disk_info_ptr + 20), 0x0000_0008);
        assert_eq!(memory.word(bytes_returned_ptr), 24);

        for offset in (0..24).step_by(4) {
            memory.map_word(disk_info_ptr + offset, 0xfeed_cafe);
        }
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_GETINFO,
                    0,
                    0,
                    disk_info_ptr,
                    24,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(disk_info_ptr), 0x4444);
        assert_eq!(memory.word(disk_info_ptr + 4), 1024);
        assert_eq!(memory.word(disk_info_ptr + 8), 4);
        assert_eq!(memory.word(disk_info_ptr + 12), 8);
        assert_eq!(memory.word(disk_info_ptr + 16), 16);
        assert_eq!(memory.word(disk_info_ptr + 20), 0x0000_0008);
        assert_eq!(memory.word(bytes_returned_ptr), 24);

        memory.map_wide_buffer(disk_name_ptr, 32);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_GETNAME,
                    0,
                    0,
                    disk_name_ptr,
                    64,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.read_wide_z(disk_name_ptr, 32), "Storage Card");
        assert_eq!(memory.word(bytes_returned_ptr), 26);

        memory.map_word(device_info_ptr, 0);
        memory.map_wide_buffer(device_info_ptr + 4, 32);
        for offset in [68, 72, 76] {
            memory.map_word(device_info_ptr + offset, 0);
        }
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_DEVICE_INFO,
                    device_info_ptr,
                    80,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(device_info_ptr), 80);
        assert_eq!(memory.read_wide_z(device_info_ptr + 4, 32), "SyntheticDisk");
        assert_eq!(memory.word(device_info_ptr + 68), 1);
        assert_eq!(memory.word(device_info_ptr + 72), 1 << 29);
        assert_eq!(memory.word(device_info_ptr + 76), 1);
        assert_eq!(memory.word(bytes_returned_ptr), 80);

        for offset in (0..16).step_by(4) {
            memory.map_word(storage_id_ptr + offset, 0xfeed_cafe);
        }
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_GET_STORAGEID,
                    0,
                    0,
                    storage_id_ptr,
                    16,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(storage_id_ptr), 16);
        assert_eq!(memory.word(storage_id_ptr + 4), 3);
        assert_eq!(memory.word(storage_id_ptr + 8), 0);
        assert_eq!(memory.word(storage_id_ptr + 12), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 16);

        memory.map_bytes(fmd_xip_mode_ptr, &[0xfe]);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_XIPMODE,
                    0,
                    0,
                    fmd_xip_mode_ptr,
                    1,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.read_u8(fmd_xip_mode_ptr).unwrap(), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_bytes(fmd_xip_mode_ptr, &[1]);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_SET_XIPMODE,
                    fmd_xip_mode_ptr,
                    1,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(kernel.fsdmgr_fmd_xip_mode(disk_ptr), Some(true));
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_bytes(fmd_xip_mode_ptr, &[0xfe]);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_XIPMODE,
                    0,
                    0,
                    fmd_xip_mode_ptr,
                    1,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(memory.read_u8(fmd_xip_mode_ptr).unwrap(), 1);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_SET_XIPMODE,
                    fmd_xip_mode_ptr,
                    0,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(kernel.fsdmgr_fmd_xip_mode(disk_ptr), Some(true));

        memory.map_word(fmd_block_lock_ptr, 7);
        memory.map_word(fmd_block_lock_ptr + 4, 3);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_LOCK_BLOCKS,
                    fmd_block_lock_ptr,
                    8,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(kernel.fsdmgr_fmd_block_locked(disk_ptr, 6), Some(false));
        assert_eq!(kernel.fsdmgr_fmd_block_locked(disk_ptr, 7), Some(true));
        assert_eq!(kernel.fsdmgr_fmd_block_locked(disk_ptr, 9), Some(true));
        assert_eq!(kernel.fsdmgr_fmd_block_locked(disk_ptr, 10), Some(false));

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_UNLOCK_BLOCKS,
                    fmd_block_lock_ptr,
                    8,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(kernel.fsdmgr_fmd_block_locked(disk_ptr, 7), Some(false));

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_LOCK_BLOCKS,
                    fmd_block_lock_ptr,
                    7,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(kernel.fsdmgr_fmd_block_locked(disk_ptr, 7), Some(false));

        for offset in (0..56).step_by(4) {
            memory.map_word(fmd_interface_ptr + offset, 0xfeed_cafe);
        }
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_INTERFACE,
                    0,
                    0,
                    fmd_interface_ptr,
                    55,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(memory.word(fmd_interface_ptr), 0xfeed_cafe);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_INTERFACE,
                    0,
                    0,
                    fmd_interface_ptr,
                    56,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(fmd_interface_ptr), 56);
        for offset in (4..56).step_by(4) {
            assert_eq!(memory.word(fmd_interface_ptr + offset), 0);
        }
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_word(fmd_sector_size_ptr, 2048);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_SET_SECTORSIZE,
                    fmd_sector_size_ptr,
                    4,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(kernel.fsdmgr_fmd_sector_size(disk_ptr), Some(2048));

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_SET_SECTORSIZE,
                    fmd_sector_size_ptr,
                    3,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(kernel.fsdmgr_fmd_sector_size(disk_ptr), Some(2048));

        memory.map_word(fmd_reserved_out_ptr, 0xfeed_cafe);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_RESERVED_TABLE,
                    0,
                    0,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);
        assert_eq!(memory.word(fmd_reserved_out_ptr), 0xfeed_cafe);

        memory.map_bytes(fmd_reserved_req_ptr, b"BOOT\0\0\0\0");
        memory.map_word(fmd_reserved_req_ptr + 8, 0);
        memory.map_word(fmd_reserved_req_ptr + 12, 4);
        memory.map_word(fmd_reserved_req_ptr + 16, fmd_reserved_buffer_ptr);
        memory.map_bytes(fmd_reserved_buffer_ptr, &[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_READ_RESERVED,
                    fmd_reserved_req_ptr,
                    19,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        let mut reserved_bytes = [0; 4];
        memory
            .read_bytes(fmd_reserved_buffer_ptr, &mut reserved_bytes)
            .unwrap();
        assert_eq!(reserved_bytes, [0x11, 0x22, 0x33, 0x44]);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_READ_RESERVED,
                    fmd_reserved_req_ptr,
                    20,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_NOT_SUPPORTED);
        memory
            .read_bytes(fmd_reserved_buffer_ptr, &mut reserved_bytes)
            .unwrap();
        assert_eq!(reserved_bytes, [0x11, 0x22, 0x33, 0x44]);

        memory.map_word(fmd_reserved_req_ptr + 16, 0x1001_d000);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_WRITE_RESERVED,
                    fmd_reserved_req_ptr,
                    20,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);

        memory.map_word(fmd_reserved_req_ptr + 16, fmd_reserved_buffer_ptr);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_WRITE_RESERVED,
                    fmd_reserved_req_ptr,
                    20,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_NOT_SUPPORTED);
        memory
            .read_bytes(fmd_reserved_buffer_ptr, &mut reserved_bytes)
            .unwrap();
        assert_eq!(reserved_bytes, [0x11, 0x22, 0x33, 0x44]);

        memory.map_word(fmd_reserved_out_ptr, 0xfeed_cafe);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_RESERVED_TABLE,
                    0,
                    0,
                    fmd_reserved_out_ptr,
                    4,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);
        assert_eq!(memory.word(fmd_reserved_out_ptr), 0xfeed_cafe);

        memory.map_word(fmd_reserved_out_ptr, 0xfeed_cafe);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_RAW_BLOCK_SIZE,
                    0,
                    0,
                    fmd_reserved_out_ptr,
                    4,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(fmd_reserved_out_ptr), 1024);
        assert_eq!(memory.word(bytes_returned_ptr), 4);

        let mut raw_block = vec![0; 1024];
        raw_block[..18].copy_from_slice(b"raw-fmd-block-data");
        memory.map_bytes(fmd_raw_write_buffer_ptr, &raw_block);
        memory.map_word(fmd_raw_write_req_ptr, 15);
        memory.map_word(fmd_raw_write_req_ptr + 4, 1);
        memory.map_word(fmd_raw_write_req_ptr + 8, fmd_raw_write_buffer_ptr);
        memory.map_word(fmd_raw_write_req_ptr + 12, 1024);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_RAW_WRITE_BLOCKS,
                    fmd_raw_write_req_ptr,
                    16,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_bytes(read_sector_ptr, &vec![0x5a; 512]);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 15, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert_eq!(&read_back[..18], b"raw-fmd-block-data");

        memory.map_word(fmd_raw_write_req_ptr, 0x4444);
        memory.map_word(fmd_raw_write_req_ptr + 4, 1);
        memory.map_word(fmd_raw_write_req_ptr + 12, 1024);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_RAW_WRITE_BLOCKS,
                    fmd_raw_write_req_ptr,
                    16,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_bytes(read_sector_ptr, &vec![0x5a; 512]);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 0x4444, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert!(read_back.iter().all(|byte| *byte == 0));

        memory.map_word(fmd_raw_write_req_ptr, 0xffff_ffff);
        memory.map_word(fmd_raw_write_req_ptr + 4, 2);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_RAW_WRITE_BLOCKS,
                    fmd_raw_write_req_ptr,
                    16,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);

        memory.map_word(fmd_raw_write_req_ptr, 16);
        memory.map_word(fmd_raw_write_req_ptr + 12, 1023);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_RAW_WRITE_BLOCKS,
                    fmd_raw_write_req_ptr,
                    16,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);

        memory.map_bytes(read_sector_ptr, &vec![0x5a; 512]);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 16, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert!(read_back.iter().all(|byte| *byte == 0));

        for (index, value) in [0, 0, 8, 8, 4, 1024, 1, 2, 8, 8, 8, 4, 1024, 0]
            .into_iter()
            .enumerate()
        {
            memory.map_word(fmd_region_table_ptr + (index as u32 * 4), value);
        }
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_SET_REGION_TABLE,
                    fmd_region_table_ptr,
                    55,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(memory.word(bytes_returned_ptr), 0);
        assert_eq!(kernel.fsdmgr_fmd_region_count(disk_ptr), Some(0));

        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_SET_REGION_TABLE,
                    fmd_region_table_ptr,
                    56,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 56);
        assert_eq!(kernel.fsdmgr_fmd_region_count(disk_ptr), Some(2));

        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_RAW_BLOCK_SIZE,
                    0,
                    0,
                    fmd_reserved_out_ptr,
                    3,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        for offset in (0..20).step_by(4) {
            memory.map_word(fmd_info_ptr + offset, 0xfeed_cafe);
        }
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_INFO,
                    0,
                    0,
                    fmd_info_ptr,
                    20,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(fmd_info_ptr), 0xfeed_cafe);
        assert_eq!(memory.word(fmd_info_ptr + 4), 1);
        assert_eq!(memory.word(fmd_info_ptr + 8), 0);
        assert_eq!(memory.word(fmd_info_ptr + 12), 2);
        assert_eq!(memory.word(fmd_info_ptr + 16), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 20);

        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_FMD_GET_INFO,
                    0,
                    0,
                    fmd_info_ptr,
                    16,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_word(sector_list_ptr, 11);
        memory.map_word(sector_list_ptr + 4, 12);
        memory.map_word(sector_addr_ptr, 0xfeed_cafe);
        memory.map_word(sector_addr_ptr + 4, 0xfeed_cafe);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_GET_SECTOR_ADDR,
                    sector_list_ptr,
                    8,
                    sector_addr_ptr,
                    8,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_NOT_SUPPORTED);
        assert_eq!(memory.word(sector_addr_ptr), 0xfeed_cafe);
        assert_eq!(memory.word(sector_addr_ptr + 4), 0xfeed_cafe);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_FLUSH_CACHE,
                    0,
                    0,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_word(copy_external_ptr, DISK_COPY_EXTERNAL_SIZE);
        memory.map_word(copy_external_ptr + 548, 16);
        memory.map_word(copy_external_ptr + DISK_COPY_EXTERNAL_SIZE, 31);
        memory.map_word(copy_external_ptr + DISK_COPY_EXTERNAL_SIZE + 4, 2);
        memory.map_word(copy_external_ptr + DISK_COPY_EXTERNAL_SIZE + 8, 41);
        memory.map_word(copy_external_ptr + DISK_COPY_EXTERNAL_SIZE + 12, 3);
        memory.map_word(copy_external_out_ptr, 0xfeed_cafe);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        for ioctl in [
            IOCTL_DISK_COPY_EXTERNAL_START,
            IOCTL_DISK_COPY_EXTERNAL_COMPLETE,
        ] {
            memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
            assert_eq!(
                table.dispatch_trap(
                    &mut kernel,
                    &mut memory,
                    11,
                    IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                    [
                        disk_ptr,
                        ioctl,
                        copy_external_ptr,
                        DISK_COPY_EXTERNAL_SIZE + 16,
                        copy_external_out_ptr,
                        4,
                        bytes_returned_ptr,
                        0,
                    ],
                ),
                Some(0)
            );
            assert_eq!(kernel.threads.get_last_error(11), ERROR_NOT_SUPPORTED);
            assert_eq!(memory.word(copy_external_ptr + DISK_COPY_EXTERNAL_SIZE), 31);
            assert_eq!(
                memory.word(copy_external_ptr + DISK_COPY_EXTERNAL_SIZE + 8),
                41
            );
            assert_eq!(memory.word(copy_external_out_ptr), 0xfeed_cafe);
            assert_eq!(memory.word(bytes_returned_ptr), 0);
        }

        memory.map_word(copy_external_ptr + 548, 12);
        for ioctl in [
            IOCTL_DISK_COPY_EXTERNAL_START,
            IOCTL_DISK_COPY_EXTERNAL_COMPLETE,
        ] {
            assert_eq!(
                table.dispatch_trap(
                    &mut kernel,
                    &mut memory,
                    11,
                    IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                    [
                        disk_ptr,
                        ioctl,
                        copy_external_ptr,
                        DISK_COPY_EXTERNAL_SIZE + 12,
                        copy_external_out_ptr,
                        4,
                        bytes_returned_ptr,
                        0,
                    ],
                ),
                Some(0)
            );
            assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        }

        for offset in (0..DISK_POWER_TIMINGS_SIZE).step_by(4) {
            memory.map_word(power_timings_ptr + offset, 0xfeed_cafe);
        }
        memory.map_word(power_timings_ptr, DISK_POWER_TIMINGS_SIZE);
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_GETPMTIMINGS,
                    power_timings_ptr,
                    DISK_POWER_TIMINGS_SIZE,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(power_timings_ptr), DISK_POWER_TIMINGS_SIZE);
        for offset in (4..DISK_POWER_TIMINGS_SIZE).step_by(4) {
            assert_eq!(memory.word(power_timings_ptr + offset), 0);
        }
        assert_eq!(memory.word(bytes_returned_ptr), 0);

        memory.map_word(power_timings_ptr, DISK_POWER_TIMINGS_SIZE - 4);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_GETPMTIMINGS,
                    power_timings_ptr,
                    DISK_POWER_TIMINGS_SIZE,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);

        sector_bytes[..16].copy_from_slice(b"secure-wipe-kept");
        memory.map_bytes(write_sector_ptr, &sector_bytes);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, 13, 1, write_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.map_word(delete_info_ptr, 12);
        memory.map_word(delete_info_ptr + 4, 13);
        memory.map_word(delete_info_ptr + 8, 1);
        memory.map_word(delete_info_ptr + 12, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_SET_SECURE_WIPE_FLAG,
                    delete_info_ptr,
                    16,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_SET_SECURE_WIPE_FLAG,
                    delete_info_ptr,
                    12,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 13, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert_eq!(&read_back[..16], b"secure-wipe-kept");

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_SECURE_WIPE,
                    delete_info_ptr,
                    12,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 13, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert!(read_back.iter().all(|byte| *byte == 0));

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_GET_SECTOR_ADDR,
                    sector_list_ptr,
                    8,
                    sector_addr_ptr,
                    4,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_INITIALIZED,
                    0,
                    0,
                    0,
                    0,
                    bytes_returned_ptr,
                    0
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);

        for ioctl in [
            IOCTL_DISK_SET_STANDBY_TIMER,
            IOCTL_DISK_STANDBY_NOW,
            IOCTL_DISK_DELETE_CLUSTER,
            IOCTL_DISK_READ_CDROM,
            IOCTL_DISK_WRITE_CDROM,
        ] {
            memory.map_word(delete_info_ptr, 0xfeed_cafe);
            memory.map_word(copy_external_out_ptr, 0xfeed_cafe);
            memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
            assert_eq!(
                table.dispatch_trap(
                    &mut kernel,
                    &mut memory,
                    11,
                    IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                    [
                        disk_ptr,
                        ioctl,
                        delete_info_ptr,
                        4,
                        copy_external_out_ptr,
                        4,
                        bytes_returned_ptr,
                        0,
                    ],
                ),
                Some(0)
            );
            assert_eq!(kernel.threads.get_last_error(11), ERROR_NOT_SUPPORTED);
            assert_eq!(memory.word(delete_info_ptr), 0xfeed_cafe);
            assert_eq!(memory.word(copy_external_out_ptr), 0xfeed_cafe);
            assert_eq!(memory.word(bytes_returned_ptr), 0);
        }

        memory.map_word(delete_info_ptr, 12);
        memory.map_word(delete_info_ptr + 4, 11);
        memory.map_word(delete_info_ptr + 8, 1);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_DELETE_SECTORS,
                    delete_info_ptr,
                    16,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_INVALID_PARAMETER);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 11, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert_eq!(&read_back[..17], b"direct-disk-write");
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_DELETE_SECTORS,
                    delete_info_ptr,
                    12,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 11, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert!(read_back.iter().all(|byte| *byte == 0));

        sector_bytes[..17].copy_from_slice(b"scan-volume-keeps");
        memory.map_bytes(write_sector_ptr, &sector_bytes);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, 14, 1, write_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_SCAN_VOLUME,
                    0,
                    0,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 14, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert_eq!(&read_back[..17], b"scan-volume-keeps");

        memory.map_word(bytes_returned_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    IOCTL_DISK_FORMAT_VOLUME,
                    0,
                    0,
                    0,
                    0,
                    bytes_returned_ptr,
                    0,
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(bytes_returned_ptr), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 14, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert!(read_back.iter().all(|byte| *byte == 0));

        sector_bytes[..16].copy_from_slice(b"format-will-zero");
        memory.map_bytes(write_sector_ptr, &sector_bytes);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, 12, 1, write_sector_ptr, 512],
            ),
            Some(0)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [
                    disk_ptr,
                    DISK_IOCTL_FORMAT_MEDIA,
                    0,
                    0,
                    0,
                    0,
                    bytes_returned_ptr,
                    0
                ],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, 12, 1, read_sector_ptr, 512],
            ),
            Some(0)
        );
        memory.read_bytes(read_sector_ptr, &mut read_back).unwrap();
        assert!(read_back.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn fsdmgr_direct_disk_info_and_name_imports_use_ce_metadata_contract() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![
                ImportThunk {
                    thunk_rva: 0x2000,
                    iat_rva: 0x3000,
                    import: ImportBy::Ordinal(16),
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_GetDiskName".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let disk_ptr = 0x1000_6000;
        let disk_info_ptr = 0x1000_7000;
        let disk_name_ptr = 0x1000_8000;

        for offset in (0..24).step_by(4) {
            memory.map_word(disk_info_ptr + offset, 0xfeed_cafe);
        }
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, disk_info_ptr]
            ),
            Some(0)
        );
        assert_eq!(memory.word(disk_info_ptr), 0x0002_0000);
        assert_eq!(memory.word(disk_info_ptr + 4), 512);
        assert_eq!(memory.word(disk_info_ptr + 8), 1);
        assert_eq!(memory.word(disk_info_ptr + 12), 1);
        assert_eq!(memory.word(disk_info_ptr + 16), 0x0002_0000);
        assert_eq!(memory.word(disk_info_ptr + 20), 0);

        memory.map_wide_buffer(disk_name_ptr, 260);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, disk_name_ptr],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.read_wide_z(disk_name_ptr, 260), "Storage Card");

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, 0]
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_INVALID_PARAMETER
        );
    }

    #[test]
    fn fsdmgr_registry_imports_clear_missing_outputs_like_ce() {
        const ERROR_GEN_FAILURE: u32 = 31;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_GetRegistryValue".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(19),
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_GetRegistryFlag".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let disk_ptr = 0x1000_6000;
        let value_name_ptr = 0x1000_7000;
        let dword_out_ptr = 0x1000_8000;
        let string_out_ptr = 0x1000_9000;
        let flag_out_ptr = 0x1000_a000;

        memory.map_wide_z(value_name_ptr, "CacheDll");
        memory.map_word(dword_out_ptr, 0xfeed_cafe);
        memory.map_wide_z(string_out_ptr, "unchanged");
        memory.map_word(flag_out_ptr, 0x0000_00f0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, value_name_ptr, dword_out_ptr],
            ),
            Some(0)
        );
        assert_eq!(memory.word(dword_out_ptr), 0);
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_FILE_NOT_FOUND
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, value_name_ptr, string_out_ptr, 16],
            ),
            Some(0)
        );
        assert_eq!(memory.read_wide_z(string_out_ptr, 16), "");
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_FILE_NOT_FOUND
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [disk_ptr, value_name_ptr, flag_out_ptr, 0x0000_0008],
            ),
            Some(0)
        );
        assert_eq!(memory.word(flag_out_ptr), 0x0000_00f0);
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_FILE_NOT_FOUND
        );

        memory.map_word(dword_out_ptr, 0xfeed_cafe);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [0, value_name_ptr, dword_out_ptr],
            ),
            Some(0)
        );
        assert_eq!(memory.word(dword_out_ptr), 0xfeed_cafe);
        assert_eq!(kernel.threads.get_last_error(11), ERROR_GEN_FAILURE);
    }

    #[test]
    fn fsdmgr_registry_imports_read_configured_logical_disk_roots() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_RegisterVolume".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_GetRegistryValue".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Ordinal(19),
                },
                ImportThunk {
                    thunk_rva: 0x200c,
                    iat_rva: 0x300c,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_GetRegistryFlag".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        kernel.files.mount(MountConfig {
            name: Some("Profiled Store".to_owned()),
            device_name: Some("DSK9:".to_owned()),
            bus_name: None,
            guest_root: r"\Profiled Disk".to_owned(),
            host_root: None,
            total_mbytes: 128,
            free_mbytes: 64,
            writable: true,
            removable: true,
            system: false,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: vec![
                r"HKLM\System\StorageManager\Profiles\Profiled".to_owned(),
                r"HKLM\System\StorageManager".to_owned(),
            ],
            registry_subkey: Some("FATFS".to_owned()),
        });
        kernel.registry.set_value(
            r"HKLM\System\StorageManager\Profiles\Profiled\FATFS",
            "CacheDll",
            RegistryValue::string("profilecache.dll"),
        );
        kernel.registry.set_value(
            r"HKLM\System\StorageManager\Profiles\Profiled\FATFS",
            "EnableFileCache",
            RegistryValue::dword(1),
        );
        kernel.registry.set_value(
            r"HKLM\System\StorageManager\Profiles\Profiled\FATFS",
            "LockIOBuffers",
            RegistryValue::dword(0),
        );
        kernel.registry.set_value(
            r"HKLM\System\StorageManager\FATFS",
            "FallbackValue",
            RegistryValue::dword(0x1234_5678),
        );

        let mut memory = TestMemory::default();
        let disk_ptr = 0x1000_6000;
        let mount_name_ptr = 0x1000_6800;
        let value_name_ptr = 0x1000_7000;
        let dword_out_ptr = 0x1000_8000;
        let string_out_ptr = 0x1000_9000;
        let flag_out_ptr = 0x1000_a000;
        memory.map_wide_z(mount_name_ptr, r"\Profiled Disk");
        memory.map_word(dword_out_ptr, 0);
        memory.map_wide_buffer(string_out_ptr, 32);
        memory.map_word(flag_out_ptr, 0x0000_00f0);

        let volume = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, mount_name_ptr, 0x0bad_f00d],
            )
            .unwrap();
        assert_ne!(volume, 0);
        assert_eq!(kernel.threads.get_last_error(11), 0);

        memory.map_wide_z(value_name_ptr, "FallbackValue");
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_ptr, value_name_ptr, dword_out_ptr],
            ),
            Some(1)
        );
        assert_eq!(memory.word(dword_out_ptr), 0x1234_5678);
        assert_eq!(kernel.threads.get_last_error(11), 0);

        memory.map_wide_z(value_name_ptr, "CacheDll");
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [disk_ptr, value_name_ptr, string_out_ptr, 32],
            ),
            Some(1)
        );
        assert_eq!(memory.read_wide_z(string_out_ptr, 32), "profilecache.dll");
        assert_eq!(kernel.threads.get_last_error(11), 0);

        memory.map_wide_z(value_name_ptr, "EnableFileCache");
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3,
                [disk_ptr, value_name_ptr, flag_out_ptr, 0x0000_0008],
            ),
            Some(1)
        );
        assert_eq!(memory.word(flag_out_ptr), 0x0000_00f8);
        assert_eq!(kernel.threads.get_last_error(11), 0);

        memory.map_wide_z(value_name_ptr, "LockIOBuffers");
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3,
                [disk_ptr, value_name_ptr, flag_out_ptr, 0x0000_0008],
            ),
            Some(1)
        );
        assert_eq!(memory.word(flag_out_ptr), 0x0000_00f0);
        assert_eq!(kernel.threads.get_last_error(11), 0);
    }

    #[test]
    fn fsdmgr_device_handle_and_util_imports_match_ce_helper_shape() {
        const ERROR_GEN_FAILURE: u32 = 31;
        const ERROR_MOD_NOT_FOUND: u32 = 126;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_DeviceHandleToHDSK".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(15),
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_ScanVolume".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let disk_handle = 0x1000_6000;
        let mount_root = r"\Utility Disk";

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_handle]
            ),
            Some(disk_handle)
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_handle, 0],
            ),
            Some(crate::ce::thread::ERROR_FILE_NOT_FOUND)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_FILE_NOT_FOUND
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [0, 0],
            ),
            Some(ERROR_GEN_FAILURE)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_GEN_FAILURE);

        kernel.files.mount(MountConfig {
            name: Some("Utility Store".to_owned()),
            device_name: Some("DSK8:".to_owned()),
            bus_name: None,
            guest_root: mount_root.to_owned(),
            host_root: None,
            total_mbytes: 128,
            free_mbytes: 64,
            writable: true,
            removable: true,
            system: false,
            hidden: false,
            interface_classes: Vec::new(),
            registry_roots: vec![r"HKLM\System\StorageManager\Profiles\Utility".to_owned()],
            registry_subkey: Some("FATFS".to_owned()),
        });
        kernel.registry.set_value(
            r"HKLM\System\StorageManager\Profiles\Utility\FATFS",
            "Util",
            RegistryValue::string("fatutil.dll"),
        );
        kernel
            .fsdmgr_register_volume(disk_handle, mount_root, 0x0bad_f00d)
            .unwrap();

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [disk_handle, 0],
            ),
            Some(ERROR_MOD_NOT_FOUND)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_MOD_NOT_FOUND);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [disk_handle, 0],
            ),
            Some(ERROR_MOD_NOT_FOUND)
        );
        assert_eq!(kernel.threads.get_last_error(11), ERROR_MOD_NOT_FOUND);
    }

    #[test]
    fn fsdmgr_async_volume_imports_lock_registered_hvol_shape() {
        const ERROR_DEVICE_REMOVED: u32 = 1617;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSDMGR_RegisterVolume".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Ordinal(80),
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_AsyncExitVolume".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let mount_name_ptr = 0x1000_1000;
        let lock_ptr = 0x1000_2000;
        let lock_data_ptr = 0x1000_2004;
        let disk_ptr = 0x1000_6000;
        memory.map_wide_z(mount_name_ptr, "\\Async Disk");
        memory.map_word(lock_ptr, 0xfeed_cafe);
        memory.map_word(lock_data_ptr, 0xdead_beef);

        let volume_handle = table
            .dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [disk_ptr, mount_name_ptr, 0x0bad_f00d],
            )
            .unwrap();
        assert_ne!(volume_handle, 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [volume_handle, lock_ptr, lock_data_ptr],
            ),
            Some(0)
        );
        assert_ne!(memory.word(lock_ptr), 0);
        assert_ne!(memory.word(lock_ptr), volume_handle);
        assert_eq!(memory.word(lock_data_ptr), volume_handle);

        let lock_handle = memory.word(lock_ptr);
        let lock_data = memory.word(lock_data_ptr);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [lock_handle, 0x4444_0000],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [lock_handle, lock_data],
            ),
            Some(0)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [lock_handle, lock_data],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [0x4444_0000, lock_ptr, lock_data_ptr],
            ),
            Some(ERROR_DEVICE_REMOVED)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [volume_handle, lock_ptr, 0],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [volume_handle, lock_ptr, lock_data_ptr],
            ),
            Some(0)
        );
        let stale_lock_handle = memory.word(lock_ptr);
        let stale_lock_data = memory.word(lock_data_ptr);
        assert!(kernel.unmount_volume_handle(volume_handle).unwrap());
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [stale_lock_handle, stale_lock_data],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );
    }

    #[test]
    fn fsdmgr_parse_security_descriptor_import_uses_ce_secdeschdr_size() {
        const ERROR_INVALID_SECURITY_DESCR: u32 = 1338;
        const SECURITY_ATTRIBUTES_SIZE: u32 = 12;

        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![ImportThunk {
                thunk_rva: 0x2000,
                iat_rva: 0x3000,
                import: ImportBy::Ordinal(82),
            }],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let security_attributes_ptr = 0x1000_1000;
        let security_descriptor_ptr = 0x8000_3000;
        let descriptor_out_ptr = 0x1000_2000;
        let size_out_ptr = 0x1000_2004;
        memory.map_word(security_attributes_ptr, SECURITY_ATTRIBUTES_SIZE);
        memory.map_word(security_attributes_ptr + 4, security_descriptor_ptr);
        memory.map_word(security_attributes_ptr + 8, 0);
        memory.map_halfword(security_descriptor_ptr + 2, 0x003c);
        memory.map_word(descriptor_out_ptr, 0xfeed_cafe);
        memory.map_word(size_out_ptr, 0xdead_beef);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [0, descriptor_out_ptr, size_out_ptr],
            ),
            Some(0)
        );
        assert_eq!(memory.word(descriptor_out_ptr), 0);
        assert_eq!(memory.word(size_out_ptr), 0);

        memory.map_word(descriptor_out_ptr, 0xfeed_cafe);
        memory.map_word(size_out_ptr, 0xdead_beef);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [security_attributes_ptr, descriptor_out_ptr, size_out_ptr,],
            ),
            Some(0)
        );
        assert_eq!(memory.word(descriptor_out_ptr), security_descriptor_ptr);
        assert_eq!(memory.word(size_out_ptr), 0x003c);

        memory.map_word(security_attributes_ptr + 8, 1);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [security_attributes_ptr, descriptor_out_ptr, size_out_ptr,],
            ),
            Some(ERROR_INVALID_SECURITY_DESCR)
        );
        memory.map_word(security_attributes_ptr + 8, 0);
        memory.map_word(security_attributes_ptr + 4, 0x1000_3000);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [security_attributes_ptr, descriptor_out_ptr, size_out_ptr,],
            ),
            Some(ERROR_INVALID_SECURITY_DESCR)
        );
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [security_attributes_ptr, 0, size_out_ptr],
            ),
            Some(crate::ce::thread::ERROR_INVALID_PARAMETER)
        );
    }

    #[test]
    fn fsdmgr_disk_ex_imports_scatter_gather_sparse_sectors() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
            original_first_thunk: 0x2000,
            time_date_stamp: 0,
            forwarder_chain: 0,
            name_rva: 0x2040,
            first_thunk: 0x3000,
            imports: vec![
                ImportThunk {
                    thunk_rva: 0x2000,
                    iat_rva: 0x3000,
                    import: ImportBy::Ordinal(36),
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSDMGR_ReadDiskEx".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let disk_ptr = 0x1000_6000;
        let write_info = 0x1000_b000;
        let write_buffers = 0x1000_b100;
        let write_results = 0x1000_b200;
        let write_a = 0x1000_c000;
        let write_b = 0x1000_d000;
        let read_info = 0x1000_e000;
        let read_buffers = 0x1000_e100;
        let read_results = 0x1000_e200;
        let read_a = 0x1000_f000;
        let read_b = 0x1001_0000;

        let mut sector_bytes = vec![0; 1024];
        let marker = b"ex-scatter-writeback";
        sector_bytes[..marker.len()].copy_from_slice(marker);
        for (index, byte) in sector_bytes.iter_mut().enumerate().skip(marker.len()) {
            *byte = (index as u8).wrapping_mul(37).wrapping_add(11);
        }
        memory.map_bytes(write_a, &sector_bytes[..300]);
        memory.map_bytes(write_b, &sector_bytes[300..]);
        memory.map_bytes(read_a, &vec![0xa5; 512]);
        memory.map_bytes(read_b, &vec![0x5a; 512]);
        for ptr in [write_results, read_results] {
            memory.map_word(ptr, 0xfeed_cafe);
            memory.map_word(ptr + 4, 0xfeed_cafe);
        }

        for (info, buffers) in [(write_info, write_buffers), (read_info, read_buffers)] {
            memory.map_word(info, 0);
            memory.map_word(info + 4, disk_ptr);
            memory.map_word(info + 8, 21);
            memory.map_word(info + 12, 2);
            memory.map_word(info + 16, 0);
            memory.map_word(info + 20, 2);
            memory.map_word(info + 24, buffers);
            memory.map_word(info + 28, 0);
        }
        memory.map_word(write_buffers, write_a);
        memory.map_word(write_buffers + 4, 300);
        memory.map_word(write_buffers + 8, write_b);
        memory.map_word(write_buffers + 12, 724);
        memory.map_word(read_buffers, read_a);
        memory.map_word(read_buffers + 4, 512);
        memory.map_word(read_buffers + 8, read_b);
        memory.map_word(read_buffers + 12, 512);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [write_info, write_results],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(write_results), 0);
        assert_eq!(memory.word(write_results + 4), 2);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [read_info, read_results],
            ),
            Some(0)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
        assert_eq!(memory.word(read_results), 0);
        assert_eq!(memory.word(read_results + 4), 2);

        let mut read_back = vec![0; 1024];
        memory.read_bytes(read_a, &mut read_back[..512]).unwrap();
        memory.read_bytes(read_b, &mut read_back[512..]).unwrap();
        assert_eq!(read_back, sector_bytes);
    }

    #[test]
    fn fsdmgr_notification_import_first_change_preserves_owner() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSEXT_FindFirstChangeNotificationW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_FindNextChangeNotification".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Ordinal(73),
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let path = 0x1000_0000;
        memory.map_wide_z(path, "\\");

        kernel.set_current_process_id(77);
        let handle = table
            .dispatch_trap(&mut kernel, &mut memory, 11, IMPORT_TRAP_BASE, [path, 1, 1])
            .expect("FSDMGR first-change import should dispatch");
        assert_ne!(handle, u32::MAX);
        assert_eq!(kernel.threads.get_last_error(11), 0);

        kernel.set_current_process_id(88);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [handle],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_ACCESS_DENIED
        );

        kernel.set_current_process_id(77);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [handle],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
    }

    #[test]
    fn fsdmgr_internal_notification_imports_use_internal_owner() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSINT_FindFirstChangeNotificationW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_FindNextChangeNotification".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2008,
                    iat_rva: 0x3008,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSINT_FindNextChangeNotification".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x200c,
                    iat_rva: 0x300c,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_GetFileNotificationInfoW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2010,
                    iat_rva: 0x3010,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSINT_GetFileNotificationInfoW".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2014,
                    iat_rva: 0x3014,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSINT_FindCloseChangeNotification".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let path = 0x1000_0000;
        let returned_ptr = 0x1000_0100;
        let available_ptr = 0x1000_0104;
        memory.map_wide_z(path, "\\");
        memory.map_word(returned_ptr, 0xfeed_cafe);
        memory.map_word(available_ptr, 0xabcd_1234);

        kernel.set_current_process_id(77);
        let handle = table
            .dispatch_trap(&mut kernel, &mut memory, 11, IMPORT_TRAP_BASE, [path, 1, 1])
            .expect("FSINT first-change import should dispatch");
        assert_ne!(handle, u32::MAX);
        assert_eq!(kernel.threads.get_last_error(11), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [handle],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_ACCESS_DENIED
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 2,
                [handle],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3,
                [handle, 0, 0, 0, returned_ptr, available_ptr],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_ACCESS_DENIED
        );
        assert_eq!(memory.word(returned_ptr), 0xfeed_cafe);
        assert_eq!(memory.word(available_ptr), 0xabcd_1234);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 4,
                [handle, 0, 0, 0, returned_ptr, available_ptr],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::registry::ERROR_NO_MORE_ITEMS
        );
        assert_eq!(memory.word(returned_ptr), 0);
        assert_eq!(memory.word(available_ptr), 0);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 5,
                [handle],
            ),
            Some(1)
        );
        assert_eq!(kernel.threads.get_last_error(11), 0);
    }

    #[test]
    fn fsdmgr_internal_close_rejects_wrong_handle_without_consuming_it() {
        let imports = vec![ImportDescriptor {
            module_name: "fsdmgr.dll".to_owned(),
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
                        name: "FSINT_FindCloseChangeNotification".to_owned(),
                    },
                },
                ImportThunk {
                    thunk_rva: 0x2004,
                    iat_rva: 0x3004,
                    import: ImportBy::Name {
                        hint: 0,
                        name: "FSEXT_FindCloseChangeNotification".to_owned(),
                    },
                },
            ],
        }];
        let mut mapped = vec![0; 0x4000];
        let table = patch_supported_imports(
            &mut mapped,
            0x0040_0000,
            &imports,
            &CoredllExportTable::default(),
            IMPORT_TRAP_BASE,
        )
        .unwrap();
        let mut kernel = CeKernel::boot(RuntimeConfig::load_default().unwrap());
        let mut memory = TestMemory::default();
        let internal_wrong = kernel.create_event_w(None, true, false);
        let external_wrong = kernel.create_event_w(None, true, false);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE,
                [internal_wrong],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_INVALID_HANDLE
        );
        assert!(
            kernel.close_handle(internal_wrong).is_ok(),
            "FSINT close must not consume a non-notification handle"
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                11,
                IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE,
                [external_wrong],
            ),
            Some(0)
        );
        assert_eq!(
            kernel.threads.get_last_error(11),
            crate::ce::thread::ERROR_INVALID_HANDLE
        );
        assert!(
            matches!(
                kernel.close_handle(external_wrong),
                Err(Error::InvalidHandle(_))
            ),
            "FSEXT close should consume a valid but wrong caller handle"
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

        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory::default();
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

        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory::default();
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
    fn ole_cocreateinstance_import_uses_com_registry_and_writes_ppv() {
        let mut table = ImportTrapTable::new();
        table.insert(ImportTrap {
            address: IMPORT_TRAP_BASE,
            module_kind: ImportModuleKind::Ole,
            module_name: "ole32.dll".to_owned(),
            ordinal: None,
            name: Some("CoCreateInstance".to_owned()),
            iat_va: 0x4000,
        });

        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestMemory::default();
        let clsid_ptr = 0x1000;
        let same_clsid_ptr = 0x1100;
        let iid_ptr = 0x2000;
        let ppv = 0x3000;
        let clsid = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88,
        ];
        let iid = [0x01, 0, 0, 0, 0, 0, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0, 0x46];
        memory.map_bytes(clsid_ptr, &clsid);
        memory.map_bytes(same_clsid_ptr, &clsid);
        memory.map_bytes(iid_ptr, &iid);
        memory.map_word(ppv, 0xfeed_face);

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                17,
                IMPORT_TRAP_BASE,
                vec![clsid_ptr, 0, 0x17, iid_ptr, 0],
            ),
            Some(crate::ce::com::E_POINTER)
        );

        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                17,
                IMPORT_TRAP_BASE,
                vec![clsid_ptr, 0, 0x17, iid_ptr, ppv],
            ),
            Some(crate::ce::com::REGDB_E_CLASSNOTREG)
        );
        assert_eq!(memory.word(ppv), 0);

        kernel.com.register_class_guid(clsid, 0x55aa);
        assert_eq!(
            table.dispatch_trap(
                &mut kernel,
                &mut memory,
                17,
                IMPORT_TRAP_BASE,
                vec![same_clsid_ptr, 0, 0x17, iid_ptr, ppv],
            ),
            Some(crate::ce::com::S_OK)
        );
        let object = memory.word(ppv);
        assert_eq!(object, 0x55aa);
        let record = kernel.com.object(object).expect("registered COM object");
        assert_eq!(record.clsid_ptr, same_clsid_ptr);
        assert_eq!(record.clsid, Some(clsid));
        assert_eq!(record.iid_ptr, iid_ptr);
        assert_eq!(record.iid, Some(iid));
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
        let mut external = ExternalImportTable::default();
        external.add_module_exports(
            "commctrl.dll",
            0x6200_0000,
            [("InitCommonControlsEx".to_owned(), 0x6200_1234)],
            [(17, 0x6200_5678)],
        );

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
    fn external_table_resolves_forwarded_exports_through_loaded_modules() {
        let mut external = ExternalImportTable::default();
        external.add_module_exports(
            "target.dll",
            0x6300_0000,
            [("RealExport".to_owned(), 0x6300_1234)],
            [(7, 0x6300_7000)],
        );
        external.modules.insert(
            normalize_module("forwarder.dll"),
            ExternalImportModule {
                module_name: "forwarder.dll".to_owned(),
                image_base: 0x6400_0000,
                by_name: BTreeMap::from([(
                    normalize_symbol("AliasExport"),
                    ExternalImportTarget::Forwarder("target.RealExport".to_owned()),
                )]),
                by_ordinal: BTreeMap::from([(
                    4,
                    ExternalImportTarget::Forwarder("target.#7".to_owned()),
                )]),
            },
        );

        assert_eq!(
            external.resolve(
                "forwarder.dll",
                &ImportBy::Name {
                    hint: 0,
                    name: "AliasExport".to_owned()
                }
            ),
            Some(0x6300_1234)
        );
        assert_eq!(
            external.resolve("forwarder.dll", &ImportBy::Ordinal(4)),
            Some(0x6300_7000)
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
        let mut external = ExternalImportTable::default();
        external.add_module_exports("commctrl.dll", 0x4015_0000, [], [(2, 0x4017_9d5c)]);

        let mut mapped = vec![0; 0x4000];
        mapped[0x3000..0x3004].copy_from_slice(&0x8000_0002u32.to_le_bytes());

        patch_external_imports(&mut mapped, &imports, &external).unwrap();

        assert_eq!(
            u32::from_le_bytes(mapped[0x3000..0x3004].try_into().unwrap()),
            0x4017_9d5c
        );
    }

    #[test]
    fn external_table_accepts_runtime_loaded_module_exports() {
        let mut external = ExternalImportTable::default();
        external.add_module_exports(
            "Dynamic.DLL",
            0x6500_0000,
            [("MixedCaseExport".to_owned(), 0x6500_2340)],
            [(7, 0x6500_7770)],
        );

        assert_eq!(
            external.resolve(
                "dynamic.dll",
                &ImportBy::Name {
                    hint: 0,
                    name: "mixedcaseexport".to_owned(),
                },
            ),
            Some(0x6500_2340)
        );
        assert_eq!(
            external.resolve("DYNAMIC", &ImportBy::Ordinal(7)),
            Some(0x6500_7770)
        );
    }

    #[test]
    fn next_static_trap_base_tracks_static_slots_without_dynamic_range() {
        let mut table = ImportTrapTable::new();
        assert_eq!(table.next_static_trap_base(0).unwrap(), IMPORT_TRAP_BASE);
        table.insert(ImportTrap {
            address: IMPORT_TRAP_BASE,
            module_kind: ImportModuleKind::Coredll,
            module_name: "coredll.dll".to_owned(),
            ordinal: Some(1),
            name: None,
            iat_va: 0x1000,
        });
        table.insert(ImportTrap {
            address: IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 3,
            module_kind: ImportModuleKind::Winsock,
            module_name: "winsock.dll".to_owned(),
            ordinal: None,
            name: Some("socket".to_owned()),
            iat_va: 0x2000,
        });
        table.insert(ImportTrap {
            address: DYNAMIC_COREDLL_PROC_TRAP_BASE,
            module_kind: ImportModuleKind::Coredll,
            module_name: "coredll.dll".to_owned(),
            ordinal: Some(7),
            name: None,
            iat_va: 0,
        });

        assert_eq!(
            table.next_static_trap_base(0).unwrap(),
            IMPORT_TRAP_BASE + IMPORT_TRAP_STRIDE * 4
        );
    }

    #[test]
    fn next_static_trap_base_rejects_reserved_dynamic_space() {
        let mut table = ImportTrapTable::new();
        table.insert(ImportTrap {
            address: DYNAMIC_COREDLL_PROC_TRAP_BASE - IMPORT_TRAP_STRIDE,
            module_kind: ImportModuleKind::Coredll,
            module_name: "coredll.dll".to_owned(),
            ordinal: Some(1),
            name: None,
            iat_va: 0,
        });

        assert!(table.next_static_trap_base(IMPORT_TRAP_STRIDE).is_err());
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

    #[test]
    fn external_table_resolves_multi_level_forwarding_chain() {
        // A.Foo → B.Bar → C.Baz → 0x7fff_1000 (three-level chain)
        let mut external = ExternalImportTable::default();
        external.add_module_exports("c.dll", 0x7fff_0000, [("Baz".to_owned(), 0x7fff_1000)], []);
        external.modules.insert(
            normalize_module("b.dll"),
            ExternalImportModule {
                module_name: "b.dll".to_owned(),
                image_base: 0x7ffe_0000,
                by_name: BTreeMap::from([(
                    normalize_symbol("Bar"),
                    ExternalImportTarget::Forwarder("c.Baz".to_owned()),
                )]),
                by_ordinal: BTreeMap::new(),
            },
        );
        external.modules.insert(
            normalize_module("a.dll"),
            ExternalImportModule {
                module_name: "a.dll".to_owned(),
                image_base: 0x7ffd_0000,
                by_name: BTreeMap::from([(
                    normalize_symbol("Foo"),
                    ExternalImportTarget::Forwarder("b.Bar".to_owned()),
                )]),
                by_ordinal: BTreeMap::new(),
            },
        );

        assert_eq!(
            external.resolve(
                "a.dll",
                &ImportBy::Name {
                    hint: 0,
                    name: "Foo".to_owned()
                }
            ),
            Some(0x7fff_1000),
            "three-level forward chain must resolve to final address"
        );
    }

    #[test]
    fn external_table_cycle_detection_returns_none() {
        // A.Foo → B.Bar → A.Foo (circular)
        let mut external = ExternalImportTable::default();
        external.modules.insert(
            normalize_module("a.dll"),
            ExternalImportModule {
                module_name: "a.dll".to_owned(),
                image_base: 0x6000_0000,
                by_name: BTreeMap::from([(
                    normalize_symbol("Foo"),
                    ExternalImportTarget::Forwarder("b.Bar".to_owned()),
                )]),
                by_ordinal: BTreeMap::new(),
            },
        );
        external.modules.insert(
            normalize_module("b.dll"),
            ExternalImportModule {
                module_name: "b.dll".to_owned(),
                image_base: 0x6001_0000,
                by_name: BTreeMap::from([(
                    normalize_symbol("Bar"),
                    ExternalImportTarget::Forwarder("a.Foo".to_owned()),
                )]),
                by_ordinal: BTreeMap::new(),
            },
        );

        assert_eq!(
            external.resolve(
                "a.dll",
                &ImportBy::Name {
                    hint: 0,
                    name: "Foo".to_owned()
                }
            ),
            None,
            "circular forward chain must return None instead of looping"
        );
    }

    #[test]
    fn external_table_rejects_malformed_forwarder_strings() {
        let mut external = ExternalImportTable::default();
        external.add_module_exports(
            "target.dll",
            0x6300_0000,
            [("RealExport".to_owned(), 0x6300_1234)],
            [],
        );
        external.modules.insert(
            normalize_module("forwarder.dll"),
            ExternalImportModule {
                module_name: "forwarder.dll".to_owned(),
                image_base: 0x6400_0000,
                by_name: BTreeMap::from([
                    (
                        normalize_symbol("SpacedModule"),
                        ExternalImportTarget::Forwarder(" target.RealExport".to_owned()),
                    ),
                    (
                        normalize_symbol("SpacedSymbol"),
                        ExternalImportTarget::Forwarder("target.RealExport ".to_owned()),
                    ),
                    (
                        normalize_symbol("MissingOrdinal"),
                        ExternalImportTarget::Forwarder("target.#".to_owned()),
                    ),
                ]),
                by_ordinal: BTreeMap::new(),
            },
        );

        for name in ["SpacedModule", "SpacedSymbol", "MissingOrdinal"] {
            assert_eq!(
                external.resolve(
                    "forwarder.dll",
                    &ImportBy::Name {
                        hint: 0,
                        name: name.to_owned(),
                    }
                ),
                None,
                "malformed forwarder {name} must fail closed"
            );
        }
    }

    #[test]
    fn parse_forwarder_target_handles_name_and_ordinal_forms() {
        // Normal name form: last dot separates module from symbol.
        assert_eq!(
            parse_forwarder_target("MSVCRT.printf"),
            Some((
                "MSVCRT".to_owned(),
                ImportBy::Name {
                    hint: 0,
                    name: "printf".to_owned()
                }
            ))
        );

        // Ordinal form: symbol starting with '#' parses as ordinal.
        assert_eq!(
            parse_forwarder_target("NTDLL.#42"),
            Some(("NTDLL".to_owned(), ImportBy::Ordinal(42)))
        );

        // Multi-dot path: rsplit_once on the LAST dot.
        assert_eq!(
            parse_forwarder_target("A.B.C"),
            Some((
                "A.B".to_owned(),
                ImportBy::Name {
                    hint: 0,
                    name: "C".to_owned()
                }
            ))
        );

        // Empty module (no dot) → None.
        assert_eq!(parse_forwarder_target("NoDotsHere"), None);

        // Empty module name after split (dot at start) → None.
        assert_eq!(parse_forwarder_target(".printf"), None);

        // Empty symbol name after split (dot at end) → None.
        assert_eq!(parse_forwarder_target("MSVCRT."), None);

        // Literal PE forwarder strings are not whitespace-normalized.
        assert_eq!(parse_forwarder_target(" MSVCRT.printf"), None);
        assert_eq!(parse_forwarder_target("MSVCRT.printf "), None);
        assert_eq!(parse_forwarder_target("MSVCRT. printf"), None);

        // '#' ordinal that is not a valid u16 → None.
        assert_eq!(parse_forwarder_target("FOO.#99999"), None);

        // '#' without digits is not a valid ordinal.
        assert_eq!(parse_forwarder_target("FOO.#"), None);
    }
}
