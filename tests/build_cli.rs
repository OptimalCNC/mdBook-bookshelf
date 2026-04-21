use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_cli_emits_stock_mdbook_output_per_book() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook-bookshelf"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = make_temp_dir("chunk-011-build-cli", &repo_root);

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert_exists(output_dir.join("books/meta/index.html"));
    assert_exists(output_dir.join("books/meta/onboarding.html"));
    assert_exists(output_dir.join("books/meta/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/meta"), "book-", ".js");
    assert_exists(output_dir.join("books/parser/index.html"));
    assert_exists(output_dir.join("books/parser/grammar.html"));
    assert_exists(output_dir.join("books/parser/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/parser"), "book-", ".js");

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_resolves_relative_mdbook_paths_from_bookshelf_config_dir() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook-bookshelf"));
    let fixture_root = repo_root.join("tests/fixtures/build-cli/shared-config-root");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-011-shared-config-root", &repo_root);

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let root_css =
        assert_has_file_with_prefix(&output_dir.join("books/root/shared"), "site-", ".css");
    let child_css =
        assert_has_file_with_prefix(&output_dir.join("books/child/shared"), "site-", ".css");
    assert_eq!(root_css, child_css);

    assert_file_contains(
        output_dir.join("books/root/index.html"),
        &format!("shared/{root_css}"),
    );
    assert_file_contains(
        output_dir.join("books/child/index.html"),
        &format!("shared/{child_css}"),
    );

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

fn assert_exists(path: PathBuf) {
    assert!(path.exists(), "expected {} to exist", path.display());
}

fn assert_has_file_with_prefix(dir: &Path, prefix: &str, suffix: &str) -> String {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
    let matched = entries.filter_map(|entry| entry.ok()).find_map(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        (name.starts_with(prefix) && name.ends_with(suffix)).then(|| name.into_owned())
    });

    assert!(
        matched.is_some(),
        "expected {} to contain a file matching {}*{}",
        dir.display(),
        prefix,
        suffix
    );

    matched.expect("matching file should exist")
}

fn assert_file_contains(path: PathBuf, needle: &str) {
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    assert!(
        content.contains(needle),
        "expected {} to contain {:?}",
        path.display(),
        needle
    );
}

fn make_temp_dir(tag: &str, root: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let dir = std::env::temp_dir()
        .join("mdbook-bookshelf")
        .join(root.file_name().unwrap_or_default())
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp output directory should be created");
    dir
}
