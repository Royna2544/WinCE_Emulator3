use std::fs;

#[test]
fn host_cmd_source_is_evc4_friendly_command_shell() {
    let source = fs::read_to_string("host_progs/cmd.cpp").expect("host_progs/cmd.cpp");
    assert!(source.contains("int WINAPI WinMain"));
    assert!(source.contains("CreateWindowW"));
    assert!(source.contains("CreateProcessW"));
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
