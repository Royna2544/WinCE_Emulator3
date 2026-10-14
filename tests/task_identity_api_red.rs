// D-007 historical Stage 3 E-8 task-identity API controls.
// OpenThread aliasing is active; authentic creation-owner lookup remains red.

use wince_emulation_v3::{
    Result,
    ce::{
        coredll::{CoredllDispatch, CoredllExportTable, CoredllValue},
        coredll_ordinals::{
            ORD_CREATE_THREAD, ORD_EXIT_THREAD, ORD_GET_PROCESS_ID_OF_THREAD, ORD_OPEN_PROCESS,
            ORD_OPEN_THREAD,
        },
        kernel::{CE_CURRENT_PROCESS_PSEUDO_HANDLE, CeKernel},
    },
    config::RuntimeConfig,
};

mod support;
use support::TestGuestMemory;

#[test]
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
fn failed_create_thread_output_does_not_publish_a_live_birth() -> Result<()> {
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let (_, preceding_tid) = kernel.create_guest_thread(0x1234, 0, false);
    let unpublished_tid = preceding_tid.checked_add(1).expect("test TID range");
    let mut memory = TestGuestMemory::default();
    let invalid_nonnull_tid_ptr = 0x7000;
    let result = CoredllExportTable::default().dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        1,
        ORD_CREATE_THREAD,
        [0, 0, 0x5678, 0, 0, invalid_nonnull_tid_ptr],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::Handle(0),
            ..
        }
    ));
    assert_eq!(kernel.handles.thread_birth_key_by_id(unpublished_tid), None);
    assert_eq!(kernel.open_thread_handle(unpublished_tid), None);
    Ok(())
}

#[test]
fn exit_thread_retires_live_birth_after_last_public_alias_closes() -> Result<()> {
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let (only_handle, tid) = kernel.create_guest_thread(0x1234, 0, false);
    let birth = kernel
        .handles
        .thread_birth_key_by_id(tid)
        .expect("live worker birth");
    assert!(kernel.close_handle(only_handle)?);
    assert_eq!(kernel.guest_thread_handle_by_id(tid), None);
    assert!(kernel.handles.contains_thread_birth(birth));

    let mut memory = TestGuestMemory::default();
    let result = CoredllExportTable::default().dispatch_raw_ordinal_with_memory(
        &mut kernel,
        &mut memory,
        tid,
        ORD_EXIT_THREAD,
        [0x7654],
    );
    assert!(matches!(
        result,
        CoredllDispatch::Returned {
            value: CoredllValue::Bool(true),
            ..
        }
    ));
    assert!(
        !kernel.handles.contains_thread_birth(birth),
        "normal ExitThread must release execution ownership without a public alias"
    );
    assert_eq!(kernel.open_thread_handle(tid), None);
    Ok(())
}

#[test]
fn open_process_current_pid_returns_distinct_closable_aliases_with_shared_exit() -> Result<()> {
    let mut kernel = CeKernel::boot(RuntimeConfig::load_default()?);
    let process_id = kernel.current_process_id();
    let mut memory = TestGuestMemory::default();
    let table = CoredllExportTable::default();
    let mut aliases = Vec::new();
    for _ in 0..2 {
        let alias = match table.dispatch_raw_ordinal_with_memory(
            &mut kernel,
            &mut memory,
            1,
            ORD_OPEN_PROCESS,
            [0, 0, process_id],
        ) {
            CoredllDispatch::Returned {
                value: CoredllValue::Handle(handle),
                ..
            } => handle,
            result => panic!("OpenProcess(current PID) returned {result:?}"),
        };
        assert_ne!(alias, 0);
        assert_ne!(alias, CE_CURRENT_PROCESS_PSEUDO_HANDLE);
        assert_eq!(kernel.process_id(alias), Some(process_id));
        aliases.push(alias);
    }
    assert_ne!(aliases[0], aliases[1]);
    assert_eq!(
        kernel.process_id_for_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE),
        Some(process_id),
        "explicit opens must not replace GetCurrentProcess's pseudo-handle"
    );

    assert!(kernel.terminate_process(CE_CURRENT_PROCESS_PSEUDO_HANDLE, 0x7654));
    assert_eq!(kernel.process_exit_code(aliases[0]), Some(0x7654));
    assert_eq!(kernel.process_exit_code(aliases[1]), Some(0x7654));
    assert!(kernel.close_handle(aliases[0])?);
    assert_eq!(kernel.process_id(aliases[0]), None);
    assert_eq!(kernel.process_exit_code(aliases[1]), Some(0x7654));
    assert!(kernel.close_handle(aliases[1])?);
    assert_eq!(kernel.process_id(aliases[1]), None);
    assert_eq!(
        kernel.process_id_for_handle(CE_CURRENT_PROCESS_PSEUDO_HANDLE),
        Some(process_id),
        "closing explicit aliases must not close the current-process pseudo-handle"
    );
    Ok(())
}

#[test]
#[ignore = "expected red: raw GetProcessIdOfThread lacks immutable creation owner"]
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
