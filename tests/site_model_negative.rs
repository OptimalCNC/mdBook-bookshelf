use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::{build_site_model, load_input_catalog, BuildSiteModelError};

#[test]
fn rejects_missing_markdown_page_targets() {
    let root = make_fixture(
        "missing_markdown_target",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"
"#,
        &[("docs/SUMMARY.md", "# Summary\n\n- [Meta](missing.md)\n")],
    );
    let input_catalog = load_input_catalog(root.join("bookshelf.toml")).unwrap();

    let error = build_site_model(&input_catalog).unwrap_err();
    assert!(matches!(
        error,
        BuildSiteModelError::MissingPageTarget {
            ref book_id,
            ref summary_path,
            ref target,
            ref source_path,
            ..
        } if book_id == "meta"
            && summary_path.ends_with(Path::new("docs/SUMMARY.md"))
            && target == "missing.md"
            && source_path.ends_with(Path::new("docs/missing.md"))
    ));
}

#[test]
fn rejects_existing_non_markdown_page_targets() {
    let root = make_fixture(
        "non_markdown_target",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"
"#,
        &[
            ("docs/SUMMARY.md", "# Summary\n\n- [Meta](index.txt)\n"),
            ("docs/index.txt", "not markdown\n"),
        ],
    );
    let input_catalog = load_input_catalog(root.join("bookshelf.toml")).unwrap();

    let error = build_site_model(&input_catalog).unwrap_err();
    assert!(matches!(
        error,
        BuildSiteModelError::NonMarkdownPageTarget {
            ref book_id,
            ref summary_path,
            ref target,
            ref source_path,
            ..
        } if book_id == "meta"
            && summary_path.ends_with(Path::new("docs/SUMMARY.md"))
            && target == "index.txt"
            && source_path.ends_with(Path::new("docs/index.txt"))
    ));
}

#[test]
fn rejects_duplicate_page_ownership_across_books() {
    let root = make_fixture(
        "duplicate_page_ownership",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"

[[bookshelf.book]]
id = "parser"
title = "Parser"
summary = "modules/parser/SUMMARY.md"
"#,
        &[
            ("docs/SUMMARY.md", "# Summary\n\n- [Meta](index.md)\n"),
            (
                "modules/parser/SUMMARY.md",
                "# Summary\n\n- [Parser](../../docs/index.md)\n",
            ),
            ("docs/index.md", "# Shared page\n"),
        ],
    );
    let input_catalog = load_input_catalog(root.join("bookshelf.toml")).unwrap();

    let error = build_site_model(&input_catalog).unwrap_err();
    assert!(matches!(
        error,
        BuildSiteModelError::DuplicatePageOwnership {
            ref source_path,
            ref first_book_id,
            ref second_book_id,
        } if source_path.ends_with(Path::new("docs/index.md"))
            && first_book_id == "meta"
            && second_book_id == "parser"
    ));
}

fn make_fixture(name: &str, config: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = temp_root().join(unique_fixture_name(name));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("bookshelf.toml"), config).unwrap();

    for (relative_path, contents) in files {
        let full_path = root.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, contents).unwrap();
    }

    root
}

fn temp_root() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
}

fn unique_fixture_name(name: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{name}-{timestamp}-{counter}")
}
