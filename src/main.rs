#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fmt::Write as FmtWrite,
    fs,
    io::{self, Write},
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

#[cfg(any(
    all(windows, feature = "win32-audio"),
    all(target_os = "linux", feature = "linux-audio")
))]
use wince_emulation_v3::ce::audio::HostAudioSink;
#[allow(unused_imports)]
use wince_emulation_v3::emulator::cpu::CpuBackend as _;
use wince_emulation_v3::{
    Result,
    ce::{
        audio::WaveFormat,
        desktop::{VirtualDesktop, VirtualInputEvent},
        framebuffer::{Framebuffer, FramebufferInfo, FramebufferRect, VirtualFramebuffer},
        gwe::WM_TIMER,
        kernel::{CeKernel, LoadedModuleMetadata},
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE},
        scheduler::SchedulerBlockedWaitKind,
    },
    config::{
        DEFAULT_DEVICES_PATH, DEFAULT_MOUNT_CONFIG_PATH, DEFAULT_REGISTRY_PATH, RuntimeConfig,
    },
    emulator::{
        cpu::{
            AttemptExecution, UnicornDebugSnapshot, UnicornMips, UnicornRunLimits,
            UnicornWindowSnapshot,
        },
        dll_search::{emulator_provided_import_module, normalize_module_name, resolve_dll_path},
        memory::MemoryPerms,
    },
    pe::PeImage,
    remote_server::{RemoteServer, RemoteServerConfig},
};

