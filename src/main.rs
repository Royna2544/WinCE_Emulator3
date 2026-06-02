use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use wince_emulation_v3::{
    Result,
    ce::{
        audio::WaveFormat,
        framebuffer::{Framebuffer, VirtualFramebuffer},
        gwe::WM_TIMER,
        kernel::CeKernel,
        registry::{ERROR_SUCCESS, HKEY_LOCAL_MACHINE},
    },
    config::RuntimeConfig,
    emulator::{memory::MemoryPerms, unicorn::UnicornMips},
    pe::PeImage,
};

#[derive(Debug, Clone)]
struct Args {
    registry: PathBuf,
    devices: PathBuf,
    image: Option<PathBuf>,
    dll_search_dirs: Vec<PathBuf>,
    sdmmc_root: Option<PathBuf>,
    framebuffer_dump: Option<PathBuf>,
    cpu_instruction_limit: usize,
    run_cpu: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse()?;
    let config = RuntimeConfig::load(&args.registry, &args.devices)?;
    let mut kernel = CeKernel::boot(config);
    if let Some(sdmmc_root) = args.sdmmc_root.as_ref() {
        kernel.mount_guest_root("\\SDMMC Disk", sdmmc_root);
    }
    let mut framebuffer = VirtualFramebuffer::default_primary()?;
    kernel
        .remote
        .set_framebuffer_size(framebuffer.width(), framebuffer.height());

    let mut cpu = UnicornMips::new()?;
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
        let timer_id = kernel.timers.set_timer(Some(hwnd), None, 1000, WM_TIMER);
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
        framebuffer.width(),
        framebuffer.height(),
        framebuffer.pixel_format(),
        framebuffer.stride(),
        framebuffer.pixels().len()
    );

    let pe_image = if let Some(image_path) = args.image.as_ref() {
        let image = PeImage::inspect(image_path)?;
        kernel.set_process_module_base(image.image_base());
        kernel.set_process_module_path(ce_module_path_for_image(
            &image.path,
            args.sdmmc_root.as_deref(),
        ));
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
        Some(image)
    } else {
        None
    };

    if let Some(image) = pe_image.as_ref() {
        let dll_images = load_import_dlls(image, &args.dll_search_dirs)?;
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
        cpu.load_pe_image_with_dlls(image, &dll_images)?;
        println!("  import traps: {} slots patched", cpu.import_traps().len());
    }

    if args.run_cpu {
        if let Err(err) = cpu.run_until_import_trap_with_framebuffer_limit(
            &mut kernel,
            &mut framebuffer,
            args.cpu_instruction_limit,
        ) {
            if let Some(snapshot) = cpu.last_debug_snapshot() {
                eprintln!("  Unicorn debug: {snapshot}");
            }
            if let Some(path) = args.framebuffer_dump.as_ref() {
                framebuffer.write_ppm(path)?;
                eprintln!("  framebuffer dump: {}", path.display());
            }
            return Err(err);
        }
        if let Some(snapshot) = cpu.last_debug_snapshot() {
            println!("  Unicorn stopped: {snapshot}");
        }
    }
    if let Some(path) = args.framebuffer_dump.as_ref() {
        framebuffer.write_ppm(path)?;
        println!("  framebuffer dump: {}", path.display());
    }

    Ok(())
}

impl Args {
    fn parse() -> Result<Self> {
        let mut registry = PathBuf::from("regs.json");
        let mut devices = PathBuf::from("serial_devices.json");
        let mut image = None;
        let mut dll_search_dirs = Vec::new();
        let mut sdmmc_root = None;
        let mut framebuffer_dump = None;
        let mut cpu_instruction_limit = 0;
        let mut run_cpu = false;

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
                "--sdmmc-root" => {
                    sdmmc_root = Some(next_path(&mut args, "--sdmmc-root")?);
                }
                "--framebuffer-dump" => {
                    framebuffer_dump = Some(next_path(&mut args, "--framebuffer-dump")?);
                }
                "--cpu-instruction-limit" => {
                    cpu_instruction_limit = next_usize(&mut args, "--cpu-instruction-limit")?;
                }
                "--run-cpu" => {
                    run_cpu = true;
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
            sdmmc_root,
            framebuffer_dump,
            cpu_instruction_limit,
            run_cpu,
        })
    }
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a path")))
}

fn next_usize(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<usize> {
    let value = args.next().ok_or_else(|| {
        wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a value"))
    })?;
    value
        .parse()
        .map_err(|err| wince_emulation_v3::Error::InvalidArgument(format!("{flag}: {err}")))
}

fn print_help() {
    println!(
        "Usage: wince_emulation_v3 [--registry regs.json] [--devices serial_devices.json] [--image INavi.exe] [--dll-search-dir DIR]... [--sdmmc-root DIR] [--framebuffer-dump OUT.ppm] [--cpu-instruction-limit N] [--run-cpu]"
    );
}

fn load_import_dlls(image: &PeImage, search_dirs: &[PathBuf]) -> Result<Vec<PeImage>> {
    let mut loaded = Vec::new();
    let mut seen = BTreeSet::new();

    for descriptor in &image.imports {
        let normalized = normalize_module_name(&descriptor.module_name);
        if normalized == "coredll" || !seen.insert(normalized) {
            continue;
        }
        let Some(path) = resolve_dll_path(&descriptor.module_name, search_dirs) else {
            continue;
        };
        loaded.push(PeImage::inspect(path)?);
    }

    Ok(loaded)
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

fn ce_module_path_for_image(path: &str, sdmmc_root: Option<&Path>) -> String {
    if let Some(sdmmc_root) = sdmmc_root {
        if let Some(path) = host_path_to_guest_mount("\\SDMMC Disk", sdmmc_root, Path::new(path)) {
            return path;
        }
    }
    ce_module_path(path)
}

fn host_path_to_guest_mount(
    guest_root: &str,
    host_root: &Path,
    host_path: &Path,
) -> Option<String> {
    let relative = host_path.strip_prefix(host_root).ok()?;
    let mut guest_path = guest_root.trim_end_matches('\\').to_owned();
    for component in relative.components() {
        let std::path::Component::Normal(part) = component else {
            continue;
        };
        guest_path.push('\\');
        guest_path.push_str(&part.to_string_lossy());
    }
    Some(guest_path)
}

fn ce_module_path(path: &str) -> String {
    path.replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_image_under_sdmmc_root_to_ce_mount_path() {
        let path = ce_module_path_for_image(
            r"D:\INAVI_Emulator\INAVI\INavi\INavi.exe",
            Some(Path::new(r"D:\INAVI_Emulator\INAVI")),
        );

        assert_eq!(path, r"\SDMMC Disk\INavi\INavi.exe");
    }

    #[test]
    fn leaves_unmounted_image_path_as_ce_style_host_path() {
        let path = ce_module_path_for_image(
            r"D:\Other\INavi.exe",
            Some(Path::new(r"D:\INAVI_Emulator\INAVI")),
        );

        assert_eq!(path, r"D:\Other\INavi.exe");
    }
}
