use std::{fs, path::Path};

use crate::error::{Error, Result};

pub const IMAGE_FILE_MACHINE_R4000: u16 = 0x0166;
pub const IMAGE_FILE_RELOCS_STRIPPED: u16 = 0x0001;
pub const IMAGE_NT_OPTIONAL_HDR32_MAGIC: u16 = 0x010b;
pub const IMAGE_NT_OPTIONAL_HDR64_MAGIC: u16 = 0x020b;

pub const IMAGE_DIRECTORY_ENTRY_EXPORT: usize = 0;
pub const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
pub const IMAGE_DIRECTORY_ENTRY_RESOURCE: usize = 2;
pub const IMAGE_DIRECTORY_ENTRY_EXCEPTION: usize = 3;
pub const IMAGE_DIRECTORY_ENTRY_SECURITY: usize = 4;
pub const IMAGE_DIRECTORY_ENTRY_BASERELOC: usize = 5;
pub const IMAGE_DIRECTORY_ENTRY_DEBUG: usize = 6;
pub const IMAGE_DIRECTORY_ENTRY_COPYRIGHT: usize = 7;
pub const IMAGE_DIRECTORY_ENTRY_GLOBALPTR: usize = 8;
pub const IMAGE_DIRECTORY_ENTRY_TLS: usize = 9;
pub const IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG: usize = 10;
pub const IMAGE_DIRECTORY_ENTRY_BOUND_IMPORT: usize = 11;
pub const IMAGE_DIRECTORY_ENTRY_IAT: usize = 12;
pub const IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT: usize = 13;
pub const IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR: usize = 14;
pub const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;
const IMAGE_REL_BASED_ABSOLUTE: u8 = 0;
const IMAGE_REL_BASED_HIGH: u8 = 1;
const IMAGE_REL_BASED_LOW: u8 = 2;
const IMAGE_REL_BASED_HIGHLOW: u8 = 3;
const IMAGE_REL_BASED_HIGHADJ: u8 = 4;
const IMAGE_REL_BASED_MIPS_JMPADDR: u8 = 5;
const IMAGE_REL_BASED_MIPS_JMPADDR16: u8 = 9;

const IMAGE_DOS_SIGNATURE: u16 = 0x5a4d;
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550;
const IMAGE_ORDINAL_FLAG32: u32 = 0x8000_0000;
const COFF_HEADER_SIZE: usize = 20;
const SECTION_HEADER_SIZE: usize = 40;
const OPTIONAL_HEADER32_BASE_SIZE: usize = 96;