const FAST_START_RUN_SLICE_INSTRUCTIONS: usize = 250_000;
// Live-pump slice length, in guest instructions. This doubles as the
// round-robin tick between processes. A non-zero `emu_start` count makes
// Unicorn install a per-instruction counting hook, so 0 (unbounded; time-bound
// by the native timeout and wall-clock stops) is ~2x faster on busy slices, but
// it changes scheduling dynamics (parked helpers can starve), so the default
// stays until a real time-based quantum and native-timeout context preservation
// are validated. The env override below accepts 0 for A/B runs.
const LIVE_PUMP_RUN_SLICE_INSTRUCTIONS: usize = 1_000_000;
const HOST_LIVE_RUN_SLICE_MS: u64 = 120_000;
const HOST_IDLE_MESSAGE_POLL_SLICE_MS: u64 = 100;
const LIVE_VISIBLE_WORK_RUN_SLICE_MS: u64 = 100;
const HOST_REMOTE_BUSY_RUN_SLICE_MS: u64 = 30_000;
// Long busy slices: the in-run hooks already drain remote control
// messages and ticks the framebuffer while the guest executes, and a fully
// blocked guest exits the run on its own, so short forced exits only paid the
// fresh-Unicorn restart cost (blob copy + retranslation) every second.
const REMOTE_LIVE_RUN_SLICE_MS: u64 = HOST_REMOTE_BUSY_RUN_SLICE_MS;
const REMOTE_LIVE_RUN_SLICE_ENV: &str = "WINCE_EMU_REMOTE_LIVE_RUN_SLICE_MS";
const LIVE_PUMP_RUN_SLICE_INSTRUCTIONS_ENV: &str = "WINCE_EMU_LIVE_PUMP_RUN_SLICE_INSTRUCTIONS";
const REMOTE_DEBUG_PUBLISH_INTERVAL_MS: u64 = 500;
const REMOTE_DEBUG_PUBLISH_INTERVAL_ENV: &str = "WINCE_EMU_REMOTE_DEBUG_PUBLISH_INTERVAL_MS";
const COMPANION_START_DELAY_MS: u64 = 1_000;
const COMPANION_INSTRUCTION_LIMIT: usize = 250_000_000;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone)]
struct Args {
    registry: PathBuf,
    devices: PathBuf,
    image: Option<PathBuf>,
    companion_images: Vec<PathBuf>,
    dll_search_dirs: Vec<PathBuf>,
    mount_config: Option<PathBuf>,
    framebuffer_dump: Option<PathBuf>,
    tracefiles: Vec<(String, PathBuf)>,
    desktop: DesktopMode,
    cpu_instruction_limit: usize,
    cpu_wall_clock_limit_ms: u64,
    cpu_stop_pc: Option<u32>,
    startup_taps: Vec<(i32, i32)>,
    remote_server: Option<RemoteServerConfig>,
    run_cpu: bool,
    monitor: bool,
    verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopMode {
    Virtual,
    Host,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockedRemoteInputTarget {
    thread_id: u32,
    hwnd: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RemoteEndpointDrain {
    handled: usize,
    target_thread_ids: Vec<u32>,
}

enum DesktopRuntime {
    Virtual(VirtualDesktop),
    #[cfg(all(windows, feature = "win32-desktop"))]
    Host(
        VirtualDesktop<
            wince_emulation_v3::ce::win32_desktop::Win32Input,
            wince_emulation_v3::ce::win32_desktop::Win32Presenter,
        >,
    ),
    #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
    Host(
        VirtualDesktop<
            wince_emulation_v3::ce::linux_x11_desktop::LinuxX11Input,
            wince_emulation_v3::ce::linux_x11_desktop::LinuxX11Presenter,
        >,
    ),
}

struct CompanionProcesses {
    stop: Arc<AtomicBool>,
    children: Arc<Mutex<Vec<Child>>>,
    launcher: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
struct CompanionLaunchSpec {
    executable: PathBuf,
    target: PathBuf,
    args: Vec<OsString>,
    stdout: PathBuf,
    stderr: PathBuf,
}

#[derive(Clone)]
struct MonitorCheckpoint {
    name: String,
    cpu: UnicornMips,
    kernel: CeKernel,
    framebuffer: VirtualFramebuffer,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let args = Args::parse()?;
    let config = RuntimeConfig::load_with_mounts(
        &args.registry,
        &args.devices,
        args.mount_config.as_deref(),
    )?;
    let mut kernel = CeKernel::boot(config);
    let mut host_audio_status = attach_audio_for_desktop(&mut kernel, args.desktop);
    let mut desktop = create_desktop(args.desktop, args.image.as_deref())?;
    kernel.remote.set_framebuffer_size(
        desktop.framebuffer().width(),
        desktop.framebuffer().height(),
    );
    if let Some(config) = args.remote_server.clone() {
        let remote_audio_enabled = config.audio_enabled;
        let server = RemoteServer::start(config)?;
        if remote_audio_enabled {
            if kernel.audio.register_sink(server.audio_sink()) {
                host_audio_status.push_str("; remote websocket audio sink registered");
            } else {
                host_audio_status.push_str("; remote websocket audio sink already registered");
            }
        }
        kernel.set_remote_server(server);
        publish_remote_endpoint(
            kernel.remote_server.as_ref(),
            None,
            &kernel,
            desktop.framebuffer(),
            None,
            true,
        );
    }

    let mut cpu = UnicornMips::new()?;
    cpu.set_dll_search_dirs(args.dll_search_dirs.clone());
    if args.image.is_none() {
        cpu.map_region(
            0x0001_0000,
            0x0010_0000,
            MemoryPerms::READ | MemoryPerms::WRITE | MemoryPerms::EXEC,
            "guest-low",
        )?;
        cpu.map_region(
            0x7fff_0000,
            0x0001_0000,
            MemoryPerms::READ | MemoryPerms::WRITE,
            "ce-shim-trap-page",
        )?;
    }

    let bootstrap_handles = if args.image.is_none() {
        let hwnd = kernel.gwe.create_window(1, "FakeCEBaseWindow", "");
        let timer_id = kernel
            .timers
            .set_timer(1, Some(hwnd), None, 1000, WM_TIMER, None);
        let wave_id = kernel.audio.open_wave_out(WaveFormat::pcm_16bit(2, 44_100));
        Some((hwnd, timer_id, wave_id))
    } else {
        None
    };
    let ident_key = kernel
        .registry
        .reg_open_key_exw(HKEY_LOCAL_MACHINE, Some("Ident"), 0, 0);
    let device_name = if ident_key.status == ERROR_SUCCESS {
        let value =
            kernel
                .registry
                .reg_query_value_exw(ident_key.hkey.unwrap(), Some("Name"), Some(128));
        let _ = kernel.registry.reg_close_key(ident_key.hkey.unwrap());
        format!(
            "status={} type={:?} bytes={}",
            value.status, value.value_type, value.required_len
        )
    } else {
        format!("status={}", ident_key.status)
    };

    if args.verbose {
        println!("FakeCE base booted");
        println!(
            "  registry backing: {} ({} keys)",
            args.registry.display(),
            kernel.registry.key_count()
        );
        println!("  RegOpenKeyExW/RegQueryValueExW HKLM\\Ident\\Name: {device_name}");
        println!("  devices: {}", kernel.devices.enabled_names().join(", "));
        println!(
            "  default serial: {} {}",
            kernel.devices.default_baud(),
            kernel.devices.default_mode()
        );
        println!("  host audio: {host_audio_status}");
        if let Some((hwnd, timer_id, wave_id)) = bootstrap_handles {
            println!("  bootstrap hwnd: 0x{hwnd:08x}");
            println!("  bootstrap timer: {timer_id}");
            println!("  bootstrap waveOut: {wave_id}");
        } else {
            println!("  bootstrap demo state: skipped for PE image");
        }
        println!("  memory regions: {}", cpu.memory().regions().count());
        println!(
            "  framebuffer: {}x{} {:?} stride={} bytes={}",
            desktop.framebuffer().width(),
            desktop.framebuffer().height(),
            desktop.framebuffer().pixel_format(),
            desktop.framebuffer().stride(),
            desktop.framebuffer().pixels().len()
        );
        println!("  desktop: {}", desktop.describe());
    }
    desktop.present()?;

    let pe_image = if let Some(image_path) = args.image.as_ref() {
        let image = PeImage::inspect(image_path)?;
        kernel.set_process_module_base(image.image_base());
        kernel.set_process_module_path(ce_module_path_for_image(&kernel, &image.path));
        kernel.set_process_module_host_path(PathBuf::from(&image.path));
        if args.verbose {
            println!(
                "  PE image: {} ({} bytes, lfanew=0x{:08x}, machine=0x{:04x})",
                image.path, image.len, image.dos_lfanew, image.coff_header.machine
            );
            println!(
                "  PE layout: image_base=0x{:08x} entry_va=0x{:08x} sections={} imports={} exports={} reloc_blocks={}",
                image.image_base(),
                image.entry_point_va(),
                image.sections.len(),
                image.imports.len(),
                image
                    .exports
                    .as_ref()
                    .map_or(0, |exports| exports.functions.len()),
                image.base_relocations.len()
            );
        }
        Some(image)
    } else {
        None
    };

    if let Some(image) = pe_image.as_ref() {
        let dll_images = load_import_dlls(image, &args.dll_search_dirs)?;
        if args.verbose {
            for dll in &dll_images {
                println!(
                    "  DLL image: {} image_base=0x{:08x} size=0x{:08x} reloc_stripped={} reloc_blocks={}",
                    dll.path,
                    dll.image_base(),
                    dll.optional_header.size_of_image,
                    dll.relocations_stripped(),
                    dll.base_relocations.len()
                );
            }
        }
        cpu.load_pe_image_with_dlls(image, &dll_images)?;
        register_loaded_modules(&mut kernel, &cpu);
        if args.verbose {
            println!(
                "  loader: {} DLL(s), {} import trap(s)",
                dll_images.len(),
                cpu.import_traps().len()
            );
        }
    }

    let _companions = launch_delayed_companion_processes(&args)?;
    if args.run_cpu {
        run_cpu_loop(&mut cpu, &mut kernel, &mut desktop, &args, None)?;
        write_requested_tracefiles(&cpu, &args.tracefiles)?;
    }
    if args.monitor {
        run_monitor(&mut cpu, &mut kernel, &mut desktop, &args)?;
    }
    if let Some(path) = args.framebuffer_dump.as_ref() {
        desktop.framebuffer().write_ppm(path)?;
        println!("  framebuffer dump: {}", path.display());
    }

    Ok(())
}

fn run_cpu_loop(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    desktop: &mut DesktopRuntime,
    args: &Args,
    mut attempt_observer: Option<&mut dyn FnMut(&AttemptExecution)>,
) -> Result<()> {
    enqueue_startup_taps(kernel, &args.startup_taps)?;
    let mut reported_blocked_message_wait = false;
    let run_started = Instant::now();
    loop {
        // Scheduler maintenance can rotate/continue without a CPU slice.
        // The requested bound applies to that host work as well as emu_start.
        if wall_clock_limit_expired(args.cpu_wall_clock_limit_ms, run_started.elapsed()) {
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                print_unicorn_stop(snapshot);
            }
            break;
        }
        if cpu.active_process_execution_completed() {
            // The child CPU is gone, but a parked parent may still be blocked
            // on a wait, message, or remote input that becomes ready later.
            // Keep servicing those host events without ever running the
            // completed native owner again.
            // Requested parked-child exits must also be completed here:
            // blocked descendants are not runnable, so switching alone would
            // leave their process waits unsignaled indefinitely.
            cpu.try_prune_exited_and_active_processes_from_parked_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            )?;
            if let Some(thread_id) = cpu.parked_timer_fallback_thread_id() {
                kernel.pump_timers_to_gwe(thread_id);
            }
            if service_completed_active_desktop(cpu, desktop, kernel, args.desktop)? != 0 {
                reported_blocked_message_wait = false;
            }
            let remote_target = blocked_remote_input_target(cpu, kernel);
            service_remote_endpoint(cpu, kernel, desktop, remote_target.as_ref())?;
            if cpu.try_switch_to_next_parked_child_process(kernel)? {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.has_parked_child_processes() {
                let remaining = (args.cpu_wall_clock_limit_ms != 0).then(|| {
                    Duration::from_millis(remaining_wall_clock_limit_ms(
                        args.cpu_wall_clock_limit_ms,
                        run_started.elapsed(),
                    ))
                });
                let delay = completed_cpu_idle_delay(kernel, args.desktop, remaining);
                if let Some(server) = kernel.remote_server.as_ref() {
                    server.wait_for_control_or_timeout(delay);
                } else {
                    std::thread::sleep(delay);
                }
                continue;
            }
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                print_unicorn_stop(snapshot);
            }
            break;
        }
        cpu.try_prune_exited_and_active_processes_from_parked_with_framebuffer(
            kernel,
            Some(desktop.framebuffer_mut()),
        )?;
        // Pruning may itself complete and seal the selected child. Re-enter
        // the completed-owner maintenance lane before any native attempt.
        if cpu.active_process_execution_completed() {
            continue;
        }
        if remote_server_has_pending_control(kernel) {
            let pending_remote_target = blocked_remote_input_target(cpu, kernel);
            let pending_remote_drained =
                service_remote_endpoint(cpu, kernel, desktop, pending_remote_target.as_ref())?;
            if pending_remote_drained.handled != 0 {
                reported_blocked_message_wait = false;
                let current_cpu_targeted = pending_remote_drained
                    .target_thread_ids
                    .contains(&cpu.current_thread_id());
                if cpu.rotate_to_active_visible_receiver_thread(
                    kernel,
                    &pending_remote_drained.target_thread_ids,
                ) {
                    publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                    continue;
                }
                if cpu.rotate_to_ready_parked_threads_with_framebuffer(
                    kernel,
                    &pending_remote_drained.target_thread_ids,
                    Some(desktop.framebuffer_mut()),
                ) {
                    publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                    continue;
                }
                if !cpu.active_process_has_visible_receiver_work(kernel)
                    && cpu.rotate_to_runnable_parked_threads(
                        kernel,
                        &pending_remote_drained.target_thread_ids,
                    )
                {
                    publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                    continue;
                }
                if !current_cpu_targeted
                    && cpu.has_runnable_parked_process(kernel)
                    && cpu.rotate_to_next_parked_process(kernel)
                {
                    publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                    continue;
                }
                continue;
            }
        }
        if cpu.complete_escaped_saved_get_message_sent_callout(kernel)? {
            tracing::debug!(target: "ce.gwe", "completed escaped saved GetMessage sent callout at main-loop head");
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.complete_escaped_direct_send_message_callout(kernel)? {
            tracing::debug!(target: "ce.gwe", "completed escaped direct send callout at main-loop head");
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.clear_escaped_visible_message_callouts(kernel) {
            tracing::debug!(target: "ce.gwe", "escaped visible-message callout cleared at main-loop head");
            reported_blocked_message_wait = false;
            continue;
        }
        if args.remote_server.is_some()
            && !cpu.slice_end_rotation_pending()
            && !cpu.active_process_has_visible_receiver_work(kernel)
            && cpu.active_should_yield_to_visible_parked_process(kernel)
            && cpu.rotate_to_visible_window_parked_process(kernel)
        {
            tracing::debug!(target: "ce.quantum", "main loop yielded active to visible parked process");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if args.remote_server.is_some()
            && !cpu.slice_end_rotation_pending()
            && rotate_idle_active_to_visible_parked(cpu, kernel, desktop)
        {
            tracing::debug!(target: "ce.quantum", "main loop rotated idle active to visible parked process");
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.clear_orphaned_send_depths(kernel) {
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.clear_orphaned_direct_send_callouts(kernel) {
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.complete_orphaned_active_send_wait(kernel) {
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.complete_orphaned_parked_send_wait(kernel) {
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.clear_orphaned_cross_process_send_yield(kernel) {
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.complete_ready_active_modal_message_box_with_framebuffer(
            kernel,
            Some(desktop.framebuffer_mut()),
        ) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.try_complete_active_process_thread_exit(kernel, Some(desktop.framebuffer_mut()))? {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if rotate_to_cross_process_send_target(cpu, kernel) {
            tracing::debug!(target: "ce.quantum", "main loop rotated to cross-process send target");
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.reconcile_active_visible_window_thread(kernel) {
            tracing::debug!(target: "ce.quantum", "main loop reconciled active visible window thread");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        let early_blocked_remote_target = blocked_remote_input_target(cpu, kernel);
        let early_remote_drained =
            service_remote_endpoint(cpu, kernel, desktop, early_blocked_remote_target.as_ref())?;
        if early_remote_drained.handled != 0 {
            reported_blocked_message_wait = false;
            let current_cpu_targeted = early_remote_drained
                .target_thread_ids
                .contains(&cpu.current_thread_id());
            if cpu.rotate_to_active_visible_receiver_thread(
                kernel,
                &early_remote_drained.target_thread_ids,
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.rotate_to_ready_parked_threads_with_framebuffer(
                kernel,
                &early_remote_drained.target_thread_ids,
                Some(desktop.framebuffer_mut()),
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if !cpu.active_process_has_visible_receiver_work(kernel)
                && cpu.rotate_to_runnable_parked_threads(
                    kernel,
                    &early_remote_drained.target_thread_ids,
                )
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if !current_cpu_targeted
                && cpu.has_runnable_parked_process(kernel)
                && cpu.rotate_to_next_parked_process(kernel)
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            continue;
        }
        if cpu.prepare_active_orphaned_visible_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_cross_thread_visible_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if args.remote_server.is_some()
            && cpu.active_process_has_receiver_work(kernel)
            && cpu.rotate_to_active_receiver_thread(kernel, &[])
        {
            tracing::debug!(target: "ce.quantum", "main loop rotated to active receiver thread");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_active_sent_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        let blocked_remote_target = blocked_remote_input_target(cpu, kernel);
        let remote_drained =
            service_remote_endpoint(cpu, kernel, desktop, blocked_remote_target.as_ref())?;
        if remote_drained.handled != 0 {
            reported_blocked_message_wait = false;
            let current_cpu_targeted = remote_drained
                .target_thread_ids
                .contains(&cpu.current_thread_id());
            if cpu
                .rotate_to_active_visible_receiver_thread(kernel, &remote_drained.target_thread_ids)
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.rotate_to_ready_parked_threads_with_framebuffer(
                kernel,
                &remote_drained.target_thread_ids,
                Some(desktop.framebuffer_mut()),
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.rotate_to_ready_parked_wait_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if should_rotate_remote_input_receiver_parked_process(
                cpu.active_process_has_visible_receiver_work(kernel),
            ) && cpu.rotate_to_receiver_parked_process_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if !cpu.active_process_has_visible_receiver_work(kernel)
                && cpu.rotate_to_runnable_parked_threads(kernel, &remote_drained.target_thread_ids)
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if !current_cpu_targeted
                && cpu.has_runnable_parked_process(kernel)
                && cpu.rotate_to_next_parked_process(kernel)
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if current_cpu_targeted && cpu.prepare_active_orphaned_visible_message_callout(kernel) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.prepare_cross_thread_visible_message_callout(kernel) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if current_cpu_targeted && cpu.prepare_active_sent_message_callout(kernel) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            continue;
        }
        let desktop_queued =
            enqueue_desktop_input_for_current_wait(cpu, desktop, kernel, args.desktop)?;
        if desktop_queued != 0 {
            reported_blocked_message_wait = false;
        }
        if rotate_to_same_process_visible_receiver_thread(cpu, kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if (remote_drained.handled != 0 || desktop_queued != 0)
            && !cpu.active_process_has_visible_receiver_work(kernel)
            && cpu.rotate_to_receiver_parked_process_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            )
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if args.remote_server.is_some()
            && !cpu.slice_end_rotation_pending()
            && remote_drained.handled == 0
            && desktop_queued == 0
            && !cpu.active_process_has_visible_windows(kernel)
            && should_rotate_idle_runnable_parked_process(
                cpu.active_process_has_visible_receiver_work(kernel),
                cpu.has_runnable_parked_process(kernel),
            )
            && cpu.rotate_to_next_parked_process(kernel)
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if args.remote_server.is_some()
            && !cpu.slice_end_rotation_pending()
            && remote_drained.handled == 0
            && desktop_queued == 0
            && should_rotate_idle_remote_receiver_parked_process(
                cpu.active_process_has_receiver_work(kernel),
                cpu.active_process_has_visible_receiver_work(kernel),
            )
            && !cpu.active_process_has_visible_windows(kernel)
            && cpu.rotate_to_receiver_parked_process_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            )
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if !cpu.slice_end_rotation_pending()
            && cpu.rotate_to_ready_parked_wait_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            )
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if !cpu.slice_end_rotation_pending()
            && !cpu.active_process_has_visible_windows(kernel)
            && !cpu.active_process_has_visible_receiver_work(kernel)
            && cpu.has_runnable_parked_process(kernel)
            && cpu.rotate_to_next_parked_process(kernel)
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if rotate_to_cross_process_send_target(cpu, kernel) {
            tracing::debug!(target: "ce.quantum", "main loop rotated to cross-process send target");
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.reconcile_active_visible_window_thread(kernel) {
            tracing::debug!(target: "ce.quantum", "main loop reconciled active visible window thread");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_active_orphaned_visible_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_cross_thread_visible_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_active_sent_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if args.remote_server.is_some()
            && remote_drained.handled == 0
            && desktop_queued == 0
            && !cpu.has_saved_context()
            && !cpu.active_process_has_visible_receiver_work(kernel)
            && cpu
                .last_debug_snapshot()
                .is_some_and(|snapshot| snapshot_has_orphaned_cross_process_send(snapshot, kernel))
            && cpu.has_runnable_parked_process(kernel)
            && cpu.rotate_to_next_parked_process(kernel)
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if args.remote_server.is_some()
            && remote_drained.handled == 0
            && desktop_queued == 0
            && !cpu.has_saved_context()
            && !cpu.active_process_has_visible_receiver_work(kernel)
            && cpu.last_debug_snapshot().is_some_and(|snapshot| {
                snapshot_has_only_stale_owned_blocked_waits(snapshot, cpu, kernel)
            })
            && cpu.has_runnable_parked_process(kernel)
            && cpu.rotate_to_next_parked_process(kernel)
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if switch_completed_active_context(cpu, kernel)? {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if args.remote_server.is_some()
            && !cpu.slice_end_rotation_pending()
            && should_rotate_idle_nonrunning_runnable_parked_process(
                cpu.has_running_guest_thread(),
                cpu.active_process_has_visible_queued_receiver_work(kernel),
                cpu.has_runnable_parked_process(kernel),
            )
            && cpu.active_should_yield_to_visible_parked_process(kernel)
            && cpu.rotate_to_next_parked_process(kernel)
        {
            tracing::debug!(target: "ce.quantum", "main loop rotated idle nonrunning active to next parked");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if remote_drained.handled == 0
            && desktop_queued == 0
            && should_idle_host_message_pump(cpu, kernel, args.desktop)
        {
            desktop.present()?;
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }
        let (wall_clock_limit_ms, live_pump_slice) = effective_wall_clock_limit_ms(
            args.cpu_wall_clock_limit_ms,
            run_started.elapsed(),
            args.desktop,
            args.remote_server.is_some(),
            host_idle_message_poll_slice(cpu, args.desktop),
            cpu.active_process_has_visible_receiver_work(kernel)
                || cpu.active_process_has_receiver_work(kernel),
        );
        let instruction_limit = effective_instruction_limit(
            args.cpu_instruction_limit,
            live_pump_slice,
            std::env::var_os("WINCE_EMU_FAST_START").is_some(),
        );
        let execution = desktop.run_cpu_until_report(
            cpu,
            kernel,
            UnicornRunLimits {
                instruction_limit,
                wall_clock_limit_ms,
                stop_pc: args.cpu_stop_pc,
                live_pump: live_pump_slice,
            },
        );
        if let Some(observer) = attempt_observer.as_mut() {
            observer(&execution);
        }
        if let Err(err) = execution.result {
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                eprintln!("  Unicorn debug: {}", snapshot.summary());
            }
            write_requested_tracefiles(cpu, &args.tracefiles)?;
            if let Some(path) = args.framebuffer_dump.as_ref() {
                desktop.framebuffer().write_ppm(path)?;
                eprintln!("  framebuffer dump: {}", path.display());
            }
            if let Err(status_err) = desktop.show_stopped_message("Emulator process stopped") {
                eprintln!("  presenter status update failed: {status_err}");
            }
            return Err(err);
        }
        if requested_pc_stop_reached(
            args.cpu_stop_pc,
            cpu.last_debug_snapshot(),
            cpu.saved_context_pc(),
        ) {
            // A debugger stop owns the selected context before post-run input,
            // callback maintenance, or normal scheduler handoff can replace it.
            desktop.present()?;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                print_unicorn_stop(snapshot);
            }
            break;
        }
        if cpu.active_process_execution_completed() {
            // A native self-termination may have sealed this CPU during the
            // slice. Let the completion lane retire it before any rotation
            // can park an inert context behind a closed process handle.
            continue;
        }
        if enqueue_desktop_input_for_current_wait(cpu, desktop, kernel, args.desktop)? != 0 {
            reported_blocked_message_wait = false;
        }
        desktop.present()?;
        publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
        let blocked_remote_target = blocked_remote_input_target(cpu, kernel);
        let remote_drained =
            service_remote_endpoint(cpu, kernel, desktop, blocked_remote_target.as_ref())?;
        if remote_drained.handled != 0 {
            reported_blocked_message_wait = false;
            let current_cpu_targeted = remote_drained
                .target_thread_ids
                .contains(&cpu.current_thread_id());
            if cpu
                .rotate_to_active_visible_receiver_thread(kernel, &remote_drained.target_thread_ids)
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.rotate_to_ready_parked_threads_with_framebuffer(
                kernel,
                &remote_drained.target_thread_ids,
                Some(desktop.framebuffer_mut()),
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.rotate_to_ready_parked_wait_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if should_rotate_remote_input_receiver_parked_process(
                cpu.active_process_has_visible_receiver_work(kernel),
            ) && cpu.rotate_to_receiver_parked_process_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            ) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if !cpu.active_process_has_visible_receiver_work(kernel)
                && cpu.rotate_to_runnable_parked_threads(kernel, &remote_drained.target_thread_ids)
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if !current_cpu_targeted
                && cpu.has_runnable_parked_process(kernel)
                && cpu.rotate_to_next_parked_process(kernel)
            {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if cpu.prepare_cross_thread_visible_message_callout(kernel) {
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            continue;
        }
        if rotate_to_same_process_visible_receiver_thread(cpu, kernel) {
            tracing::debug!(target: "ce.quantum", "post-run rotated to same-process visible receiver thread");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if !cpu.slice_end_rotation_pending()
            && cpu.rotate_to_ready_parked_wait_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            )
        {
            tracing::debug!(target: "ce.quantum", "post-run rotated to ready parked wait");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if !cpu.slice_end_rotation_pending()
            && !cpu.active_process_has_visible_receiver_work(kernel)
            && !cpu.active_process_has_visible_windows(kernel)
            && cpu.rotate_to_receiver_parked_process_with_framebuffer(
                kernel,
                Some(desktop.framebuffer_mut()),
            )
        {
            tracing::debug!(target: "ce.quantum", "post-run rotated hidden active to receiver parked process");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        let total_wall_clock_expired =
            wall_clock_limit_expired(args.cpu_wall_clock_limit_ms, run_started.elapsed());
        if total_wall_clock_expired {
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                print_unicorn_stop(snapshot);
            }
            break;
        }
        if cpu.try_complete_active_process_thread_exit(kernel, Some(desktop.framebuffer_mut()))? {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if switch_completed_active_context(cpu, kernel)? {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if active_process_exited(cpu, kernel) {
            if should_keep_remote_active_exit_alive(
                args.remote_server.is_some(),
                cpu.active_process_has_visible_receiver_work(kernel),
            ) {
                reported_blocked_message_wait = false;
                std::thread::sleep(Duration::from_millis(16));
                continue;
            }
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                print_unicorn_stop(snapshot);
            }
            break;
        }
        if rotate_to_cross_process_send_target(cpu, kernel) {
            tracing::debug!(target: "ce.quantum", "main loop rotated to cross-process send target");
            reported_blocked_message_wait = false;
            continue;
        }
        if cpu.reconcile_active_visible_window_thread(kernel) {
            tracing::debug!(target: "ce.quantum", "main loop reconciled active visible window thread");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_active_orphaned_visible_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_cross_thread_visible_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if cpu.prepare_active_sent_message_callout(kernel) {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if live_pump_slice
            && !cpu.slice_end_rotation_pending()
            && should_rotate_live_pump_priority_parked_process(
                cpu.active_process_has_visible_queued_receiver_work(kernel),
                cpu.has_live_pump_priority_parked_process(kernel),
            )
            && cpu.rotate_to_next_parked_process(kernel)
        {
            tracing::debug!(target: "ce.quantum", "pre-run rotated for live-pump priority parked process");
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if live_pump_slice
            && !cpu.slice_end_rotation_pending()
            && rotate_idle_active_to_visible_parked(cpu, kernel, desktop)
        {
            tracing::debug!(target: "ce.quantum", "pre-run rotated idle active to visible parked");
            reported_blocked_message_wait = false;
            continue;
        }
        let native_service_boundary = cpu.last_native_slice_outcome().is_service_boundary();
        let snapshot_state = cpu.last_debug_snapshot().map(|snapshot| {
            (
                native_service_boundary,
                snapshot_has_live_blocked_message_waiter(snapshot, kernel),
                snapshot_has_live_non_message_blocked_wait(snapshot, kernel),
            )
        });
        if let Some((native_service_boundary, message_waiter, non_message_waiter)) =
            snapshot_state.filter(|_| !cpu.slice_end_rotation_pending())
        {
            let has_runnable_parked_process = cpu.has_runnable_parked_process(kernel);
            let should_rotate_process = should_rotate_parked_process(
                cpu.has_parked_child_processes(),
                has_runnable_parked_process,
                cpu.has_live_pump_priority_parked_process(kernel),
                cpu.has_ready_parked_send_unblock(kernel),
                cpu.has_ready_parked_wait_unblock(kernel),
                message_waiter,
                native_service_boundary,
                live_pump_slice,
                wall_clock_limit_ms == HOST_IDLE_MESSAGE_POLL_SLICE_MS,
            );
            if should_rotate_process
                && !cpu.slice_end_rotation_pending()
                && cpu.rotate_to_next_parked_process(kernel)
            {
                tracing::debug!(target: "ce.quantum", "pre-run rotated on native service boundary");
                reported_blocked_message_wait = false;
                publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                continue;
            }
            if live_pump_slice && message_waiter {
                if !should_run_active_receiver_work_for_live_message_waiter(
                    live_pump_slice,
                    message_waiter,
                    cpu.active_process_has_receiver_work(kernel),
                ) {
                    if rotate_idle_active_to_visible_parked(cpu, kernel, desktop) {
                        reported_blocked_message_wait = false;
                        continue;
                    }
                    if non_message_waiter {
                        reported_blocked_message_wait = false;
                        std::thread::sleep(Duration::from_millis(16));
                        continue;
                    }
                    if !reported_blocked_message_wait {
                        if let Some(snapshot) = cpu.last_debug_snapshot() {
                            print_unicorn_stop(snapshot);
                        }
                        if let Some(path) = args.framebuffer_dump.as_ref() {
                            desktop.framebuffer().write_ppm(path)?;
                            println!("  framebuffer dump: {}", path.display());
                        }
                        reported_blocked_message_wait = true;
                    }
                    std::thread::sleep(Duration::from_millis(16));
                    continue;
                }
            }
            if live_pump_slice && native_service_boundary {
                if rotate_idle_active_to_visible_parked(cpu, kernel, desktop) {
                    reported_blocked_message_wait = false;
                    continue;
                }
                if !cpu.active_process_has_visible_receiver_work(kernel)
                    && cpu.rotate_to_receiver_parked_process_with_framebuffer(
                        kernel,
                        Some(desktop.framebuffer_mut()),
                    )
                {
                    reported_blocked_message_wait = false;
                    publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                    continue;
                }
                if should_rotate_idle_saved_remote_active(
                    args.remote_server.is_some(),
                    cpu.has_saved_context(),
                    cpu.has_running_guest_thread(),
                    cpu.active_process_has_visible_queued_receiver_work(kernel),
                    cpu.active_process_has_visible_windows(kernel),
                    cpu.has_runnable_parked_process(kernel),
                ) && cpu.rotate_to_visible_window_parked_process(kernel)
                {
                    reported_blocked_message_wait = false;
                    publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
                    continue;
                }
                reported_blocked_message_wait = false;
                continue;
            }
            if !reported_blocked_message_wait {
                if let Some(snapshot) = cpu.last_debug_snapshot() {
                    print_unicorn_stop(snapshot);
                }
                reported_blocked_message_wait = true;
            }
        }
        if args.remote_server.is_some()
            && live_pump_slice
            && !cpu.slice_end_rotation_pending()
            && rotate_idle_active_to_visible_parked(cpu, kernel, desktop)
        {
            reported_blocked_message_wait = false;
            continue;
        }
        if !cpu.slice_end_rotation_pending()
            && should_rotate_idle_saved_remote_active(
                args.remote_server.is_some(),
                cpu.has_saved_context(),
                cpu.has_running_guest_thread(),
                cpu.active_process_has_visible_queued_receiver_work(kernel),
                cpu.active_process_has_visible_windows(kernel),
                cpu.has_runnable_parked_process(kernel),
            )
            && cpu.rotate_to_visible_window_parked_process(kernel)
        {
            reported_blocked_message_wait = false;
            publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
            continue;
        }
        if should_sleep_remote_idle(args.remote_server.is_some(), cpu.has_saved_context()) {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }
        if args.remote_server.is_some() && cpu.has_saved_context() {
            reported_blocked_message_wait = false;
            continue;
        }
        break;
    }
    desktop.show_stopped_message("Emulator process stopped")?;
    Ok(())
}

fn rotate_idle_active_to_visible_parked(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    desktop: &mut DesktopRuntime,
) -> bool {
    if cpu.active_process_has_receiver_work(kernel) {
        return false;
    }
    if !cpu.active_should_yield_to_visible_parked_process(kernel) {
        return false;
    }
    if cpu.rotate_to_visible_window_parked_process(kernel)
        || (!cpu.active_process_has_visible_windows(kernel)
            && cpu.rotate_to_next_parked_process(kernel))
    {
        publish_remote_debug_after_scheduler_change(cpu, kernel, desktop);
        true
    } else {
        false
    }
}

fn remote_server_has_pending_control(kernel: &CeKernel) -> bool {
    kernel
        .remote_server
        .as_ref()
        .is_some_and(|server| server.pending_control_count() != 0)
}

fn rotate_to_cross_process_send_target(cpu: &mut UnicornMips, kernel: &mut CeKernel) -> bool {
    let Some(yielded) = cpu
        .last_debug_snapshot()
        .and_then(|snapshot| snapshot.cross_process_send_yield.as_ref())
    else {
        return false;
    };
    if kernel.gwe.sent_message(yielded.send_id).is_none() {
        return false;
    }
    cpu.rotate_to_parked_process_id(kernel, yielded.target_process_id)
}

fn remaining_wall_clock_limit_ms(limit_ms: u64, elapsed: Duration) -> u64 {
    if limit_ms == 0 {
        return 0;
    }
    let elapsed_ms = elapsed.as_millis().min(u128::from(u64::MAX)) as u64;
    limit_ms.saturating_sub(elapsed_ms).max(1)
}

fn wall_clock_limit_expired(limit_ms: u64, elapsed: Duration) -> bool {
    limit_ms != 0 && elapsed >= Duration::from_millis(limit_ms)
}

fn completed_cpu_idle_delay(
    kernel: &CeKernel,
    desktop: DesktopMode,
    remaining_run: Option<Duration>,
) -> Duration {
    // Host input has no shared wake event yet; retain a short guard there.
    // Remote control has its own notification, so it may sleep longer when
    // no canonical wait or GWE timer is approaching.
    let guard_ms = if desktop == DesktopMode::Host {
        16
    } else {
        100
    };
    let mut delay = Duration::from_millis(guard_ms);
    if let Some(timer_ms) = kernel.timers.next_due_delay_ms() {
        delay = delay.min(Duration::from_millis(u64::from(timer_ms)));
    }
    let now = kernel.timers.tick_count();
    if let Some(wait_ms) = kernel
        .scheduler
        .blocked_waits()
        .filter(|wait| wait.timeout_ms != u32::MAX)
        .map(|wait| {
            wait.timeout_ms
                .saturating_sub(now.wrapping_sub(wait.wait_started_ms))
        })
        .min()
    {
        delay = delay.min(Duration::from_millis(u64::from(wait_ms)));
    }
    if let Some(remaining_run) = remaining_run {
        delay = delay.min(remaining_run);
    }
    delay.max(Duration::from_millis(1))
}

fn should_rotate_parked_process(
    has_parked_child_processes: bool,
    has_runnable_parked_process: bool,
    has_live_pump_priority_parked_process: bool,
    has_ready_parked_send_unblock: bool,
    has_ready_parked_wait_unblock: bool,
    message_waiter: bool,
    native_service_boundary: bool,
    live_pump_slice: bool,
    _idle_message_poll_slice: bool,
) -> bool {
    if !has_parked_child_processes {
        return false;
    }
    let ready_message_waiter = message_waiter
        && ((!live_pump_slice && has_runnable_parked_process)
            || has_ready_parked_send_unblock
            || has_ready_parked_wait_unblock);
    let ready_live_service_boundary = live_pump_slice
        && native_service_boundary
        && (has_runnable_parked_process
            || has_live_pump_priority_parked_process
            || has_ready_parked_send_unblock
            || has_ready_parked_wait_unblock
            || message_waiter);
    let ready_live_message_waiter =
        live_pump_slice && message_waiter && has_runnable_parked_process;
    ready_message_waiter || ready_live_service_boundary || ready_live_message_waiter
}

fn should_run_active_receiver_work_for_live_message_waiter(
    live_pump_slice: bool,
    message_waiter: bool,
    active_receiver_work: bool,
) -> bool {
    live_pump_slice && message_waiter && active_receiver_work
}

fn host_idle_message_poll_slice(cpu: &UnicornMips, desktop: DesktopMode) -> bool {
    should_use_host_idle_message_poll_slice(
        desktop,
        cpu.has_saved_context(),
        cpu.last_debug_snapshot()
            .is_some_and(snapshot_is_idle_message_wait_only),
    )
}

fn should_use_host_idle_message_poll_slice(
    desktop: DesktopMode,
    has_saved_context: bool,
    idle_message_wait_only: bool,
) -> bool {
    desktop == DesktopMode::Host && !has_saved_context && idle_message_wait_only
}

fn should_sleep_remote_idle(remote_server_enabled: bool, has_saved_context: bool) -> bool {
    remote_server_enabled && !has_saved_context
}

fn should_idle_host_message_pump(
    cpu: &UnicornMips,
    kernel: &CeKernel,
    desktop: DesktopMode,
) -> bool {
    desktop == DesktopMode::Host
        && cpu
            .last_debug_snapshot()
            .is_some_and(|snapshot| snapshot_has_live_saved_get_message_waiter(snapshot, kernel))
        && !cpu.has_ready_parked_send_unblock(kernel)
        && !cpu.has_ready_parked_wait_unblock(kernel)
        && !cpu.has_parked_child_processes()
}

fn snapshot_has_blocked_get_message(snapshot: &UnicornDebugSnapshot) -> bool {
    if snapshot.trap_address.is_some() {
        return false;
    }
    snapshot.blocked_get_message.is_some() || snapshot_has_saved_get_message_waiter(snapshot)
}

fn snapshot_has_saved_get_message_waiter(snapshot: &UnicornDebugSnapshot) -> bool {
    snapshot
        .active_blocked_waits
        .iter()
        .any(|wait| wait.kind == "get_message")
}

fn snapshot_has_live_saved_get_message_waiter(
    snapshot: &UnicornDebugSnapshot,
    kernel: &CeKernel,
) -> bool {
    snapshot
        .active_blocked_waits
        .iter()
        .any(|wait| wait.kind == "get_message" && kernel.blocked_waiter(wait.id).is_some())
}

fn snapshot_has_non_message_blocked_wait(snapshot: &UnicornDebugSnapshot) -> bool {
    snapshot
        .active_blocked_waits
        .iter()
        .any(|wait| wait.kind != "get_message")
}

fn snapshot_has_orphaned_cross_process_send(
    snapshot: &UnicornDebugSnapshot,
    kernel: &CeKernel,
) -> bool {
    snapshot
        .cross_process_send_yield
        .as_ref()
        .is_some_and(|yielded| kernel.gwe.sent_message(yielded.send_id).is_none())
}

fn should_rotate_remote_input_receiver_parked_process(
    active_has_visible_receiver_work: bool,
) -> bool {
    !active_has_visible_receiver_work
}

fn rotate_to_same_process_visible_receiver_thread(
    cpu: &mut UnicornMips,
    kernel: &CeKernel,
) -> bool {
    !cpu.current_thread_has_visible_receiver_work(kernel)
        && cpu.active_process_has_visible_receiver_work(kernel)
        && cpu.rotate_to_active_visible_receiver_thread(kernel, &[])
}

fn active_process_exited(cpu: &UnicornMips, kernel: &CeKernel) -> bool {
    cpu.active_process_execution_completed()
        // An encoded kernel exit can target another process's real handle.
        // Only the synchronous current-process pseudo lane makes that snapshot
        // sufficient for a caller handoff; real child completion is CPU-owned.
        || (cpu
            .last_debug_snapshot()
            .is_some_and(|snapshot| snapshot.encoded_kernel_exit.is_some())
            && kernel.is_wait_ready(
                wince_emulation_v3::ce::kernel::CE_CURRENT_PROCESS_PSEUDO_HANDLE,
                0,
            ) == Some(true))
}

fn should_keep_remote_active_exit_alive(
    remote_server: bool,
    active_visible_receiver_work: bool,
) -> bool {
    remote_server && active_visible_receiver_work
}

fn switch_completed_active_context(cpu: &mut UnicornMips, kernel: &mut CeKernel) -> Result<bool> {
    if active_process_exited(cpu, kernel) {
        return cpu.try_switch_to_next_parked_child_process(kernel);
    }
    if !cpu.last_stop_is_guest_thread_return_stub() {
        return Ok(false);
    }
    if cpu.preserve_current_on_process_handoff(kernel) {
        Ok(cpu.rotate_to_next_parked_process(kernel))
    } else {
        cpu.try_switch_to_next_parked_child_process(kernel)
    }
}

fn snapshot_has_live_blocked_message_waiter(
    snapshot: &UnicornDebugSnapshot,
    kernel: &CeKernel,
) -> bool {
    snapshot_has_blocked_get_message(snapshot)
        && snapshot_has_live_saved_get_message_waiter(snapshot, kernel)
}

fn snapshot_has_live_non_message_blocked_wait(
    snapshot: &UnicornDebugSnapshot,
    kernel: &CeKernel,
) -> bool {
    snapshot
        .active_blocked_waits
        .iter()
        .any(|wait| wait.kind != "get_message" && kernel.blocked_waiter(wait.id).is_some())
}

fn snapshot_has_only_stale_owned_blocked_waits(
    snapshot: &UnicornDebugSnapshot,
    cpu: &UnicornMips,
    kernel: &CeKernel,
) -> bool {
    let mut owned_waits = 0usize;
    for wait in &snapshot.active_blocked_waits {
        if !cpu.contains_thread(&[wait.thread_id]) {
            continue;
        }
        owned_waits += 1;
        if kernel.blocked_waiter(wait.id).is_some() {
            return false;
        }
    }
    owned_waits != 0
}

fn snapshot_is_idle_message_wait_only(snapshot: &UnicornDebugSnapshot) -> bool {
    snapshot_has_blocked_get_message(snapshot) && !snapshot_has_non_message_blocked_wait(snapshot)
}

fn remote_live_run_slice_ms() -> u64 {
    std::env::var(REMOTE_LIVE_RUN_SLICE_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value != 0)
        .unwrap_or(REMOTE_LIVE_RUN_SLICE_MS)
}

fn live_pump_run_slice_instructions() -> usize {
    std::env::var(LIVE_PUMP_RUN_SLICE_INSTRUCTIONS_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(LIVE_PUMP_RUN_SLICE_INSTRUCTIONS)
}

fn remote_debug_publish_interval() -> Duration {
    std::env::var(REMOTE_DEBUG_PUBLISH_INTERVAL_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(REMOTE_DEBUG_PUBLISH_INTERVAL_MS))
}

fn should_rotate_idle_remote_receiver_parked_process(
    active_has_receiver_work: bool,
    active_has_visible_receiver_work: bool,
) -> bool {
    !active_has_receiver_work && !active_has_visible_receiver_work
}

fn should_rotate_idle_runnable_parked_process(
    active_has_visible_receiver_work: bool,
    has_runnable_parked_process: bool,
) -> bool {
    !active_has_visible_receiver_work && has_runnable_parked_process
}

fn should_rotate_idle_nonrunning_runnable_parked_process(
    active_has_running_guest_thread: bool,
    active_has_visible_queued_receiver_work: bool,
    has_runnable_parked_process: bool,
) -> bool {
    !active_has_running_guest_thread
        && !active_has_visible_queued_receiver_work
        && has_runnable_parked_process
}

fn should_rotate_live_pump_priority_parked_process(
    active_has_visible_queued_receiver_work: bool,
    has_live_pump_priority_parked_process: bool,
) -> bool {
    !active_has_visible_queued_receiver_work && has_live_pump_priority_parked_process
}

fn should_rotate_idle_saved_remote_active(
    remote_server: bool,
    has_saved_context: bool,
    _active_has_running_guest_thread: bool,
    active_has_visible_receiver_work: bool,
    active_has_visible_windows: bool,
    has_runnable_parked_process: bool,
) -> bool {
    remote_server
        && has_saved_context
        && !active_has_visible_receiver_work
        && !active_has_visible_windows
        && has_runnable_parked_process
}

fn effective_instruction_limit(
    explicit_instruction_limit: usize,
    live_pump_slice: bool,
    fast_start_enabled: bool,
) -> usize {
    if live_pump_slice {
        if explicit_instruction_limit != 0 {
            return explicit_instruction_limit;
        }
        return live_pump_run_slice_instructions();
    }
    if explicit_instruction_limit == 0 && fast_start_enabled {
        return FAST_START_RUN_SLICE_INSTRUCTIONS;
    }
    explicit_instruction_limit
}

fn effective_wall_clock_limit_ms(
    explicit_limit_ms: u64,
    elapsed: Duration,
    desktop: DesktopMode,
    remote_server_enabled: bool,
    idle_message_poll_slice: bool,
    active_receiver_work: bool,
) -> (u64, bool) {
    let active_receiver_work_slice =
        active_receiver_work && (desktop == DesktopMode::Host || remote_server_enabled);
    if active_receiver_work_slice {
        if explicit_limit_ms == 0 {
            return (LIVE_VISIBLE_WORK_RUN_SLICE_MS, true);
        }
        return (
            remaining_wall_clock_limit_ms(explicit_limit_ms, elapsed)
                .min(LIVE_VISIBLE_WORK_RUN_SLICE_MS),
            true,
        );
    }
    if desktop == DesktopMode::Host {
        if idle_message_poll_slice {
            if explicit_limit_ms == 0 {
                return (HOST_IDLE_MESSAGE_POLL_SLICE_MS, true);
            }
            return (
                remaining_wall_clock_limit_ms(explicit_limit_ms, elapsed)
                    .min(HOST_IDLE_MESSAGE_POLL_SLICE_MS),
                true,
            );
        }
        if explicit_limit_ms == 0 {
            let live_slice = if remote_server_enabled {
                remote_live_run_slice_ms()
            } else {
                HOST_LIVE_RUN_SLICE_MS
            };
            return (live_slice, true);
        }
        let live_slice = if remote_server_enabled {
            remote_live_run_slice_ms()
        } else {
            HOST_LIVE_RUN_SLICE_MS
        };
        return (
            remaining_wall_clock_limit_ms(explicit_limit_ms, elapsed).min(live_slice),
            true,
        );
    }
    if remote_server_enabled {
        let live_slice = remote_live_run_slice_ms();
        if explicit_limit_ms == 0 {
            return (live_slice, true);
        }
        return (
            remaining_wall_clock_limit_ms(explicit_limit_ms, elapsed).min(live_slice),
            true,
        );
    }
    (
        remaining_wall_clock_limit_ms(explicit_limit_ms, elapsed),
        false,
    )
}

fn launch_delayed_companion_processes(args: &Args) -> Result<Option<CompanionProcesses>> {
    let specs = companion_launch_specs(args)?;
    if specs.is_empty() {
        return Ok(None);
    }
    let stop = Arc::new(AtomicBool::new(false));
    let children = Arc::new(Mutex::new(Vec::new()));
    let launcher_stop = Arc::clone(&stop);
    let launcher_children = Arc::clone(&children);
    let launcher = std::thread::Builder::new()
        .name("ce-companion-launcher".to_owned())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(COMPANION_START_DELAY_MS));
            if launcher_stop.load(Ordering::SeqCst) {
                return;
            }
            for (index, spec) in specs.into_iter().enumerate() {
                if launcher_stop.load(Ordering::SeqCst) {
                    return;
                }
                match spawn_companion_process(&spec) {
                    Ok(child) => {
                        println!(
                            "  companion launched #{} pid={} target={} stdout={} stderr={}",
                            index + 1,
                            child.id(),
                            spec.target.display(),
                            spec.stdout.display(),
                            spec.stderr.display()
                        );
                        if let Ok(mut children) = launcher_children.lock() {
                            children.push(child);
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "  companion launch failed #{} target={}: {err}",
                            index + 1,
                            spec.target.display()
                        );
                    }
                }
            }
        })
        .map_err(|err| {
            wince_emulation_v3::Error::Backend(format!("spawn companion launcher: {err}"))
        })?;
    Ok(Some(CompanionProcesses {
        stop,
        children,
        launcher: Some(launcher),
    }))
}

fn companion_launch_specs(args: &Args) -> Result<Vec<CompanionLaunchSpec>> {
    if args.companion_images.is_empty() {
        return Ok(Vec::new());
    }
    let executable = std::env::current_exe()
        .map_err(|err| wince_emulation_v3::Error::Backend(format!("current exe: {err}")))?;
    fs::create_dir_all("target").map_err(|err| {
        wince_emulation_v3::Error::Backend(format!("create companion log dir target: {err}"))
    })?;
    let mut specs = Vec::new();
    for (index, target) in args.companion_images.iter().enumerate() {
        if !target.is_file() {
            return Err(wince_emulation_v3::Error::InvalidArgument(format!(
                "--companion-image {} is not a file",
                target.display()
            )));
        }
        let log_stem = companion_log_stem(target, index + 1);
        specs.push(CompanionLaunchSpec {
            executable: executable.clone(),
            target: target.clone(),
            args: companion_command_args(args, target),
            stdout: PathBuf::from("target").join(format!("{log_stem}.stdout.log")),
            stderr: PathBuf::from("target").join(format!("{log_stem}.stderr.log")),
        });
    }
    Ok(specs)
}

fn companion_command_args(args: &Args, image: &Path) -> Vec<OsString> {
    let mut command_args = Vec::new();
    push_arg_pair(&mut command_args, "--registry", &args.registry);
    push_arg_pair(&mut command_args, "--devices", &args.devices);
    if let Some(mount_config) = args.mount_config.as_ref() {
        push_arg_pair(&mut command_args, "--mount-config", mount_config);
    }
    push_arg_pair(&mut command_args, "--image", image);
    for dll_search_dir in &args.dll_search_dirs {
        push_arg_pair(&mut command_args, "--dll-search-dir", dll_search_dir);
    }
    command_args.push(OsString::from("--desktop"));
    command_args.push(OsString::from("virtual"));
    command_args.push(OsString::from("--cpu-instruction-limit"));
    command_args.push(OsString::from(COMPANION_INSTRUCTION_LIMIT.to_string()));
    command_args.push(OsString::from("--run-cpu"));
    command_args
}

fn push_arg_pair(args: &mut Vec<OsString>, flag: &str, value: &Path) {
    args.push(OsString::from(flag));
    args.push(value.as_os_str().to_owned());
}

fn companion_log_stem(target: &Path, index: usize) -> String {
    let stem = target
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("companion");
    let sanitized = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("companion_{index:02}_{sanitized}")
}

fn spawn_companion_process(spec: &CompanionLaunchSpec) -> io::Result<Child> {
    let stdout = fs::File::create(&spec.stdout)?;
    let stderr = fs::File::create(&spec.stderr)?;
    let mut command = Command::new(&spec.executable);
    command
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command.spawn()
}

impl Drop for CompanionProcesses {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(launcher) = self.launcher.take() {
            let _ = launcher.join();
        }
        let Ok(mut children) = self.children.lock() else {
            return;
        };
        for child in children.iter_mut() {
            if matches!(child.try_wait(), Ok(None)) {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

fn blocked_remote_input_target(
    cpu: &UnicornMips,
    kernel: &CeKernel,
) -> Option<BlockedRemoteInputTarget> {
    if cpu.active_process_has_visible_receiver_work(kernel) {
        return None;
    }
    if let Some(blocked) = cpu
        .last_debug_snapshot()
        .and_then(|snapshot| snapshot.blocked_get_message.clone())
        .filter(|blocked| {
            blocked.hwnd.is_some()
                && remote_input_wait_target_is_visible(kernel, blocked.thread_id, blocked.hwnd)
        })
    {
        return Some(BlockedRemoteInputTarget {
            thread_id: blocked.thread_id,
            hwnd: blocked.hwnd,
        });
    }
    saved_message_remote_input_target(kernel)
}

fn saved_message_remote_input_target(kernel: &CeKernel) -> Option<BlockedRemoteInputTarget> {
    let modal_message_box = kernel.scheduler.blocked_waits().find_map(|wait| {
        let SchedulerBlockedWaitKind::ModalMessageBox { dialog_hwnd } = wait.kind else {
            return None;
        };
        match wait.owner_birth {
            Some(birth)
                if birth.thread_id() == wait.thread_id
                    && kernel.handles.thread_birth_key_by_id(wait.thread_id) == Some(birth)
                    && kernel.handles.thread_execution_live_by_id(wait.thread_id) => {}
            None if kernel
                .handles
                .thread_birth_key_by_id(wait.thread_id)
                .is_none() => {}
            _ => return None,
        }
        let dialog = kernel
            .gwe
            .window(dialog_hwnd)
            .filter(|window| {
                window.thread_id == wait.thread_id
                    && window.visible
                    && !window.destroyed
                    && window.class_name.eq_ignore_ascii_case("dialog")
            })
            .map(|_| dialog_hwnd);
        Some(BlockedRemoteInputTarget {
            thread_id: wait.thread_id,
            hwnd: dialog,
        })
    });
    if modal_message_box.is_some() {
        return modal_message_box;
    }
    let get_message = kernel.scheduler.blocked_waits().find_map(|wait| {
        matches!(wait.kind, SchedulerBlockedWaitKind::GetMessage { .. })
            .then(|| {
                let SchedulerBlockedWaitKind::GetMessage { hwnd, .. } = wait.kind else {
                    unreachable!();
                };
                let hwnd = hwnd?;
                remote_input_wait_target_is_visible(kernel, wait.thread_id, Some(hwnd)).then_some(
                    BlockedRemoteInputTarget {
                        thread_id: wait.thread_id,
                        hwnd: Some(hwnd),
                    },
                )
            })
            .flatten()
    });
    if get_message.is_some() {
        return get_message;
    }
    None
}

fn remote_input_wait_target_is_visible(
    kernel: &CeKernel,
    thread_id: u32,
    hwnd: Option<u32>,
) -> bool {
    let visible_input_window = |window: &wince_emulation_v3::ce::gwe::Window| {
        window.thread_id == thread_id
            && window.visible
            && !window.destroyed
            && window.rect.width() > 0
            && window.rect.height() > 0
    };
    if let Some(hwnd) = hwnd {
        return kernel
            .gwe
            .window(hwnd)
            .is_some_and(|window| visible_input_window(window));
    }
    kernel
        .gwe
        .windows_snapshot()
        .iter()
        .any(visible_input_window)
}

fn service_remote_endpoint(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    desktop: &mut DesktopRuntime,
    blocked_get_message: Option<&BlockedRemoteInputTarget>,
) -> Result<RemoteEndpointDrain> {
    cpu.try_prune_exited_and_active_processes_from_parked_with_framebuffer(
        kernel,
        Some(desktop.framebuffer_mut()),
    )?;
    let remote_target = blocked_get_message.cloned();
    let drained = if let Some(blocked) = remote_target.as_ref() {
        kernel.drain_remote_server_control_messages_to_thread_window_with_targets(
            blocked.thread_id,
            blocked.hwnd,
        )
    } else {
        kernel.drain_remote_server_control_messages_with_targets()
    };
    let snapshot = cpu.last_debug_snapshot().cloned();
    publish_remote_endpoint(
        kernel.remote_server.as_ref(),
        Some(cpu),
        kernel,
        desktop.framebuffer(),
        snapshot.as_ref(),
        false,
    );
    Ok(RemoteEndpointDrain {
        handled: drained.handled,
        target_thread_ids: drained.target_thread_ids,
    })
}

fn publish_remote_endpoint(
    server: Option<&RemoteServer>,
    cpu: Option<&mut UnicornMips>,
    kernel: &CeKernel,
    framebuffer: &VirtualFramebuffer,
    snapshot: Option<&UnicornDebugSnapshot>,
    publish_framebuffer: bool,
) {
    if let Some(server) = server {
        let cpu = cpu.map(|cpu| {
            cpu.prune_active_process_from_parked(kernel);
            &*cpu
        });
        server.publish_status(&kernel.remote_status());
        server.publish_recent_logs(kernel.remote.recent_log_lines(4096));
        if publish_framebuffer {
            server.publish_framebuffer(framebuffer);
        }
        let publication = match server.claim_debug_publish_slot(remote_debug_publish_interval()) {
            Ok(Some(publication)) => publication,
            Ok(None) => return,
            Err(err) => {
                tracing::error!(target: "ce.remote", "debug publication slot failed: {err}");
                return;
            }
        };
        // This is one cached debug body with its own freshness identity, not a
        // coherent epoch for status, frames or the other debug keys.
        server.publish_debug_text(
            "summary",
            format!(
                "  publication_sequence: {}\n  publication_host_monotonic_ns: {}\n  publication_utc_unix_ms: {}\n  live_guest_stable_page_budget_used_bytes: {}\n  last_stop_snapshot: {}\n{}",
                publication.sequence,
                publication.host_monotonic_ns,
                publication.utc_unix_ms,
                kernel.memory.private_commit_used_bytes(),
                snapshot
                    .map(|snapshot| snapshot.summary())
                    .unwrap_or_else(|| "none".to_owned()),
                live_kernel_waits_text(kernel)
            ),
        );
        server.publish_debug_text("windows", live_kernel_windows_text(kernel));
        server.publish_debug_text("messages", live_kernel_messages_text(kernel));
        server.publish_debug_text("queues", kernel.gwe.queue_debug_text());
        server.publish_debug_text("processes-live", live_kernel_processes_text(kernel));
        server.publish_debug_text("message-boxes", live_kernel_message_boxes_text(kernel));
        server.publish_debug_text("handles", kernel.handle_debug_text());
        server.publish_debug_text("devices", live_kernel_devices_text(kernel));
        server.publish_debug_text("display-perf", live_kernel_display_perf_text(kernel));
        server.publish_debug_text("timers", kernel.timers.debug_text());
        if let Some(cpu) = cpu {
            server.publish_debug_text("active", live_active_process_text(cpu, kernel));
            server.publish_debug_text("pending-wndproc", cpu.pending_wndproc_debug_text());
            server.publish_debug_text("parked", cpu.parked_process_debug_text(kernel));
            server.publish_debug_text("blobs", live_mapped_blob_ranges_text(cpu));
            server.publish_debug_text("trampolines", cpu.trampoline_debug_text());
        }
        server.publish_debug_text("sends", kernel.gwe.sent_message_debug_text());
        if let Some(snapshot) = snapshot {
            server.publish_debug_text("processes", monitor_trace_text(snapshot, "processes"));
            server.publish_debug_text("wndproc", monitor_trace_text(snapshot, "wndproc"));
            server.publish_debug_text("imports", monitor_trace_text(snapshot, "imports"));
            server.publish_debug_text(
                "window-imports",
                monitor_trace_text(snapshot, "window-imports"),
            );
            server.publish_debug_text("presentation", monitor_trace_text(snapshot, "presentation"));
            server.publish_debug_text("milestones", monitor_trace_text(snapshot, "milestones"));
            server.publish_debug_text("counts", monitor_trace_text(snapshot, "counts"));
            server.publish_debug_text("calls", monitor_trace_text(snapshot, "calls"));
            server.publish_debug_text("code", monitor_trace_text(snapshot, "code"));
            server.publish_debug_text("blocks", monitor_trace_text(snapshot, "blocks"));
            server.publish_debug_text("files", monitor_trace_text(snapshot, "files"));
            server.publish_debug_text("files-full", monitor_trace_text(snapshot, "files-full"));
            server.publish_debug_text("events", monitor_trace_text(snapshot, "events"));
            server.publish_debug_text("render", monitor_trace_text(snapshot, "render"));
            server.publish_debug_text("controller", monitor_trace_text(snapshot, "controller"));
            server.publish_debug_text("resource", monitor_trace_text(snapshot, "resource"));
        }
    }
}

fn publish_remote_debug_after_scheduler_change(
    cpu: &mut UnicornMips,
    kernel: &CeKernel,
    desktop: &DesktopRuntime,
) {
    let snapshot = cpu.last_debug_snapshot().cloned();
    publish_remote_endpoint(
        kernel.remote_server.as_ref(),
        Some(cpu),
        kernel,
        desktop.framebuffer(),
        snapshot.as_ref(),
        true,
    );
}

fn live_active_process_text(cpu: &UnicornMips, kernel: &CeKernel) -> String {
    use std::fmt::Write as _;

    let state = kernel.current_process_state();
    let active_threads = cpu.active_thread_ids();
    let active_thread_text = active_threads
        .iter()
        .map(|thread_id| thread_id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let last_stop = cpu
        .last_debug_snapshot()
        .map(|snapshot| snapshot.summary())
        .unwrap_or_else(|| "none".to_owned());
    let mut out = format!(
        "  active: pid={} tid={} active_threads=[{}] saved_context={} visible_receiver_work={} receiver_work={} yield_visible_parked={} parked_processes={}\n  saved_context_detail: {}\n  active_last_stop_snapshot: {}\n",
        state.process_id,
        cpu.current_thread_id(),
        active_thread_text,
        cpu.has_saved_context(),
        cpu.active_process_has_visible_receiver_work(kernel),
        cpu.active_process_has_receiver_work(kernel),
        cpu.active_should_yield_to_visible_parked_process(kernel),
        cpu.parked_child_process_count_for_kernel(kernel),
        cpu.saved_context_debug_text(),
        last_stop
    );
    out.push_str("  active thread work:\n");
    for thread_id in active_threads {
        let dirty_visible_windows = kernel
            .gwe
            .windows_snapshot()
            .into_iter()
            .filter(|window| {
                window.thread_id == thread_id
                    && window.visible
                    && !window.destroyed
                    && window.rect.width() > 0
                    && window.rect.height() > 0
                    && (window.update_pending || window.erase_pending)
            })
            .count();
        let _ = writeln!(
            &mut out,
            "    tid={thread_id} sent={} messages={} visible_messages={} dirty_visible_windows={dirty_visible_windows}",
            kernel.thread_has_pending_sent_message(thread_id),
            kernel.gwe.has_message_filtered(thread_id, None, 0, 0),
            kernel
                .gwe
                .has_visible_window_message_filtered(thread_id, None, 0, 0)
        );
    }
    out.push_str(&cpu.blocked_wait_debug_text(kernel));
    out.push_str(&live_kernel_waits_text(kernel));
    out
}

fn live_mapped_blob_ranges_text(cpu: &UnicornMips) -> String {
    let mut out = String::from("  mapped blobs:\n");
    let blobs = cpu.mapped_blob_ranges();
    if blobs.is_empty() {
        out.push_str("    none\n");
        return out;
    }
    for blob in blobs {
        let end = blob.base.saturating_add(blob.size);
        out.push_str(&format!(
            "    0x{:08x}-0x{:08x} size=0x{:x} {}\n",
            blob.base, end, blob.size, blob.name
        ));
    }
    out
}

fn print_unicorn_stop(snapshot: &wince_emulation_v3::emulator::cpu::UnicornDebugSnapshot) {
    if let Some(exit) = snapshot.encoded_kernel_exit.as_ref() {
        println!(
            "  CE process exited: process=0x{:08x} code=0x{:08x}; {}",
            exit.process,
            exit.exit_code,
            snapshot.summary()
        );
    } else {
        println!("  Unicorn stopped: {}", snapshot.summary());
    }
}

fn enqueue_desktop_input_for_current_wait(
    cpu: &UnicornMips,
    desktop: &mut DesktopRuntime,
    kernel: &mut CeKernel,
    desktop_mode: DesktopMode,
) -> Result<usize> {
    let blocked_get_message = if desktop_mode == DesktopMode::Host {
        blocked_remote_input_target(cpu, kernel)
    } else {
        None
    };
    let queued = enqueue_desktop_input(desktop, kernel)?;
    if queued != 0 {
        if let Some(blocked) = blocked_get_message {
            kernel.drain_remote_input_to_thread_window(blocked.thread_id, blocked.hwnd);
        }
    }
    Ok(queued)
}

fn service_completed_active_desktop(
    cpu: &UnicornMips,
    desktop: &mut DesktopRuntime,
    kernel: &mut CeKernel,
    desktop_mode: DesktopMode,
) -> Result<usize> {
    let queued = enqueue_desktop_input_for_current_wait(cpu, desktop, kernel, desktop_mode)?;
    if desktop_mode == DesktopMode::Host {
        desktop.present()?;
    }
    Ok(queued)
}

fn write_requested_tracefiles(cpu: &UnicornMips, tracefiles: &[(String, PathBuf)]) -> Result<()> {
    if tracefiles.is_empty() {
        return Ok(());
    }
    let Some(snapshot) = cpu.preferred_trace_snapshot() else {
        return Err(wince_emulation_v3::Error::Backend(
            "tracefile requested but no Unicorn snapshot is available".to_owned(),
        ));
    };
    for (selector, path) in tracefiles {
        let text = monitor_trace_text(snapshot, selector);
        fs::write(path, text).map_err(|err| {
            wince_emulation_v3::Error::Backend(format!("write tracefile {}: {err}", path.display()))
        })?;
        println!("  trace {selector} written to {}", path.display());
    }
    Ok(())
}

fn run_monitor(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    desktop: &mut DesktopRuntime,
    args: &Args,
) -> Result<()> {
    enqueue_startup_taps(kernel, &args.startup_taps)?;
    println!("  monitor: interactive commands enabled; type `help`");
    let stdin = io::stdin();
    let mut checkpoints = Vec::<MonitorCheckpoint>::new();
    let mut recovery_required = false;
    loop {
        print!("fakece> ");
        io::stdout().flush().map_err(|err| {
            wince_emulation_v3::Error::Backend(format!("flush monitor prompt: {err}"))
        })?;
        let mut line = String::new();
        let read = stdin.read_line(&mut line).map_err(|err| {
            wince_emulation_v3::Error::Backend(format!("read monitor command: {err}"))
        })?;
        if read == 0 {
            println!();
            break;
        }
        let mut words = line.split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        if recovery_required && monitor_command_blocked_by_recovery(command) {
            println!(
                "  stopped-process exit recovery required; restore an earlier checkpoint or quit before running, queuing input, or saving a new checkpoint"
            );
            continue;
        }
        match command {
            "help" | "h" | "?" => print_monitor_help(),
            "continue" | "c" => {
                let wall_clock_limit_ms = words
                    .next()
                    .map(parse_monitor_u64)
                    .transpose()?
                    .unwrap_or_else(|| {
                        if args.cpu_wall_clock_limit_ms == 0 && args.cpu_instruction_limit == 0 {
                            1000
                        } else {
                            args.cpu_wall_clock_limit_ms
                        }
                    });
                let instruction_limit = words
                    .next()
                    .map(parse_monitor_usize)
                    .transpose()?
                    .unwrap_or(args.cpu_instruction_limit);
                recovery_required |= monitor_run_and_report(
                    cpu,
                    kernel,
                    desktop,
                    UnicornRunLimits {
                        instruction_limit,
                        wall_clock_limit_ms,
                        stop_pc: None,
                        live_pump: monitor_live_pump(args),
                    },
                    args.framebuffer_dump.as_deref(),
                );
            }
            "until" | "run-until" => {
                let address = words.next().ok_or_else(|| {
                    wince_emulation_v3::Error::InvalidArgument(
                        "until needs ADDRESS [wall_ms] [insns]".to_owned(),
                    )
                })?;
                let stop_pc = parse_monitor_u32(address)?;
                let wall_clock_limit_ms = words
                    .next()
                    .map(parse_monitor_u64)
                    .transpose()?
                    .unwrap_or_else(|| {
                        if args.cpu_wall_clock_limit_ms == 0 && args.cpu_instruction_limit == 0 {
                            1000
                        } else {
                            args.cpu_wall_clock_limit_ms
                        }
                    });
                let instruction_limit = words
                    .next()
                    .map(parse_monitor_usize)
                    .transpose()?
                    .unwrap_or(args.cpu_instruction_limit);
                recovery_required |= monitor_run_and_report(
                    cpu,
                    kernel,
                    desktop,
                    UnicornRunLimits {
                        instruction_limit,
                        wall_clock_limit_ms,
                        stop_pc: Some(stop_pc),
                        live_pump: monitor_live_pump(args),
                    },
                    args.framebuffer_dump.as_deref(),
                );
            }
            "step" | "s" | "si" => {
                println!(
                    "  step needs persistent Unicorn CPU/RAM state; use until/continue for bounded stops"
                );
            }
            "tap" => {
                let x = words.next().ok_or_else(|| {
                    wince_emulation_v3::Error::InvalidArgument("tap needs X Y".to_owned())
                })?;
                let y = words.next().ok_or_else(|| {
                    wince_emulation_v3::Error::InvalidArgument("tap needs X Y".to_owned())
                })?;
                let x = parse_monitor_i32(x)?;
                let y = parse_monitor_i32(y)?;
                kernel.remote.enqueue_touch("tap", x, y).map_err(|err| {
                    wince_emulation_v3::Error::Backend(format!("monitor tap: {err}"))
                })?;
                println!("  queued tap {x},{y}");
            }
            "dump" => {
                let path = words
                    .next()
                    .map(PathBuf::from)
                    .or_else(|| args.framebuffer_dump.clone())
                    .ok_or_else(|| {
                        wince_emulation_v3::Error::InvalidArgument(
                            "dump needs a path or --framebuffer-dump".to_owned(),
                        )
                    })?;
                desktop.framebuffer().write_ppm(&path)?;
                println!("  framebuffer dump: {}", path.display());
            }
            "present" => {
                desktop.present()?;
                println!("  presented framebuffer");
            }
            "regs" | "snapshot" | "info" => {
                if let Some(snapshot) = cpu.last_debug_snapshot() {
                    println!("  Unicorn stopped: {}", snapshot.summary());
                } else {
                    println!("  no Unicorn snapshot yet");
                }
            }
            "trace" | "detail" => {
                let selector = words.next().unwrap_or("all");
                let Some(snapshot) = cpu.preferred_trace_snapshot() else {
                    println!("  no Unicorn snapshot yet");
                    continue;
                };
                print_monitor_trace(snapshot, selector);
            }
            "tracefile" | "trace-file" => {
                let selector = words.next().ok_or_else(|| {
                    wince_emulation_v3::Error::InvalidArgument(
                        "tracefile needs KIND PATH".to_owned(),
                    )
                })?;
                let path = words.next().ok_or_else(|| {
                    wince_emulation_v3::Error::InvalidArgument(
                        "tracefile needs KIND PATH".to_owned(),
                    )
                })?;
                let Some(snapshot) = cpu.last_debug_snapshot() else {
                    println!("  no Unicorn snapshot yet");
                    continue;
                };
                let text = monitor_trace_text(snapshot, selector);
                fs::write(path, text).map_err(|err| {
                    wince_emulation_v3::Error::Backend(format!("write tracefile {path}: {err}"))
                })?;
                println!("  trace {selector} written to {path}");
            }
            "map" | "regions" => {
                println!("  memory regions:");
                for region in cpu.memory().regions() {
                    println!(
                        "    0x{base:08x}-0x{end:08x} {perms:?} {name}",
                        base = region.base,
                        end = region.base.saturating_add(region.size),
                        perms = region.perms,
                        name = &region.name
                    );
                }
                println!("  mapped blobs:");
                for blob in cpu.mapped_blob_ranges() {
                    println!(
                        "    0x{base:08x}-0x{end:08x} {name}",
                        base = blob.base,
                        end = blob.base.saturating_add(blob.size),
                        name = blob.name
                    );
                }
            }
            "x" | "examine" => {
                let address = words.next().ok_or_else(|| {
                    wince_emulation_v3::Error::InvalidArgument("x needs ADDRESS [LEN]".to_owned())
                })?;
                let address = parse_monitor_u32(address)?;
                let len = words
                    .next()
                    .map(parse_monitor_usize)
                    .transpose()?
                    .unwrap_or(64)
                    .min(4096);
                let Some(bytes) = cpu.read_mapped_bytes(address, len) else {
                    println!("  no mapped static bytes at 0x{address:08x} for {len} byte(s)");
                    continue;
                };
                print_monitor_hexdump(address, &bytes);
            }
            "disasm" | "u32" | "words" => {
                let address = words.next().ok_or_else(|| {
                    wince_emulation_v3::Error::InvalidArgument(
                        "disasm needs ADDRESS [WORDS]".to_owned(),
                    )
                })?;
                let address = parse_monitor_u32(address)?;
                let words_count = words
                    .next()
                    .map(parse_monitor_usize)
                    .transpose()?
                    .unwrap_or(8)
                    .min(128);
                let len = words_count.saturating_mul(4);
                let Some(bytes) = cpu.read_mapped_bytes(address, len) else {
                    println!(
                        "  no mapped static words at 0x{address:08x} for {words_count} word(s)"
                    );
                    continue;
                };
                for (index, chunk) in bytes.chunks_exact(4).enumerate() {
                    let value = u32::from_le_bytes(chunk.try_into().unwrap());
                    let pc = address.wrapping_add((index * 4) as u32);
                    println!("  0x{pc:08x}: 0x{value:08x}");
                }
            }
            "checkpoint" | "save" => {
                let name = words
                    .next()
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("#{}", checkpoints.len()));
                let checkpoint = match try_monitor_checkpoint_save(
                    name.clone(),
                    cpu,
                    kernel,
                    desktop.framebuffer(),
                ) {
                    Ok(checkpoint) => checkpoint,
                    Err(err) => {
                        println!("  checkpoint not saved: {err}");
                        continue;
                    }
                };
                checkpoints.push(checkpoint);
                println!("  checkpoint {} saved as {name}", checkpoints.len() - 1);
            }
            "checkpoints" | "saves" => {
                if checkpoints.is_empty() {
                    println!("  no checkpoints");
                } else {
                    for (index, checkpoint) in checkpoints.iter().enumerate() {
                        println!("  {index}: {}", checkpoint.name);
                    }
                }
            }
            "rewind" | "restore" => {
                let selector = words.next().unwrap_or("#last");
                let Some(checkpoint) = select_checkpoint(&checkpoints, selector) else {
                    println!("  no checkpoint matching `{selector}`");
                    continue;
                };
                let (restored_cpu, restored_kernel, restored_framebuffer) =
                    match try_monitor_checkpoint_restore(checkpoint, kernel) {
                        Ok(restored) => restored,
                        Err(err) => {
                            println!("  checkpoint not restored: {err}");
                            continue;
                        }
                    };
                *cpu = restored_cpu;
                *kernel = restored_kernel;
                *desktop.framebuffer_mut() = restored_framebuffer;
                desktop.present()?;
                recovery_required = false;
                println!("  restored checkpoint {}", checkpoint.name);
            }
            "quit" | "q" | "exit" => break,
            other => {
                println!("  unknown monitor command `{other}`; type `help`");
            }
        }
    }
    Ok(())
}

fn monitor_command_blocked_by_recovery(command: &str) -> bool {
    matches!(
        command,
        "continue" | "c" | "until" | "run-until" | "tap" | "checkpoint" | "save"
    )
}

fn ensure_monitor_checkpointable(kernel: &CeKernel) -> Result<()> {
    if kernel.memory.has_stable_private_slabs() {
        return Err(wince_emulation_v3::Error::Unsupported(
            "slab-backed private VM needs machine-level checkpoint relineage".to_owned(),
        ));
    }
    if kernel.memory.has_stable_writable_image_runs() {
        return Err(wince_emulation_v3::Error::Unsupported(
            "writable image backing needs machine-level checkpoint relineage".to_owned(),
        ));
    }
    Ok(())
}

fn try_monitor_checkpoint_save(
    name: String,
    cpu: &UnicornMips,
    kernel: &CeKernel,
    framebuffer: &VirtualFramebuffer,
) -> Result<MonitorCheckpoint> {
    ensure_monitor_checkpointable(kernel)?;
    if cpu.has_sealed_image_runs() {
        return Err(wince_emulation_v3::Error::Unsupported(
            "sealed main image needs machine-level checkpoint relineage".to_owned(),
        ));
    }
    Ok(MonitorCheckpoint {
        name,
        cpu: cpu.clone(),
        kernel: kernel.clone(),
        framebuffer: framebuffer.clone(),
    })
}

fn try_monitor_checkpoint_restore(
    checkpoint: &MonitorCheckpoint,
    destination_kernel: &CeKernel,
) -> Result<(UnicornMips, CeKernel, VirtualFramebuffer)> {
    ensure_monitor_checkpointable(destination_kernel)?;
    ensure_monitor_checkpointable(&checkpoint.kernel)?;
    if checkpoint.cpu.has_sealed_image_runs() {
        return Err(wince_emulation_v3::Error::Unsupported(
            "sealed main image needs machine-level checkpoint relineage".to_owned(),
        ));
    }
    Ok((
        checkpoint.cpu.clone(),
        checkpoint.kernel.clone(),
        checkpoint.framebuffer.clone(),
    ))
}

fn select_checkpoint<'a>(
    checkpoints: &'a [MonitorCheckpoint],
    selector: &str,
) -> Option<&'a MonitorCheckpoint> {
    if selector == "#last" || selector == "last" {
        return checkpoints.last();
    }
    if let Ok(index) = selector.parse::<usize>() {
        return checkpoints.get(index);
    }
    checkpoints
        .iter()
        .rev()
        .find(|checkpoint| checkpoint.name == selector)
}

fn monitor_run_once(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    desktop: &mut DesktopRuntime,
    limits: UnicornRunLimits,
    framebuffer_dump: Option<&Path>,
    mut attempt_observer: Option<&mut dyn FnMut(&AttemptExecution)>,
) -> Result<()> {
    let input_before = enqueue_desktop_input(desktop, kernel)?;
    if input_before != 0 {
        println!("  drained {input_before} host input event(s)");
    }
    let mut continued = 0usize;
    let run_started = Instant::now();
    loop {
        let current_limits = UnicornRunLimits {
            wall_clock_limit_ms: remaining_wall_clock_limit_ms(
                limits.wall_clock_limit_ms,
                run_started.elapsed(),
            ),
            ..limits
        };
        let execution = desktop.run_cpu_until_report(cpu, kernel, current_limits);
        if let Some(observer) = attempt_observer.as_mut() {
            observer(&execution);
        }
        if let Err(err) = execution.result {
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                eprintln!("  Unicorn debug: {}", snapshot.summary());
            }
            if let Some(path) = framebuffer_dump {
                desktop.framebuffer().write_ppm(path)?;
                eprintln!("  framebuffer dump: {}", path.display());
            }
            if let Err(status_err) = desktop.show_stopped_message("Emulator process stopped") {
                eprintln!("  presenter status update failed: {status_err}");
            }
            return Err(err);
        }
        if requested_pc_stop_reached(
            limits.stop_pc,
            cpu.last_debug_snapshot(),
            cpu.saved_context_pc(),
        ) {
            break;
        }
        desktop.present()?;
        let wall_expired =
            wall_clock_limit_expired(limits.wall_clock_limit_ms, run_started.elapsed());
        if wall_expired
            && !monitor_has_immediate_process_handoff(cpu, kernel)
            && !cpu.has_ready_parked_send_unblock(kernel)
        {
            break;
        }
        if !monitor_continue_process_handoff(cpu, kernel, current_limits, &mut continued)? {
            break;
        }
    }
    if !requested_pc_stop_reached(
        limits.stop_pc,
        cpu.last_debug_snapshot(),
        cpu.saved_context_pc(),
    ) {
        let input_after = enqueue_desktop_input(desktop, kernel)?;
        if input_after != 0 {
            println!("  drained {input_after} host input event(s)");
        }
    }
    desktop.present()?;
    if let Some(snapshot) = cpu.last_debug_snapshot() {
        println!("  Unicorn stopped: {}", snapshot.summary());
    }
    if let Some(path) = framebuffer_dump {
        desktop.framebuffer().write_ppm(path)?;
        println!("  framebuffer dump: {}", path.display());
    }
    desktop.show_stopped_message("Emulator process stopped")?;
    Ok(())
}

fn monitor_continue_process_handoff(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    limits: UnicornRunLimits,
    continued: &mut usize,
) -> Result<bool> {
    if requested_pc_stop_reached(
        limits.stop_pc,
        cpu.last_debug_snapshot(),
        cpu.saved_context_pc(),
    ) {
        return Ok(false);
    }
    const MAX_MONITOR_CONTINUATIONS: usize = 16;
    if *continued >= MAX_MONITOR_CONTINUATIONS {
        return Ok(false);
    }
    if cpu.clear_orphaned_cross_process_send_yield(kernel) {
        *continued += 1;
        return Ok(true);
    }
    let active_exit_completed = active_process_exited(cpu, kernel);
    let active_context_returned_without_continuation = cpu.last_stop_is_guest_thread_return_stub();
    let process_handoff_switched = if active_exit_completed {
        cpu.try_switch_to_next_parked_child_process(kernel)?
    } else if active_context_returned_without_continuation {
        if cpu.preserve_current_on_process_handoff(kernel) {
            cpu.rotate_to_next_parked_process(kernel)
        } else {
            cpu.try_switch_to_next_parked_child_process(kernel)?
        }
    } else {
        false
    };
    if process_handoff_switched {
        *continued += 1;
        return Ok(true);
    }
    if let Some(target_process_id) = cpu
        .last_debug_snapshot()
        .and_then(|snapshot| snapshot.cross_process_send_yield.as_ref())
        .map(|yielded| yielded.target_process_id)
        && cpu.rotate_to_parked_process_id(kernel, target_process_id)
    {
        *continued += 1;
        return Ok(true);
    }
    let Some(snapshot) = cpu.last_debug_snapshot() else {
        return Ok(false);
    };
    let native_service_boundary = cpu.last_native_slice_outcome().is_service_boundary();
    let message_waiter = snapshot_has_live_blocked_message_waiter(snapshot, kernel);
    let should_rotate_process = should_rotate_parked_process(
        cpu.has_parked_child_processes(),
        cpu.has_runnable_parked_process(kernel),
        cpu.has_live_pump_priority_parked_process(kernel),
        cpu.has_ready_parked_send_unblock(kernel),
        cpu.has_ready_parked_wait_unblock(kernel),
        message_waiter,
        native_service_boundary,
        limits.live_pump,
        false,
    );
    if should_rotate_process && cpu.rotate_to_next_parked_process(kernel) {
        *continued += 1;
        return Ok(true);
    }
    Ok(false)
}

fn monitor_has_immediate_process_handoff(cpu: &UnicornMips, kernel: &CeKernel) -> bool {
    cpu.last_stop_is_guest_thread_return_stub()
        || active_process_exited(cpu, kernel)
        || cpu
            .last_debug_snapshot()
            .is_some_and(|snapshot| snapshot.cross_process_send_yield.is_some())
}

fn requested_pc_stop_reached(
    requested_pc: Option<u32>,
    snapshot: Option<&UnicornDebugSnapshot>,
    saved_pc: Option<u32>,
) -> bool {
    requested_pc.is_some_and(|target| {
        target != 0
            && saved_pc == Some(target)
            && snapshot.is_some_and(|snapshot| {
                snapshot.pc == target
                    && snapshot
                        .pc_stop
                        .as_ref()
                        .is_some_and(|stop| stop.pc == target)
            })
    })
}

fn monitor_run_and_report(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    desktop: &mut DesktopRuntime,
    limits: UnicornRunLimits,
    framebuffer_dump: Option<&Path>,
) -> bool {
    if let Err(err) = monitor_run_once(cpu, kernel, desktop, limits, framebuffer_dump, None) {
        if err.is_host_exit_recovery_required() {
            eprintln!("  stopped for host-exit recovery: {err}");
            return true;
        } else if cpu.last_debug_snapshot().is_some() {
            eprintln!("  stopped; use regs or trace for detail");
        } else {
            eprintln!("  stopped: {err}");
        }
    }
    false
}

fn print_monitor_help() {
    println!("  continue [wall_ms] [insns]  run until stop or bounded limit; default 1000 ms");
    println!("  until ADDRESS [wall] [insns] run until mapped PC or bounded limit");
    println!("  step                        report why live stepping is unavailable");
    println!("  tap X Y                     queue a touch tap");
    println!("  dump [path]                 write framebuffer PPM");
    println!("  present                     present the current framebuffer");
    println!("  regs                        print compact stop/register summary");
    println!(
        "  trace [kind]                print detailed trace: all/imports/calls/code/blocks/messages/render"
    );
    println!("  tracefile KIND PATH         write selected trace detail to a file");
    println!("  map                         list memory regions and mapped static blobs");
    println!("  x ADDRESS [LEN]             hexdump mapped static PE/DLL/trap bytes");
    println!("  disasm ADDRESS [WORDS]      print mapped static MIPS instruction words");
    println!("  checkpoint [name]           save CPU wrapper, CE kernel, and framebuffer");
    println!("  checkpoints                 list saved checkpoints");
    println!("  rewind [name|index]         restore a saved checkpoint, default last");
    println!("  quit                        exit the monitor");
    println!("  note: x/disasm read mapped static bytes; live memory needs persistent CPU state");
}

fn monitor_live_pump(args: &Args) -> bool {
    args.desktop == DesktopMode::Host || args.remote_server.is_some()
}

fn parse_monitor_u64(value: &str) -> Result<u64> {
    parse_u64_value(value).map_err(|err| {
        wince_emulation_v3::Error::InvalidArgument(format!("monitor integer {value}: {err}"))
    })
}

fn parse_monitor_usize(value: &str) -> Result<usize> {
    parse_u64_value(value)
        .and_then(|value| {
            usize::try_from(value)
                .map_err(|err| wince_emulation_v3::Error::InvalidArgument(err.to_string()))
        })
        .map_err(|err| {
            wince_emulation_v3::Error::InvalidArgument(format!("monitor integer {value}: {err}"))
        })
}

fn parse_monitor_u32(value: &str) -> Result<u32> {
    let parsed = parse_monitor_u64(value)?;
    u32::try_from(parsed).map_err(|err| {
        wince_emulation_v3::Error::InvalidArgument(format!("monitor integer {value}: {err}"))
    })
}

fn parse_monitor_i32(value: &str) -> Result<i32> {
    let parsed = parse_i64_value(value).map_err(|err| {
        wince_emulation_v3::Error::InvalidArgument(format!("monitor integer {value}: {err}"))
    })?;
    i32::try_from(parsed).map_err(|err| {
        wince_emulation_v3::Error::InvalidArgument(format!("monitor integer {value}: {err}"))
    })
}

fn print_monitor_hexdump(base: u32, bytes: &[u8]) {
    for (line_index, chunk) in bytes.chunks(16).enumerate() {
        let address = base.wrapping_add((line_index * 16) as u32);
        print!("  0x{address:08x}:");
        for index in 0..16 {
            if let Some(byte) = chunk.get(index) {
                print!(" {byte:02x}");
            } else {
                print!("   ");
            }
        }
        print!("  |");
        for byte in chunk {
            let ascii = if byte.is_ascii_graphic() || *byte == b' ' {
                char::from(*byte)
            } else {
                '.'
            };
            print!("{ascii}");
        }
        println!("|");
    }
}

fn print_monitor_trace(snapshot: &UnicornDebugSnapshot, selector: &str) {
    print!("{}", monitor_trace_text(snapshot, selector));
}

fn monitor_trace_text(snapshot: &UnicornDebugSnapshot, selector: &str) -> String {
    let mut out = String::new();
    match selector {
        "all" | "full" => {
            let _ = writeln!(&mut out, "  Unicorn detail: {snapshot}");
        }
        "summary" | "regs" => {
            let _ = writeln!(&mut out, "  Unicorn stopped: {}", snapshot.summary());
        }
        "imports" => push_monitor_records(&mut out, "imports", &snapshot.last_imports),
        "milestones" | "import-milestones" => {
            push_monitor_records(&mut out, "import milestones", &snapshot.import_milestones)
        }
        "counts" | "import-counts" => {
            push_monitor_records(&mut out, "import counts", &snapshot.import_counts)
        }
        "loader" | "loader-summary" => {
            push_monitor_loader_summary(&mut out, snapshot.runtime_loader_stats);
        }
        "calls" => push_monitor_records(&mut out, "calls", &snapshot.last_calls),
        "code" => push_monitor_records(&mut out, "code", &snapshot.last_code),
        "blocks" => push_monitor_records(&mut out, "blocks", &snapshot.last_blocks),
        "messages" | "msgs" => {
            push_monitor_records(&mut out, "message ops", &snapshot.recent_message_ops);
            push_monitor_records(&mut out, "messages", &snapshot.last_messages);
        }
        "window-imports" | "winimports" => {
            push_monitor_records(&mut out, "window imports", &snapshot.window_imports)
        }
        "presentation" | "present" | "presentation-imports" => push_monitor_records(
            &mut out,
            "presentation imports",
            &snapshot.presentation_imports,
        ),
        "windows" | "wnd" => {
            push_monitor_windows(&mut out, &snapshot.z_order, &snapshot.windows);
        }
        "wndproc" => {
            push_monitor_records(&mut out, "wndproc returns", &snapshot.last_wndproc_returns);
            push_monitor_records(
                &mut out,
                "wndproc calls",
                &snapshot.last_wndproc_call_traces,
            );
        }
        "render" => {
            push_monitor_records(
                &mut out,
                "presentation imports",
                &snapshot.presentation_imports,
            );
            push_monitor_records(&mut out, "inavi display", &snapshot.last_inavi_display);
            push_monitor_records(
                &mut out,
                "inavi controller",
                &snapshot.last_inavi_controller,
            );
            push_monitor_records(
                &mut out,
                "inavi render milestones",
                &snapshot.inavi_render_milestones,
            );
        }
        "controller" | "inavi-controller" => {
            push_monitor_records(
                &mut out,
                "inavi controller",
                &snapshot.last_inavi_controller,
            );
            push_monitor_records(
                &mut out,
                "inavi render milestones",
                &snapshot.inavi_render_milestones,
            );
        }
        "guest" | "guest-entry" | "guest-entries" => {
            push_monitor_records(&mut out, "guest entries", &snapshot.guest_entry_traces);
        }
        "resource" | "resources" => {
            let resource_records: Vec<_> = snapshot
                .inavi_render_milestones
                .iter()
                .filter(|trace| is_resource_trace_label(trace.label))
                .collect();
            push_monitor_records(&mut out, "resource milestones", &resource_records);
        }
        "files" | "file-summary" => {
            push_monitor_file_summary(
                &mut out,
                snapshot.file_io_stats,
                &snapshot.recent_file_open_ops,
                &snapshot.recent_file_ops,
            );
        }
        "files-full" | "file-full" => {
            push_monitor_records(&mut out, "file opens", &snapshot.recent_file_open_ops);
            push_monitor_records(&mut out, "file ops", &snapshot.recent_file_ops);
        }
        "processes" | "process" | "proc" => {
            push_monitor_records(&mut out, "process ops", &snapshot.recent_process_ops);
        }
        "events" | "event" => {
            push_monitor_records(&mut out, "event ops", &snapshot.recent_event_ops);
        }
        other => {
            let _ = writeln!(
                &mut out,
                "  unknown trace kind `{other}`; use all/imports/milestones/counts/loader/calls/code/blocks/messages/window-imports/presentation/windows/wndproc/render/controller/guest/resource/files/files-full/processes/events"
            );
        }
    }
    out
}

fn live_kernel_messages_text(kernel: &CeKernel) -> String {
    let mut out = String::new();
    push_monitor_records(&mut out, "live message ops", kernel.recent_message_ops());
    out
}

fn live_kernel_processes_text(kernel: &CeKernel) -> String {
    let mut out = String::new();
    push_monitor_records(&mut out, "live process ops", kernel.recent_process_ops());
    out
}

fn live_kernel_message_boxes_text(kernel: &CeKernel) -> String {
    let mut out = String::new();
    let records = kernel.shell.message_boxes().collect::<Vec<_>>();
    if records.is_empty() {
        let _ = writeln!(out, "  message boxes: none");
        return out;
    }
    let _ = writeln!(out, "  message boxes:");
    for (index, record) in records.iter().enumerate() {
        let dialog_live = kernel.gwe.is_window(record.dialog_hwnd);
        let dialog_visible = dialog_live && kernel.gwe.is_window_visible(record.dialog_hwnd);
        let _ = writeln!(
            out,
            "    {index}: tid={} owner=0x{:08x} dialog=0x{:08x} style=0x{:08x} result={} rendered={} live={} visible={} was_active={} caller_pc={} trap_pc={} caller_module={:?} caption={:?} text={:?}",
            record.thread_id,
            record.owner_hwnd,
            record.dialog_hwnd,
            record.style,
            record.result,
            record.rendered,
            dialog_live,
            dialog_visible,
            record.dialog_was_active,
            record
                .caller_pc
                .map(|pc| format!("0x{pc:08x}"))
                .unwrap_or_else(|| "none".to_owned()),
            record
                .trap_pc
                .map(|pc| format!("0x{pc:08x}"))
                .unwrap_or_else(|| "none".to_owned()),
            record.caller_module,
            record.caption,
            record.text
        );
    }
    out
}

fn live_kernel_devices_text(kernel: &CeKernel) -> String {
    let mut out = kernel.device_debug_text();
    push_monitor_records(&mut out, "live device ops", kernel.recent_device_ops());
    out
}

fn live_kernel_display_perf_text(kernel: &CeKernel) -> String {
    const ROW_BYTES: usize = 64;
    const PARAM_COUNT: usize = 8;
    let bytes = kernel.display_perf_timing_bytes();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  display perf: unhandled={}",
        kernel.display_perf_unhandled()
    );
    let mut rows = 0usize;
    for (index, row) in bytes.chunks_exact(ROW_BYTES).enumerate() {
        let read_u32 = |offset: usize| -> u32 {
            u32::from_le_bytes(row[offset..offset + 4].try_into().unwrap())
        };
        let rop = read_u32(0);
        let c_gpe = read_u32(4);
        let dw_gpe_time = read_u32(8);
        let c_emul = read_u32(12);
        let c_hardware = read_u32(20);
        let dw_wait_time = read_u32(28);
        if rop == 0 && c_gpe == 0 && c_emul == 0 && c_hardware == 0 {
            continue;
        }
        let params = (0..PARAM_COUNT)
            .map(|param| read_u32(32 + param * 4).to_string())
            .collect::<Vec<_>>()
            .join(",");
        let _ = writeln!(
            out,
            "    {index}: rop=0x{rop:08x} gpe={c_gpe} gpe_time={dw_gpe_time} emul={c_emul} hardware={c_hardware} wait={dw_wait_time} params=[{params}]"
        );
        rows += 1;
    }
    if rows == 0 {
        out.push_str("    none\n");
    }
    out
}

fn live_kernel_waits_text(kernel: &CeKernel) -> String {
    let waits = kernel.blocked_waiters().collect::<Vec<_>>();
    if waits.is_empty() {
        return "  live scheduler waits: none\n".to_owned();
    }
    let mut out = String::new();
    let _ = writeln!(out, "  live scheduler waits:");
    for wait in waits {
        let handles = if wait.wait_handles.is_empty() {
            "none".to_owned()
        } else {
            wait.wait_handles
                .iter()
                .map(|handle| format!("0x{handle:08x}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        let _ = writeln!(
            out,
            "    id={} tid={} kind={:?} timeout={} started={} handles={}",
            wait.id, wait.thread_id, wait.kind, wait.timeout_ms, wait.wait_started_ms, handles
        );
    }
    out
}

fn live_kernel_windows_text(kernel: &CeKernel) -> String {
    let mut out = String::new();
    let z_order = kernel.gwe.z_order_snapshot();
    if !z_order.is_empty() {
        let _ = write!(out, "  z-order:");
        for hwnd in z_order {
            let _ = write!(out, " 0x{hwnd:08x}");
        }
        let _ = writeln!(out);
    }
    let windows = kernel.gwe.windows_snapshot();
    if windows.is_empty() {
        let _ = writeln!(out, "  windows: none");
        return out;
    }
    let _ = writeln!(out, "  windows:");
    for window in windows {
        let parent = window
            .parent
            .map(|hwnd| format!("0x{hwnd:08x}"))
            .unwrap_or_else(|| "<none>".to_owned());
        let owner = window
            .owner
            .map(|hwnd| format!("0x{hwnd:08x}"))
            .unwrap_or_else(|| "<none>".to_owned());
        let update = window
            .update_rects
            .iter()
            .copied()
            .reduce(wince_emulation_v3::ce::gwe::Rect::union)
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "    0x{:08x} tid={} pid={} parent={} owner={} class=`{}` title=`{}` vis={} enabled={} destroying={} dead={} update_pending={} erase_pending={} update={},{}-{},{} style=0x{:08x} ex=0x{:08x} rect={},{}-{},{} client={},{}-{},{} wndproc=0x{:08x}",
            window.hwnd,
            window.thread_id,
            window.process_id,
            parent,
            owner,
            window.class_name,
            window.title,
            window.visible,
            window.enabled,
            window.being_destroyed,
            window.destroyed,
            window.update_pending,
            window.erase_pending,
            update.left,
            update.top,
            update.right,
            update.bottom,
            window.style,
            window.ex_style,
            window.rect.left,
            window.rect.top,
            window.rect.right,
            window.rect.bottom,
            window.client_rect.left,
            window.client_rect.top,
            window.client_rect.right,
            window.client_rect.bottom,
            window.wndproc,
        );
    }
    out
}

fn push_monitor_loader_summary(
    out: &mut String,
    stats: wince_emulation_v3::ce::kernel::RuntimeLoaderStats,
) {
    let _ = writeln!(
        out,
        "  loader counters: load_attempt_count={} successful_map_count={} dependency_load_count={} export_lookup_count={} export_lookup_miss_count={} forwarded_export_count={} tls_callback_count={} dllmain_attach_count={} dllmain_detach_count={} loud_failure_count={}",
        stats.load_attempt_count,
        stats.successful_map_count,
        stats.dependency_load_count,
        stats.export_lookup_count,
        stats.export_lookup_miss_count,
        stats.forwarded_export_count,
        stats.tls_callback_count,
        stats.dllmain_attach_count,
        stats.dllmain_detach_count,
        stats.loud_failure_count
    );
}

fn is_resource_trace_label(label: &str) -> bool {
    label.starts_with("resource_")
        || label.starts_with("query_5237_")
        || matches!(
            label,
            "init_dialog_resource_check" | "app_query_thunk_entry" | "app_query_thunk_target"
        )
}

fn push_monitor_windows(out: &mut String, z_order: &[u32], windows: &[UnicornWindowSnapshot]) {
    if !z_order.is_empty() {
        let _ = write!(out, "  z-order:");
        for hwnd in z_order {
            let _ = write!(out, " 0x{hwnd:08x}");
        }
        let _ = writeln!(out);
    }
    if windows.is_empty() {
        let _ = writeln!(out, "  windows: none");
        return;
    }
    let _ = writeln!(out, "  windows:");
    for window in windows {
        let parent = window
            .parent
            .map(|hwnd| format!("0x{hwnd:08x}"))
            .unwrap_or_else(|| "<none>".to_owned());
        let owner = window
            .owner
            .map(|hwnd| format!("0x{hwnd:08x}"))
            .unwrap_or_else(|| "<none>".to_owned());
        let _ = writeln!(
            out,
            "    0x{:08x} tid={} parent={} owner={} class=`{}` title=`{}` vis={} destroying={} dead={} style=0x{:08x} ex=0x{:08x} upd={} erase={} rect={},{}-{},{} client={},{}-{},{} update={},{}-{},{} wndproc=0x{:08x}",
            window.hwnd,
            window.thread_id,
            parent,
            owner,
            window.class_name,
            window.title,
            window.visible,
            window.being_destroyed,
            window.destroyed,
            window.style,
            window.ex_style,
            window.update_pending,
            window.erase_pending,
            window.rect.left,
            window.rect.top,
            window.rect.right,
            window.rect.bottom,
            window.client_rect.left,
            window.client_rect.top,
            window.client_rect.right,
            window.client_rect.bottom,
            window.update_rect.left,
            window.update_rect.top,
            window.update_rect.right,
            window.update_rect.bottom,
            window.wndproc,
        );
    }
}

fn push_monitor_file_summary(
    out: &mut String,
    stats: wince_emulation_v3::ce::file::FileIoStats,
    open_records: &[wince_emulation_v3::ce::kernel::FileTraceRecord],
    records: &[wince_emulation_v3::ce::kernel::FileTraceRecord],
) {
    let _ = writeln!(
        out,
        "  file counters: host_file_open_count={} host_file_read_count={} host_file_read_bytes={} memory_backed_open_count={} max_read_request={}",
        stats.host_file_open_count,
        stats.host_file_read_count,
        stats.host_file_read_bytes,
        stats.memory_backed_open_count,
        stats.max_read_request
    );
    if open_records.is_empty() && records.is_empty() {
        let _ = writeln!(out, "  file summary: none");
        return;
    }

    let _ = writeln!(out, "  file opens:");
    if open_records.is_empty() {
        let _ = writeln!(out, "    none");
    } else {
        for record in open_records {
            let path = record.path.as_deref().unwrap_or("<unknown>");
            let handle = record
                .handle
                .map(|handle| format!("0x{handle:08x}"))
                .unwrap_or_else(|| "-".to_owned());
            let result = record
                .result
                .map(|result| format!("0x{result:08x}"))
                .unwrap_or_else(|| "-".to_owned());
            let requested = record
                .requested
                .map(|requested| format!(" req=0x{requested:08x}"))
                .unwrap_or_default();
            let position = record
                .position
                .map(|position| format!(" pos=0x{position:08x}"))
                .unwrap_or_default();
            let detail = record.preview.as_deref().unwrap_or("");
            let error = record.error.as_deref().unwrap_or("");
            let _ = writeln!(
                out,
                "    {} handle={} result={}{}{} {}",
                record.op, handle, result, requested, position, path
            );
            if !detail.is_empty() {
                let _ = writeln!(out, "      {detail}");
            }
            if !error.is_empty() {
                let _ = writeln!(out, "      error={error}");
            }
        }
    }

    #[derive(Default)]
    struct FileSummary {
        count: usize,
        requested: u64,
        transferred: u64,
        last_position: Option<u64>,
        last_preview: Option<String>,
        last_error: Option<String>,
    }

    let mut summaries: BTreeMap<(String, &'static str), FileSummary> = BTreeMap::new();
    for record in records {
        if record.op.ends_with("Arg") || record.op == "CreateFileW" || record.op == "FindFirstFileW"
        {
            continue;
        }
        let key = (
            record
                .path
                .clone()
                .unwrap_or_else(|| "<unknown>".to_owned()),
            record.op,
        );
        let summary = summaries.entry(key).or_default();
        summary.count += 1;
        summary.requested += u64::from(record.requested.unwrap_or(0));
        summary.transferred += u64::from(record.transferred.unwrap_or(0));
        summary.last_position = record.position;
        summary.last_preview = record.preview.clone();
        summary.last_error = record.error.clone();
    }

    let _ = writeln!(out, "  file activity:");
    if summaries.is_empty() {
        let _ = writeln!(out, "    none");
    } else {
        for ((path, op), summary) in summaries {
            let last_position = summary
                .last_position
                .map(|position| position.to_string())
                .unwrap_or_else(|| "-".to_owned());
            let last_preview = summary.last_preview.unwrap_or_default();
            let last_error = summary.last_error.unwrap_or_default();
            let _ = writeln!(
                out,
                "    {op} count={} requested={} transferred={} last_pos={} {}",
                summary.count, summary.requested, summary.transferred, last_position, path
            );
            if !last_preview.is_empty() {
                let _ = writeln!(out, "      {last_preview}");
            }
            if !last_error.is_empty() {
                let _ = writeln!(out, "      error={last_error}");
            }
        }
    }
}

fn push_monitor_records<T: std::fmt::Debug>(out: &mut String, label: &str, records: &[T]) {
    if records.is_empty() {
        let _ = writeln!(out, "  {label}: none");
        return;
    }
    let _ = writeln!(out, "  {label}:");
    for (index, record) in records.iter().enumerate() {
        let _ = writeln!(out, "    {index}: {record:?}");
    }
}

fn parse_u64_value(value: &str) -> Result<u64> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
            .map_err(|err| wince_emulation_v3::Error::InvalidArgument(err.to_string()))
    } else {
        value
            .parse()
            .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{err}")))
    }
}

fn parse_i64_value(value: &str) -> Result<i64> {
    let (sign, digits) = value
        .strip_prefix('-')
        .map(|digits| (-1i64, digits))
        .unwrap_or((1, value));
    let parsed = parse_u64_value(digits)?;
    i64::try_from(parsed)
        .map(|parsed| sign.saturating_mul(parsed))
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(err.to_string()))
}

impl DesktopRuntime {
    fn framebuffer(&self) -> &VirtualFramebuffer {
        match self {
            Self::Virtual(desktop) => desktop.framebuffer(),
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(desktop) => desktop.framebuffer(),
            #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
            Self::Host(desktop) => desktop.framebuffer(),
        }
    }

    fn framebuffer_mut(&mut self) -> &mut VirtualFramebuffer {
        match self {
            Self::Virtual(desktop) => desktop.framebuffer_mut(),
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(desktop) => desktop.framebuffer_mut(),
            #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
            Self::Host(desktop) => desktop.framebuffer_mut(),
        }
    }

    fn present(&mut self) -> Result<()> {
        match self {
            Self::Virtual(desktop) => {
                let _ = desktop.present()?;
            }
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(desktop) => {
                let _ = desktop.present()?;
            }
            #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
            Self::Host(desktop) => {
                let _ = desktop.present()?;
            }
        }
        Ok(())
    }

    fn show_stopped_message(&mut self, _message: &str) -> Result<()> {
        match self {
            Self::Virtual(_) => {}
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(desktop) => {
                desktop.presenter_mut().show_stopped_message(_message)?;
            }
            #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
            Self::Host(desktop) => {
                desktop.presenter_mut().show_stopped_message(_message)?;
            }
        }
        Ok(())
    }

    /// Preserve the native attempt's detached evidence across desktop
    /// wrappers. Presentation failure happens before any CPU attempt.
    fn run_cpu_until_report(
        &mut self,
        cpu: &mut UnicornMips,
        kernel: &mut CeKernel,
        limits: UnicornRunLimits,
    ) -> wince_emulation_v3::emulator::cpu::AttemptExecution {
        match self {
            Self::Virtual(desktop) => {
                if limits.live_pump
                    && let Some(server) = kernel.remote_server.clone()
                {
                    let mut live_framebuffer =
                        RemoteLiveFramebuffer::new(desktop.framebuffer_mut(), server);
                    cpu.run_until_import_trap_with_framebuffer_limits_report(
                        kernel,
                        &mut live_framebuffer,
                        limits,
                    )
                } else {
                    cpu.run_until_import_trap_with_framebuffer_limits_report(
                        kernel,
                        desktop.framebuffer_mut(),
                        limits,
                    )
                }
            }
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(desktop) => {
                use wince_emulation_v3::emulator::cpu::{AttemptExecution, NativeAttemptReport};

                let (framebuffer, presenter) = desktop.framebuffer_and_presenter_mut();
                if let Err(err) = presenter.blit(framebuffer) {
                    return AttemptExecution {
                        report: NativeAttemptReport::no_attempt_failure(limits, "presenter_blit"),
                        result: Err(err),
                    };
                }
                let mut live_framebuffer =
                    HostLiveFramebuffer::new(framebuffer, presenter, kernel.remote_server.clone());
                cpu.run_until_import_trap_with_framebuffer_limits_report(
                    kernel,
                    &mut live_framebuffer,
                    limits,
                )
            }
            #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
            Self::Host(desktop) => {
                use wince_emulation_v3::emulator::cpu::{AttemptExecution, NativeAttemptReport};

                let (framebuffer, presenter) = desktop.framebuffer_and_presenter_mut();
                if let Err(err) = presenter.blit(framebuffer) {
                    return AttemptExecution {
                        report: NativeAttemptReport::no_attempt_failure(limits, "presenter_blit"),
                        result: Err(err),
                    };
                }
                let mut live_framebuffer = LinuxHostLiveFramebuffer::new(
                    framebuffer,
                    presenter,
                    kernel.remote_server.clone(),
                );
                cpu.run_until_import_trap_with_framebuffer_limits_report(
                    kernel,
                    &mut live_framebuffer,
                    limits,
                )
            }
        }
    }

    fn poll_input(&mut self) -> Result<Vec<VirtualInputEvent>> {
        match self {
            Self::Virtual(desktop) => desktop.poll_input(),
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(desktop) => desktop.poll_input(),
            #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
            Self::Host(desktop) => desktop.poll_input(),
        }
    }

    fn describe(&self) -> &'static str {
        match self {
            Self::Virtual(_) => "virtual/null presenter",
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(_) => "win32 host presenter",
            #[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
            Self::Host(_) => "linux X11 host presenter",
        }
    }
}

struct RemoteLiveFramebuffer<'a> {
    framebuffer: &'a mut VirtualFramebuffer,
    remote_server: RemoteServer,
    last_publish: Instant,
    publish_interval: Duration,
    pending_guest_dirty: bool,
}

impl<'a> RemoteLiveFramebuffer<'a> {
    fn new(framebuffer: &'a mut VirtualFramebuffer, remote_server: RemoteServer) -> Self {
        let publish_interval = remote_server.video_frame_interval();
        Self {
            framebuffer,
            remote_server,
            last_publish: Instant::now()
                .checked_sub(Duration::from_millis(16))
                .unwrap_or_else(Instant::now),
            publish_interval,
            pending_guest_dirty: false,
        }
    }

    fn publish_if_due(&mut self, force: bool) {
        if !self.pending_guest_dirty {
            return;
        }
        let now = Instant::now();
        if !force && now.duration_since(self.last_publish) < self.publish_interval {
            return;
        }
        self.remote_server.publish_framebuffer(self.framebuffer);
        self.last_publish = now;
        self.pending_guest_dirty = false;
    }
}

impl Framebuffer for RemoteLiveFramebuffer<'_> {
    fn info(&self) -> FramebufferInfo {
        self.framebuffer.info()
    }

    fn pixels(&self) -> &[u8] {
        self.framebuffer.pixels()
    }

    fn pixels_mut(&mut self) -> &mut [u8] {
        self.framebuffer.pixels_mut()
    }

    fn mark_dirty(&mut self, rect: FramebufferRect) {
        self.framebuffer.mark_dirty(rect);
        self.pending_guest_dirty = true;
        self.publish_if_due(is_large_dirty_rect(rect, self.framebuffer.info()));
    }

    fn dirty_rects(&self) -> &[FramebufferRect] {
        self.framebuffer.dirty_rects()
    }

    fn take_dirty_rects(&mut self) -> Vec<FramebufferRect> {
        self.framebuffer.take_dirty_rects()
    }

    fn emulator_tick(&mut self) -> Result<()> {
        self.publish_if_due(false);
        Ok(())
    }
}

#[cfg(all(windows, feature = "win32-desktop"))]
struct HostLiveFramebuffer<'a> {
    framebuffer: &'a mut VirtualFramebuffer,
    presenter: &'a mut wince_emulation_v3::ce::win32_desktop::Win32Presenter,
    remote_server: Option<RemoteServer>,
    last_blit: Instant,
    blit_interval: Duration,
    pending_guest_dirty: bool,
    pending_error: Option<wince_emulation_v3::Error>,
}

#[cfg(all(windows, feature = "win32-desktop"))]
impl<'a> HostLiveFramebuffer<'a> {
    fn new(
        framebuffer: &'a mut VirtualFramebuffer,
        presenter: &'a mut wince_emulation_v3::ce::win32_desktop::Win32Presenter,
        remote_server: Option<RemoteServer>,
    ) -> Self {
        let blit_interval = remote_server
            .as_ref()
            .map(RemoteServer::video_frame_interval)
            .unwrap_or_else(|| Duration::from_millis(16));
        Self {
            framebuffer,
            presenter,
            remote_server,
            last_blit: Instant::now()
                .checked_sub(Duration::from_millis(16))
                .unwrap_or_else(Instant::now),
            blit_interval,
            pending_guest_dirty: false,
            pending_error: None,
        }
    }

    fn blit_if_due(&mut self, force: bool) -> Result<()> {
        self.presenter.pump_messages();
        if !self.pending_guest_dirty {
            return Ok(());
        }
        let now = Instant::now();
        if !force && now.duration_since(self.last_blit) < self.blit_interval {
            return Ok(());
        }
        self.presenter.blit(self.framebuffer)?;
        if let Some(server) = self.remote_server.as_ref() {
            server.publish_framebuffer(self.framebuffer);
        }
        self.last_blit = now;
        self.pending_guest_dirty = false;
        Ok(())
    }
}

#[cfg(all(windows, feature = "win32-desktop"))]
impl Framebuffer for HostLiveFramebuffer<'_> {
    fn info(&self) -> FramebufferInfo {
        self.framebuffer.info()
    }

    fn pixels(&self) -> &[u8] {
        self.framebuffer.pixels()
    }

    fn pixels_mut(&mut self) -> &mut [u8] {
        self.framebuffer.pixels_mut()
    }

    fn mark_dirty(&mut self, rect: FramebufferRect) {
        self.framebuffer.mark_dirty(rect);
        self.pending_guest_dirty = true;
        if let Err(err) = self.blit_if_due(is_large_dirty_rect(rect, self.framebuffer.info())) {
            self.pending_error = Some(err);
        }
    }

    fn dirty_rects(&self) -> &[FramebufferRect] {
        self.framebuffer.dirty_rects()
    }

    fn take_dirty_rects(&mut self) -> Vec<FramebufferRect> {
        self.framebuffer.take_dirty_rects()
    }

    fn emulator_tick(&mut self) -> Result<()> {
        if let Some(err) = self.pending_error.take() {
            return Err(err);
        }
        self.blit_if_due(false)
    }
}

#[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
struct LinuxHostLiveFramebuffer<'a> {
    framebuffer: &'a mut VirtualFramebuffer,
    presenter: &'a mut wince_emulation_v3::ce::linux_x11_desktop::LinuxX11Presenter,
    remote_server: Option<RemoteServer>,
    last_blit: Instant,
    blit_interval: Duration,
    pending_guest_dirty: bool,
    pending_error: Option<wince_emulation_v3::Error>,
}

#[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
impl<'a> LinuxHostLiveFramebuffer<'a> {
    fn new(
        framebuffer: &'a mut VirtualFramebuffer,
        presenter: &'a mut wince_emulation_v3::ce::linux_x11_desktop::LinuxX11Presenter,
        remote_server: Option<RemoteServer>,
    ) -> Self {
        let blit_interval = remote_server
            .as_ref()
            .map(RemoteServer::video_frame_interval)
            .unwrap_or_else(|| Duration::from_millis(16));
        Self {
            framebuffer,
            presenter,
            remote_server,
            last_blit: Instant::now()
                .checked_sub(Duration::from_millis(16))
                .unwrap_or_else(Instant::now),
            blit_interval,
            pending_guest_dirty: false,
            pending_error: None,
        }
    }

    fn blit_if_due(&mut self, force: bool) -> Result<()> {
        self.presenter.pump_messages();
        if !self.pending_guest_dirty {
            return Ok(());
        }
        let now = Instant::now();
        if !force && now.duration_since(self.last_blit) < self.blit_interval {
            return Ok(());
        }
        self.presenter.blit(self.framebuffer)?;
        if let Some(server) = self.remote_server.as_ref() {
            server.publish_framebuffer(self.framebuffer);
        }
        self.last_blit = now;
        self.pending_guest_dirty = false;
        Ok(())
    }
}

#[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
impl Framebuffer for LinuxHostLiveFramebuffer<'_> {
    fn info(&self) -> FramebufferInfo {
        self.framebuffer.info()
    }

    fn pixels(&self) -> &[u8] {
        self.framebuffer.pixels()
    }

    fn pixels_mut(&mut self) -> &mut [u8] {
        self.framebuffer.pixels_mut()
    }

    fn mark_dirty(&mut self, rect: FramebufferRect) {
        self.framebuffer.mark_dirty(rect);
        self.pending_guest_dirty = true;
        if let Err(err) = self.blit_if_due(is_large_dirty_rect(rect, self.framebuffer.info())) {
            self.pending_error = Some(err);
        }
    }

    fn dirty_rects(&self) -> &[FramebufferRect] {
        self.framebuffer.dirty_rects()
    }

    fn take_dirty_rects(&mut self) -> Vec<FramebufferRect> {
        self.framebuffer.take_dirty_rects()
    }

    fn emulator_tick(&mut self) -> Result<()> {
        if let Some(err) = self.pending_error.take() {
            return Err(err);
        }
        self.blit_if_due(false)
    }
}

fn is_large_dirty_rect(rect: FramebufferRect, info: FramebufferInfo) -> bool {
    let dirty_area = u64::from(rect.width).saturating_mul(u64::from(rect.height));
    let frame_area = u64::from(info.width).saturating_mul(u64::from(info.height));
    frame_area != 0 && dirty_area.saturating_mul(4) >= frame_area
}

fn create_desktop(mode: DesktopMode, image_path: Option<&Path>) -> Result<DesktopRuntime> {
    match mode {
        DesktopMode::Virtual => Ok(DesktopRuntime::Virtual(VirtualDesktop::default_primary()?)),
        DesktopMode::Host => create_host_desktop(image_path),
    }
}

#[cfg(all(windows, feature = "win32-desktop"))]
fn create_host_desktop(image_path: Option<&Path>) -> Result<DesktopRuntime> {
    let framebuffer = VirtualFramebuffer::default_primary()?;
    let title = image_path
        .map(|path| format!("WinCE virtual desktop - {}", path.display()))
        .unwrap_or_else(|| "WinCE virtual desktop".to_owned());
    let presenter = wince_emulation_v3::ce::win32_desktop::Win32Presenter::new(
        framebuffer.width(),
        framebuffer.height(),
        title,
        image_path,
    )?;
    Ok(DesktopRuntime::Host(VirtualDesktop::with_parts(
        framebuffer,
        wince_emulation_v3::ce::win32_desktop::Win32Input::new(),
        presenter,
    )))
}

#[cfg(all(target_os = "linux", feature = "linux-x11-desktop"))]
fn create_host_desktop(image_path: Option<&Path>) -> Result<DesktopRuntime> {
    let framebuffer = VirtualFramebuffer::default_primary()?;
    let title = image_path
        .map(|path| format!("WinCE virtual desktop - {}", path.display()))
        .unwrap_or_else(|| "WinCE virtual desktop".to_owned());
    let presenter = wince_emulation_v3::ce::linux_x11_desktop::LinuxX11Presenter::new(
        framebuffer.width(),
        framebuffer.height(),
        title,
        image_path,
    )?;
    Ok(DesktopRuntime::Host(VirtualDesktop::with_parts(
        framebuffer,
        wince_emulation_v3::ce::linux_x11_desktop::LinuxX11Input::new(),
        presenter,
    )))
}

#[cfg(not(any(
    all(windows, feature = "win32-desktop"),
    all(target_os = "linux", feature = "linux-x11-desktop")
)))]
fn create_host_desktop(_image_path: Option<&Path>) -> Result<DesktopRuntime> {
    let requirement = if cfg!(windows) {
        "Windows and the `win32-desktop` feature"
    } else if cfg!(target_os = "linux") {
        "Linux and the `linux-x11-desktop` feature"
    } else {
        "a supported host desktop backend"
    };
    Err(wince_emulation_v3::Error::InvalidArgument(format!(
        "--desktop host requires {requirement}"
    )))
}

fn enqueue_desktop_input(desktop: &mut DesktopRuntime, kernel: &mut CeKernel) -> Result<usize> {
    let mut queued = 0;
    for event in desktop.poll_input()? {
        match event {
            VirtualInputEvent::Key {
                virtual_key,
                pressed,
            } => {
                let phase = if pressed { "down" } else { "up" };
                kernel
                    .remote
                    .enqueue_key(phase, virtual_key)
                    .map_err(|err| {
                        wince_emulation_v3::Error::Backend(format!("host key input: {err}"))
                    })?;
                queued += 1;
            }
            VirtualInputEvent::TouchDown { x, y } => {
                if enqueue_host_touch(kernel, "down", x, y, "host touch down")? {
                    queued += 1;
                }
            }
            VirtualInputEvent::TouchMove { x, y } => {
                if enqueue_host_touch(kernel, "move", x, y, "host touch move")? {
                    queued += 1;
                }
            }
            VirtualInputEvent::TouchUp { x, y } => {
                if enqueue_host_touch(kernel, "up", x, y, "host touch up")? {
                    queued += 1;
                }
            }
        }
    }
    Ok(queued)
}

fn enqueue_host_touch(
    kernel: &mut CeKernel,
    phase: &str,
    x: i32,
    y: i32,
    context: &str,
) -> Result<bool> {
    match kernel.remote.enqueue_touch(phase, x, y) {
        Ok(()) => Ok(true),
        Err(wince_emulation_v3::ce::remote::RemoteError::TouchOutsideFramebuffer { .. }) => {
            Ok(false)
        }
        Err(err) => Err(wince_emulation_v3::Error::Backend(format!(
            "{context}: {err}"
        ))),
    }
}

fn enqueue_startup_taps(kernel: &mut CeKernel, taps: &[(i32, i32)]) -> Result<()> {
    for &(x, y) in taps {
        kernel
            .remote
            .enqueue_touch("tap", x, y)
            .map_err(|err| wince_emulation_v3::Error::Backend(format!("startup tap: {err}")))?;
    }
    Ok(())
}

fn attach_audio_for_desktop(kernel: &mut CeKernel, desktop: DesktopMode) -> String {
    match desktop {
        DesktopMode::Virtual => attach_virtual_audio(kernel),
        DesktopMode::Host => attach_host_audio(kernel),
    }
}

fn attach_virtual_audio(kernel: &mut CeKernel) -> String {
    #[cfg(debug_assertions)]
    {
        let registered =
            kernel
                .audio
                .register_sink(wince_emulation_v3::ce::audio::LoggingAudioSink::new(
                    "virtual-log",
                    32,
                ));
        if registered {
            "virtual desktop logging sink registered".to_owned()
        } else {
            "virtual desktop logging sink already registered".to_owned()
        }
    }
    #[cfg(not(debug_assertions))]
    {
        let registered =
            kernel
                .audio
                .register_sink(wince_emulation_v3::ce::audio::NullAudioSink::new(
                    "virtual-null",
                ));
        if registered {
            "virtual desktop null sink registered".to_owned()
        } else {
            "virtual desktop null sink already registered".to_owned()
        }
    }
}

fn attach_host_audio(kernel: &mut CeKernel) -> String {
    #[cfg(all(windows, feature = "win32-audio"))]
    {
        let sink = HostAudioSink::winmm("host", 32);
        let status = match sink.backend() {
            wince_emulation_v3::ce::audio::HostAudioBackend::Unplugged => {
                "host sink is unplugged".to_owned()
            }
            wince_emulation_v3::ce::audio::HostAudioBackend::Winmm(backend) => {
                let device_count = backend.device_count();
                if sink.is_connected() {
                    format!("winmm host sink registered ({device_count} output device(s))")
                } else {
                    "winmm host sink registered, but no output devices were reported".to_owned()
                }
            }
        };
        let registered = kernel.audio.register_sink(sink);
        if registered {
            status
        } else {
            "winmm host sink already registered".to_owned()
        }
    }
    #[cfg(all(windows, not(feature = "win32-audio")))]
    {
        let _ = kernel;
        "winmm host sink unavailable; enable the `win32-audio` feature".to_owned()
    }
    #[cfg(all(target_os = "linux", feature = "linux-audio"))]
    {
        let sink = HostAudioSink::alsa("host", 32);
        let registered = kernel.audio.register_sink(sink);
        if registered {
            "alsa host sink registered".to_owned()
        } else {
            "alsa host sink already registered".to_owned()
        }
    }
    #[cfg(all(target_os = "linux", not(feature = "linux-audio")))]
    {
        let _ = kernel;
        "alsa host sink unavailable; enable the `linux-audio` feature".to_owned()
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = kernel;
        "not registered on unsupported host platform".to_owned()
    }
}

impl Args {
    fn parse() -> Result<Self> {
        let mut registry = PathBuf::from(DEFAULT_REGISTRY_PATH);
        let mut devices = PathBuf::from(DEFAULT_DEVICES_PATH);
        let mut image = None;
        let mut companion_images = Vec::new();
        let mut dll_search_dirs = Vec::new();
        let mut mount_config = None;
        let mut framebuffer_dump = None;
        let mut tracefiles = Vec::new();
        let mut desktop = DesktopMode::Virtual;
        let mut cpu_instruction_limit = 0;
        let mut cpu_wall_clock_limit_ms = 0;
        let mut cpu_stop_pc = None;
        let mut startup_taps = Vec::new();
        let mut remote_server = None::<RemoteServerConfig>;
        let mut run_cpu = false;
        let mut monitor = false;
        let mut verbose = false;

        let mut args = std::env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--registry" => {
                    registry = next_path(&mut args, "--registry")?;
                }
                "--devices" => {
                    devices = next_path(&mut args, "--devices")?;
                }
                "--image" => {
                    image = Some(next_path(&mut args, "--image")?);
                }
                "--companion-image" | "--companion-target" => {
                    companion_images.push(next_path(&mut args, "--companion-image")?);
                }
                "--dll-search-dir" => {
                    dll_search_dirs.push(next_path(&mut args, "--dll-search-dir")?);
                }
                "--mount-config" => {
                    mount_config = Some(next_path(&mut args, "--mount-config")?);
                }
                "--framebuffer-dump" => {
                    framebuffer_dump = Some(next_path(&mut args, "--framebuffer-dump")?);
                }
                "--tracefile" | "--trace-file" => {
                    let selector = next_string(&mut args, "--tracefile")?;
                    let path = next_path(&mut args, "--tracefile")?;
                    tracefiles.push((selector, path));
                }
                "--desktop" => {
                    desktop = next_desktop_mode(&mut args, "--desktop")?;
                }
                "--cpu-instruction-limit" => {
                    cpu_instruction_limit = next_usize(&mut args, "--cpu-instruction-limit")?;
                }
                "--cpu-wall-clock-limit-ms" => {
                    cpu_wall_clock_limit_ms = next_u64(&mut args, "--cpu-wall-clock-limit-ms")?;
                }
                "--cpu-stop-pc" => {
                    let value = next_string(&mut args, "--cpu-stop-pc")?;
                    cpu_stop_pc = Some(parse_monitor_u32(&value)?);
                }
                "--tap" => {
                    startup_taps.push(next_tap(&mut args, "--tap")?);
                }
                "--remote-server" => {
                    let config = remote_server.get_or_insert_with(RemoteServerConfig::default);
                    if let Some(value) = args.next_if(|value| !value.starts_with("--")) {
                        config.addr = parse_socket_addr(&value, "--remote-server")?;
                    }
                }
                "--remote-bind" => {
                    let ip = next_string(&mut args, "--remote-bind")?
                        .parse::<IpAddr>()
                        .map_err(|err| {
                            wince_emulation_v3::Error::InvalidArgument(format!(
                                "--remote-bind: {err}"
                            ))
                        })?;
                    let config = remote_server.get_or_insert_with(RemoteServerConfig::default);
                    config.addr = SocketAddr::new(ip, config.addr.port());
                }
                "--remote-port" => {
                    let port = next_u16(&mut args, "--remote-port")?;
                    let config = remote_server.get_or_insert_with(RemoteServerConfig::default);
                    config.addr = SocketAddr::new(config.addr.ip(), port);
                }
                "--remote-token" => {
                    let token = next_string(&mut args, "--remote-token")?;
                    remote_server
                        .get_or_insert_with(RemoteServerConfig::default)
                        .token = Some(token);
                }
                "--remote-video-fps" => {
                    let fps = next_u32(&mut args, "--remote-video-fps")?;
                    remote_server
                        .get_or_insert_with(RemoteServerConfig::default)
                        .video_fps = fps.clamp(1, 60);
                }
                "--remote-jpeg-quality" => {
                    let quality = next_u8(&mut args, "--remote-jpeg-quality")?;
                    remote_server
                        .get_or_insert_with(RemoteServerConfig::default)
                        .jpeg_quality = quality.clamp(1, 100);
                }
                "--remote-audio" => {
                    remote_server
                        .get_or_insert_with(RemoteServerConfig::default)
                        .audio_enabled = true;
                }
                "--remote-audio-sample-rate" => {
                    let sample_rate = next_u32(&mut args, "--remote-audio-sample-rate")?;
                    remote_server
                        .get_or_insert_with(RemoteServerConfig::default)
                        .audio_sample_rate = sample_rate;
                }
                "--remote-audio-channels" => {
                    let channels = next_u16(&mut args, "--remote-audio-channels")?;
                    remote_server
                        .get_or_insert_with(RemoteServerConfig::default)
                        .audio_channels = channels;
                }
                "--remote-audio-format" => {
                    let format = next_string(&mut args, "--remote-audio-format")?;
                    remote_server
                        .get_or_insert_with(RemoteServerConfig::default)
                        .audio_format = format;
                }
                "--run-cpu" => {
                    run_cpu = true;
                }
                "--monitor" | "--debugger" => {
                    monitor = true;
                }
                "--verbose" | "-v" => {
                    verbose = true;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => {
                    return Err(wince_emulation_v3::Error::InvalidArgument(format!(
                        "unknown argument {other}"
                    )));
                }
            }
        }

        Ok(Self {
            registry,
            devices,
            image,
            companion_images,
            dll_search_dirs,
            mount_config,
            framebuffer_dump,
            tracefiles,
            desktop,
            cpu_instruction_limit,
            cpu_wall_clock_limit_ms,
            cpu_stop_pc,
            startup_taps,
            remote_server,
            run_cpu,
            monitor,
            verbose,
        })
    }
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a path")))
}

fn next_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value")))
}

fn next_usize(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<usize> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value"))
    })?;
    value
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag}: {err}")))
}

fn next_u64(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<u64> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value"))
    })?;
    value
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag}: {err}")))
}

fn next_u32(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<u32> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value"))
    })?;
    value
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag}: {err}")))
}

