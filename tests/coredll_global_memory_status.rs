use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{ORD_GLOBAL_MEMORY_STATUS, ORD_VIRTUAL_ALLOC, ORD_VIRTUAL_FREE},
        kernel::CeKernel,
        memory::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PreparedPrivateVm, PreparedPrivateVmChange},
    },
    config::RuntimeConfig,
};

mod support;
use support::TestGuestMemory;

/// API-level adapter: the MemorySystem's stable pages are this test's backing;
/// there is no native engine to map or unmap.
#[derive(Default)]
struct MemoryStatusGuestMemory(TestGuestMemory);

impl CoredllGuestMemory for MemoryStatusGuestMemory {
    fn read_u8(&self, addr: u32) -> Result<u8> {
        self.0.read_u8(addr)
    }

    fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        self.0.write_u8(addr, value)
    }

    fn read_u32(&self, addr: u32) -> Result<u32> {
        self.0.read_u32(addr)
    }

    fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.0.write_u32(addr, value)
    }

    fn read_u16(&self, addr: u32) -> Result<u16> {
        self.0.read_u16(addr)
    }

    fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        self.0.write_u16(addr, value)
    }

    fn apply_prepared_private_vm(
        &mut self,
        _kernel: &CeKernel,
        _prepared: &PreparedPrivateVm,
    ) -> Result<()> {
        Ok(())
    }

    fn apply_prepared_private_vm_change(
        &mut self,
        _kernel: &CeKernel,
        _change: &PreparedPrivateVmChange,
    ) -> Result<()> {
        Ok(())
    }
}

#[test]
fn public_global_memory_status_uses_configured_guest_ram_budget() -> Result<()> {
    const STATUS_PTR: u32 = 0x12_000;
    const MIB: u64 = 1024 * 1024;
    let table = CoredllExportTable::default();

    for physical_bytes in [96 * MIB, 192 * MIB, u64::from(u32::MAX) + 1] {
        let mut config = RuntimeConfig::load_default()?;
        config.guest_physical_ram_bytes = physical_bytes;
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestGuestMemory::default();
        memory.map_words(STATUS_PTR, 8);
        assert_eq!(kernel.memory.private_commit_limit_bytes(), physical_bytes);
        assert_eq!(kernel.memory.private_commit_used_bytes(), 0);

        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                1,
                ORD_GLOBAL_MEMORY_STATUS,
                [STATUS_PTR],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::U32(0),
                ..
            }
        ));
        let reported_bytes = physical_bytes.min(u64::from(u32::MAX)) as u32;
        assert_eq!(memory.read_u32(STATUS_PTR)?, 32);
        assert_eq!(memory.read_u32(STATUS_PTR + 4)?, 0);
        assert_eq!(memory.read_u32(STATUS_PTR + 8)?, reported_bytes);
        assert_eq!(memory.read_u32(STATUS_PTR + 12)?, reported_bytes);
        assert_eq!(memory.read_u32(STATUS_PTR + 16)?, reported_bytes);
        assert_eq!(memory.read_u32(STATUS_PTR + 20)?, reported_bytes);
        assert_eq!(kernel.threads.get_last_error(1), 0);
    }

    Ok(())
}

#[test]
fn public_global_memory_status_tracks_committed_private_page_delta() -> Result<()> {
    const STATUS_PTR: u32 = 0x12_000;
    const TOTAL_BYTES: u64 = 16 * 1024;
    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.guest_physical_ram_bytes = TOTAL_BYTES;
    let mut kernel = CeKernel::boot(config);
    let mut memory = MemoryStatusGuestMemory::default();
    memory.0.map_words(STATUS_PTR, 8);

    let private_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        1,
        ORD_VIRTUAL_ALLOC,
        [0, 4096, MEM_RESERVE | MEM_COMMIT, 0x04], // PAGE_READWRITE
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("VirtualAlloc result: {other:?}"),
    };
    assert_ne!(private_ptr, 0);
    assert_eq!(kernel.memory.private_commit_used_bytes(), 4096);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            1,
            ORD_GLOBAL_MEMORY_STATUS,
            [STATUS_PTR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(STATUS_PTR + 4)?, 25);
    assert_eq!(memory.read_u32(STATUS_PTR + 8)?, TOTAL_BYTES as u32);
    assert_eq!(memory.read_u32(STATUS_PTR + 12)?, 12 * 1024);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            1,
            ORD_VIRTUAL_FREE,
            [private_ptr, 0, MEM_RELEASE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(kernel.memory.private_commit_used_bytes(), 0);
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            1,
            ORD_GLOBAL_MEMORY_STATUS,
            [STATUS_PTR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(memory.read_u32(STATUS_PTR + 4)?, 0);
    assert_eq!(memory.read_u32(STATUS_PTR + 12)?, TOTAL_BYTES as u32);
    Ok(())
}
