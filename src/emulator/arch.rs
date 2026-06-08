// Hook interface for architecture-specific CPU operations.
// Implement for each target ISA. run_with_unicorn is currently monolithic MIPS code;
// this trait provides the interface for plugging in other architectures in the future.

#[cfg(feature = "unicorn")]
use crate::error::Result;
#[cfg(feature = "unicorn")]
use unicorn_engine::Unicorn;

#[cfg(feature = "unicorn")]
pub(crate) trait ArchHooks {
    /// Saved register context for context-switch save/restore.
    type Context: Default + Clone + std::fmt::Debug;

    fn read_pc<D>(uc: &Unicorn<'_, D>) -> u32;
    fn write_pc<D>(uc: &mut Unicorn<'_, D>, val: u32);
    fn read_sp<D>(uc: &Unicorn<'_, D>) -> u32;
    fn read_return_addr<D>(uc: &Unicorn<'_, D>) -> u32;
    fn write_return_val<D>(uc: &mut Unicorn<'_, D>, val: u32);

    fn save_context<D>(uc: &Unicorn<'_, D>) -> Self::Context;
    fn restore_context<D>(uc: &mut Unicorn<'_, D>, ctx: &Self::Context) -> Result<()>;

    /// Returns true if `pc` is inside a branch-delay slot.
    /// `read_u32` reads a guest word at the given address.
    fn is_delay_slot(pc: u32, read_u32: &dyn Fn(u32) -> Option<u32>) -> bool;

    fn gpr_name(n: usize) -> &'static str;
}
