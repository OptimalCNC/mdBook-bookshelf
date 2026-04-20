use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_cli_produces_the_expected_canonical_output_tree() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let relative_config_path = PathBuf::from("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = temp_root().join(unique_dir_name("build-cli-example"));

    let binary = env!("CARGO_BIN_EXE_mdbook-bookshelf");
    let status = Command::new(binary)
        .current_dir(&repo_root)
        .arg("build")
        .arg(&relative_config_path)
        .arg(&output_dir)
        .status()
        .expect("build command should run");

    assert!(status.success());

    let files = collect_files(&output_dir);
    assert_eq!(
        files,
        vec![
            PathBuf::from("docs/architecture.html"),
            PathBuf::from("docs/index.html"),
            PathBuf::from("docs/onboarding.html"),
            PathBuf::from("index.html"),
            PathBuf::from("modules/parser/docs/grammar.html"),
            PathBuf::from("modules/parser/docs/index.html"),
            PathBuf::from("modules/parser/docs/runtime.html"),
            PathBuf::from("modules/ui/docs/diagnostics.html"),
            PathBuf::from("modules/ui/docs/index.html"),
            PathBuf::from("modules/ui/docs/navigation.html"),
            PathBuf::from("searchindex.json"),
        ]
    );
}

fn collect_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_recursive(root, root, &mut files);
    files.sort();
    files
}

fn collect_files_recursive(root: &PathBuf, current: &PathBuf, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(current).expect("directory should be readable") {
        let entry = entry.expect("directory entry should exist");
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("file should be under root")
                    .to_path_buf(),
            );
        }
    }
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
