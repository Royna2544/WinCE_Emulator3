use std::fs;

#[test]
fn host_cmd_source_is_evc4_friendly_command_shell() {
    let source = fs::read_to_string("host_progs/cmd.cpp").expect("host_progs/cmd.cpp");
    assert!(source.contains("int WINAPI WinMain"));
    assert!(source.contains("CreateWindowW"));
    assert!(source.contains("CreateProcessW"));
    assert!(source.contains("DynTerminateProcess"));
    assert!(source.contains("ExecuteCommand"));
    for command in [
        "\"cd\"",
        "\"dir\"",
        "\"type\"",
        "\"echo\"",
        "\"copy\"",
        "\"del\"",
        "\"mkdir\"",
        "\"rmdir\"",
        "\"cls\"",
        "\"_kill\"",
        "\"exit\"",
    ] {
        assert!(
            source.contains(command),
            "host cmd source should include built-in command {command}"
        );
    }
    assert!(!source.contains("#include <string>"));
    assert!(!source.contains("#include <vector>"));
    assert!(!source.contains("#include <afx"));
}

#[test]
fn host_tools_sources_cover_requested_utilities() {
    let snip = fs::read_to_string("host_progs/snip.cpp").expect("host_progs/snip.cpp");
    assert!(snip.contains("SNIP_FROM_YEAR 2005"));
    assert!(snip.contains("DEFAULT_SNIP_COPY_DIR L\"\\\\SDMMC Disk\""));
    assert!(snip.contains("SnipTool_%lu.bmp"));
    assert!(snip.contains("BitBlt"));
    assert!(snip.contains("MessageBoxW"));

    let taskmgr = fs::read_to_string("host_progs/taskmgr.cpp").expect("host_progs/taskmgr.cpp");
    assert!(taskmgr.contains("GlobalMemoryStatus"));
    assert!(taskmgr.contains("GetDiskFreeSpaceExW"));
    assert!(taskmgr.contains("FILE_ATTRIBUTE_TEMPORARY"));
    assert!(taskmgr.contains("DynCreateToolhelp32Snapshot"));
    assert!(taskmgr.contains("CPU Usage"));
    assert!(taskmgr.contains("Processes"));
    assert!(taskmgr.contains("PID(hex)"));

    let easy = fs::read_to_string("host_progs/easy_iNavi.cpp").expect("host_progs/easy_iNavi.cpp");
    assert!(easy.contains("\\\\SDMMC Disk\\\\INavi\\\\iNavi_main.exe"));
    assert!(easy.contains("\\\\Windows\\\\explorer.exe"));
    assert!(easy.contains("CreateMutexW"));
    assert!(easy.contains("Already running"));
    assert!(easy.contains("cwd = "));
    assert!(easy.contains("process args = ["));
    assert!(easy.contains("process env = {"));
    assert!(easy.contains("CreateProcessW"));

    let helper =
        fs::read_to_string("host_progs/dyn_resolve_helper.cpp").expect("dyn_resolve_helper.cpp");
    assert!(helper.contains("LoadLibraryW"));
    assert!(helper.contains("GetProcAddress"));
    assert!(helper.contains("toolhelp.dll"));
}