fn next_u16(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<u16> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value"))
    })?;
    value
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag}: {err}")))
}

fn next_u8(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<u8> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value"))
    })?;
    value
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag}: {err}")))
}

fn parse_socket_addr(value: &str, flag: &str) -> Result<SocketAddr> {
    value
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag}: {err}")))
}

fn next_desktop_mode(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<DesktopMode> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value"))
    })?;
    match value.as_str() {
        "virtual" | "null" | "headless" => Ok(DesktopMode::Virtual),
        "host" | "win32" => Ok(DesktopMode::Host),
        other => Err(wince_emulation_v3::Error::InvalidArgument(format!(
            "{flag}: expected virtual or host, got {other}"
        ))),
    }
}

fn next_tap(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<(i32, i32)> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs X,Y coordinates"))
    })?;
    let Some((x, y)) = value.split_once(',') else {
        return Err(wince_emulation_v3::Error::InvalidArgument(format!(
            "{flag}: expected X,Y, got {value}"
        )));
    };
    let x = x
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag} x: {err}")))?;
    let y = y
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag} y: {err}")))?;
    Ok((x, y))
}

fn print_help() {
    println!(
        "Usage: wince_emulation_v3 [--registry {DEFAULT_REGISTRY_PATH}] [--devices {DEFAULT_DEVICES_PATH}] [--mount-config {DEFAULT_MOUNT_CONFIG_PATH}] [--image INavi.exe] [--companion-image MultiTBT.exe]... [--dll-search-dir DIR]... [--desktop virtual|host] [--remote-server [IP:PORT]] [--remote-bind IP] [--remote-port PORT] [--remote-token TOKEN] [--remote-video-fps N] [--remote-jpeg-quality N] [--remote-audio] [--remote-audio-sample-rate N] [--remote-audio-channels N] [--remote-audio-format s16le] [--framebuffer-dump OUT.ppm] [--tracefile KIND OUT.txt]... [--cpu-instruction-limit N] [--cpu-wall-clock-limit-ms N] [--cpu-stop-pc ADDR] [--tap X,Y]... [--run-cpu] [--monitor] [--verbose]"
    );
}

