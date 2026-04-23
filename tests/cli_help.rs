use std::path::PathBuf;
use std::process::Command;

#[test]
fn top_level_help_lists_build_and_serve_subcommands() {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));

    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .expect("help command should run");

    assert!(
        output.status.success(),
        "help command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"));
    assert!(stdout.contains("serve"));
}

#[test]
fn serve_help_lists_network_flags() {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));

    let output = Command::new(&bin)
        .arg("serve")
        .arg("--help")
        .output()
        .expect("serve help command should run");

    assert!(
        output.status.success(),
        "serve help command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BOOKSHELF_TOML"));
    assert!(stdout.contains("--hostname"));
    assert!(stdout.contains("--port"));
    assert!(stdout.contains("--dest-dir"));
}
