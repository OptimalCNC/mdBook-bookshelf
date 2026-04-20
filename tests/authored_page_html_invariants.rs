use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::{
    build_reader_context_model, build_render_manifest, build_sidebar_model, build_site_model,
    load_input_catalog, render_site,
};

#[test]
fn renders_only_manifest_defined_outputs_without_bookshelf_aliases() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");
    let render_manifest =
        build_render_manifest(&site_model, &reader_context).expect("render manifest should build");
    let output_dir = temp_root().join(unique_dir_name("authored-page-html-invariants"));

    render_site(
        &output_dir,
        &site_model,
        &sidebar_model,
        &reader_context,
        &render_manifest,
    )
    .expect("site render should succeed");

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
        ]
    );

    assert!(!output_dir.join("bookshelf/index.html").exists());
    let docs_index =
        fs::read_to_string(output_dir.join("docs/index.html")).expect("docs index should exist");
    assert!(docs_index.contains("class=\"affix\" href=\"../index.html\">Bookshelf</a>"));
    assert!(!docs_index.contains("href=\"../bookshelf/\""));
    assert!(!docs_index.contains("href=\"../bookshelf/index.html\""));

    for relative_path in files {
        let html = fs::read_to_string(output_dir.join(&relative_path))
            .expect("rendered html should exist");
        assert!(
            !html.contains(".md\""),
            "rendered html should not contain raw .md hrefs: {}",
            relative_path.display()
        );
        assert!(
            !html.contains(".markdown\""),
            "rendered html should not contain raw .markdown hrefs: {}",
            relative_path.display()
        );
    }
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
