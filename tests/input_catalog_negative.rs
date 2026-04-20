use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::{load_input_catalog, LoadInputCatalogError};

#[test]
fn rejects_missing_config_file() {
    let error = load_input_catalog(temp_root().join("missing/bookshelf.toml")).unwrap_err();
    assert!(matches!(error, LoadInputCatalogError::MissingConfig { .. }));
}

#[test]
fn rejects_missing_root_book() {
    let root = make_fixture(
        "missing_root_book",
        r#"[bookshelf]

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"
"#,
        &[("docs/SUMMARY.md", "# Summary\n\n- [Meta](index.md)\n")],
    );

    let error = load_input_catalog(root.join("bookshelf.toml")).unwrap_err();
    assert!(matches!(
        error,
        LoadInputCatalogError::MissingRootBook { .. }
    ));
}

#[test]
fn rejects_duplicate_book_ids() {
    let root = make_fixture(
        "duplicate_book_ids",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"

[[bookshelf.book]]
id = "meta"
title = "Meta Again"
summary = "other/SUMMARY.md"
"#,
        &[
            ("docs/SUMMARY.md", "# Summary\n\n- [Meta](index.md)\n"),
            ("other/SUMMARY.md", "# Summary\n\n- [Other](index.md)\n"),
        ],
    );

    let error = load_input_catalog(root.join("bookshelf.toml")).unwrap_err();
    assert!(matches!(
        error,
        LoadInputCatalogError::DuplicateBookId { ref book_id } if book_id == "meta"
    ));
}

#[test]
fn rejects_missing_summary_field() {
    let root = make_fixture(
        "missing_summary_field",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
"#,
        &[],
    );

    let error = load_input_catalog(root.join("bookshelf.toml")).unwrap_err();
    assert!(matches!(
        error,
        LoadInputCatalogError::MissingSummary { ref book_id } if book_id == "meta"
    ));
}

#[test]
fn rejects_empty_summary_file() {
    let root = make_fixture(
        "empty_summary_file",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"
"#,
        &[("docs/SUMMARY.md", "# Summary\n")],
    );

    let error = load_input_catalog(root.join("bookshelf.toml")).unwrap_err();
    assert!(matches!(
        error,
        LoadInputCatalogError::EmptySummary {
            ref book_id,
            ref summary_path,
        } if book_id == "meta" && summary_path.ends_with(Path::new("docs/SUMMARY.md"))
    ));
}

#[test]
fn rejects_root_book_that_is_not_configured() {
    let root = make_fixture(
        "missing_root_book_entry",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "parser"
title = "Parser"
summary = "docs/SUMMARY.md"
"#,
        &[("docs/SUMMARY.md", "# Summary\n\n- [Parser](index.md)\n")],
    );

    let error = load_input_catalog(root.join("bookshelf.toml")).unwrap_err();
    assert!(matches!(
        error,
        LoadInputCatalogError::RootBookNotConfigured { ref root_book_id } if root_book_id == "meta"
    ));
}

#[test]
fn rejects_authored_bookshelf_entries_in_canonical_summary() {
    let root = make_fixture(
        "authored_bookshelf_entry",
        r#"[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"
"#,
        &[(
            "docs/SUMMARY.md",
            "# Summary\n\n- [Meta](index.md)\n- [Bookshelf](bookshelf.md)\n",
        )],
    );

    let error = load_input_catalog(root.join("bookshelf.toml")).unwrap_err();
    assert!(matches!(
        error,
        LoadInputCatalogError::AuthoredBookshelfEntry {
            ref book_id,
            ref summary_path,
            ..
        } if book_id == "meta" && summary_path.ends_with(Path::new("docs/SUMMARY.md"))
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
