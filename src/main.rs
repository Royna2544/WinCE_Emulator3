use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as FmtWrite,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use wince_emulation_v3::{
    Result,
    ce::{
        audio::{HostAudioSink, WaveFormat},
        desktop::{VirtualDesktop, VirtualInputEvent},
        framebuffer::{Framebuffer, FramebufferInfo, FramebufferRect, VirtualFramebuffer},
        gwe::WM_TIMER,
        kernel::CeKernel,
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE},
    },
    config::RuntimeConfig,
    emulator::{
        memory::MemoryPerms,
        unicorn::{UnicornDebugSnapshot, UnicornMips, UnicornRunLimits, UnicornWindowSnapshot},
    },
    pe::PeImage,
};

#[derive(Debug, Clone)]
struct Args {
    registry: PathBuf,
    devices: PathBuf,
    image: Option<PathBuf>,
    dll_search_dirs: Vec<PathBuf>,
    mount_config: Option<PathBuf>,
    framebuffer_dump: Option<PathBuf>,
    tracefiles: Vec<(String, PathBuf)>,
    desktop: DesktopMode,
    cpu_instruction_limit: usize,
    cpu_wall_clock_limit_ms: u64,
    cpu_stop_pc: Option<u32>,
    startup_taps: Vec<(i32, i32)>,
    run_cpu: bool,
    monitor: bool,
    verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopMode {
    Virtual,
    Host,
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
    let host_audio_status = attach_audio_for_desktop(&mut kernel, args.desktop);
    let mut desktop = create_desktop(args.desktop, args.image.as_deref())?;
    kernel.remote.set_framebuffer_size(
        desktop.framebuffer().width(),
        desktop.framebuffer().height(),
    );

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

    if args.run_cpu {
        run_cpu_loop(&mut cpu, &mut kernel, &mut desktop, &args)?;
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
) -> Result<()> {
    enqueue_startup_taps(kernel, &args.startup_taps)?;
    let mut reported_blocked_message_wait = false;
    loop {
        if enqueue_desktop_input_for_current_wait(cpu, desktop, kernel, args.desktop)? != 0 {
            reported_blocked_message_wait = false;
        }
        if let Err(err) = desktop.run_cpu_until(
            cpu,
            kernel,
            UnicornRunLimits {
                instruction_limit: args.cpu_instruction_limit,
                wall_clock_limit_ms: args.cpu_wall_clock_limit_ms,
                stop_pc: args.cpu_stop_pc,
            },
        ) {
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                eprintln!("  Unicorn debug: {}", snapshot.summary());
            }
            write_requested_tracefiles(cpu, &args.tracefiles)?;
            if let Some(path) = args.framebuffer_dump.as_ref() {
                desktop.framebuffer().write_ppm(path)?;
                eprintln!("  framebuffer dump: {}", path.display());
            }
            return Err(err);
        }
        if enqueue_desktop_input_for_current_wait(cpu, desktop, kernel, args.desktop)? != 0 {
            reported_blocked_message_wait = false;
        }
        desktop.present()?;
        if let Some(snapshot) = cpu.last_debug_snapshot() {
            if args.desktop == DesktopMode::Host && snapshot.blocked_get_message.is_some() {
                if !reported_blocked_message_wait {
                    println!("  Unicorn stopped: {}", snapshot.summary());
                    if let Some(path) = args.framebuffer_dump.as_ref() {
                        desktop.framebuffer().write_ppm(path)?;
                        println!("  framebuffer dump: {}", path.display());
                    }
                    reported_blocked_message_wait = true;
                }
                std::thread::sleep(Duration::from_millis(16));
                continue;
            }
            println!("  Unicorn stopped: {}", snapshot.summary());
        }
        break;
    }
    Ok(())
}

fn enqueue_desktop_input_for_current_wait(
    cpu: &UnicornMips,
    desktop: &mut DesktopRuntime,
    kernel: &mut CeKernel,
    desktop_mode: DesktopMode,
) -> Result<usize> {
    let blocked_get_message = if desktop_mode == DesktopMode::Host {
        cpu.last_debug_snapshot()
            .and_then(|snapshot| snapshot.blocked_get_message.clone())
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

fn write_requested_tracefiles(cpu: &UnicornMips, tracefiles: &[(String, PathBuf)]) -> Result<()> {
    if tracefiles.is_empty() {
        return Ok(());
    }
    let Some(snapshot) = cpu.last_debug_snapshot() else {
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
                monitor_run_and_report(
                    cpu,
                    kernel,
                    desktop,
                    UnicornRunLimits {
                        instruction_limit,
                        wall_clock_limit_ms,
                        stop_pc: None,
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
                monitor_run_and_report(
                    cpu,
                    kernel,
                    desktop,
                    UnicornRunLimits {
                        instruction_limit,
                        wall_clock_limit_ms,
                        stop_pc: Some(stop_pc),
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
                let Some(snapshot) = cpu.last_debug_snapshot() else {
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
                checkpoints.push(MonitorCheckpoint {
                    name: name.clone(),
                    cpu: cpu.clone(),
                    kernel: kernel.clone(),
                    framebuffer: desktop.framebuffer().clone(),
                });
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
                *cpu = checkpoint.cpu.clone();
                *kernel = checkpoint.kernel.clone();
                *desktop.framebuffer_mut() = checkpoint.framebuffer.clone();
                desktop.present()?;
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
) -> Result<()> {
    let input_before = enqueue_desktop_input(desktop, kernel)?;
    if input_before != 0 {
        println!("  drained {input_before} host input event(s)");
    }
    if let Err(err) = desktop.run_cpu_until(cpu, kernel, limits) {
        if let Some(snapshot) = cpu.last_debug_snapshot() {
            eprintln!("  Unicorn debug: {}", snapshot.summary());
        }
        if let Some(path) = framebuffer_dump {
            desktop.framebuffer().write_ppm(path)?;
            eprintln!("  framebuffer dump: {}", path.display());
        }
        return Err(err);
    }
    let input_after = enqueue_desktop_input(desktop, kernel)?;
    if input_after != 0 {
        println!("  drained {input_after} host input event(s)");
    }
    desktop.present()?;
    if let Some(snapshot) = cpu.last_debug_snapshot() {
        println!("  Unicorn stopped: {}", snapshot.summary());
    }
    if let Some(path) = framebuffer_dump {
        desktop.framebuffer().write_ppm(path)?;
        println!("  framebuffer dump: {}", path.display());
    }
    Ok(())
}

fn monitor_run_and_report(
    cpu: &mut UnicornMips,
    kernel: &mut CeKernel,
    desktop: &mut DesktopRuntime,
    limits: UnicornRunLimits,
    framebuffer_dump: Option<&Path>,
) {
    if let Err(err) = monitor_run_once(cpu, kernel, desktop, limits, framebuffer_dump) {
        if cpu.last_debug_snapshot().is_some() {
            eprintln!("  stopped; use regs or trace for detail");
        } else {
            eprintln!("  stopped: {err}");
        }
    }
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
        "calls" => push_monitor_records(&mut out, "calls", &snapshot.last_calls),
        "code" => push_monitor_records(&mut out, "code", &snapshot.last_code),
        "blocks" => push_monitor_records(&mut out, "blocks", &snapshot.last_blocks),
        "messages" | "msgs" => push_monitor_records(&mut out, "messages", &snapshot.last_messages),
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
                "  unknown trace kind `{other}`; use all/imports/milestones/counts/calls/code/blocks/messages/window-imports/presentation/windows/wndproc/render/resource/files/files-full/processes/events"
            );
        }
    }
    out
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
            let _ = writeln!(
                out,
                "    {} handle={} result={}{}{} {}",
                record.op, handle, result, requested, position, path
            );
            if !detail.is_empty() {
                let _ = writeln!(out, "      {detail}");
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
        }
    }

    fn framebuffer_mut(&mut self) -> &mut VirtualFramebuffer {
        match self {
            Self::Virtual(desktop) => desktop.framebuffer_mut(),
            #[cfg(all(windows, feature = "win32-desktop"))]
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
        }
        Ok(())
    }

    fn run_cpu_until(
        &mut self,
        cpu: &mut UnicornMips,
        kernel: &mut CeKernel,
        limits: UnicornRunLimits,
    ) -> Result<()> {
        match self {
            Self::Virtual(desktop) => cpu.run_until_import_trap_with_framebuffer_limits(
                kernel,
                desktop.framebuffer_mut(),
                limits,
            ),
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(desktop) => {
                let (framebuffer, presenter) = desktop.framebuffer_and_presenter_mut();
                let mut live_framebuffer = HostLiveFramebuffer::new(framebuffer, presenter);
                cpu.run_until_import_trap_with_framebuffer_limits(
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
        }
    }

    fn describe(&self) -> &'static str {
        match self {
            Self::Virtual(_) => "virtual/null presenter",
            #[cfg(all(windows, feature = "win32-desktop"))]
            Self::Host(_) => "win32 host presenter",
        }
    }
}

#[cfg(all(windows, feature = "win32-desktop"))]
struct HostLiveFramebuffer<'a> {
    framebuffer: &'a mut VirtualFramebuffer,
    presenter: &'a mut wince_emulation_v3::ce::win32_desktop::Win32Presenter,
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
    ) -> Self {
        Self {
            framebuffer,
            presenter,
            last_blit: Instant::now()
                .checked_sub(Duration::from_millis(16))
                .unwrap_or_else(Instant::now),
            blit_interval: Duration::from_millis(16),
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

#[cfg(all(windows, feature = "win32-desktop"))]
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

#[cfg(not(all(windows, feature = "win32-desktop")))]
fn create_host_desktop(_image_path: Option<&Path>) -> Result<DesktopRuntime> {
    Err(wince_emulation_v3::Error::InvalidArgument(
        "--desktop host requires Windows and the `win32-desktop` feature".to_owned(),
    ))
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
                kernel.remote.enqueue_touch("down", x, y).map_err(|err| {
                    wince_emulation_v3::Error::Backend(format!("host touch down: {err}"))
                })?;
                queued += 1;
            }
            VirtualInputEvent::TouchMove { x, y } => {
                kernel.remote.enqueue_touch("move", x, y).map_err(|err| {
                    wince_emulation_v3::Error::Backend(format!("host touch move: {err}"))
                })?;
                queued += 1;
            }
            VirtualInputEvent::TouchUp { x, y } => {
                kernel.remote.enqueue_touch("up", x, y).map_err(|err| {
                    wince_emulation_v3::Error::Backend(format!("host touch up: {err}"))
                })?;
                queued += 1;
            }
        }
    }
    Ok(queued)
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
    #[cfg(windows)]
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
    #[cfg(not(windows))]
    {
        let _ = kernel;
        "not registered on non-Windows host".to_owned()
    }
}

impl Args {
    fn parse() -> Result<Self> {
        let mut registry = PathBuf::from("regs.json");
        let mut devices = PathBuf::from("serial_devices.json");
        let mut image = None;
        let mut dll_search_dirs = Vec::new();
        let mut mount_config = None;
        let mut framebuffer_dump = None;
        let mut tracefiles = Vec::new();
        let mut desktop = DesktopMode::Virtual;
        let mut cpu_instruction_limit = 0;
        let mut cpu_wall_clock_limit_ms = 0;
        let mut cpu_stop_pc = None;
        let mut startup_taps = Vec::new();
        let mut run_cpu = false;
        let mut monitor = false;
        let mut verbose = false;

        let mut args = std::env::args().skip(1);
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
            dll_search_dirs,
            mount_config,
            framebuffer_dump,
            tracefiles,
            desktop,
            cpu_instruction_limit,
            cpu_wall_clock_limit_ms,
            cpu_stop_pc,
            startup_taps,
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
        "Usage: wince_emulation_v3 [--registry regs.json] [--devices serial_devices.json] [--mount-config mounts.toml] [--image INavi.exe] [--dll-search-dir DIR]... [--desktop virtual|host] [--framebuffer-dump OUT.ppm] [--tracefile KIND OUT.txt]... [--cpu-instruction-limit N] [--cpu-wall-clock-limit-ms N] [--cpu-stop-pc ADDR] [--tap X,Y]... [--run-cpu] [--monitor] [--verbose]"
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
        let path = resolve_dll_path(&descriptor.module_name, search_dirs).ok_or_else(|| {
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
    if let Some(path) = resolve_dll_path(module_name, search_dirs) {
        loaded.push(PeImage::inspect(path)?);
    }
    Ok(())
}

fn register_loaded_modules(kernel: &mut CeKernel, cpu: &UnicornMips) {
    for module in cpu.loaded_modules() {
        kernel.register_loaded_module(
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
        );
    }
}

fn emulator_provided_import_module(normalized_module_name: &str) -> bool {
    matches!(
        normalized_module_name,
        "coredll" | "winsock" | "ws2" | "ws2_32" | "ole32" | "oleaut32" | "olece"
    )
}

fn resolve_dll_path(module_name: &str, search_dirs: &[PathBuf]) -> Option<PathBuf> {
    let candidates = [
        module_name.to_owned(),
        module_name.to_ascii_lowercase(),
        module_name.to_ascii_uppercase(),
    ];
    for dir in search_dirs {
        for candidate in &candidates {
            let path = dir.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
        if Path::new(module_name).extension().is_none() {
            let path = dir.join(format!("{module_name}.dll"));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn normalize_module_name(module_name: &str) -> String {
    module_name
        .trim()
        .trim_end_matches('\0')
        .trim_end_matches(".dll")
        .trim_end_matches(".DLL")
        .to_ascii_lowercase()
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

    #[test]
    fn maps_image_under_configured_mount_to_ce_mount_path() {
        let mut config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
        config
            .storage
            .mounts
            .push(wince_emulation_v3::config::MountConfig {
                name: Some("sdmmc".to_owned()),
                guest_root: "\\SDMMC Disk".to_owned(),
                host_root: Some(PathBuf::from(r"D:\INAVI_Emulator\INAVI")),
                total_mbytes: 8192,
                free_mbytes: 4096,
                writable: true,
                removable: true,
                system: false,
                hidden: false,
            });
        let kernel = CeKernel::boot(config);
        let path = ce_module_path_for_image(&kernel, r"D:\INAVI_Emulator\INAVI\INavi\INavi.exe");

        assert_eq!(path, r"\SDMMC Disk\INavi\INavi.exe");
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
        let config = RuntimeConfig::load_with_mounts(
            "regs.json",
            "serial_devices.json",
            Some("mounts.toml"),
        )
        .unwrap();
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
        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
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

        let mixed_case = resolve_dll_path("MFCcE400.DLL", std::slice::from_ref(&root)).unwrap();
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
            resolve_dll_path("mfcce400", std::slice::from_ref(&root)).unwrap(),
            dll
        );
        assert!(resolve_dll_path("missing.dll", &[root.clone()]).is_none());

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
        let config = RuntimeConfig::load("regs.json", "serial_devices.json").unwrap();
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
