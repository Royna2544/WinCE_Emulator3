// D-007 historical Stage 3 E-8 task-identity API controls.
// The expected-red tests are compiled but ignored until the Stage 3 gate opens.

use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllValue},
        coredll_ordinals::{ORD_GET_PROCESS_ID_OF_THREAD, ORD_OPEN_THREAD},
        kernel::CeKernel,
    },
    config::RuntimeConfig,
};

mod support;
use support::TestGuestMemory;

#[test]
#[ignore = "expected red: E-8 Stage 3 task identity API"]
fn open_thread_returns_independent_alias_to_same_live_task() -> Result<()> {
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let (original, tid) = kernel.create_guest_thread(0x1234, 0x5678, false);
    let mut memory = TestGuestMemory::default();
    let opened = match CoredllExportTable::default().dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        1,
        ORD_OPEN_THREAD,
        [0, 0, tid],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(handle),
            ..
        } => handle,
        result => panic!("OpenThread returned {result:?}"),
    };
    assert_ne!(opened, 0);
    assert_ne!(
        opened, original,
        "OpenThread must allocate a new public alias"
    );
    assert_eq!(kernel.guest_thread_id(opened), Some(tid));
    assert_eq!(kernel.guest_thread_id(original), Some(tid));

    kernel.close_handle(opened)?;
    assert_eq!(kernel.guest_thread_id(original), Some(tid));
    kernel.close_handle(original)?;
    assert_eq!(kernel.guest_thread_handle_by_id(tid), None);
    Ok(())
}

#[test]
#[ignore = "expected red: E-8 Stage 3 task identity API"]
fn get_process_id_of_thread_reports_immutable_creation_owner() -> Result<()> {
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let child = kernel.queue_process_launch(Some("\\Test\\child.exe".to_owned()), None);
    let _drained = kernel.take_pending_process_launches();
    assert_ne!(child.process_id, kernel.current_process_id());
    let mut memory = TestGuestMemory::default();
    let owner = match CoredllExportTable::default().dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        1,
        ORD_GET_PROCESS_ID_OF_THREAD,
        [child.thread_handle],
    ) {
        CoredllDispatch::Returned {
            value: CoredllValue::U32(pid),
            ..
        } => pid,
        result => panic!("GetProcessIdOfThread returned {result:?}"),
    };
    assert_eq!(owner, child.process_id);
    Ok(())
}
