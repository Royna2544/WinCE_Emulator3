// ARM architecture stub — placeholder for future ARM emulation support.

use crate::error::Result;
use unicorn_engine::Unicorn;

use super::arch::ArchHooks;

pub(crate) struct ArmArch;

#[derive(Debug, Clone, Default)]
pub(crate) struct ArmGuestContext {
    pub regs: [u32; 16],
    pub cpsr: u32,
}

impl ArchHooks for ArmArch {
    type Context = ArmGuestContext;

    fn read_pc<D>(_uc: &Unicorn<'_, D>) -> u32 {
        todo!("ARM read_pc")
    }

    fn write_pc<D>(_uc: &mut Unicorn<'_, D>, _val: u32) {
        todo!("ARM write_pc")
    }

    fn read_sp<D>(_uc: &Unicorn<'_, D>) -> u32 {
        todo!("ARM read_sp")
    }

    fn read_return_addr<D>(_uc: &Unicorn<'_, D>) -> u32 {
        todo!("ARM read_return_addr")
    }

    fn write_return_val<D>(_uc: &mut Unicorn<'_, D>, _val: u32) {
        todo!("ARM write_return_val")
    }

    fn save_context<D>(_uc: &Unicorn<'_, D>) -> ArmGuestContext {
        todo!("ARM save_context")
    }

    fn restore_context<D>(_uc: &mut Unicorn<'_, D>, _ctx: &ArmGuestContext) -> Result<()> {
        todo!("ARM restore_context")
    }

    fn is_delay_slot(_pc: u32, _read_u32: &dyn Fn(u32) -> Option<u32>) -> bool {
        false // ARM Thumb/A32 has no branch-delay slots
    }

    fn gpr_name(n: usize) -> &'static str {
        match n {
            0 => "r0",
            1 => "r1",
            2 => "r2",
            3 => "r3",
            4 => "r4",
            5 => "r5",
            6 => "r6",
            7 => "r7",
            8 => "r8",
            9 => "r9",
            10 => "r10",
            11 => "fp",
            12 => "ip",
            13 => "sp",
            14 => "lr",
            15 => "pc",
            _ => "?",
        }
    }
}