fn load_import_dlls(image: &PeImage, search_dirs: &[PathBuf]) -> Result<Vec<PeImage>> {
    let mut loaded = Vec::new();
    let mut seen = BTreeSet::new();

    for descriptor in &image.imports {
        let normalized = normalize_module_name(&descriptor.module_name);
        if emulator_provided_import_module(&normalized) || !seen.insert(normalized) {
            continue;
        }
        let path =
            resolve_dll_path(&descriptor.module_name, None, search_dirs).ok_or_else(|| {
                wince_emulation_v3::Error::MissingImportDll {
                    dll: descriptor.module_name.clone(),
                }
            })?;
        loaded.push(PeImage::inspect(path)?);
    }
    preload_search_dll_if_present("commctrl.dll", search_dirs, &mut seen, &mut loaded)?;
    preload_image_directory_dlls(image, &mut seen, &mut loaded)?;

    Ok(loaded)
}

fn preload_image_directory_dlls(
    image: &PeImage,
    seen: &mut BTreeSet<String>,
    loaded: &mut Vec<PeImage>,
) -> Result<()> {
    for path in image_directory_dll_paths(Path::new(&image.path))? {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let normalized = normalize_module_name(file_name);
        if emulator_provided_import_module(&normalized) || !seen.insert(normalized) {
            continue;
        }
        loaded.push(PeImage::inspect(path)?);
    }
    Ok(())
}

