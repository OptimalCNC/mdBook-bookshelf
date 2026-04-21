use mdbook_bookshelf::build_html_site;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_html_preprocess() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = repo_root.join("tests/fixtures/build-html-preprocess");

    let valid_config = fixture_root.join("valid/bookshelf.toml");
    let valid_out = make_temp_dir("chunk-009-preprocess-valid");
    build_html_site(&valid_config, &valid_out).expect("valid preprocess fixture should build");
    let valid_html = fs::read_to_string(valid_out.join("root/p0000.html"))
        .expect("root content page should be readable");
    assert!(valid_html.contains("Included text from preprocess include."));
    assert!(!valid_html.contains("{{#include included.md}}"));
    fs::remove_dir_all(&valid_out).expect("temp valid output should be removed");

    let invalid_config = fixture_root.join("invalid-preprocess/bookshelf.toml");
    let invalid_out = make_temp_dir("chunk-009-preprocess-invalid");
    let err = match build_html_site(&invalid_config, &invalid_out) {
        Ok(_) => panic!("invalid preprocess fixture must fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string()
            .contains("book 'root' failed to preprocess for renderer 'markdown'"),
        "expected deterministic book-scoped preprocess failure, got: {}",
        err
    );
    if invalid_out.exists() {
        fs::remove_dir_all(&invalid_out).expect("temp invalid output should be removed");
    }
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
