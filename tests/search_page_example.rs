use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::build_site;

#[test]
fn self_contained_example_emits_search_page_and_shared_search_affordances() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = temp_root().join(unique_dir_name("search-page-example"));

    build_site(&config_path, &output_dir).expect("site build should succeed");

    let search_html =
        fs::read_to_string(output_dir.join("search.html")).expect("search page should exist");
    assert!(
        search_html.contains("<h1>Search</h1>") || search_html.contains("aria-label=\"Search\"")
    );
    assert!(search_html.contains("Example Core"));
    assert!(search_html.contains("Example Parser"));
    assert!(search_html.contains("Example UI"));
    assert!(search_html.contains("href=\"modules/parser/docs/grammar.html\""));
    assert!(search_html.contains(">Example Parser<"));

    let root_html =
        fs::read_to_string(output_dir.join("index.html")).expect("root page should exist");
    assert!(root_html.contains("class=\"search-link\" href=\"search.html\""));

    let grammar_html = fs::read_to_string(output_dir.join("modules/parser/docs/grammar.html"))
        .expect("grammar page should exist");
    assert!(grammar_html.contains("class=\"search-link\" href=\"../../../search.html\""));
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