fn image_directory_dll_paths(image_path: &Path) -> Result<Vec<PathBuf>> {
    let Some(parent) = image_path.parent() else {
        return Ok(Vec::new());
    };
    let mut paths = Vec::new();
    for entry in fs::read_dir(parent)
        .map_err(|err| wince_emulation_v3::Error::Backend(format!("read image dir: {err}")))?
    {
        let entry = entry.map_err(|err| {
            wince_emulation_v3::Error::Backend(format!("read image dir entry: {err}"))
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_dll = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"));
        if is_dll {
            paths.push(path);
        }
    }
    paths.sort_by_key(|path| path.file_name().map(|name| name.to_os_string()));
    Ok(paths)
}

fn preload_search_dll_if_present(
    module_name: &str,
    search_dirs: &[PathBuf],
    seen: &mut BTreeSet<String>,
    loaded: &mut Vec<PeImage>,
) -> Result<()> {
    let normalized = normalize_module_name(module_name);
    if emulator_provided_import_module(&normalized) || !seen.insert(normalized) {
        return Ok(());
    }
    if let Some(path) = resolve_dll_path(module_name, None, search_dirs) {
        loaded.push(PeImage::inspect(path)?);
    }
    Ok(())
}

fn register_loaded_modules(kernel: &mut CeKernel, cpu: &UnicornMips) {
    for module in cpu.loaded_modules() {
        kernel.register_loaded_module_with_metadata(
            module.name.clone(),
            module.base,
            module
                .exports_by_name
                .iter()
                .map(|(name, address)| (name.clone(), *address))
                .collect::<BTreeMap<_, _>>(),
            module
                .exports_by_ordinal
                .iter()
                .map(|(ordinal, address)| (*ordinal, *address))
                .collect::<BTreeMap<_, _>>(),
            LoadedModuleMetadata {
                guest_path: module.guest_path.clone(),
                host_path: module.host_path.clone(),
                image_size: module.image_size,
                entry_point: module.entry_point,
                dependencies: module.dependencies.clone(),
                tls_callbacks: module.tls_callbacks.clone(),
                forwarders_by_name: module
                    .forwarders_by_name
                    .iter()
                    .map(|(name, forwarder)| (name.clone(), forwarder.clone()))
                    .collect(),
                forwarders_by_ordinal: module
                    .forwarders_by_ordinal
                    .iter()
                    .map(|(ordinal, forwarder)| (*ordinal, forwarder.clone()))
                    .collect(),
                ref_count: 1,
                load_flags: 0,
                dynamic: module.dynamic,
                thread_library_calls_disabled: false,
            },
        );
    }
}

fn ce_module_path_for_image(kernel: &CeKernel, path: &str) -> String {
    if let Some(path) = kernel.host_path_to_guest_mount(Path::new(path)) {
        return path;
    }
    ce_module_path(path)
}

fn ce_module_path(path: &str) -> String {
    path.replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read as _;
    use wince_emulation_v3::ce::memory::{
        MEM_COMMIT, MEM_RESERVE, VirtualAllocationSource, WritableImageRunIdentity,
    };

    #[test]
    fn monitor_recovery_rejects_post_refusal_checkpoint_save() {
        for command in [
            "continue",
            "c",
            "until",
            "run-until",
            "tap",
            "checkpoint",
            "save",
        ] {
            assert!(
                monitor_command_blocked_by_recovery(command),
                "recovery hold must block `{command}`"
            );
        }
        for command in ["restore", "rewind", "quit", "regs", "trace"] {
            assert!(!monitor_command_blocked_by_recovery(command));
        }
    }

    fn kernel_with_committed_private_slab() -> CeKernel {
        let config = RuntimeConfig::load_default().expect("checkpoint test config loads");
        let mut kernel = CeKernel::boot(config);
        let owner = kernel
            .current_process_birth_key()
            .expect("boot process birth");
        let prepared = kernel
            .memory
            .prepare_private_vm(
                owner,
                VirtualAllocationSource::Private,
                0,
                0x1000,
                MEM_RESERVE | MEM_COMMIT,
                4,
            )
            .expect("prepare a committed private slab");
        assert_ne!(kernel.memory.commit_prepared_private_vm(prepared), None);
        assert!(kernel.memory.has_stable_private_slabs());
        kernel
    }

    fn kernel_with_writable_image_run() -> CeKernel {
        let config = RuntimeConfig::load_default().expect("checkpoint test config loads");
        let mut kernel = CeKernel::boot(config);
        let owner = kernel
            .current_process_birth_key()
            .expect("boot process birth");
        let identity = WritableImageRunIdentity::new(owner, 0x6000_0000, 1, 0x6000_0000)
            .expect("aligned writable image identity");
        let prepared = kernel
            .memory
            .prepare_writable_image_run(identity, &[0; 4096])
            .expect("prepare writable image run");
        assert!(
            kernel
                .memory
                .commit_prepared_writable_image_run(prepared)
                .is_some()
        );
        assert!(kernel.memory.has_stable_writable_image_runs());
        kernel
    }

    #[test]
    fn remote_summary_publishes_stable_page_budget_and_freshness_identity() {
        let kernel = kernel_with_committed_private_slab();
        let expected_used = kernel.memory.private_commit_used_bytes();
        assert_eq!(expected_used, 4096);
        let server = RemoteServer::start(RemoteServerConfig {
            addr: "127.0.0.1:0".parse().expect("loopback address"),
            ..RemoteServerConfig::default()
        })
        .expect("loopback remote server starts");
        let framebuffer = VirtualFramebuffer::default();

        publish_remote_endpoint(Some(&server), None, &kernel, &framebuffer, None, false);

        let fetch_summary = || {
            let mut stream =
                std::net::TcpStream::connect_timeout(&server.local_addr(), Duration::from_secs(1))
                    .expect("connect to loopback remote server");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("set summary read timeout");
            stream
                .write_all(b"GET /api/v1/debug/summary.txt HTTP/1.1\r\nHost: local\r\n\r\n")
                .expect("request summary");
            let mut response = String::new();
            stream
                .read_to_string(&mut response)
                .expect("read summary response");
            assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
            response
                .split_once("\r\n\r\n")
                .expect("HTTP response has a body")
                .1
                .to_owned()
        };
        let first = fetch_summary();
        let repeated = fetch_summary();
        assert_eq!(repeated, first, "a GET must not mint a new publication");

        let field = |name: &str| -> &str {
            first
                .lines()
                .find_map(|line| line.trim().strip_prefix(name)?.strip_prefix(": "))
                .unwrap_or_else(|| panic!("summary is missing {name}: {first}"))
        };
        assert_eq!(
            field("live_guest_stable_page_budget_used_bytes")
                .parse::<u64>()
                .expect("stable-page budget is numeric"),
            expected_used
        );
        assert!(
            field("publication_sequence")
                .parse::<u64>()
                .expect("publication sequence is numeric")
                > 0
        );
        assert!(
            field("publication_host_monotonic_ns")
                .parse::<u128>()
                .expect("publication monotonic timestamp is numeric")
                > 0
        );
        assert!(
            field("publication_utc_unix_ms")
                .parse::<u64>()
                .expect("publication UTC milliseconds are numeric")
                > 0
        );
    }

    #[test]
    fn monitor_checkpoint_save_rejects_slab_backed_private_vm() {
        let kernel = kernel_with_committed_private_slab();
        let owner = kernel
            .current_process_birth_key()
            .expect("boot process birth");
        let before = kernel.memory.virtual_allocations_for(owner).count();
        let cpu = UnicornMips::new().expect("checkpoint CPU initializes");
        let framebuffer = VirtualFramebuffer::default();

        let result = try_monitor_checkpoint_save("slab".to_owned(), &cpu, &kernel, &framebuffer);
        assert!(matches!(
            result,
            Err(wince_emulation_v3::Error::Unsupported(message))
                if message.contains("machine-level checkpoint relineage")
        ));
        assert_eq!(kernel.memory.virtual_allocations_for(owner).count(), before);
        assert!(kernel.memory.has_stable_private_slabs());
    }

    #[test]
    fn monitor_checkpoint_save_and_restore_reject_writable_image_backing() {
        let clean = CeKernel::boot(RuntimeConfig::load_default().expect("checkpoint config loads"));
        let cpu = UnicornMips::new().expect("checkpoint CPU initializes");
        let framebuffer = VirtualFramebuffer::default();
        let clean_checkpoint =
            try_monitor_checkpoint_save("clean".to_owned(), &cpu, &clean, &framebuffer)
                .expect("clean checkpoint saves");
        let image_kernel = kernel_with_writable_image_run();

        assert!(matches!(
            try_monitor_checkpoint_save("image".to_owned(), &cpu, &image_kernel, &framebuffer),
            Err(wince_emulation_v3::Error::Unsupported(message))
                if message.contains("writable image backing")
        ));
        assert!(matches!(
            try_monitor_checkpoint_restore(&clean_checkpoint, &image_kernel),
            Err(wince_emulation_v3::Error::Unsupported(message))
                if message.contains("writable image backing")
        ));
        let image_checkpoint = MonitorCheckpoint {
            name: "pre-guard image snapshot".to_owned(),
            cpu,
            kernel: image_kernel,
            framebuffer,
        };
        assert!(matches!(
            try_monitor_checkpoint_restore(&image_checkpoint, &clean),
            Err(wince_emulation_v3::Error::Unsupported(message))
                if message.contains("writable image backing")
        ));
    }

    #[test]
    fn monitor_checkpoint_restore_rejects_slab_backed_source_or_destination() {
        let clean = CeKernel::boot(RuntimeConfig::load_default().expect("checkpoint config loads"));
        let cpu = UnicornMips::new().expect("checkpoint CPU initializes");
        let framebuffer = VirtualFramebuffer::default();
        let clean_checkpoint =
            try_monitor_checkpoint_save("clean".to_owned(), &cpu, &clean, &framebuffer)
                .expect("clean checkpoint saves");

        let destination = kernel_with_committed_private_slab();
        let destination_owner = destination
            .current_process_birth_key()
            .expect("destination process birth");
        let destination_allocations = destination
            .memory
            .virtual_allocations_for(destination_owner)
            .count();
        assert!(matches!(
            try_monitor_checkpoint_restore(&clean_checkpoint, &destination),
            Err(wince_emulation_v3::Error::Unsupported(message))
                if message.contains("machine-level checkpoint relineage")
        ));
        assert_eq!(
            destination
                .memory
                .virtual_allocations_for(destination_owner)
                .count(),
            destination_allocations
        );

        // A saved snapshot from an older path must also fail before it can be
        // restored, even when the destination is still clean.
        let slab_checkpoint = MonitorCheckpoint {
            name: "pre-guard slab snapshot".to_owned(),
            cpu: cpu.clone(),
            kernel: destination,
            framebuffer: framebuffer.clone(),
        };
        assert!(matches!(
            try_monitor_checkpoint_restore(&slab_checkpoint, &clean),
            Err(wince_emulation_v3::Error::Unsupported(message))
                if message.contains("machine-level checkpoint relineage")
        ));
        assert!(!clean.memory.has_stable_private_slabs());
    }

    #[test]
    fn requested_pc_stop_requires_matching_fresh_stop_and_retained_context() {
        let target = 0x0040_1004;
        let mut snapshot = UnicornDebugSnapshot {
            pc: target,
            pc_stop: Some(wince_emulation_v3::emulator::cpu::UnicornPcStop {
                pc: target,
                ra: 0x0040_2000,
                sp: 0x7fff_f000,
                instruction: Some(0),
            }),
            ..Default::default()
        };
        assert!(requested_pc_stop_reached(
            Some(target),
            Some(&snapshot),
            Some(target),
        ));
        assert!(!requested_pc_stop_reached(
            None,
            Some(&snapshot),
            Some(target)
        ));
        assert!(!requested_pc_stop_reached(Some(target), None, Some(target)));
        assert!(!requested_pc_stop_reached(
            Some(target),
            Some(&snapshot),
            None
        ));
        assert!(!requested_pc_stop_reached(
            Some(target),
            Some(&snapshot),
            Some(target + 4),
        ));
        snapshot.pc = target + 4;
        assert!(!requested_pc_stop_reached(
            Some(target),
            Some(&snapshot),
            Some(target),
        ));
        snapshot.pc = target;
        snapshot.pc_stop.as_mut().unwrap().pc = target + 4;
        assert!(!requested_pc_stop_reached(
            Some(target),
            Some(&snapshot),
            Some(target),
        ));
        snapshot.pc_stop = None;
        assert!(!requested_pc_stop_reached(
            Some(target),
            Some(&snapshot),
            Some(target),
        ));
        snapshot.pc = 0;
        snapshot.pc_stop = Some(wince_emulation_v3::emulator::cpu::UnicornPcStop {
            pc: 0,
            ra: 0,
            sp: 0,
            instruction: None,
        });
        assert!(!requested_pc_stop_reached(
            Some(0),
            Some(&snapshot),
            Some(0)
        ));
    }

    #[test]
    fn maps_image_under_configured_mount_to_ce_mount_path() {
        let mut config = RuntimeConfig::load_default().unwrap();
        config
            .storage
            .mounts
            .push(wince_emulation_v3::config::MountConfig {
                name: Some("sdmmc".to_owned()),
                device_name: None,
                bus_name: None,
                guest_root: "\\SDMMC Disk".to_owned(),
                host_root: Some(PathBuf::from(r"D:\INAVI_Emulator\INAVI")),
                total_mbytes: 8192,
                free_mbytes: 4096,
                writable: true,
                removable: true,
                system: false,
                hidden: false,
                interface_classes: Vec::new(),
                registry_roots: Vec::new(),
                registry_subkey: None,
            });
        let kernel = CeKernel::boot(config);
        let path = ce_module_path_for_image(&kernel, r"D:\INAVI_Emulator\INAVI\INavi\INavi.exe");

        assert_eq!(path, r"\SDMMC Disk\INavi\INavi.exe");
    }

    #[test]
    fn host_loop_wall_clock_budget_is_total_not_per_burst() {
        assert_eq!(remaining_wall_clock_limit_ms(0, Duration::from_secs(10)), 0);
        assert_eq!(
            remaining_wall_clock_limit_ms(500, Duration::from_millis(125)),
            375
        );
        assert_eq!(
            remaining_wall_clock_limit_ms(500, Duration::from_millis(500)),
            1
        );
        assert!(!wall_clock_limit_expired(500, Duration::from_millis(499)));
        assert!(wall_clock_limit_expired(500, Duration::from_millis(500)));
        assert!(wall_clock_limit_expired(500, Duration::from_secs(10)));
        assert!(!wall_clock_limit_expired(0, Duration::from_secs(10)));
    }

    #[test]
    fn completed_cpu_idle_delay_uses_canonical_wait_deadline_and_run_cap() {
        let config = RuntimeConfig::load_default().expect("default config");
        let mut kernel = CeKernel::boot(config);
        assert_eq!(
            completed_cpu_idle_delay(&kernel, DesktopMode::Virtual, None),
            Duration::from_millis(100)
        );
        let started = kernel.timers.tick_count();
        let _ = kernel.register_blocked_waiter(
            1,
            0,
            vec![],
            SchedulerBlockedWaitKind::Sleep,
            started,
            7,
        );
        let delay = completed_cpu_idle_delay(&kernel, DesktopMode::Virtual, None);
        assert!(delay <= Duration::from_millis(7));
        assert!(delay >= Duration::from_millis(1));
        assert_eq!(
            completed_cpu_idle_delay(
                &kernel,
                DesktopMode::Virtual,
                Some(Duration::from_millis(1))
            ),
            Duration::from_millis(1)
        );
    }

    #[cfg(not(feature = "unicorn"))]
    #[test]
    fn monitor_no_feature_error_publishes_outgoing_no_attempt_report() {
        use wince_emulation_v3::emulator::cpu::{
            AttemptExecution, AttemptObservation, NativeSliceOutcome,
        };

        let mut kernel = CeKernel::boot(
            RuntimeConfig::load_default().expect("runtime config loads for monitor attempt test"),
        );
        let mut cpu = UnicornMips::new().expect("stub CPU initializes for monitor attempt test");
        let mut desktop = DesktopRuntime::Virtual(
            VirtualDesktop::default_primary().expect("virtual desktop initializes"),
        );
        let mut observed = Vec::new();
        let result = {
            let mut observe = |execution: &AttemptExecution| {
                observed.push((execution.report.clone(), execution.result.is_err()));
            };
            monitor_run_once(
                &mut cpu,
                &mut kernel,
                &mut desktop,
                UnicornRunLimits {
                    wall_clock_limit_ms: 100,
                    ..Default::default()
                },
                None,
                Some(&mut observe),
            )
        };

        let error = result.expect_err("the no-feature CPU must reject execution");
        assert!(
            error.to_string().contains("without the `unicorn` feature"),
            "monitor should reach the CPU, not fail during setup: {error}"
        );
        assert_eq!(
            observed.len(),
            1,
            "monitor must publish the outgoing report before returning the CPU error"
        );
        let (report, result_was_error) = &observed[0];
        assert!(*result_was_error);
        assert_eq!(report.final_outcome(), NativeSliceOutcome::NoAttempt);
        assert_eq!(report.failure_operation(), Some("unsupported_cpu_backend"));
        assert_eq!(report.attempt_id(), None);
        assert_eq!(report.native_result(), &AttemptObservation::Uncollected);
        assert_eq!(report.count_stop_query(), &AttemptObservation::Uncollected);
        assert_eq!(report.timeout_query(), &AttemptObservation::Uncollected);
    }

    #[cfg(not(feature = "unicorn"))]
    #[test]
    fn live_no_feature_error_publishes_outgoing_no_attempt_report() {
        use wince_emulation_v3::emulator::cpu::{AttemptExecution, NativeSliceOutcome};

        let args = Args {
            registry: PathBuf::from(DEFAULT_REGISTRY_PATH),
            devices: PathBuf::from(DEFAULT_DEVICES_PATH),
            image: None,
            companion_images: Vec::new(),
            dll_search_dirs: Vec::new(),
            mount_config: None,
            framebuffer_dump: None,
            tracefiles: Vec::new(),
            desktop: DesktopMode::Virtual,
            cpu_instruction_limit: 0,
            cpu_wall_clock_limit_ms: 1_000,
            cpu_stop_pc: None,
            startup_taps: Vec::new(),
            remote_server: None,
            run_cpu: true,
            monitor: false,
            verbose: false,
        };
        let mut kernel = CeKernel::boot(
            RuntimeConfig::load_default().expect("runtime config loads for live attempt test"),
        );
        let mut cpu = UnicornMips::new().expect("stub CPU initializes for live attempt test");
        let mut desktop = DesktopRuntime::Virtual(
            VirtualDesktop::default_primary().expect("virtual desktop initializes"),
        );
        let mut observed = Vec::new();
        let result = {
            let mut observe = |execution: &AttemptExecution| {
                observed.push((execution.report.clone(), execution.result.is_err()));
            };
            run_cpu_loop(
                &mut cpu,
                &mut kernel,
                &mut desktop,
                &args,
                Some(&mut observe),
            )
        };

        let error = result.expect_err("the no-feature CPU must reject execution");
        assert!(
            error.to_string().contains("without the `unicorn` feature"),
            "live loop should reach the CPU, not fail during setup: {error}"
        );
        assert_eq!(
            observed.len(),
            1,
            "live loop must publish before error return"
        );
        let (report, result_was_error) = &observed[0];
        assert!(*result_was_error);
        assert_eq!(report.final_outcome(), NativeSliceOutcome::NoAttempt);
        assert_eq!(report.failure_operation(), Some("unsupported_cpu_backend"));
        assert_eq!(report.attempt_id(), None);
    }

    #[test]
    fn host_no_wall_run_uses_implicit_live_slice() {
        assert_eq!(
            effective_wall_clock_limit_ms(
                0,
                Duration::from_secs(10),
                DesktopMode::Host,
                false,
                false,
                false,
            ),
            (HOST_LIVE_RUN_SLICE_MS, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                0,
                Duration::from_secs(10),
                DesktopMode::Host,
                true,
                false,
                false,
            ),
            (HOST_REMOTE_BUSY_RUN_SLICE_MS, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                0,
                Duration::from_secs(10),
                DesktopMode::Virtual,
                false,
                false,
                false,
            ),
            (0, false)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                0,
                Duration::from_secs(10),
                DesktopMode::Virtual,
                true,
                false,
                false,
            ),
            (REMOTE_LIVE_RUN_SLICE_MS, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                500,
                Duration::from_millis(125),
                DesktopMode::Host,
                false,
                false,
                false,
            ),
            (375, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                2_000,
                Duration::from_millis(125),
                DesktopMode::Host,
                true,
                false,
                false,
            ),
            (2_000 - 125, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                500,
                Duration::from_millis(125),
                DesktopMode::Host,
                false,
                true,
                false,
            ),
            (HOST_IDLE_MESSAGE_POLL_SLICE_MS, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                500,
                Duration::from_millis(125),
                DesktopMode::Virtual,
                true,
                true,
                false,
            ),
            (375, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                0,
                Duration::from_secs(10),
                DesktopMode::Host,
                true,
                false,
                true,
            ),
            (LIVE_VISIBLE_WORK_RUN_SLICE_MS, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                500,
                Duration::from_millis(125),
                DesktopMode::Host,
                true,
                false,
                true,
            ),
            (LIVE_VISIBLE_WORK_RUN_SLICE_MS, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                500,
                Duration::from_millis(450),
                DesktopMode::Host,
                true,
                false,
                true,
            ),
            (50, true)
        );
        assert_eq!(
            effective_wall_clock_limit_ms(
                0,
                Duration::from_secs(10),
                DesktopMode::Virtual,
                true,
                false,
                true,
            ),
            (LIVE_VISIBLE_WORK_RUN_SLICE_MS, true)
        );
    }

    #[test]
    fn host_idle_message_poll_slice_requires_no_saved_context() {
        assert!(should_use_host_idle_message_poll_slice(
            DesktopMode::Host,
            false,
            true,
        ));
        assert!(!should_use_host_idle_message_poll_slice(
            DesktopMode::Host,
            true,
            true,
        ));
        assert!(!should_use_host_idle_message_poll_slice(
            DesktopMode::Virtual,
            false,
            true,
        ));
        assert!(!should_use_host_idle_message_poll_slice(
            DesktopMode::Host,
            false,
            false,
        ));
    }

    #[test]
    fn remote_idle_sleep_only_without_saved_context() {
        assert!(should_sleep_remote_idle(true, false));
        assert!(!should_sleep_remote_idle(true, true));
        assert!(!should_sleep_remote_idle(false, false));
    }

    #[test]
    fn idle_saved_remote_hidden_active_rotates_to_runnable_parked_process() {
        assert!(should_rotate_idle_saved_remote_active(
            true, true, false, false, false, true
        ));
        assert!(should_rotate_idle_saved_remote_active(
            true, true, true, false, false, true
        ));
        assert!(!should_rotate_idle_saved_remote_active(
            false, true, false, false, false, true
        ));
        assert!(!should_rotate_idle_saved_remote_active(
            true, false, false, false, false, true
        ));
        assert!(!should_rotate_idle_saved_remote_active(
            true, true, false, true, false, true
        ));
        assert!(!should_rotate_idle_saved_remote_active(
            true, true, false, false, true, true
        ));
        assert!(!should_rotate_idle_saved_remote_active(
            true, true, false, false, false, false
        ));
    }

    #[test]
    fn live_pump_priority_parked_rotation_waits_for_visible_queue_to_drain() {
        assert!(should_rotate_live_pump_priority_parked_process(false, true));
        assert!(!should_rotate_live_pump_priority_parked_process(true, true));
        assert!(!should_rotate_live_pump_priority_parked_process(
            false, false
        ));
    }

    #[test]
    fn live_pump_uses_bounded_instruction_slice_for_host_service() {
        assert_eq!(
            effective_instruction_limit(0, true, false),
            LIVE_PUMP_RUN_SLICE_INSTRUCTIONS
        );
        assert_eq!(
            effective_instruction_limit(5_000_000, true, false),
            5_000_000
        );
        assert_eq!(
            effective_instruction_limit(0, false, true),
            FAST_START_RUN_SLICE_INSTRUCTIONS
        );
        assert_eq!(effective_instruction_limit(1234, false, true), 1234);
    }

    #[test]
    fn live_service_boundary_rotates_only_for_ready_work() {
        assert!(!should_rotate_parked_process(
            true, false, false, false, false, false, true, true, false
        ));
        assert!(should_rotate_parked_process(
            true, false, false, true, false, true, false, false, true
        ));
        assert!(should_rotate_parked_process(
            true, true, false, false, false, true, false, false, true
        ));
        assert!(should_rotate_parked_process(
            true, true, false, false, false, true, false, true, true
        ));
        assert!(should_rotate_parked_process(
            true, true, false, false, false, true, false, true, false
        ));
        assert!(!should_rotate_parked_process(
            true, false, false, false, false, true, false, false, true
        ));
        assert!(should_rotate_parked_process(
            true, false, false, false, true, false, true, true, true
        ));
        assert!(should_rotate_parked_process(
            true, true, true, false, false, false, true, true, true
        ));
        assert!(should_rotate_parked_process(
            true, true, false, false, false, false, true, true, true
        ));
        assert!(!should_rotate_parked_process(
            true, false, false, false, false, false, true, true, true
        ));
        assert!(!should_rotate_parked_process(
            true, false, false, false, false, false, true, false, false
        ));
        assert!(!should_rotate_parked_process(
            false, false, false, true, true, true, true, true, false
        ));
    }

    #[test]
    fn live_message_waiter_runs_active_receiver_work_instead_of_idling() {
        assert!(should_run_active_receiver_work_for_live_message_waiter(
            true, true, true
        ));
        assert!(!should_run_active_receiver_work_for_live_message_waiter(
            true, true, false
        ));
        assert!(!should_run_active_receiver_work_for_live_message_waiter(
            true, false, true
        ));
        assert!(!should_run_active_receiver_work_for_live_message_waiter(
            false, true, true
        ));
    }

    #[test]
    fn idle_remote_receiver_rotation_preserves_active_ui_work() {
        assert!(should_rotate_idle_remote_receiver_parked_process(
            false, false
        ));
        assert!(!should_rotate_idle_remote_receiver_parked_process(
            true, false
        ));
        assert!(!should_rotate_idle_remote_receiver_parked_process(
            false, true
        ));
        assert!(!should_rotate_idle_remote_receiver_parked_process(
            true, true
        ));
    }

    #[test]
    fn idle_runnable_parked_rotation_preserves_active_visible_ui_work() {
        assert!(should_rotate_idle_runnable_parked_process(false, true));
        assert!(!should_rotate_idle_runnable_parked_process(true, true));
        assert!(!should_rotate_idle_runnable_parked_process(false, false));
    }

    #[test]
    fn idle_nonrunning_runnable_parked_rotation_preserves_queued_visible_ui_work() {
        assert!(should_rotate_idle_nonrunning_runnable_parked_process(
            false, false, true
        ));
        assert!(!should_rotate_idle_nonrunning_runnable_parked_process(
            true, false, true
        ));
        assert!(!should_rotate_idle_nonrunning_runnable_parked_process(
            false, true, true
        ));
        assert!(!should_rotate_idle_nonrunning_runnable_parked_process(
            false, false, false
        ));
    }

    #[test]
    fn remote_input_receiver_rotation_preserves_active_visible_ui_work() {
        assert!(should_rotate_remote_input_receiver_parked_process(false));
        assert!(!should_rotate_remote_input_receiver_parked_process(true));
    }

    #[test]
    fn remote_active_exit_keeps_visible_ui_serviceable() {
        assert!(should_keep_remote_active_exit_alive(true, true));
        assert!(!should_keep_remote_active_exit_alive(false, true));
        assert!(!should_keep_remote_active_exit_alive(true, false));
    }

    #[test]
    fn idle_poll_detects_saved_get_message_waiter() {
        let mut snapshot = UnicornDebugSnapshot::default();
        assert!(!snapshot_has_blocked_get_message(&snapshot));
        assert!(!snapshot_has_non_message_blocked_wait(&snapshot));

        snapshot.active_blocked_waits.push(
            wince_emulation_v3::emulator::cpu::UnicornBlockedWaitSnapshot {
                id: 1,
                thread_id: 1,
                thread_handle: 0x120,
                kind: "get_message".to_owned(),
                wait_started_ms: 10,
                timeout_ms: u32::MAX,
                handles: Vec::new(),
            },
        );

        assert!(snapshot_has_blocked_get_message(&snapshot));
        assert!(!snapshot_has_non_message_blocked_wait(&snapshot));
        assert!(snapshot_is_idle_message_wait_only(&snapshot));

        snapshot.trap_address = Some(0x7fff_2a00);
        snapshot.trap_ordinal = Some(wince_emulation_v3::ce::coredll_ordinals::ORD_MESSAGE_BOX_W);
        assert!(!snapshot_has_blocked_get_message(&snapshot));
        assert!(!snapshot_is_idle_message_wait_only(&snapshot));
    }

    #[test]
    fn idle_poll_requires_a_live_get_message_waiter() {
        let mut kernel = CeKernel::boot(
            RuntimeConfig::load_default().expect("runtime config loads for idle waiter test"),
        );
        let wait_id = kernel.register_blocked_waiter(
            7,
            0x120,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            10,
            u32::MAX,
        );
        let mut snapshot = UnicornDebugSnapshot::default();
        snapshot.active_blocked_waits.push(
            wince_emulation_v3::emulator::cpu::UnicornBlockedWaitSnapshot {
                id: wait_id,
                thread_id: 7,
                thread_handle: 0x120,
                kind: "get_message".to_owned(),
                wait_started_ms: 10,
                timeout_ms: u32::MAX,
                handles: Vec::new(),
            },
        );
        assert!(snapshot_is_idle_message_wait_only(&snapshot));
        assert!(snapshot_has_live_saved_get_message_waiter(
            &snapshot, &kernel
        ));
        assert!(snapshot_has_live_blocked_message_waiter(&snapshot, &kernel));
        snapshot.trap_address = Some(0x7fff_2a00);
        assert!(snapshot_has_live_saved_get_message_waiter(
            &snapshot, &kernel
        ));
        assert!(!snapshot_has_live_blocked_message_waiter(
            &snapshot, &kernel
        ));
        snapshot.trap_address = None;
        assert!(kernel.remove_blocked_waiter(wait_id).is_some());
        assert!(snapshot_is_idle_message_wait_only(&snapshot));
        assert!(!snapshot_has_live_saved_get_message_waiter(
            &snapshot, &kernel
        ));
        assert!(!snapshot_has_live_blocked_message_waiter(
            &snapshot, &kernel
        ));
    }

    #[test]
    fn idle_poll_detects_mixed_message_and_sleep_waiters() {
        let mut snapshot = UnicornDebugSnapshot::default();

        snapshot.active_blocked_waits.push(
            wince_emulation_v3::emulator::cpu::UnicornBlockedWaitSnapshot {
                id: 1,
                thread_id: 3,
                thread_handle: 0x303,
                kind: "get_message".to_owned(),
                wait_started_ms: 10,
                timeout_ms: u32::MAX,
                handles: Vec::new(),
            },
        );
        snapshot.active_blocked_waits.push(
            wince_emulation_v3::emulator::cpu::UnicornBlockedWaitSnapshot {
                id: 2,
                thread_id: 1,
                thread_handle: u32::MAX,
                kind: "sleep".to_owned(),
                wait_started_ms: 20,
                timeout_ms: 501,
                handles: Vec::new(),
            },
        );

        assert!(snapshot_has_blocked_get_message(&snapshot));
        assert!(snapshot_has_non_message_blocked_wait(&snapshot));
        assert!(!snapshot_is_idle_message_wait_only(&snapshot));
    }

    #[test]
    fn saved_remote_input_target_uses_saved_get_message_waiter() {
        let mut kernel = CeKernel::boot(
            RuntimeConfig::load_default().expect("runtime config loads for saved waiter test"),
        );
        let hwnd = kernel.create_window_ex_w(
            7,
            "REMOTE_INPUT_TARGET",
            "",
            None,
            0,
            wince_emulation_v3::ce::gwe::WS_VISIBLE,
            0,
        );
        kernel.register_blocked_waiter(
            7,
            0x120,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(hwnd),
                min_msg: 0,
                max_msg: 0,
            },
            10,
            u32::MAX,
        );

        assert_eq!(
            saved_message_remote_input_target(&kernel),
            Some(BlockedRemoteInputTarget {
                thread_id: 7,
                hwnd: Some(hwnd),
            })
        );
    }

    #[test]
    fn completed_cpu_idle_polls_host_input_for_parked_message_waiter() {
        let mut kernel = CeKernel::boot(
            RuntimeConfig::load_default().expect("runtime config loads for parked input test"),
        );
        let cpu = UnicornMips::new().expect("CPU initializes for parked input test");
        let hwnd = kernel.create_window_ex_w(
            7,
            "PARKED_HOST_INPUT_TARGET",
            "",
            None,
            0,
            wince_emulation_v3::ce::gwe::WS_VISIBLE,
            0,
        );
        let wait_id = kernel.register_blocked_waiter(
            7,
            0x120,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(hwnd),
                min_msg: 0,
                max_msg: 0,
            },
            kernel.timers.tick_count(),
            u32::MAX,
        );
        let mut virtual_desktop =
            VirtualDesktop::default_primary().expect("virtual desktop initializes");
        virtual_desktop.input_mut().push_touch_down(12, 34);
        let mut desktop = DesktopRuntime::Virtual(virtual_desktop);
        kernel.remote.set_framebuffer_size(
            desktop.framebuffer().width(),
            desktop.framebuffer().height(),
        );

        assert_eq!(
            service_completed_active_desktop(&cpu, &mut desktop, &mut kernel, DesktopMode::Host)
                .expect("completed CPU idle service succeeds"),
            1
        );
        assert!(kernel.recent_message_ops().iter().any(|record| {
            record.op == "remote_touch_target"
                && record.thread_id == 7
                && record.hwnd == Some(hwnd)
                && record.msg == Some(wince_emulation_v3::ce::gwe::WM_LBUTTONDOWN)
        }));
        assert_eq!(
            kernel.select_ready_blocked_waiter(1, 0, |blocked, kernel| {
                matches!(blocked.kind, SchedulerBlockedWaitKind::GetMessage { .. })
                    && kernel.gwe.has_queue_input(blocked.thread_id, u32::MAX)
            }),
            Some(wait_id)
        );
        if let DesktopRuntime::Virtual(virtual_desktop) = &desktop {
            assert_eq!(virtual_desktop.presenter().presentation_count(), 1);
        } else {
            panic!("test desktop must remain virtual");
        }
    }

    #[test]
    fn remote_input_target_ignores_any_get_message_without_visible_window() {
        let mut kernel = CeKernel::boot(
            RuntimeConfig::load_default().expect("runtime config loads for hidden waiter test"),
        );
        let mut cpu = UnicornMips::new().expect("unicorn initializes for hidden waiter test");
        cpu.set_initial_thread_id(1);
        let _hidden = kernel.create_window_ex_w(6, "HIDDEN_REMOTE_HELPER", "", None, 0, 0, 0);
        kernel.register_blocked_waiter(
            6,
            0x606,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: None,
                min_msg: 0,
                max_msg: 0,
            },
            10,
            u32::MAX,
        );

        assert_eq!(blocked_remote_input_target(&cpu, &kernel), None);
    }

    #[test]
    fn remote_input_target_preserves_active_visible_receiver_work() {
        let mut kernel = CeKernel::boot(
            RuntimeConfig::load_default().expect("runtime config loads for active receiver test"),
        );
        let mut cpu = UnicornMips::new().expect("unicorn initializes for active receiver test");
        cpu.set_initial_thread_id(1);
        let active = kernel.create_window_ex_w(
            1,
            "ACTIVE_REMOTE_INPUT_TARGET",
            "",
            None,
            0,
            wince_emulation_v3::ce::gwe::WS_VISIBLE,
            0,
        );
        assert!(kernel.post_message_w(active, wince_emulation_v3::ce::gwe::WM_LBUTTONDOWN, 1, 0));

        let parked = kernel.create_window_ex_w(
            6,
            "PARKED_REMOTE_INPUT_TARGET",
            "",
            None,
            0,
            wince_emulation_v3::ce::gwe::WS_VISIBLE,
            0,
        );
        kernel.register_blocked_waiter(
            6,
            0x606,
            Vec::new(),
            SchedulerBlockedWaitKind::GetMessage {
                hwnd: Some(parked),
                min_msg: 0,
                max_msg: 0,
            },
            10,
            u32::MAX,
        );

        assert_eq!(blocked_remote_input_target(&cpu, &kernel), None);
    }

    #[test]
    fn companion_command_uses_shared_config_without_remote_or_nested_companions() {
        let args = Args {
            registry: PathBuf::from(DEFAULT_REGISTRY_PATH),
            devices: PathBuf::from(DEFAULT_DEVICES_PATH),
            image: Some(PathBuf::from(r"D:\INAVI_Emulator\INAVI\INavi\iNavi.exe")),
            companion_images: vec![PathBuf::from(r"D:\INAVI_Emulator\INAVI\TBT\MultiTBT.exe")],
            dll_search_dirs: vec![PathBuf::from(r"D:\INAVI_Emulator\DUMPPLZ\Windows")],
            mount_config: Some(PathBuf::from(DEFAULT_MOUNT_CONFIG_PATH)),
            framebuffer_dump: None,
            tracefiles: Vec::new(),
            desktop: DesktopMode::Host,
            cpu_instruction_limit: 0,
            cpu_wall_clock_limit_ms: 240_000,
            cpu_stop_pc: None,
            startup_taps: Vec::new(),
            remote_server: Some(RemoteServerConfig::default()),
            run_cpu: true,
            monitor: false,
            verbose: false,
        };

        let command_args = companion_command_args(
            &args,
            Path::new(r"D:\INAVI_Emulator\INAVI\TBT\MultiTBT.exe"),
        )
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

        assert_eq!(
            command_args,
            vec![
                "--registry",
                DEFAULT_REGISTRY_PATH,
                "--devices",
                DEFAULT_DEVICES_PATH,
                "--mount-config",
                DEFAULT_MOUNT_CONFIG_PATH,
                "--image",
                r"D:\INAVI_Emulator\INAVI\TBT\MultiTBT.exe",
                "--dll-search-dir",
                r"D:\INAVI_Emulator\DUMPPLZ\Windows",
                "--desktop",
                "virtual",
                "--cpu-instruction-limit",
                "250000000",
                "--run-cpu",
            ]
        );
        assert!(!command_args.iter().any(|arg| arg == "--remote-server"));
        assert!(!command_args.iter().any(|arg| arg == "--companion-image"));
        assert!(!command_args.iter().any(|arg| arg == "--companion-target"));
    }

    #[test]
    fn image_directory_dll_paths_lists_sibling_dlls_case_insensitively() {
        let root =
            std::env::temp_dir().join(format!("wince-image-dir-dlls-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("app.exe"), b"").unwrap();
        fs::write(root.join("AuthLibrary.dll"), b"").unwrap();
        fs::write(root.join("TpSysAuth.DLL"), b"").unwrap();
        fs::write(root.join("notes.txt"), b"").unwrap();

        let paths = image_directory_dll_paths(&root.join("app.exe")).unwrap();
        let names = paths
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["AuthLibrary.dll", "TpSysAuth.DLL"]);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn maps_image_under_loaded_mount_config_to_ce_mount_path() {
        let config = RuntimeConfig::load_default_with_mounts().unwrap();
        assert_eq!(config.storage.mounts.len(), 3);
        assert_eq!(
            config.storage.mounts[0].host_root.as_deref(),
            Some(Path::new(r"D:\INAVI_Emulator\INAVI"))
        );
        let kernel = CeKernel::boot(config);
        let path = ce_module_path_for_image(&kernel, r"D:\INAVI_Emulator\INAVI\INavi\iNavi.exe");

        assert_eq!(path, r"\SDMMC Disk\INavi\iNavi.exe");
    }

    #[test]
    fn leaves_unmounted_image_path_as_ce_style_host_path() {
        let config = RuntimeConfig::load_default().unwrap();
        let kernel = CeKernel::boot(config);
        let path = ce_module_path_for_image(&kernel, r"D:\Other\INavi.exe");

        assert_eq!(path, r"D:\Other\INavi.exe");
    }

    #[test]
    fn resolves_dll_path_with_case_variants_and_optional_extension() {
        let root = std::env::temp_dir().join(format!("wince-dll-resolve-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let dll = root.join("mfcce400.dll");
        std::fs::write(&dll, []).unwrap();

        let mixed_case =
            resolve_dll_path("MFCcE400.DLL", None, std::slice::from_ref(&root)).unwrap();
        assert!(mixed_case.is_file());
        assert_eq!(
            mixed_case
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_ascii_lowercase(),
            "mfcce400.dll"
        );
        assert_eq!(
            resolve_dll_path("mfcce400", None, std::slice::from_ref(&root)).unwrap(),
            dll
        );
        assert!(resolve_dll_path("missing.dll", None, std::slice::from_ref(&root)).is_none());

        let _ = std::fs::remove_file(dll);
        let _ = std::fs::remove_dir(root);
    }

    #[test]
    fn commctrl_imports_are_loaded_from_search_dirs_not_emulator_provided() {
        assert!(!emulator_provided_import_module("commctrl"));
        assert!(!emulator_provided_import_module("commctrlce"));
        assert!(emulator_provided_import_module("coredll"));
        assert!(emulator_provided_import_module("winsock"));
        assert!(emulator_provided_import_module("ole32"));
    }

    #[test]
    fn virtual_desktop_uses_headless_audio_sink() {
        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        let status = attach_audio_for_desktop(&mut kernel, DesktopMode::Virtual);

        #[cfg(debug_assertions)]
        {
            assert_eq!(status, "virtual desktop logging sink registered");
            assert_eq!(kernel.audio.sink_names(), vec!["virtual-log".to_owned()]);
        }
        #[cfg(not(debug_assertions))]
        {
            assert_eq!(status, "virtual desktop null sink registered");
            assert_eq!(kernel.audio.sink_names(), vec!["virtual-null".to_owned()]);
        }
    }
}
