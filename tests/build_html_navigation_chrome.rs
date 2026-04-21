use mdbook_bookshelf::build_html_site;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_html_navigation_chrome() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = repo_root.join("tests/fixtures/build-html-nav/valid");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-008-build-html-nav");

    build_html_site(&config_path, &output_dir).expect("build_html_site should succeed");

    let root_first = fs::read_to_string(output_dir.join("root/p0000.html"))
        .expect("root first page should be readable");
    let root_last = fs::read_to_string(output_dir.join("root/p0001.html"))
        .expect("root last page should be readable");
    let child_first = fs::read_to_string(output_dir.join("child/p0000.html"))
        .expect("child first page should be readable");

    assert!(root_first.contains("<a class=\"bookshelf-return\" href=\"/index.html\">Bookshelf</a>"));
    assert!(child_first.contains("<a class=\"bookshelf-return\" href=\"/index.html\">Bookshelf</a>"));

    assert!(root_first.contains("<aside class=\"sidebar\" data-active-book=\"root\">"));
    assert!(root_first.contains("data-page-id=\"root:0000\""));
    assert!(root_first.contains("data-page-id=\"root:0001\""));
    assert!(!root_first.contains("data-page-id=\"child:0000\""));
    assert!(!root_first.contains("data-page-id=\"child:0001\""));

    assert!(child_first.contains("<aside class=\"sidebar\" data-active-book=\"child\">"));
    assert!(child_first.contains("data-page-id=\"child:0000\""));
    assert!(child_first.contains("data-page-id=\"child:0001\""));
    assert!(!child_first.contains("data-page-id=\"root:0000\""));
    assert!(!child_first.contains("data-page-id=\"root:0001\""));

    assert!(root_first.contains("<div class=\"breadcrumb\">Root Book / Root Intro</div>"));
    assert!(child_first.contains("<div class=\"breadcrumb\">Child Book / Child Intro</div>"));

    assert!(root_first.contains("<span class=\"prev missing\"></span>"));
    assert!(root_first.contains("<a class=\"next\" href=\"/root/p0001.html\">Next</a>"));

    assert!(root_last.contains("<a class=\"prev\" href=\"/root/p0000.html\">Prev</a>"));
    assert!(root_last.contains("<span class=\"next missing\"></span>"));
    assert!(!root_last.contains("/child/p0000.html"));

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
