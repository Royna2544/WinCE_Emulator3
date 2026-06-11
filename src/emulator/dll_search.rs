use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::ce::kernel::CeKernel;

pub fn emulator_provided_import_module(normalized_module_name: &str) -> bool {
    matches!(
        normalized_module_name,
        "coredll" | "winsock" | "ws2" | "ws2_32" | "ole32" | "oleaut32" | "olece"
    )
}

pub fn normalize_module_name(module_name: &str) -> String {
    let name = module_name.trim().trim_end_matches('\0').replace('/', "\\");
    let file_name = name.rsplit('\\').next().unwrap_or(&name);
    file_name
        .trim_end_matches(".dll")
        .trim_end_matches(".DLL")
        .to_ascii_lowercase()
}

pub fn resolve_dll_path(
    module_name: &str,
    kernel: Option<&CeKernel>,
    search_dirs: &[PathBuf],
) -> Option<PathBuf> {
    let module_name = module_name.trim().trim_end_matches('\0');
    if module_name.is_empty() {
        return None;
    }

    if let Some(kernel) = kernel {
        if is_ce_absolute_path(module_name) {
            for candidate in ce_path_candidates(module_name) {
                if let Ok(path) = kernel.host_path_for_guest(&candidate)
                    && path.is_file()
                {
                    return Some(path);
                }
            }
        }
    }

    let file_name = module_file_name(module_name);
    let candidates = file_name_candidates(file_name);

    if let Some(kernel) = kernel {
        if let Some(parent) = kernel
            .process_module_host_path()
            .and_then(|path| path.parent())
        {
            if let Some(path) = resolve_in_host_dir(parent, &candidates) {
                return Some(path);
            }
        }

        if let Some(parent) = ce_parent_path(kernel.process_module_path()) {
            for candidate in &candidates {
                let ce_path = format!("{parent}\\{candidate}");
                if let Ok(path) = kernel.host_path_for_guest(&ce_path)
                    && path.is_file()
                {
                    return Some(path);
                }
            }
        }
    }

    for dir in search_dirs {
        if let Some(path) = resolve_in_host_dir(dir, &candidates) {
            return Some(path);
        }
    }

    if let Some(kernel) = kernel {
        for candidate in &candidates {
            let ce_path = format!("\\Windows\\{candidate}");
            if let Ok(path) = kernel.host_path_for_guest(&ce_path)
                && path.is_file()
            {
                return Some(path);
            }
        }
    }

    None
}

fn resolve_in_host_dir(dir: &Path, candidates: &[String]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(|candidate| dir.join(candidate))
        .find(|candidate| candidate.is_file())
}

fn is_ce_absolute_path(path: &str) -> bool {
    path.starts_with('\\') || path.starts_with('/')
}

fn module_file_name(module_name: &str) -> &str {
    module_name
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(module_name)
}

fn file_name_candidates(file_name: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    let base_variants = [
        file_name.to_owned(),
        file_name.to_ascii_lowercase(),
        file_name.to_ascii_uppercase(),
    ];
    for variant in base_variants {
        push_candidate(&mut seen, &mut candidates, variant.clone());
        if Path::new(&variant).extension().is_none() {
            push_candidate(&mut seen, &mut candidates, format!("{variant}.dll"));
            push_candidate(&mut seen, &mut candidates, format!("{variant}.DLL"));
        }
    }
    candidates
}

fn ce_path_candidates(path: &str) -> Vec<String> {
    let normalized = path.replace('/', "\\");
    let Some((parent, file_name)) = normalized.rsplit_once('\\') else {
        return file_name_candidates(&normalized);
    };
    file_name_candidates(file_name)
        .into_iter()
        .map(|candidate| format!("{parent}\\{candidate}"))
        .collect()
}

fn ce_parent_path(path: &str) -> Option<String> {
    let normalized = path.replace('/', "\\");
    normalized
        .rsplit_once('\\')
        .map(|(parent, _name)| parent.to_owned())
        .filter(|parent| !parent.is_empty())
}

fn push_candidate(seen: &mut BTreeSet<String>, candidates: &mut Vec<String>, candidate: String) {
    if seen.insert(candidate.to_ascii_lowercase()) {
        candidates.push(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ce::kernel::CeKernel, config::RuntimeConfig};

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("wince-dll-search-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn resolves_case_variants_and_optional_extension_from_configured_dirs() {
        let root = temp_root("case");
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

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_search_prefers_process_dir_then_configured_dirs_then_windows() {
        let root = temp_root("order");
        let process_dir = root.join("app");
        let configured_dir = root.join("configured");
        let windows_dir = root.join("Windows");
        std::fs::create_dir_all(&process_dir).unwrap();
        std::fs::create_dir_all(&configured_dir).unwrap();
        std::fs::create_dir_all(&windows_dir).unwrap();
        std::fs::write(process_dir.join("foo.dll"), b"process").unwrap();
        std::fs::write(configured_dir.join("foo.dll"), b"configured").unwrap();
        std::fs::write(windows_dir.join("foo.dll"), b"windows").unwrap();

        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        kernel.set_file_root(&root);
        kernel.mount_guest_root("\\Program Files\\App", &process_dir);
        kernel.mount_guest_root("\\Windows", &windows_dir);
        kernel.set_process_module_path("\\Program Files\\App\\app.exe");
        kernel.set_process_module_host_path(process_dir.join("app.exe"));

        assert_eq!(
            resolve_dll_path(
                "foo.dll",
                Some(&kernel),
                std::slice::from_ref(&configured_dir)
            )
            .unwrap(),
            process_dir.join("foo.dll")
        );
        std::fs::remove_file(process_dir.join("foo.dll")).unwrap();
        assert_eq!(
            resolve_dll_path(
                "foo.dll",
                Some(&kernel),
                std::slice::from_ref(&configured_dir)
            )
            .unwrap(),
            configured_dir.join("foo.dll")
        );
        std::fs::remove_file(configured_dir.join("foo.dll")).unwrap();
        assert_eq!(
            resolve_dll_path(
                "foo.dll",
                Some(&kernel),
                std::slice::from_ref(&configured_dir)
            )
            .unwrap(),
            windows_dir.join("foo.dll")
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn runtime_search_resolves_exact_ce_paths_through_mounts() {
        let root = temp_root("ce-path");
        let windows_dir = root.join("Windows");
        std::fs::create_dir_all(&windows_dir).unwrap();
        let dll = windows_dir.join("bar.dll");
        std::fs::write(&dll, []).unwrap();

        let config = RuntimeConfig::load_default().unwrap();
        let mut kernel = CeKernel::boot(config);
        kernel.set_file_root(&root);
        kernel.mount_guest_root("\\Windows", &windows_dir);

        assert_eq!(
            resolve_dll_path("\\Windows\\bar", Some(&kernel), &[]).unwrap(),
            dll
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn commctrl_is_not_emulator_provided() {
        assert!(!emulator_provided_import_module("commctrl"));
        assert!(!emulator_provided_import_module("commctrlce"));
        assert!(emulator_provided_import_module("coredll"));
        assert!(emulator_provided_import_module("winsock"));
        assert!(emulator_provided_import_module("ole32"));
    }
}
