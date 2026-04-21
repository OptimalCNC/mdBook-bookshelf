use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn cli_build_smoke() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = make_temp_dir("chunk-010-cli-build");

    let status = Command::new(env!("CARGO_BIN_EXE_mdbook-bookshelf"))
        .arg("build")
        .arg("--config")
        .arg(&config_path)
        .arg("--dest")
        .arg(&output_dir)
        .status()
        .expect("cli should launch");
    assert!(status.success(), "cli build command should succeed");

    assert!(output_dir.join("index.html").is_file(), "root index.html should exist");
    assert!(
        output_dir.join("meta/p0000.html").is_file(),
        "meta content page should exist"
    );
    assert!(
        output_dir.join("parser/p0000.html").is_file(),
        "parser content page should exist"
    );
    assert!(
        output_dir.join("ui/p0000.html").is_file(),
        "ui content page should exist"
    );

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

fn make_temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let dir = std::env::temp_dir()
        .join("mdbook-bookshelf")
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}
