use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::build_site;

#[test]
fn search_page_stays_root_level_and_uses_canonical_result_hrefs() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = temp_root().join(unique_dir_name("search-page-invariants"));

    build_site(&config_path, &output_dir).expect("site build should succeed");

    assert!(output_dir.join("search.html").exists());
    assert!(!output_dir.join("bookshelf/search.html").exists());

    let search_html =
        fs::read_to_string(output_dir.join("search.html")).expect("search page should exist");
    assert!(!search_html.contains("bookshelf/"));
    assert!(!search_html.contains(".md\""));
    assert!(!search_html.contains("href=\"/\""));
    assert!(search_html.contains("docs/onboarding.html"));
    assert!(search_html.contains("modules/parser/docs/grammar.html"));
    assert!(search_html.contains("modules/ui/docs/index.html"));
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
