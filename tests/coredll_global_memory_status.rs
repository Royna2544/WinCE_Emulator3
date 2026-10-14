use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllGuestMemory, CoredllValue},
        coredll_ordinals::{ORD_GLOBAL_MEMORY_STATUS, ORD_VIRTUAL_ALLOC, ORD_VIRTUAL_FREE},
        kernel::CeKernel,
        memory::{
            MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PreparedPrivateVm, PreparedPrivateVmChange,
        },
        thread::ERROR_NOT_ENOUGH_MEMORY,
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

const STATUS_PTR: u32 = 0x12_000;
const DEVICE_TOTAL_VIRTUAL_BYTES: u32 = 1_073_741_824;
const DEVICE_AVAIL_VIRTUAL_BYTES: u32 = 1_073_152_000;

fn query_status<M: CoredllGuestMemory>(
    table: &CoredllExportTable,
    kernel: &mut CeKernel,
    memory: &mut M,
) -> Result<[u32; 8]> {
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            kernel,
            memory,
            1,
            ORD_GLOBAL_MEMORY_STATUS,
            [STATUS_PTR],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::U32(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(1), 0);
    let mut fields = [0; 8];
    for (index, field) in fields.iter_mut().enumerate() {
        *field = memory.read_u32(STATUS_PTR + (index as u32) * 4)?;
    }
    Ok(fields)
}

#[test]
fn public_global_memory_status_matches_device_boot_report() -> Result<()> {
    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.guest_physical_ram_bytes = 195_350_528;
    let mut kernel = CeKernel::boot(config);
    let mut memory = TestGuestMemory::default();
    memory.map_words(STATUS_PTR, 8);

    assert_eq!(
        query_status(&table, &mut kernel, &mut memory)?,
        [
            32,
            16,
            195_350_528,
            164_368_384,
            195_350_528,
            164_368_384,
            DEVICE_TOTAL_VIRTUAL_BYTES,
            DEVICE_AVAIL_VIRTUAL_BYTES,
        ]
    );
    Ok(())
}

#[test]
fn public_global_memory_status_scales_and_clamps_configured_guest_ram() -> Result<()> {
    let table = CoredllExportTable::default();
    // Independent exact ratios from the device's 195_350_528/164_368_384
    // physical-memory report. The final case exceeds both u32 field limits.
    for (physical_bytes, available_bytes, reported_total, reported_available) in [
        (100_663_296, 84_698_329, 100_663_296, 84_698_329),
        (201_326_592, 169_396_658, 201_326_592, 169_396_658),
        (4_294_967_296, 3_613_795_370, u32::MAX, 3_613_795_370),
        (8_589_934_592, 7_227_590_741, u32::MAX, u32::MAX),
    ] {
        let mut config = RuntimeConfig::load_default()?;
        config.guest_physical_ram_bytes = physical_bytes;
        let mut kernel = CeKernel::boot(config);
        let mut memory = TestGuestMemory::default();
        memory.map_words(STATUS_PTR, 8);
        assert_eq!(kernel.memory.private_commit_limit_bytes(), physical_bytes);
        assert_eq!(kernel.memory.private_commit_used_bytes(), 0);
        assert_eq!(
            available_bytes.min(u64::from(u32::MAX)) as u32,
            reported_available
        );

        assert_eq!(
            query_status(&table, &mut kernel, &mut memory)?,
            [
                32,
                16,
                reported_total,
                reported_available,
                reported_total,
                reported_available,
                DEVICE_TOTAL_VIRTUAL_BYTES,
                DEVICE_AVAIL_VIRTUAL_BYTES,
            ],
            "configured physical bytes: {physical_bytes}"
        );
    }
    Ok(())
}

