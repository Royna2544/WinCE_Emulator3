use std::path::PathBuf;

use wince_emulation_v3::{
    Result,
    ce::{
        audio::WaveFormat,
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

    let mut cpu = UnicornMips::new()?;
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

    let hwnd = kernel.gwe.create_window(1, "FakeCEBaseWindow", "");
    let timer_id = kernel.timers.set_timer(Some(hwnd), None, 1000, WM_TIMER);
    let wave_id = kernel.audio.open_wave_out(WaveFormat::pcm_16bit(2, 44_100));
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
    println!("  bootstrap hwnd: 0x{hwnd:08x}");
    println!("  bootstrap timer: {timer_id}");
    println!("  bootstrap waveOut: {wave_id}");
    println!("  memory regions: {}", cpu.memory().regions().count());

    if let Some(image_path) = args.image {
        let image = PeImage::inspect(image_path)?;
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

    if args.run_cpu {
        cpu.run_until_import_trap()?;
    }

    Ok(())
}

impl Args {
    fn parse() -> Result<Self> {
        let mut registry = PathBuf::from("regs.json");
        let mut devices = PathBuf::from("serial_devices.json");
        let mut image = None;
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
            run_cpu,
        })
    }
}

fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| wince_emulation_v3::Error::InvalidArgument(format!("{flag} needs a path")))
}

fn print_help() {
    println!(
        "Usage: wince_emulation_v3 [--registry regs.json] [--devices serial_devices.json] [--image INavi.exe] [--run-cpu]"
    );
}
