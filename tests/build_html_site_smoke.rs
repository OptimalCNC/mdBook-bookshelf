use mdbook_bookshelf::build_html_site;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_html_site_smoke() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = repo_root.join("tests/fixtures/build-html/valid");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-007b-build-html");

    build_html_site(&config_path, &output_dir).expect("build_html_site should succeed");

    let root_index = output_dir.join("index.html");
    assert!(root_index.is_file(), "root index.html should exist");
    let root_html = fs::read_to_string(&root_index).expect("root index should be readable");
    assert!(root_html.contains("<h1>Bookshelf</h1>"));
    assert!(root_html.contains("<html lang=\"en\">"));
    assert!(root_html.contains("data-mdbook-default-theme=\"light\""));
    assert!(root_html.contains("data-book-id=\"root\"><a href=\"/root/p0000.html\">Root Book</a>"));
    assert!(root_html.contains("data-book-id=\"child\"><a href=\"/child/p0000.html\">Child Book</a>"));

    let root_content = output_dir.join("root/p0000.html");
    let child_content = output_dir.join("child/p0000.html");
    assert!(root_content.is_file(), "root book content page should exist");
    assert!(child_content.is_file(), "child book content page should exist");

    let root_content_html =
        fs::read_to_string(&root_content).expect("root content page should be readable");
    assert!(root_content_html.contains("<html lang=\"en\">"));
    assert!(root_content_html.contains("data-mdbook-default-theme=\"light\""));
    assert!(root_content_html.contains("Root Book / Root Intro"));
    assert!(root_content_html.contains("Next"));

    let child_content_html =
        fs::read_to_string(&child_content).expect("child content page should be readable");
    assert!(child_content_html.contains("Child Book / Child Intro"));

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
