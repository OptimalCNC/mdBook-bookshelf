use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::{
    build_reader_context_model, build_render_manifest, build_search_index, build_sidebar_model,
    build_site_model, load_input_catalog, render_site_with_search_index,
};

#[test]
fn search_index_is_site_wide_and_excludes_the_synthetic_bookshelf_page() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");
    let render_manifest =
        build_render_manifest(&site_model, &reader_context).expect("render manifest should build");
    let search_index =
        build_search_index(&reader_context, &render_manifest).expect("search index should build");
    let output_dir = temp_root().join(unique_dir_name("search-index-invariants"));

    render_site_with_search_index(
        &output_dir,
        &site_model,
        &sidebar_model,
        &reader_context,
        &render_manifest,
    )
    .expect("site render with search should succeed");

    assert_eq!(search_index.documents.len(), 9);
    assert!(search_index
        .documents
        .iter()
        .all(|document| document.title != "Bookshelf"));
    assert!(search_index
        .documents
        .iter()
        .any(|document| document.book_label == "Example Core"));
    assert!(search_index
        .documents
        .iter()
        .any(|document| document.book_label == "Example Parser"));
    assert!(search_index
        .documents
        .iter()
        .any(|document| document.book_label == "Example UI"));
    assert!(search_index
        .documents
        .iter()
        .all(|document| document.href.ends_with(".html")));
    assert!(search_index
        .documents
        .iter()
        .all(|document| !document.href.ends_with(".md")));
    assert!(search_index
        .documents
        .iter()
        .all(|document| document.href != "/"));
    assert!(search_index
        .documents
        .iter()
        .all(|document| !document.href.contains("bookshelf/")));

    let json = fs::read_to_string(output_dir.join("searchindex.json"))
        .expect("searchindex.json should be readable");
    assert!(output_dir.join("searchindex.json").exists());
    assert!(!json.contains("\"title\":\"Bookshelf\""));
    assert!(!json.contains(".md\""));
    assert!(!json.contains("\"href\":\"/\""));
    assert!(!json.contains("bookshelf/"));
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
