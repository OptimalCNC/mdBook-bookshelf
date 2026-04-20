use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::{
    build_reader_context_model, build_render_manifest, build_sidebar_model, build_site_model,
    load_input_catalog, render_bookshelf_root_page,
};

#[test]
fn renders_the_bookshelf_root_html_for_the_self_contained_example() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");
    let render_manifest =
        build_render_manifest(&site_model, &reader_context).expect("render manifest should build");
    let output_dir = temp_root().join(unique_dir_name("bookshelf-root-html-example"));

    let written_path = render_bookshelf_root_page(&output_dir, &site_model, &render_manifest)
        .expect("render should succeed");
    let html = fs::read_to_string(&written_path).expect("rendered html should be readable");

    assert_eq!(written_path, output_dir.join("index.html"));
    assert!(html.contains("<h1>Bookshelf</h1>"));
    assert!(html.contains("class=\"page-wrapper\""));
    assert!(html.contains("class=\"content\""));
    assert!(html.matches("class=\"bookshelf-link\"").count() == 3);

    for title in ["Example Core", "Example Parser", "Example UI"] {
        assert!(html.contains(title));
    }
    for description in [
        "Repository-wide onboarding and architecture notes.",
        "Parser-specific reference pages with their own reading order.",
        "Interface and runtime guides for the UI book.",
    ] {
        assert!(html.contains(description));
    }

    for href in [
        "href=\"docs/index.html\"",
        "href=\"modules/parser/docs/index.html\"",
        "href=\"modules/ui/docs/index.html\"",
    ] {
        assert!(html.contains(href));
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
