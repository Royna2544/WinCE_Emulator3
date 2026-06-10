pub use super::types::*;

use crate::{
    ce::{framebuffer::Framebuffer, kernel::CeKernel},
    emulator::{imports::ImportTrapTable, memory::MemoryMap},
    error::Result,
    pe::PeImage,
};

pub trait CpuBackend: Clone + std::fmt::Debug {
    fn new() -> Result<Self>
    where
        Self: Sized;

    fn set_dll_search_dirs(&mut self, dirs: Vec<std::path::PathBuf>);
    fn map_region(
        &mut self,
        base: u32,
        size: u32,
        perms: crate::emulator::memory::MemoryPerms,
        name: &str,
    ) -> Result<()>;

    fn memory(&self) -> &MemoryMap;
    fn import_traps(&self) -> &ImportTrapTable;
    fn loaded_modules(&self) -> &[LoadedPeModuleInfo];

    fn last_debug_snapshot(&self) -> Option<&UnicornDebugSnapshot>;
    fn preferred_trace_snapshot(&self) -> Option<&UnicornDebugSnapshot>;

    fn has_parked_child_processes(&self) -> bool;
    fn has_ready_parked_send_unblock(&self, kernel: &CeKernel) -> bool;
    fn has_ready_parked_wait_unblock(&self, kernel: &CeKernel) -> bool;
    fn rotate_to_ready_parked_wait(&mut self, kernel: &mut CeKernel) -> bool;
    fn last_stop_is_guest_thread_return_stub(&self) -> bool;
    fn switch_to_next_parked_child_process(&mut self, kernel: &mut CeKernel) -> bool;
    fn rotate_to_next_parked_process(&mut self, kernel: &mut CeKernel) -> bool;
    fn has_parked_process_id(&self, process_id: u32) -> bool;
    fn rotate_to_parked_process_id(&mut self, kernel: &mut CeKernel, process_id: u32) -> bool;

    fn mapped_blob_ranges(&self) -> Vec<UnicornMappedBlobRange>;
    fn read_mapped_bytes(&self, address: u32, len: usize) -> Option<Vec<u8>>;

    fn load_pe_image(&mut self, image: &PeImage) -> Result<()>;
    fn load_pe_image_with_dlls(&mut self, image: &PeImage, dlls: &[PeImage]) -> Result<()>;

    fn run_until_import_trap_with_framebuffer_limits(
        &mut self,
        kernel: &mut CeKernel,
        framebuffer: &mut dyn Framebuffer,
        limits: UnicornRunLimits,
    ) -> Result<()>;
}

#[cfg(feature = "unicorn")]
pub use super::unicorn::UnicornMips;

#[cfg(not(feature = "unicorn"))]
pub use super::stub::UnicornMips;