#[test]
fn public_global_memory_status_tracks_committed_private_page_delta() -> Result<()> {
    const TOTAL_BYTES: u64 = 16 * 1024;
    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.guest_physical_ram_bytes = TOTAL_BYTES;
    let mut kernel = CeKernel::boot(config);
    let mut memory = MemoryStatusGuestMemory::default();
    memory.0.map_words(STATUS_PTR, 8);

    let baseline = [
        32,
        16,
        TOTAL_BYTES as u32,
        13_785,
        TOTAL_BYTES as u32,
        13_785,
        DEVICE_TOTAL_VIRTUAL_BYTES,
        DEVICE_AVAIL_VIRTUAL_BYTES,
    ];
    assert_eq!(query_status(&table, &mut kernel, &mut memory)?, baseline);

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

    assert_eq!(
        query_status(&table, &mut kernel, &mut memory)?,
        [
            32,
            41,
            TOTAL_BYTES as u32,
            9_689,
            TOTAL_BYTES as u32,
            9_689,
            DEVICE_TOTAL_VIRTUAL_BYTES,
            DEVICE_AVAIL_VIRTUAL_BYTES
        ]
    );

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
    assert_eq!(query_status(&table, &mut kernel, &mut memory)?, baseline);
    Ok(())
}

#[test]
fn public_private_commit_stops_when_reported_physical_memory_is_consumed() -> Result<()> {
    // This configured total scales the device's boot-available bytes to
    // exactly two 4-KiB pages, allowing an exact zero-availability boundary.
    const TOTAL_BYTES: u64 = 9_737;
    const AVAILABLE_BYTES: u32 = 8_192;
    let table = CoredllExportTable::default();
    let mut config = RuntimeConfig::load_default()?;
    config.guest_physical_ram_bytes = TOTAL_BYTES;
    let mut kernel = CeKernel::boot(config);
    let mut memory = MemoryStatusGuestMemory::default();
    memory.0.map_words(STATUS_PTR, 8);

    assert_eq!(
        kernel.memory.private_commit_available_bytes(),
        u64::from(AVAILABLE_BYTES)
    );
    assert_eq!(
        query_status(&table, &mut kernel, &mut memory)?[3],
        AVAILABLE_BYTES
    );
    let reserved_ptr = match table.dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        1,
        ORD_VIRTUAL_ALLOC,
        [0, 4096, MEM_RESERVE, 0x04],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(ptr),
            ..
        } => ptr,
        other => panic!("VirtualAlloc reserve result: {other:?}"),
    };
    assert_ne!(reserved_ptr, 0);
    assert_eq!(
        kernel.memory.private_commit_available_bytes(),
        u64::from(AVAILABLE_BYTES)
    );
    let mut private_ptrs = Vec::new();
    for expected_available in [4_096, 0] {
        let ptr = match table.dispatch_raw_ordinal_with_memory(
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
        assert_ne!(ptr, 0);
        private_ptrs.push(ptr);
        assert_eq!(
            kernel.memory.private_commit_available_bytes(),
            expected_available
        );
        assert_eq!(
            query_status(&table, &mut kernel, &mut memory)?[3],
            expected_available as u32
        );
    }
    assert_eq!(kernel.memory.private_commit_used_bytes(), 8_192);
    assert_eq!(query_status(&table, &mut kernel, &mut memory)?[1], 100);

    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            1,
            ORD_VIRTUAL_ALLOC,
            [reserved_ptr, 4096, MEM_COMMIT, 0x04],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.threads.get_last_error(1), ERROR_NOT_ENOUGH_MEMORY);
    assert_eq!(kernel.memory.private_commit_used_bytes(), 8_192);
    assert_eq!(query_status(&table, &mut kernel, &mut memory)?[3], 0);

    for (ptr, expected_available) in private_ptrs.into_iter().zip([4_096, 8_192]) {
        assert!(matches!(
            table.dispatch_raw_ordinal_with_memory(
                &mut kernel,
                &mut memory,
                1,
                ORD_VIRTUAL_FREE,
                [ptr, 0, MEM_RELEASE],
            ),
            CoredllDispatch::Returned {
                value: CoredllValue::Bool(true),
                ..
            }
        ));
        assert_eq!(
            kernel.memory.private_commit_available_bytes(),
            expected_available
        );
        assert_eq!(
            query_status(&table, &mut kernel, &mut memory)?[3],
            expected_available as u32
        );
    }
    assert!(matches!(
        table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            1,
            ORD_VIRTUAL_FREE,
            [reserved_ptr, 0, MEM_RELEASE],
        ),
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert_eq!(
        kernel.memory.private_commit_available_bytes(),
        u64::from(AVAILABLE_BYTES)
    );
    Ok(())
}