#[derive(Debug, Clone)]
pub struct PeImage {
    pub path: String,
    pub len: usize,
    pub dos_lfanew: u32,
    pub coff_header: CoffHeader,
    pub optional_header: OptionalHeader32,
    pub sections: Vec<SectionHeader>,
    pub imports: Vec<ImportDescriptor>,
    pub exports: Option<ExportDirectory>,
    pub base_relocations: Vec<BaseRelocationBlock>,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoffHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalHeader32 {
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
    pub image_base: u32,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub checksum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u32,
    pub size_of_stack_commit: u32,
    pub size_of_heap_reserve: u32,
    pub size_of_heap_commit: u32,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directories: [DataDirectory; IMAGE_NUMBEROF_DIRECTORY_ENTRIES],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DataDirectory {
    pub virtual_address: u32,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionHeader {
    pub name: String,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDescriptor {
    pub module_name: String,
    pub original_first_thunk: u32,
    pub time_date_stamp: u32,
    pub forwarder_chain: u32,
    pub name_rva: u32,
    pub first_thunk: u32,
    pub imports: Vec<ImportThunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportThunk {
    pub thunk_rva: u32,
    pub iat_rva: u32,
    pub import: ImportBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportBy {
    Name { hint: u16, name: String },
    Ordinal(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportDirectory {
    pub characteristics: u32,
    pub time_date_stamp: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub name: Option<String>,
    pub ordinal_base: u32,
    pub functions: Vec<ExportFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFunction {
    pub ordinal: u32,
    pub name: Option<String>,
    pub rva: u32,
    pub forwarder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseRelocationBlock {
    pub page_rva: u32,
    pub entries: Vec<BaseRelocationEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseRelocationEntry {
    pub raw: u16,
    pub relocation_type: u8,
    pub offset: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeResourceString {
    pub id: u32,
    pub data_rva: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeResourceData {
    pub kind: u32,
    pub name: u32,
    pub name_string: Option<String>,
    pub data_rva: u32,
    pub size: u32,
}

impl PeImage {
    pub fn inspect(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let bytes = fs::read(path_ref).map_err(|source| Error::Read {
            path: path_ref.to_path_buf(),
            source,
        })?;

        Self::parse_bytes(path_ref.display().to_string(), &bytes)
    }

    pub fn parse_bytes(path: impl Into<String>, bytes: &[u8]) -> Result<Self> {
        let path = path.into();
        let reader = PeReader::new(&path, bytes);

        if reader.read_u16(0)? != IMAGE_DOS_SIGNATURE {
            return Err(pe_error(&path, "missing MZ DOS signature"));
        }
        let dos_lfanew = reader.read_u32(0x3c)?;
        let nt_offset = usize_from_u32(dos_lfanew, &path, "e_lfanew")?;
        if reader.read_u32(nt_offset)? != IMAGE_NT_SIGNATURE {
            return Err(pe_error(&path, "missing PE NT signature"));
        }

        let coff_offset = checked_add(nt_offset, 4, &path, "COFF header offset")?;
        let coff_header = parse_coff_header(&reader, coff_offset)?;
        let optional_offset = checked_add(
            coff_offset,
            COFF_HEADER_SIZE,
            &path,
            "optional header offset",
        )?;
        let optional_header = parse_optional_header32(&reader, optional_offset, coff_header)?;
        let section_table_offset = checked_add(
            optional_offset,
            coff_header.size_of_optional_header as usize,
            &path,
            "section table offset",
        )?;
        let sections = parse_sections(&reader, section_table_offset, coff_header)?;

        let mut image = Self {
            path,
            len: bytes.len(),
            dos_lfanew,
            coff_header,
            optional_header,
            sections,
            imports: Vec::new(),
            exports: None,
            base_relocations: Vec::new(),
            bytes: bytes.to_vec(),
        };

        image.imports = image.parse_imports()?;
        image.exports = image.parse_exports()?;
        image.base_relocations = image.parse_base_relocations()?;

        Ok(image)
    }

    pub fn raw_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn entry_point_va(&self) -> u32 {
        self.optional_header
            .image_base
            .wrapping_add(self.optional_header.address_of_entry_point)
    }

    pub fn image_base(&self) -> u32 {
        self.optional_header.image_base
    }

    pub fn relocations_stripped(&self) -> bool {
        self.coff_header.characteristics & IMAGE_FILE_RELOCS_STRIPPED != 0
    }

    pub fn data_directory(&self, index: usize) -> Option<DataDirectory> {
        self.optional_header.data_directories.get(index).copied()
    }

    pub fn resource_strings(&self) -> Result<Vec<PeResourceString>> {
        const RT_STRING: u32 = 6;

        let Some(directory) = self.data_directory(IMAGE_DIRECTORY_ENTRY_RESOURCE) else {
            return Ok(Vec::new());
        };
        if directory.is_empty() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for type_entry in
            self.resource_directory_entries(directory.virtual_address, directory.virtual_address)?
        {
            if type_entry.id != Some(RT_STRING) || !type_entry.is_directory {
                continue;
            }
            let name_dir_rva = directory
                .virtual_address
                .checked_add(type_entry.offset)
                .ok_or_else(|| pe_error(&self.path, "resource type offset overflow"))?;
            for name_entry in
                self.resource_directory_entries(directory.virtual_address, name_dir_rva)?
            {
                let Some(block_id) = name_entry.id else {
                    continue;
                };
                if !name_entry.is_directory {
                    continue;
                }
                let lang_dir_rva = directory
                    .virtual_address
                    .checked_add(name_entry.offset)
                    .ok_or_else(|| pe_error(&self.path, "resource name offset overflow"))?;
                for lang_entry in
                    self.resource_directory_entries(directory.virtual_address, lang_dir_rva)?
                {
                    if lang_entry.is_directory {
                        continue;
                    }
                    let entry_rva = directory
                        .virtual_address
                        .checked_add(lang_entry.offset)
                        .ok_or_else(|| pe_error(&self.path, "resource data offset overflow"))?;
                    let data_rva = self.read_u32_rva(entry_rva)?;
                    let size = self.read_u32_rva(rva_add(entry_rva, 4, &self.path)?)?;
                    self.parse_string_table_block(block_id, data_rva, size, &mut out)?;
                }
            }
        }
        Ok(out)
    }

    pub fn resource_data_entries(&self) -> Result<Vec<PeResourceData>> {
        let Some(directory) = self.data_directory(IMAGE_DIRECTORY_ENTRY_RESOURCE) else {
            return Ok(Vec::new());
        };
        if directory.is_empty() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for type_entry in
            self.resource_directory_entries(directory.virtual_address, directory.virtual_address)?
        {
            let Some(kind) = type_entry.id else {
                continue;
            };
            if !type_entry.is_directory {
                continue;
            }
            let name_dir_rva = directory
                .virtual_address
                .checked_add(type_entry.offset)
                .ok_or_else(|| pe_error(&self.path, "resource type offset overflow"))?;
            for name_entry in
                self.resource_directory_entries(directory.virtual_address, name_dir_rva)?
            {
                let name_string = name_entry.name.clone();
                let name = name_entry.id.unwrap_or(0);
                if !name_entry.is_directory {
                    continue;
                }
                let lang_dir_rva = directory
                    .virtual_address
                    .checked_add(name_entry.offset)
                    .ok_or_else(|| pe_error(&self.path, "resource name offset overflow"))?;
                for lang_entry in
                    self.resource_directory_entries(directory.virtual_address, lang_dir_rva)?
                {
                    if lang_entry.is_directory {
                        continue;
                    }
                    let entry_rva = directory
                        .virtual_address
                        .checked_add(lang_entry.offset)
                        .ok_or_else(|| pe_error(&self.path, "resource data offset overflow"))?;
                    out.push(PeResourceData {
                        kind,
                        name,
                        name_string: name_string.clone(),
                        data_rva: self.read_u32_rva(entry_rva)?,
                        size: self.read_u32_rva(rva_add(entry_rva, 4, &self.path)?)?,
                    });
                }
            }
        }
        Ok(out)
    }

    pub fn section_for_rva(&self, rva: u32) -> Option<&SectionHeader> {
        self.sections
            .iter()
            .find(|section| section.contains_rva(rva))
    }

    pub fn rva_to_file_offset(&self, rva: u32) -> Option<usize> {
        if rva < self.optional_header.size_of_headers {
            let offset = rva as usize;
            return (offset < self.bytes.len()).then_some(offset);
        }

        let section = self.section_for_rva(rva)?;
        let delta = rva.checked_sub(section.virtual_address)? as usize;
        if delta >= section.size_of_raw_data as usize || section.pointer_to_raw_data == 0 {
            return None;
        }

        let offset = section.pointer_to_raw_data as usize + delta;
        (offset < self.bytes.len()).then_some(offset)
    }

    pub fn mapped_image(&self) -> Result<Vec<u8>> {
        let size_of_image = usize_from_u32(
            self.optional_header.size_of_image,
            &self.path,
            "SizeOfImage",
        )?;
        let mut mapped = vec![0; size_of_image];

        let header_len = self
            .optional_header
            .size_of_headers
            .min(self.bytes.len() as u32)
            .min(self.optional_header.size_of_image) as usize;
        mapped[..header_len].copy_from_slice(&self.bytes[..header_len]);

        for section in &self.sections {
            if section.size_of_raw_data == 0 || section.pointer_to_raw_data == 0 {
                continue;
            }
            let src_start = usize_from_u32(
                section.pointer_to_raw_data,
                &self.path,
                "section raw pointer",
            )?;
            let src_len = section.size_of_raw_data as usize;
            let src_end = checked_add(src_start, src_len, &self.path, "section raw end")?;
            if src_end > self.bytes.len() {
                return Err(pe_error(
                    &self.path,
                    format!("section {} raw data extends past EOF", section.name),
                ));
            }

            let dst_start = usize_from_u32(
                section.virtual_address,
                &self.path,
                "section virtual address",
            )?;
            let copy_len = src_len.min(section.virtual_size.max(section.size_of_raw_data) as usize);
            let dst_end = checked_add(dst_start, copy_len, &self.path, "mapped section end")?;
            if dst_end > mapped.len() {
                return Err(pe_error(
                    &self.path,
                    format!("section {} extends past SizeOfImage", section.name),
                ));
            }
            mapped[dst_start..dst_end]
                .copy_from_slice(&self.bytes[src_start..src_start + copy_len]);
        }

        Ok(mapped)
    }

    pub fn mapped_image_at(&self, load_base: u32) -> Result<Vec<u8>> {
        let mut mapped = self.mapped_image()?;
        let delta = load_base.wrapping_sub(self.image_base());
        if delta == 0 {
            return Ok(mapped);
        }
        if self.relocations_stripped() {
            return Err(pe_error(
                &self.path,
                format!("image has relocations stripped and cannot load at 0x{load_base:08x}"),
            ));
        }
        if self.base_relocations.is_empty() {
            return Err(pe_error(
                &self.path,
                format!("image has no base relocations for load base 0x{load_base:08x}"),
            ));
        }

        for block in &self.base_relocations {
            let mut index = 0;
            while index < block.entries.len() {
                let entry = block.entries[index];
                let rva = rva_add(block.page_rva, u32::from(entry.offset), &self.path)?;
                match entry.relocation_type {
                    IMAGE_REL_BASED_ABSOLUTE => {}
                    IMAGE_REL_BASED_HIGH => {
                        let value = read_mapped_u16(&mapped, rva, &self.path)?;
                        write_mapped_u16(
                            &mut mapped,
                            rva,
                            value.wrapping_add((delta >> 16) as u16),
                            &self.path,
                        )?;
                    }
                    IMAGE_REL_BASED_LOW => {
                        let value = read_mapped_u16(&mapped, rva, &self.path)?;
                        write_mapped_u16(
                            &mut mapped,
                            rva,
                            value.wrapping_add(delta as u16),
                            &self.path,
                        )?;
                    }
                    IMAGE_REL_BASED_HIGHLOW => {
                        let value = read_mapped_u32(&mapped, rva, &self.path)?;
                        write_mapped_u32(&mut mapped, rva, value.wrapping_add(delta), &self.path)?;
                    }
                    IMAGE_REL_BASED_HIGHADJ => {
                        let Some(next) = block.entries.get(index + 1).copied() else {
                            return Err(pe_error(&self.path, "HIGHADJ relocation missing pair"));
                        };
                        let high = read_mapped_u16(&mapped, rva, &self.path)? as i32;
                        let low_adjust = next.raw as i16 as i32;
                        let adjusted = ((high << 16) + low_adjust)
                            .wrapping_add(delta as i32)
                            .wrapping_add(0x8000);
                        write_mapped_u16(&mut mapped, rva, (adjusted >> 16) as u16, &self.path)?;
                        index += 1;
                    }
                    IMAGE_REL_BASED_MIPS_JMPADDR | IMAGE_REL_BASED_MIPS_JMPADDR16 => {
                        let instruction = read_mapped_u32(&mapped, rva, &self.path)?;
                        let target = (instruction & 0x03ff_ffff)
                            .wrapping_shl(2)
                            .wrapping_add(delta);
                        let relocated = (instruction & 0xfc00_0000) | ((target >> 2) & 0x03ff_ffff);
                        write_mapped_u32(&mut mapped, rva, relocated, &self.path)?;
                    }
                    other => {
                        return Err(pe_error(
                            &self.path,
                            format!("unsupported base relocation type {other} at RVA 0x{rva:08x}"),
                        ));
                    }
                }
                index += 1;
            }
        }

        Ok(mapped)
    }

    fn parse_imports(&self) -> Result<Vec<ImportDescriptor>> {
        let Some(directory) = self.data_directory(IMAGE_DIRECTORY_ENTRY_IMPORT) else {
            return Ok(Vec::new());
        };
        if directory.is_empty() {
            return Ok(Vec::new());
        }

        let mut descriptors = Vec::new();
        let mut descriptor_rva = directory.virtual_address;
        for _ in 0..4096 {
            let original_first_thunk = self.read_u32_rva(descriptor_rva)?;
            let time_date_stamp = self.read_u32_rva(rva_add(descriptor_rva, 4, &self.path)?)?;
            let forwarder_chain = self.read_u32_rva(rva_add(descriptor_rva, 8, &self.path)?)?;
            let name_rva = self.read_u32_rva(rva_add(descriptor_rva, 12, &self.path)?)?;
            let first_thunk = self.read_u32_rva(rva_add(descriptor_rva, 16, &self.path)?)?;

            if original_first_thunk == 0
                && time_date_stamp == 0
                && forwarder_chain == 0
                && name_rva == 0
                && first_thunk == 0
            {
                return Ok(descriptors);
            }

            let module_name = self.read_c_string_rva(name_rva)?;
            let thunk_table_rva = if original_first_thunk != 0 {
                original_first_thunk
            } else {
                first_thunk
            };
            let imports = self.parse_import_thunks(thunk_table_rva, first_thunk)?;
            descriptors.push(ImportDescriptor {
                module_name,
                original_first_thunk,
                time_date_stamp,
                forwarder_chain,
                name_rva,
                first_thunk,
                imports,
            });

            descriptor_rva = rva_add(descriptor_rva, 20, &self.path)?;
        }

        Err(pe_error(
            &self.path,
            "import descriptor table did not terminate",
        ))
    }

    fn parse_import_thunks(
        &self,
        thunk_table_rva: u32,
        first_thunk: u32,
    ) -> Result<Vec<ImportThunk>> {
        let mut imports = Vec::new();
        let mut thunk_rva = thunk_table_rva;
        for index in 0..8192u32 {
            let thunk = self.read_u32_rva(thunk_rva)?;
            if thunk == 0 {
                return Ok(imports);
            }

            let import = if thunk & IMAGE_ORDINAL_FLAG32 != 0 {
                ImportBy::Ordinal((thunk & 0xffff) as u16)
            } else {
                let hint = self.read_u16_rva(thunk)?;
                let name = self.read_c_string_rva(rva_add(thunk, 2, &self.path)?)?;
                ImportBy::Name { hint, name }
            };
            imports.push(ImportThunk {
                thunk_rva,
                iat_rva: rva_add(first_thunk, index * 4, &self.path)?,
                import,
            });

            thunk_rva = rva_add(thunk_rva, 4, &self.path)?;
        }

        Err(pe_error(&self.path, "import thunk table did not terminate"))
    }

    fn parse_exports(&self) -> Result<Option<ExportDirectory>> {
        let Some(directory) = self.data_directory(IMAGE_DIRECTORY_ENTRY_EXPORT) else {
            return Ok(None);
        };
        if directory.is_empty() {
            return Ok(None);
        }

        let base = directory.virtual_address;
        let characteristics = self.read_u32_rva(base)?;
        let time_date_stamp = self.read_u32_rva(rva_add(base, 4, &self.path)?)?;
        let major_version = self.read_u16_rva(rva_add(base, 8, &self.path)?)?;
        let minor_version = self.read_u16_rva(rva_add(base, 10, &self.path)?)?;
        let name_rva = self.read_u32_rva(rva_add(base, 12, &self.path)?)?;
        let ordinal_base = self.read_u32_rva(rva_add(base, 16, &self.path)?)?;
        let number_of_functions = self.read_u32_rva(rva_add(base, 20, &self.path)?)?;
        let number_of_names = self.read_u32_rva(rva_add(base, 24, &self.path)?)?;
        let address_of_functions = self.read_u32_rva(rva_add(base, 28, &self.path)?)?;
        let address_of_names = self.read_u32_rva(rva_add(base, 32, &self.path)?)?;
        let address_of_name_ordinals = self.read_u32_rva(rva_add(base, 36, &self.path)?)?;
        if number_of_functions > 1_000_000 || number_of_names > 1_000_000 {
            return Err(pe_error(
                &self.path,
                "export table counts are implausibly large",
            ));
        }

        let mut names_by_index = vec![None; number_of_functions as usize];
        for index in 0..number_of_names {
            let export_name_rva =
                self.read_u32_rva(rva_add(address_of_names, index * 4, &self.path)?)?;
            let ordinal_index =
                self.read_u16_rva(rva_add(address_of_name_ordinals, index * 2, &self.path)?)?
                    as usize;
            if ordinal_index >= names_by_index.len() {
                return Err(pe_error(
                    &self.path,
                    format!("export name ordinal index {ordinal_index} is out of range"),
                ));
            }
            names_by_index[ordinal_index] = Some(self.read_c_string_rva(export_name_rva)?);
        }

        let mut functions = Vec::with_capacity(number_of_functions as usize);
        for index in 0..number_of_functions {
            let function_rva =
                self.read_u32_rva(rva_add(address_of_functions, index * 4, &self.path)?)?;
            let forwarder = if directory.contains_rva(function_rva) && function_rva != 0 {
                Some(self.read_c_string_rva(function_rva)?)
            } else {
                None
            };
            functions.push(ExportFunction {
                ordinal: ordinal_base + index,
                name: names_by_index[index as usize].clone(),
                rva: function_rva,
                forwarder,
            });
        }

        Ok(Some(ExportDirectory {
            characteristics,
            time_date_stamp,
            major_version,
            minor_version,
            name: (name_rva != 0)
                .then(|| self.read_c_string_rva(name_rva))
                .transpose()?,
            ordinal_base,
            functions,
        }))
    }

    fn parse_base_relocations(&self) -> Result<Vec<BaseRelocationBlock>> {
        let Some(directory) = self.data_directory(IMAGE_DIRECTORY_ENTRY_BASERELOC) else {
            return Ok(Vec::new());
        };
        if directory.is_empty() {
            return Ok(Vec::new());
        }

        let mut blocks = Vec::new();
        let mut cursor = directory.virtual_address;
        let end = directory
            .virtual_address
            .checked_add(directory.size)
            .ok_or_else(|| pe_error(&self.path, "base relocation directory overflow"))?;

        while cursor < end {
            let page_rva = self.read_u32_rva(cursor)?;
            let block_size = self.read_u32_rva(rva_add(cursor, 4, &self.path)?)?;
            if page_rva == 0 && block_size == 0 {
                break;
            }
            if block_size < 8 || block_size % 2 != 0 {
                return Err(pe_error(
                    &self.path,
                    format!("invalid base relocation block size {block_size}"),
                ));
            }

            let entry_count = (block_size - 8) / 2;
            let mut entries = Vec::with_capacity(entry_count as usize);
            let mut entry_rva = rva_add(cursor, 8, &self.path)?;
            for _ in 0..entry_count {
                let raw = self.read_u16_rva(entry_rva)?;
                entries.push(BaseRelocationEntry {
                    raw,
                    relocation_type: (raw >> 12) as u8,
                    offset: raw & 0x0fff,
                });
                entry_rva = rva_add(entry_rva, 2, &self.path)?;
            }

            blocks.push(BaseRelocationBlock { page_rva, entries });
            cursor = rva_add(cursor, block_size, &self.path)?;
        }

        Ok(blocks)
    }

    fn read_u16_rva(&self, rva: u32) -> Result<u16> {
        let bytes = self.read_mapped_bytes_rva::<2>(rva)?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32_rva(&self, rva: u32) -> Result<u32> {
        let bytes = self.read_mapped_bytes_rva::<4>(rva)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_c_string_rva(&self, rva: u32) -> Result<String> {
        let mut cursor = rva;
        let mut bytes = Vec::new();
        loop {
            let byte = self.read_mapped_u8_rva(cursor)?;
            if byte == 0 {
                return Ok(String::from_utf8_lossy(&bytes).into_owned());
            }
            bytes.push(byte);
            cursor = rva_add(cursor, 1, &self.path)?;
        }
    }

    fn read_mapped_bytes_rva<const N: usize>(&self, rva: u32) -> Result<[u8; N]> {
        let mut out = [0; N];
        for (index, byte) in out.iter_mut().enumerate() {
            let delta = u32::try_from(index)
                .map_err(|_| pe_error(&self.path, "mapped RVA read length overflow"))?;
            *byte = self.read_mapped_u8_rva(rva_add(rva, delta, &self.path)?)?;
        }
        Ok(out)
    }

    fn read_mapped_u8_rva(&self, rva: u32) -> Result<u8> {
        if let Some(offset) = self.rva_to_file_offset(rva) {
            return PeReader::new(&self.path, &self.bytes).read_u8(offset);
        }
        if rva < self.optional_header.size_of_image {
            return Ok(0);
        }
        Err(pe_error(
            &self.path,
            format!("RVA 0x{rva:08x} is outside mapped image"),
        ))
    }

    fn read_resource_name_rva(&self, rva: u32) -> Result<String> {
        let len = self.read_u16_rva(rva)? as u32;
        let mut cursor = rva_add(rva, 2, &self.path)?;
        let mut units = Vec::with_capacity(len as usize);
        for _ in 0..len {
            units.push(self.read_u16_rva(cursor)?);
            cursor = rva_add(cursor, 2, &self.path)?;
        }
        Ok(String::from_utf16_lossy(&units))
    }

    fn resource_directory_entries(
        &self,
        resource_root_rva: u32,
        directory_rva: u32,
    ) -> Result<Vec<ResourceDirectoryEntry>> {
        let named = self.read_u16_rva(rva_add(directory_rva, 12, &self.path)?)? as u32;
        let ids = self.read_u16_rva(rva_add(directory_rva, 14, &self.path)?)? as u32;
        let count = named.saturating_add(ids);
        let mut entries = Vec::new();
        let mut entry_rva = rva_add(directory_rva, 16, &self.path)?;
        for _ in 0..count {
            let name_or_id = self.read_u32_rva(entry_rva)?;
            let offset_to_data = self.read_u32_rva(rva_add(entry_rva, 4, &self.path)?)?;
            let name = if name_or_id & 0x8000_0000 != 0 {
                Some(self.read_resource_name_rva(
                    resource_root_rva.wrapping_add(name_or_id & 0x7fff_ffff),
                )?)
            } else {
                None
            };
            entries.push(ResourceDirectoryEntry {
                id: (name_or_id & 0x8000_0000 == 0).then_some(name_or_id & 0xffff),
                name,
                is_directory: offset_to_data & 0x8000_0000 != 0,
                offset: offset_to_data & 0x7fff_ffff,
            });
            entry_rva = rva_add(entry_rva, 8, &self.path)?;
        }
        Ok(entries)
    }

    fn parse_string_table_block(
        &self,
        block_id: u32,
        data_rva: u32,
        size: u32,
        out: &mut Vec<PeResourceString>,
    ) -> Result<()> {
        let mut cursor = data_rva;
        let end = data_rva
            .checked_add(size)
            .ok_or_else(|| pe_error(&self.path, "resource string block overflow"))?;
        for index in 0..16 {
            if cursor >= end {
                break;
            }
            let len = self.read_u16_rva(cursor)? as u32;
            cursor = rva_add(cursor, 2, &self.path)?;
            let mut units = Vec::new();
            for _ in 0..len {
                if cursor >= end {
                    return Err(pe_error(&self.path, "truncated resource string"));
                }
                units.push(self.read_u16_rva(cursor)?);
                cursor = rva_add(cursor, 2, &self.path)?;
            }
            if len != 0 {
                out.push(PeResourceString {
                    id: block_id.saturating_sub(1).saturating_mul(16) + index,
                    data_rva,
                    text: String::from_utf16_lossy(&units),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceDirectoryEntry {
    id: Option<u32>,
    name: Option<String>,
    is_directory: bool,
    offset: u32,
}

fn read_mapped_u16(mapped: &[u8], rva: u32, path: &str) -> Result<u16> {
    let offset = rva as usize;
    let bytes = mapped
        .get(offset..offset + 2)
        .ok_or_else(|| pe_error(path, format!("relocation RVA 0x{rva:08x} is outside image")))?;
    Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
}

fn write_mapped_u16(mapped: &mut [u8], rva: u32, value: u16, path: &str) -> Result<()> {
    let offset = rva as usize;
    let bytes = mapped
        .get_mut(offset..offset + 2)
        .ok_or_else(|| pe_error(path, format!("relocation RVA 0x{rva:08x} is outside image")))?;
    bytes.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn read_mapped_u32(mapped: &[u8], rva: u32, path: &str) -> Result<u32> {
    let offset = rva as usize;
    let bytes = mapped
        .get(offset..offset + 4)
        .ok_or_else(|| pe_error(path, format!("relocation RVA 0x{rva:08x} is outside image")))?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn write_mapped_u32(mapped: &mut [u8], rva: u32, value: u32, path: &str) -> Result<()> {
    let offset = rva as usize;
    let bytes = mapped
        .get_mut(offset..offset + 4)
        .ok_or_else(|| pe_error(path, format!("relocation RVA 0x{rva:08x} is outside image")))?;
    bytes.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

impl DataDirectory {
    pub fn is_empty(self) -> bool {
        self.virtual_address == 0 || self.size == 0
    }

    pub fn contains_rva(self, rva: u32) -> bool {
        let Some(end) = self.virtual_address.checked_add(self.size) else {
            return false;
        };
        rva >= self.virtual_address && rva < end
    }
}

impl SectionHeader {
    pub fn contains_rva(&self, rva: u32) -> bool {
        let span = self.virtual_size.max(self.size_of_raw_data);
        let Some(end) = self.virtual_address.checked_add(span) else {
            return false;
        };
        span != 0 && rva >= self.virtual_address && rva < end
    }
}

fn parse_coff_header(reader: &PeReader<'_>, offset: usize) -> Result<CoffHeader> {
    Ok(CoffHeader {
        machine: reader.read_u16(offset)?,
        number_of_sections: reader.read_u16(offset + 2)?,
        time_date_stamp: reader.read_u32(offset + 4)?,
        pointer_to_symbol_table: reader.read_u32(offset + 8)?,
        number_of_symbols: reader.read_u32(offset + 12)?,
        size_of_optional_header: reader.read_u16(offset + 16)?,
        characteristics: reader.read_u16(offset + 18)?,
    })
}

fn parse_optional_header32(
    reader: &PeReader<'_>,
    offset: usize,
    coff_header: CoffHeader,
) -> Result<OptionalHeader32> {
    let optional_size = coff_header.size_of_optional_header as usize;
    if optional_size < OPTIONAL_HEADER32_BASE_SIZE {
        return Err(pe_error(
            reader.path,
            format!("optional header is too small: {optional_size} bytes"),
        ));
    }
    reader.slice(offset, optional_size)?;

    let magic = reader.read_u16(offset)?;
    if magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC {
        return Err(pe_error(
            reader.path,
            "PE32+ images are not supported by the CE/MIPS loader yet",
        ));
    }
    if magic != IMAGE_NT_OPTIONAL_HDR32_MAGIC {
        return Err(pe_error(
            reader.path,
            format!("unsupported optional header magic 0x{magic:04x}"),
        ));
    }

    let number_of_rva_and_sizes = reader.read_u32(offset + 92)?;
    let mut data_directories = [DataDirectory::default(); IMAGE_NUMBEROF_DIRECTORY_ENTRIES];
    let available_directories = ((optional_size - OPTIONAL_HEADER32_BASE_SIZE) / 8)
        .min(number_of_rva_and_sizes as usize)
        .min(IMAGE_NUMBEROF_DIRECTORY_ENTRIES);
    for (index, directory) in data_directories
        .iter_mut()
        .enumerate()
        .take(available_directories)
    {
        let directory_offset = offset + OPTIONAL_HEADER32_BASE_SIZE + index * 8;
        *directory = DataDirectory {
            virtual_address: reader.read_u32(directory_offset)?,
            size: reader.read_u32(directory_offset + 4)?,
        };
    }

    Ok(OptionalHeader32 {
        magic,
        major_linker_version: reader.read_u8(offset + 2)?,
        minor_linker_version: reader.read_u8(offset + 3)?,
        size_of_code: reader.read_u32(offset + 4)?,
        size_of_initialized_data: reader.read_u32(offset + 8)?,
        size_of_uninitialized_data: reader.read_u32(offset + 12)?,
        address_of_entry_point: reader.read_u32(offset + 16)?,
        base_of_code: reader.read_u32(offset + 20)?,
        base_of_data: reader.read_u32(offset + 24)?,
        image_base: reader.read_u32(offset + 28)?,
        section_alignment: reader.read_u32(offset + 32)?,
        file_alignment: reader.read_u32(offset + 36)?,
        major_operating_system_version: reader.read_u16(offset + 40)?,
        minor_operating_system_version: reader.read_u16(offset + 42)?,
        major_image_version: reader.read_u16(offset + 44)?,
        minor_image_version: reader.read_u16(offset + 46)?,
        major_subsystem_version: reader.read_u16(offset + 48)?,
        minor_subsystem_version: reader.read_u16(offset + 50)?,
        win32_version_value: reader.read_u32(offset + 52)?,
        size_of_image: reader.read_u32(offset + 56)?,
        size_of_headers: reader.read_u32(offset + 60)?,
        checksum: reader.read_u32(offset + 64)?,
        subsystem: reader.read_u16(offset + 68)?,
        dll_characteristics: reader.read_u16(offset + 70)?,
        size_of_stack_reserve: reader.read_u32(offset + 72)?,
        size_of_stack_commit: reader.read_u32(offset + 76)?,
        size_of_heap_reserve: reader.read_u32(offset + 80)?,
        size_of_heap_commit: reader.read_u32(offset + 84)?,
        loader_flags: reader.read_u32(offset + 88)?,
        number_of_rva_and_sizes,
        data_directories,
    })
}

fn parse_sections(
    reader: &PeReader<'_>,
    section_table_offset: usize,
    coff_header: CoffHeader,
) -> Result<Vec<SectionHeader>> {
    let mut sections = Vec::with_capacity(coff_header.number_of_sections as usize);
    for index in 0..coff_header.number_of_sections as usize {
        let offset = checked_add(
            section_table_offset,
            index * SECTION_HEADER_SIZE,
            reader.path,
            "section header offset",
        )?;
        let name_bytes = reader.slice(offset, 8)?;
        let name_len = name_bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name_bytes.len());
        let name = String::from_utf8_lossy(&name_bytes[..name_len]).into_owned();

        sections.push(SectionHeader {
            name,
            virtual_size: reader.read_u32(offset + 8)?,
            virtual_address: reader.read_u32(offset + 12)?,
            size_of_raw_data: reader.read_u32(offset + 16)?,
            pointer_to_raw_data: reader.read_u32(offset + 20)?,
            pointer_to_relocations: reader.read_u32(offset + 24)?,
            pointer_to_linenumbers: reader.read_u32(offset + 28)?,
            number_of_relocations: reader.read_u16(offset + 32)?,
            number_of_linenumbers: reader.read_u16(offset + 34)?,
            characteristics: reader.read_u32(offset + 36)?,
        });
    }

    Ok(sections)
}

struct PeReader<'a> {
    path: &'a str,
    bytes: &'a [u8],
}

impl<'a> PeReader<'a> {
    fn new(path: &'a str, bytes: &'a [u8]) -> Self {
        Self { path, bytes }
    }

    fn read_u8(&self, offset: usize) -> Result<u8> {
        Ok(*self
            .bytes
            .get(offset)
            .ok_or_else(|| pe_error(self.path, format!("read past EOF at 0x{offset:x}")))?)
    }

    fn read_u16(&self, offset: usize) -> Result<u16> {
        let bytes = self.slice(offset, 2)?;
        Ok(u16::from_le_bytes(
            bytes.try_into().expect("slice length checked"),
        ))
    }

    fn read_u32(&self, offset: usize) -> Result<u32> {
        let bytes = self.slice(offset, 4)?;
        Ok(u32::from_le_bytes(
            bytes.try_into().expect("slice length checked"),
        ))
    }

    fn slice(&self, offset: usize, len: usize) -> Result<&'a [u8]> {
        let end = checked_add(offset, len, self.path, "slice end")?;
        self.bytes.get(offset..end).ok_or_else(|| {
            pe_error(
                self.path,
                format!(
                    "need {len} bytes at file offset 0x{offset:x}, len is {}",
                    self.bytes.len()
                ),
            )
        })
    }
}

fn checked_add(lhs: usize, rhs: usize, path: &str, context: &str) -> Result<usize> {
    lhs.checked_add(rhs)
        .ok_or_else(|| pe_error(path, format!("{context} overflow")))
}

fn usize_from_u32(value: u32, path: &str, context: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| pe_error(path, format!("{context} does not fit in usize")))
}

fn rva_add(base: u32, delta: u32, path: &str) -> Result<u32> {
    base.checked_add(delta)
        .ok_or_else(|| pe_error(path, "RVA arithmetic overflow"))
}

fn pe_error(path: &str, message: impl Into<String>) -> Error {
    Error::InvalidArgument(format!("PE parse error in {path}: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mips_pe32_imports_exports_relocations_and_mapping() -> Result<()> {
        let bytes = synthetic_pe32();
        let image = PeImage::parse_bytes("synthetic-mips.exe", &bytes)?;

        assert_eq!(image.dos_lfanew, 0x80);
        assert_eq!(image.coff_header.machine, IMAGE_FILE_MACHINE_R4000);
        assert_eq!(image.optional_header.address_of_entry_point, 0x1000);
        assert_eq!(image.entry_point_va(), 0x0040_1000);
        assert_eq!(image.sections.len(), 3);
        assert_eq!(image.sections[1].name, ".idata");
        assert_eq!(image.rva_to_file_offset(0x2040), Some(0x440));

        assert_eq!(image.imports.len(), 1);
        assert_eq!(image.imports[0].module_name, "COREDLL.dll");
        assert_eq!(image.imports[0].imports.len(), 2);
        assert_eq!(
            image.imports[0].imports[0].import,
            ImportBy::Name {
                hint: 168,
                name: "CreateFileW".to_owned()
            }
        );
        assert_eq!(image.imports[0].imports[0].iat_rva, 0x2070);
        assert_eq!(image.imports[0].imports[1].import, ImportBy::Ordinal(461));

        let exports = image.exports.as_ref().expect("export directory");
        assert_eq!(exports.name.as_deref(), Some("SYNTH.dll"));
        assert_eq!(exports.ordinal_base, 1);
        assert_eq!(exports.functions.len(), 1);
        assert_eq!(
            exports.functions[0].name.as_deref(),
            Some("SyntheticExport")
        );
        assert_eq!(exports.functions[0].rva, 0x1010);

        assert_eq!(image.base_relocations.len(), 1);
        assert_eq!(image.base_relocations[0].page_rva, 0x1000);
        assert_eq!(
            image.base_relocations[0].entries,
            vec![
                BaseRelocationEntry {
                    raw: 0x3010,
                    relocation_type: 3,
                    offset: 0x010,
                },
                BaseRelocationEntry {
                    raw: 0,
                    relocation_type: 0,
                    offset: 0,
                },
            ]
        );

        let mapped = image.mapped_image()?;
        assert_eq!(&mapped[0x2040..0x204b], b"COREDLL.dll");
        assert_eq!(read_mapped_u32(&mapped, 0x1010, &image.path)?, 0x0040_1010);

        let relocated = image.mapped_image_at(0x0050_0000)?;
        assert_eq!(
            read_mapped_u32(&relocated, 0x1010, &image.path)?,
            0x0050_1010
        );

        Ok(())
    }

    #[test]
    fn rejects_relocating_stripped_images() {
        let mut bytes = synthetic_pe32();
        put_u16(&mut bytes, 0x84 + 18, 0x0103);
        let image = PeImage::parse_bytes("stripped-mips.exe", &bytes).unwrap();

        assert!(image.relocations_stripped());
        let err = image.mapped_image_at(0x0050_0000).unwrap_err();
        assert!(err.to_string().contains("relocations stripped"));
    }

    #[test]
    fn rejects_non_pe_images() {
        let err = PeImage::parse_bytes("not-pe.bin", b"nope").unwrap_err();
        assert!(err.to_string().contains("missing MZ"));
    }

    #[test]
    fn mapped_rva_reads_zero_filled_section_tail() -> Result<()> {
        let mut bytes = synthetic_pe32();
        let idata_section = 0x178 + SECTION_HEADER_SIZE;
        put_u32(&mut bytes, idata_section + 8, 0x300);

        let image = PeImage::parse_bytes("zero-fill-tail.exe", &bytes)?;

        assert_eq!(image.rva_to_file_offset(0x2200), None);
        assert_eq!(image.read_u32_rva(0x2200)?, 0);
        assert_eq!(image.read_c_string_rva(0x2200)?, "");

        Ok(())
    }

    #[test]
    fn mapped_rva_reads_zero_filled_image_gaps() -> Result<()> {
        let image = PeImage::parse_bytes("zero-fill-gap.exe", &synthetic_pe32())?;

        assert_eq!(image.section_for_rva(0x4f00), None);
        assert_eq!(image.rva_to_file_offset(0x4f00), None);
        assert_eq!(image.read_u32_rva(0x4f00)?, 0);
        assert_eq!(image.read_c_string_rva(0x4f00)?, "");

        Ok(())
    }

    fn synthetic_pe32() -> Vec<u8> {
        let mut bytes = vec![0; 0x800];
        put_bytes(&mut bytes, 0, b"MZ");
        put_u32(&mut bytes, 0x3c, 0x80);
        put_bytes(&mut bytes, 0x80, b"PE\0\0");

        let coff = 0x84;
        put_u16(&mut bytes, coff, IMAGE_FILE_MACHINE_R4000);
        put_u16(&mut bytes, coff + 2, 3);
        put_u16(&mut bytes, coff + 16, 0xe0);
        put_u16(&mut bytes, coff + 18, 0x0102);

        let optional = 0x98;
        put_u16(&mut bytes, optional, IMAGE_NT_OPTIONAL_HDR32_MAGIC);
        bytes[optional + 2] = 8;
        put_u32(&mut bytes, optional + 4, 0x200);
        put_u32(&mut bytes, optional + 8, 0x600);
        put_u32(&mut bytes, optional + 16, 0x1000);
        put_u32(&mut bytes, optional + 20, 0x1000);
        put_u32(&mut bytes, optional + 24, 0x2000);
        put_u32(&mut bytes, optional + 28, 0x0040_0000);
        put_u32(&mut bytes, optional + 32, 0x1000);
        put_u32(&mut bytes, optional + 36, 0x200);
        put_u16(&mut bytes, optional + 40, 4);
        put_u16(&mut bytes, optional + 48, 4);
        put_u32(&mut bytes, optional + 56, 0x5000);
        put_u32(&mut bytes, optional + 60, 0x200);
        put_u16(&mut bytes, optional + 68, 9);
        put_u32(&mut bytes, optional + 72, 0x10000);
        put_u32(&mut bytes, optional + 76, 0x1000);
        put_u32(&mut bytes, optional + 80, 0x10000);
        put_u32(&mut bytes, optional + 84, 0x1000);
        put_u32(
            &mut bytes,
            optional + 92,
            IMAGE_NUMBEROF_DIRECTORY_ENTRIES as u32,
        );

        put_directory(
            &mut bytes,
            optional,
            IMAGE_DIRECTORY_ENTRY_EXPORT,
            0x2100,
            0x80,
        );
        put_directory(
            &mut bytes,
            optional,
            IMAGE_DIRECTORY_ENTRY_IMPORT,
            0x2000,
            0x28,
        );
        put_directory(
            &mut bytes,
            optional,
            IMAGE_DIRECTORY_ENTRY_BASERELOC,
            0x4000,
            0x0c,
        );

        let section = 0x178;
        put_section(
            &mut bytes,
            section,
            b".text\0\0\0",
            0x100,
            0x1000,
            0x200,
            0x200,
            0x6000_0020,
        );
        put_section(
            &mut bytes,
            section + SECTION_HEADER_SIZE,
            b".idata\0\0",
            0x200,
            0x2000,
            0x200,
            0x400,
            0xc000_0040,
        );
        put_section(
            &mut bytes,
            section + SECTION_HEADER_SIZE * 2,
            b".reloc\0\0",
            0x100,
            0x4000,
            0x200,
            0x600,
            0x4200_0040,
        );

        put_u32(&mut bytes, 0x210, 0x0040_1010);

        put_u32(&mut bytes, 0x400, 0x2050);
        put_u32(&mut bytes, 0x40c, 0x2040);
        put_u32(&mut bytes, 0x410, 0x2070);
        put_bytes(&mut bytes, 0x440, b"COREDLL.dll\0");
        put_u32(&mut bytes, 0x450, 0x2060);
        put_u32(&mut bytes, 0x454, IMAGE_ORDINAL_FLAG32 | 461);
        put_u32(&mut bytes, 0x458, 0);
        put_u16(&mut bytes, 0x460, 168);
        put_bytes(&mut bytes, 0x462, b"CreateFileW\0");
        put_u32(&mut bytes, 0x470, 0x2060);
        put_u32(&mut bytes, 0x474, IMAGE_ORDINAL_FLAG32 | 461);
        put_u32(&mut bytes, 0x478, 0);

        put_u32(&mut bytes, 0x600, 0x1000);
        put_u32(&mut bytes, 0x604, 0x0c);
        put_u16(&mut bytes, 0x608, 0x3010);
        put_u16(&mut bytes, 0x60a, 0x0000);

        let export = 0x500;
        put_u32(&mut bytes, export + 12, 0x2160);
        put_u32(&mut bytes, export + 16, 1);
        put_u32(&mut bytes, export + 20, 1);
        put_u32(&mut bytes, export + 24, 1);
        put_u32(&mut bytes, export + 28, 0x2140);
        put_u32(&mut bytes, export + 32, 0x2144);
        put_u32(&mut bytes, export + 36, 0x2148);
        put_u32(&mut bytes, 0x540, 0x1010);
        put_u32(&mut bytes, 0x544, 0x2170);
        put_u16(&mut bytes, 0x548, 0);
        put_bytes(&mut bytes, 0x560, b"SYNTH.dll\0");
        put_bytes(&mut bytes, 0x570, b"SyntheticExport\0");

        bytes
    }

    fn put_directory(bytes: &mut [u8], optional: usize, index: usize, rva: u32, size: u32) {
        let offset = optional + OPTIONAL_HEADER32_BASE_SIZE + index * 8;
        put_u32(bytes, offset, rva);
        put_u32(bytes, offset + 4, size);
    }

    fn put_section(
        bytes: &mut [u8],
        offset: usize,
        name: &[u8; 8],
        virtual_size: u32,
        virtual_address: u32,
        raw_size: u32,
        raw_pointer: u32,
        characteristics: u32,
    ) {
        put_bytes(bytes, offset, name);
        put_u32(bytes, offset + 8, virtual_size);
        put_u32(bytes, offset + 12, virtual_address);
        put_u32(bytes, offset + 16, raw_size);
        put_u32(bytes, offset + 20, raw_pointer);
        put_u32(bytes, offset + 36, characteristics);
    }

    fn put_bytes(bytes: &mut [u8], offset: usize, value: &[u8]) {
        bytes[offset..offset + value.len()].copy_from_slice(value);
    }

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
