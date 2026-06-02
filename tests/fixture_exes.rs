#[test]
#[ignore = "requires eVC4 MIPSII toolchain configuration"]
fn evc4_mipsii_fixture_exes() -> wince_emulation_v3::Result<()> {
    #[cfg(not(all(feature = "unicorn", feature = "evc4-fixtures")))]
    {
        eprintln!(
            "skipping eVC4 MIPSII fixtures: enable features `unicorn evc4-fixtures` to build and run fixture EXEs"
        );
        Ok(())
    }

    #[cfg(all(feature = "unicorn", feature = "evc4-fixtures"))]
    {
        fixtures::build_and_run_all()
    }
}

#[cfg(all(feature = "unicorn", feature = "evc4-fixtures"))]
mod fixtures {
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
        time::SystemTime,
    };

    use wince_emulation_v3::{
        Error, Result,
        ce::{framebuffer::VirtualFramebuffer, kernel::CeKernel},
        config::RuntimeConfig,
        emulator::unicorn::UnicornMips,
        pe::PeImage,
    };

    const SOURCE_ROOT: &str = "tests/test_progs";
    const OUTPUT_ROOT: &str = "target/wince-fixtures/mipsii";
    const SDMMC_ROOT: &str = "target/wince-fixtures/sdmmc";
    const GUEST_SDMMC: &str = "\\SDMMC Disk";
    const DEFAULT_INSTRUCTION_LIMIT: usize = 2_000_000;

    #[derive(Debug, Clone)]
    struct Fixture {
        name: String,
        source_dir: PathBuf,
        cpp_sources: Vec<PathBuf>,
        rc_sources: Vec<PathBuf>,
        output_dir: PathBuf,
        exe_path: PathBuf,
        expected_exit_code: u32,
        run_standalone: bool,
    }

    #[derive(Debug, Clone)]
    struct ToolConfig {
        cc: PathBuf,
        link: PathBuf,
        rc: Option<PathBuf>,
        include_dirs: Vec<PathBuf>,
        lib_dirs: Vec<PathBuf>,
        cflags: Vec<String>,
        lflags: Vec<String>,
        force_rebuild: bool,
    }

    pub fn build_and_run_all() -> Result<()> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let Some(config) = ToolConfig::from_env()? else {
            eprintln!(
                "skipping eVC4 MIPSII fixtures: set WINCE_EVC4_MIPSII_CC, WINCE_EVC4_MIPSII_LINK, WINCE_EVC4_MIPSII_INCLUDE, and WINCE_EVC4_MIPSII_LIB"
            );
            return Ok(());
        };

        let fixtures = discover_fixtures(&manifest_dir)?;
        if fixtures.is_empty() {
            return Err(Error::Backend(format!(
                "no fixture source directories found under {}",
                manifest_dir.join(SOURCE_ROOT).display()
            )));
        }

        for fixture in &fixtures {
            if needs_rebuild(&fixture, &manifest_dir, config.force_rebuild)? {
                build_fixture(&fixture, &config, &manifest_dir)?;
            } else {
                eprintln!(
                    "fixture {} is fresh: {}",
                    fixture.name,
                    fixture.exe_path.display()
                );
            }
        }

        for fixture in &fixtures {
            if !fixture.run_standalone {
                eprintln!(
                    "fixture {} is child-only; built but not run standalone",
                    fixture.name
                );
                continue;
            }
            run_fixture(&fixture, &manifest_dir)?;
        }

        Ok(())
    }

    impl ToolConfig {
        fn from_env() -> Result<Option<Self>> {
            let required = [
                "WINCE_EVC4_MIPSII_CC",
                "WINCE_EVC4_MIPSII_LINK",
                "WINCE_EVC4_MIPSII_INCLUDE",
                "WINCE_EVC4_MIPSII_LIB",
            ];
            if required.iter().any(|key| env::var_os(key).is_none()) {
                return Ok(None);
            }

            let rc = env::var_os("WINCE_EVC4_RC").map(PathBuf::from);
            let config = Self {
                cc: PathBuf::from(env::var_os("WINCE_EVC4_MIPSII_CC").unwrap()),
                link: PathBuf::from(env::var_os("WINCE_EVC4_MIPSII_LINK").unwrap()),
                rc,
                include_dirs: split_path_list("WINCE_EVC4_MIPSII_INCLUDE"),
                lib_dirs: split_path_list("WINCE_EVC4_MIPSII_LIB"),
                cflags: split_flags("WINCE_EVC4_MIPSII_CFLAGS"),
                lflags: split_flags("WINCE_EVC4_MIPSII_LFLAGS"),
                force_rebuild: env::var_os("WINCE_EVC4_FORCE_REBUILD").is_some_and(|value| {
                    value != "0" && !value.to_string_lossy().eq_ignore_ascii_case("false")
                }),
            };
            config.validate()?;
            Ok(Some(config))
        }

        fn validate(&self) -> Result<()> {
            require_file("WINCE_EVC4_MIPSII_CC", &self.cc)?;
            require_file("WINCE_EVC4_MIPSII_LINK", &self.link)?;
            if let Some(rc) = self.rc.as_ref() {
                require_file("WINCE_EVC4_RC", rc)?;
            }
            if self.include_dirs.is_empty() {
                return Err(Error::Backend(
                    "WINCE_EVC4_MIPSII_INCLUDE must contain at least one include directory"
                        .to_owned(),
                ));
            }
            if self.lib_dirs.is_empty() {
                return Err(Error::Backend(
                    "WINCE_EVC4_MIPSII_LIB must contain at least one library directory".to_owned(),
                ));
            }
            for dir in &self.include_dirs {
                require_dir("WINCE_EVC4_MIPSII_INCLUDE", dir)?;
            }
            for dir in &self.lib_dirs {
                require_dir("WINCE_EVC4_MIPSII_LIB", dir)?;
            }
            Ok(())
        }
    }

    fn discover_fixtures(manifest_dir: &Path) -> Result<Vec<Fixture>> {
        let source_root = manifest_dir.join(SOURCE_ROOT);
        let output_root = manifest_dir.join(OUTPUT_ROOT);
        let mut fixtures = Vec::new();

        for entry in fs::read_dir(&source_root).map_err(|source| Error::Read {
            path: source_root.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| Error::Read {
                path: source_root.clone(),
                source,
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if matches!(name.as_str(), "asm" | "build_notes" | "common" | "tools") {
                continue;
            }

            let cpp_sources = sorted_files_with_extension(&path, "cpp")?;
            if cpp_sources.is_empty() {
                continue;
            }
            let rc_sources = sorted_files_with_extension(&path, "rc")?;
            let output_dir = output_root.join(&name);
            let exe_path = output_dir.join(format!("{name}.exe"));
            let run_standalone = !matches!(
                name.as_str(),
                "037_ipc_child" | "039_exit_marker_child" | "041_commandline_child"
            );
            fixtures.push(Fixture {
                name,
                source_dir: path,
                cpp_sources,
                rc_sources,
                output_dir,
                exe_path,
                expected_exit_code: 0,
                run_standalone,
            });
        }

        fixtures.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
        Ok(fixtures)
    }

    fn sorted_files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(dir).map_err(|source| Error::Read {
            path: dir.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| Error::Read {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(extension))
            {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    fn needs_rebuild(fixture: &Fixture, manifest_dir: &Path, force: bool) -> Result<bool> {
        if force || !fixture.exe_path.is_file() {
            return Ok(true);
        }
        let output_time = modified_at(&fixture.exe_path)?;
        for source in fixture_inputs(fixture, manifest_dir)? {
            if modified_at(&source)? > output_time {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn fixture_inputs(fixture: &Fixture, manifest_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut inputs = Vec::new();
        collect_files_recursive(&fixture.source_dir, &mut inputs)?;
        collect_files_recursive(&manifest_dir.join(SOURCE_ROOT).join("common"), &mut inputs)?;
        inputs.sort();
        Ok(inputs)
    }

    fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir).map_err(|source| Error::Read {
            path: dir.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| Error::Read {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
        Ok(())
    }

    fn build_fixture(fixture: &Fixture, config: &ToolConfig, manifest_dir: &Path) -> Result<()> {
        fs::create_dir_all(&fixture.output_dir).map_err(|source| Error::Io {
            path: fixture.output_dir.clone(),
            source,
        })?;

        eprintln!("building eVC4 MIPSII fixture {}", fixture.name);
        let mut objects = Vec::new();
        for source in &fixture.cpp_sources {
            let obj = fixture
                .output_dir
                .join(format!("{}.obj", file_stem(source)?));
            run_compile(config, manifest_dir, source, &obj)?;
            objects.push(obj);
        }

        let mut resources = Vec::new();
        for source in &fixture.rc_sources {
            let Some(rc) = config.rc.as_ref() else {
                return Err(Error::Backend(format!(
                    "fixture {} has resource source {} but WINCE_EVC4_RC is not set",
                    fixture.name,
                    source.display()
                )));
            };
            let res = fixture
                .output_dir
                .join(format!("{}.res", file_stem(source)?));
            run_resource_compile(config, rc, manifest_dir, source, &res)?;
            resources.push(res);
        }

        run_link(
            config,
            &objects,
            &resources,
            &fixture.exe_path,
            fixture_uses_mfc(fixture)?,
        )?;
        Ok(())
    }

    fn run_compile(
        config: &ToolConfig,
        manifest_dir: &Path,
        source: &Path,
        object: &Path,
    ) -> Result<()> {
        let mut command = Command::new(&config.cc);
        command.current_dir(manifest_dir);
        command.args([
            "/nologo",
            "/c",
            "/W3",
            "/O2",
            "/D_WIN32_WCE=420",
            "/DUNDER_CE=420",
            "/DWIN32_PLATFORM_STANDARDSDK_420",
            "/DMIPS",
            "/D_MIPS_",
            "/DMIPSII",
            "/D_MIPSII_",
            "/DNDEBUG",
        ]);
        for include_dir in &config.include_dirs {
            command.arg(format!("/I{}", include_dir.display()));
        }
        command.arg(format!(
            "/I{}",
            manifest_dir.join(SOURCE_ROOT).join("common").display()
        ));
        command.args(&config.cflags);
        command.arg(format!("/Fo{}", object.display()));
        command.arg(source);
        run_command("compile fixture source", &mut command)
    }

    fn run_resource_compile(
        config: &ToolConfig,
        rc: &Path,
        manifest_dir: &Path,
        source: &Path,
        resource: &Path,
    ) -> Result<()> {
        let mut command = Command::new(rc);
        command.current_dir(manifest_dir);
        command.args([
            "/r",
            "/d_WIN32_WCE=420",
            "/dUNDER_CE=420",
            "/dWIN32_PLATFORM_STANDARDSDK_420",
        ]);
        for include_dir in &config.include_dirs {
            command.arg(format!("/i{}", include_dir.display()));
        }
        command.arg(format!(
            "/i{}",
            manifest_dir.join(SOURCE_ROOT).join("common").display()
        ));
        command.arg(format!("/fo{}", resource.display()));
        command.arg(source);
        run_command("compile fixture resource", &mut command)
    }

    fn run_link(
        config: &ToolConfig,
        objects: &[PathBuf],
        resources: &[PathBuf],
        exe: &Path,
        uses_mfc: bool,
    ) -> Result<()> {
        let mut command = Command::new(&config.link);
        command.args([
            "/nologo",
            "/MACHINE:MIPS",
            "/SUBSYSTEM:WINDOWSCE,4.20",
            "/ENTRY:WinMainCRTStartup",
        ]);
        command.arg(format!("/OUT:{}", exe.display()));
        for lib_dir in &config.lib_dirs {
            command.arg(format!("/LIBPATH:{}", lib_dir.display()));
        }
        if uses_mfc {
            command.arg("/FORCE:MULTIPLE");
        }
        command.args(&config.lflags);
        command.args(objects);
        command.args(resources);
        command.args(["coredll.lib", "corelibc.lib"]);
        run_command("link fixture exe", &mut command)
    }

    fn fixture_uses_mfc(fixture: &Fixture) -> Result<bool> {
        for source in &fixture.cpp_sources {
            let text = fs::read_to_string(source).map_err(|source_err| Error::Read {
                path: source.clone(),
                source: source_err,
            })?;
            if text.contains("<afxwin.h>") {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn run_fixture(fixture: &Fixture, manifest_dir: &Path) -> Result<()> {
        eprintln!("running fixture {}", fixture.name);
        let image = PeImage::inspect(&fixture.exe_path)?;
        let config = RuntimeConfig::load(
            manifest_dir.join("regs.json"),
            manifest_dir.join("serial_devices.json"),
        )?;
        let mut kernel = CeKernel::boot(config);
        kernel.set_process_module_base(image.image_base());
        kernel.set_process_module_path(format!("{}\\{}.exe", GUEST_SDMMC, fixture.name));
        kernel.set_process_module_host_path(fixture.exe_path.clone());
        let sdmmc_root = manifest_dir.join(SDMMC_ROOT).join(&fixture.name);
        fs::create_dir_all(&sdmmc_root).map_err(|source| Error::Io {
            path: sdmmc_root.clone(),
            source,
        })?;
        kernel.mount_guest_root(GUEST_SDMMC, sdmmc_root);

        let mut framebuffer = VirtualFramebuffer::default_primary()?;
        let mut cpu = UnicornMips::new()?;
        cpu.load_pe_image(&image)?;
        cpu.run_until_import_trap_with_framebuffer_limit(
            &mut kernel,
            &mut framebuffer,
            DEFAULT_INSTRUCTION_LIMIT,
        )?;

        let snapshot = cpu.last_debug_snapshot().ok_or_else(|| {
            Error::Backend(format!(
                "fixture {} produced no Unicorn snapshot",
                fixture.name
            ))
        })?;
        let Some(exit) = snapshot.encoded_kernel_exit.as_ref() else {
            return Err(Error::Backend(format!(
                "fixture {} did not reach a decoded process exit; snapshot: {snapshot}",
                fixture.name
            )));
        };
        if exit.exit_code != fixture.expected_exit_code {
            return Err(Error::Backend(format!(
                "fixture {} exited with 0x{:08x}, expected 0x{:08x}; snapshot: {snapshot}",
                fixture.name, exit.exit_code, fixture.expected_exit_code
            )));
        }

        Ok(())
    }

    fn run_command(label: &str, command: &mut Command) -> Result<()> {
        let output = command.output().map_err(|source| Error::Io {
            path: command.get_program().into(),
            source,
        })?;
        if output.status.success() {
            return Ok(());
        }

        Err(Error::Backend(format!(
            "{label} failed with status {}\ncommand: {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            command,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )))
    }

    fn split_path_list(key: &str) -> Vec<PathBuf> {
        env::var_os(key)
            .map(|value| {
                env::split_paths(&value)
                    .filter(|path| !path.as_os_str().is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn split_flags(key: &str) -> Vec<String> {
        env::var(key)
            .map(|value| value.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default()
    }

    fn require_file(key: &str, path: &Path) -> Result<()> {
        if path.is_file() {
            Ok(())
        } else {
            Err(Error::Backend(format!(
                "{key} points to a missing file: {}",
                path.display()
            )))
        }
    }

    fn require_dir(key: &str, path: &Path) -> Result<()> {
        if path.is_dir() {
            Ok(())
        } else {
            Err(Error::Backend(format!(
                "{key} contains a missing directory: {}",
                path.display()
            )))
        }
    }

    fn modified_at(path: &Path) -> Result<SystemTime> {
        Ok(fs::metadata(path)
            .map_err(|source| Error::Read {
                path: path.to_path_buf(),
                source,
            })?
            .modified()
            .map_err(|source| Error::Read {
                path: path.to_path_buf(),
                source,
            })?)
    }

    fn file_stem(path: &Path) -> Result<String> {
        path.file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .ok_or_else(|| Error::Backend(format!("path has no file stem: {}", path.display())))
    }
}
