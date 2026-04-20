use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_cli_reports_missing_arguments_cleanly() {
    let binary = env!("CARGO_BIN_EXE_mdbook-bookshelf");
    let output = Command::new(binary)
        .arg("build")
        .output()
        .expect("build command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("missing required <bookshelf.toml> path"));
    assert!(stderr.contains("usage: mdbook-bookshelf build <bookshelf.toml> <out-dir>"));
}

#[test]
fn build_cli_preserves_root_only_bookshelf_routing_and_no_markdown_outputs() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = temp_root().join(unique_dir_name("build-cli-invariants"));
    let binary = env!("CARGO_BIN_EXE_mdbook-bookshelf");

    let status = Command::new(binary)
        .arg("build")
        .arg(&config_path)
        .arg(&output_dir)
        .status()
        .expect("build command should run");

    assert!(status.success());
    assert!(output_dir.join("index.html").exists());
    assert!(output_dir.join("searchindex.json").exists());
    assert!(!output_dir.join("bookshelf/index.html").exists());
    assert!(!output_dir.join("docs/index.md").exists());
    assert!(!output_dir.join("modules/parser/docs/grammar.md").exists());
}

fn temp_root() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
}

fn unique_dir_name(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{timestamp}-{counter}")
}
