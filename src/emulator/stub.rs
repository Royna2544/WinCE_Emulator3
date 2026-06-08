use crate::{
    ce::{framebuffer::Framebuffer, kernel::CeKernel},
    emulator::{
        imports::ImportTrapTable,
        memory::{MemoryMap, MemoryPerms},
        pe_loader::{MappedBlob, MappedResource, MappedResourceString, PeLoadResult},
        types::*,
    },
    error::{Error, Result},
    pe::PeImage,
};

use super::cpu::CpuBackend;

#[derive(Debug, Clone)]
pub struct UnicornMips {
    memory: MemoryMap,
    import_traps: ImportTrapTable,
    loaded_modules: Vec<LoadedPeModuleInfo>,
    dll_search_dirs: Vec<std::path::PathBuf>,
    mapped_blobs: Vec<MappedBlob>,
    stack_top: Option<u32>,
    entry: Option<u32>,
    entry_image_base: Option<u32>,
    resource_strings: Vec<MappedResourceString>,
    resources: Vec<MappedResource>,
}

impl UnicornMips {
    fn apply_pe_load_result(&mut self, result: PeLoadResult) -> Result<()> {
        self.loaded_modules.clear();
        for (base, size, perms, name) in result.memory_regions {
            self.map_region(base, size, perms, &name)?;
        }
        for (base, size, perms, name) in result.shared_memory_regions {
            if !self.memory.contains_range(base, size) {
                self.map_region(base, size, perms, &name)?;
            }
        }
        self.entry = Some(result.entry);
        self.entry_image_base = Some(result.entry_image_base);
        self.stack_top = Some(result.stack_top);
        self.import_traps.merge(result.import_traps);
        self.mapped_blobs.extend(result.mapped_blobs);
        self.loaded_modules = result.loaded_modules;
        self.resource_strings.extend(result.resource_strings);
        self.resources.extend(result.resources);
        Ok(())
    }
}

impl CpuBackend for UnicornMips {
    fn new() -> Result<Self> {
        Ok(Self {
            memory: MemoryMap::default(),
            import_traps: ImportTrapTable::new(),
            loaded_modules: Vec::new(),
            dll_search_dirs: Vec::new(),
            mapped_blobs: Vec::new(),
            stack_top: None,
            entry: None,
            entry_image_base: None,
            resource_strings: Vec::new(),
            resources: Vec::new(),
        })
    }

    fn set_dll_search_dirs(&mut self, dirs: Vec<std::path::PathBuf>) {
        self.dll_search_dirs = dirs;
    }

    fn map_region(&mut self, base: u32, size: u32, perms: MemoryPerms, name: &str) -> Result<()> {
        self.memory.map(base, size, perms, name)
    }

    fn memory(&self) -> &MemoryMap {
        &self.memory
    }

    fn import_traps(&self) -> &ImportTrapTable {
        &self.import_traps
    }

    fn loaded_modules(&self) -> &[LoadedPeModuleInfo] {
        &self.loaded_modules
    }

    fn last_debug_snapshot(&self) -> Option<&UnicornDebugSnapshot> {
        None
    }

    fn preferred_trace_snapshot(&self) -> Option<&UnicornDebugSnapshot> {
        None
    }

    fn has_parked_child_processes(&self) -> bool {
        false
    }

    fn has_ready_parked_send_unblock(&self, _kernel: &CeKernel) -> bool {
        false
    }

    fn has_ready_parked_wait_unblock(&self, _kernel: &CeKernel) -> bool {
        false
    }

    fn rotate_to_ready_parked_wait(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    fn last_stop_is_guest_thread_return_stub(&self) -> bool {
        false
    }

    fn switch_to_next_parked_child_process(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    fn rotate_to_next_parked_process(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    fn has_parked_process_id(&self, _process_id: u32) -> bool {
        false
    }

    fn rotate_to_parked_process_id(&mut self, _kernel: &mut CeKernel, _process_id: u32) -> bool {
        false
    }

    fn mapped_blob_ranges(&self) -> Vec<UnicornMappedBlobRange> {
        Vec::new()
    }

    fn read_mapped_bytes(&self, _address: u32, _len: usize) -> Option<Vec<u8>> {
        None
    }

    fn load_pe_image(&mut self, image: &PeImage) -> Result<()> {
        self.load_pe_image_with_dlls(image, &[])
    }

    fn load_pe_image_with_dlls(&mut self, image: &PeImage, dlls: &[PeImage]) -> Result<()> {
        let result = super::pe_loader::load_pe_image_with_dlls(image, dlls)?;
        self.apply_pe_load_result(result)
    }

    fn run_until_import_trap_with_framebuffer_limits(
        &mut self,
        _kernel: &mut CeKernel,
        _framebuffer: &mut dyn Framebuffer,
        _limits: UnicornRunLimits,
    ) -> Result<()> {
        Err(Error::Backend(
            "built without the `unicorn` feature; core state is ready but CPU execution is disabled"
                .to_owned(),
        ))
    }
}
