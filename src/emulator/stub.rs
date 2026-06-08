use crate::{
    ce::{framebuffer::Framebuffer, kernel::CeKernel},
    emulator::{
        imports::ImportTrapTable,
        memory::{MemoryMap, MemoryPerms},
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
}

impl CpuBackend for UnicornMips {
    fn new() -> Result<Self> {
        Ok(Self {
            memory: MemoryMap::default(),
            import_traps: ImportTrapTable::new(),
            loaded_modules: Vec::new(),
            dll_search_dirs: Vec::new(),
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

    fn load_pe_image(&mut self, _image: &PeImage) -> Result<()> {
        Err(Error::Backend(
            "unicorn feature not enabled — PE loading unavailable".to_owned(),
        ))
    }

    fn load_pe_image_with_dlls(&mut self, _image: &PeImage, _dlls: &[PeImage]) -> Result<()> {
        Err(Error::Backend(
            "unicorn feature not enabled — PE loading unavailable".to_owned(),
        ))
    }

    fn run_until_import_trap_with_framebuffer_limits(
        &mut self,
        _kernel: &mut CeKernel,
        _framebuffer: &mut dyn Framebuffer,
        _limits: UnicornRunLimits,
    ) -> Result<()> {
        Err(Error::Backend(
            "unicorn feature not enabled — CPU execution unavailable".to_owned(),
        ))
    }
}
