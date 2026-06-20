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
    initial_thread_id: u32,
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

    pub fn pending_wndproc_debug_text(&self) -> String {
        "  pending wndproc returns: 0\n    none\n".to_owned()
    }

    pub fn set_initial_thread_id(&mut self, thread_id: u32) {
        self.initial_thread_id = thread_id;
    }

    pub fn current_thread_id(&self) -> u32 {
        self.initial_thread_id
    }

    pub fn has_saved_context(&self) -> bool {
        false
    }

    pub fn has_running_guest_thread(&self) -> bool {
        false
    }

    pub fn saved_context_at_import(
        &self,
        _module_kind: crate::emulator::imports::ImportModuleKind,
        _ordinal: u32,
    ) -> bool {
        false
    }

    pub fn active_thread_ids(&self) -> Vec<u32> {
        vec![self.initial_thread_id]
    }

    pub fn contains_thread(&self, thread_ids: &[u32]) -> bool {
        thread_ids.contains(&self.initial_thread_id)
    }

    pub fn saved_context_debug_text(&self) -> String {
        "none".to_owned()
    }

    pub fn trampoline_debug_text(&self) -> String {
        "  trampoline jumps: unavailable\n".to_owned()
    }

    pub fn active_process_has_visible_receiver_work(&self, kernel: &CeKernel) -> bool {
        self.thread_has_visible_receiver_work(self.initial_thread_id, kernel)
    }

    pub fn active_process_has_visible_queued_receiver_work(&self, kernel: &CeKernel) -> bool {
        self.thread_has_visible_queued_receiver_work(self.initial_thread_id, kernel)
    }

    pub fn active_process_has_visible_windows(&self, _kernel: &CeKernel) -> bool {
        false
    }

    pub fn current_thread_has_visible_receiver_work(&self, kernel: &CeKernel) -> bool {
        self.thread_has_visible_receiver_work(self.initial_thread_id, kernel)
    }

    pub fn active_process_has_receiver_work(&self, _kernel: &CeKernel) -> bool {
        false
    }

    fn thread_has_visible_receiver_work(&self, thread_id: u32, kernel: &CeKernel) -> bool {
        kernel.thread_has_pending_sent_message(thread_id)
            || kernel
                .gwe
                .has_visible_window_message_filtered(thread_id, None, 0, 0)
            || kernel.gwe.thread_has_dirty_visible_window(thread_id)
    }

    fn thread_has_visible_queued_receiver_work(&self, thread_id: u32, kernel: &CeKernel) -> bool {
        kernel.thread_has_pending_sent_message(thread_id)
            || kernel
                .gwe
                .has_visible_queued_message_filtered(thread_id, None, 0, 0)
    }

    pub fn has_runnable_parked_process(&self, _kernel: &CeKernel) -> bool {
        false
    }

    pub fn has_live_pump_priority_parked_process(&self, _kernel: &CeKernel) -> bool {
        false
    }

    pub fn parked_child_process_count_for_kernel(&self, _kernel: &CeKernel) -> usize {
        0
    }

    pub fn preserve_current_on_process_handoff(&self, _kernel: &CeKernel) -> bool {
        false
    }

    pub fn prune_active_process_from_parked(&mut self, _kernel: &CeKernel) {}

    pub fn prune_exited_and_active_processes_from_parked_with_framebuffer(
        &mut self,
        _kernel: &mut CeKernel,
        _framebuffer: Option<&mut dyn Framebuffer>,
    ) {
    }

    pub fn complete_escaped_saved_get_message_sent_callout(
        &mut self,
        _kernel: &mut CeKernel,
    ) -> bool {
        false
    }

    pub fn complete_escaped_direct_send_message_callout(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn clear_escaped_visible_message_callouts(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn clear_orphaned_send_depths(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn clear_orphaned_direct_send_callouts(&mut self, _kernel: &CeKernel) -> bool {
        false
    }

    pub fn complete_orphaned_active_send_wait(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn complete_orphaned_parked_send_wait(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn complete_ready_active_modal_message_box_with_framebuffer(
        &mut self,
        _kernel: &mut CeKernel,
        _framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        false
    }

    pub fn complete_active_process_thread_exit_with_framebuffer(
        &mut self,
        _kernel: &mut CeKernel,
        _framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        false
    }

    pub fn prepare_active_orphaned_visible_message_callout(
        &mut self,
        _kernel: &mut CeKernel,
    ) -> bool {
        false
    }

    pub fn prepare_cross_thread_visible_message_callout(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn prepare_active_sent_message_callout(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn rotate_to_ready_parked_threads_with_framebuffer(
        &mut self,
        _kernel: &mut CeKernel,
        _thread_ids: &[u32],
        _framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        false
    }

    pub fn rotate_to_ready_parked_wait_with_framebuffer(
        &mut self,
        _kernel: &mut CeKernel,
        _framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        false
    }

    pub fn rotate_to_receiver_parked_process_with_framebuffer(
        &mut self,
        _kernel: &mut CeKernel,
        _framebuffer: Option<&mut dyn Framebuffer>,
    ) -> bool {
        false
    }

    pub fn rotate_to_visible_window_parked_process(&mut self, _kernel: &mut CeKernel) -> bool {
        false
    }

    pub fn rotate_to_runnable_parked_threads(
        &mut self,
        _kernel: &mut CeKernel,
        _thread_ids: &[u32],
    ) -> bool {
        false
    }

    pub fn rotate_to_active_visible_receiver_thread(
        &mut self,
        _kernel: &CeKernel,
        _target_thread_ids: &[u32],
    ) -> bool {
        false
    }

    pub fn rotate_to_active_receiver_thread(
        &mut self,
        _kernel: &CeKernel,
        _target_thread_ids: &[u32],
    ) -> bool {
        false
    }

    pub fn reconcile_active_visible_window_thread(&mut self, _kernel: &CeKernel) -> bool {
        false
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
            initial_thread_id: 1,
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

    fn clear_orphaned_cross_process_send_yield(&mut self, _kernel: &CeKernel) -> bool {
        false
    }

    fn has_parked_child_processes(&self) -> bool {
        false
    }

    fn parked_process_debug_text(&self, _kernel: &CeKernel) -> String {
        "  parked processes: none\n".to_owned()
    }

    fn blocked_wait_debug_text(&self, _kernel: &CeKernel) -> String {
        "  cpu blocked waits: none\n".to_owned()
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
