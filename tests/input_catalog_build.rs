use mdbook_bookshelf::build_input_catalog;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn builds_source_derived_catalog_for_root_and_child_books() {
    let temp = TempDir::new("chunk-06a-input-catalog-valid");
    write_file(
        temp.path(),
        "root-book/docs/SUMMARY.md",
        "# Summary\n\n- [Root Intro](index.md)\n",
    );
    write_file(
        temp.path(),
        "modules/child-book/docs/SUMMARY.md",
        "# Summary\n\n- [Child Intro](index.md)\n",
    );
    let config_path = write_file(
        temp.path(),
        "bookshelf.toml",
        r#"
[book]
title = "Root Book"
src = "root-book/docs"

[bookshelf]

[[bookshelf.book]]
title = "Child Book"
src = "modules/child-book/docs"
"#,
    );

    let catalog = build_input_catalog(&config_path).expect("valid fixture should build catalog");
    assert_eq!(2, catalog.books.len());
    assert_eq!(Path::new(".mdbook/bookshelf"), catalog.asset_dir.as_path());

    let root = catalog.root_book().expect("root book should exist");
    assert_eq!("root-book/docs", root.id);
    assert!(root.is_root_book);
    assert_eq!(Path::new("root-book/docs"), root.output_rel.as_path());
    assert_eq!(Path::new("."), root.book_root_rel.as_path());
    assert_eq!(Path::new("root-book/docs"), root.book_src_rel.as_path());
    assert_eq!(
        temp.path().join("root-book/docs/SUMMARY.md"),
        root.summary_abs
    );

    let child = catalog
        .books
        .iter()
        .find(|book| !book.is_root_book)
        .expect("child book should exist");
    assert_eq!("modules/child-book/docs", child.id);
    assert_eq!(
        Path::new("modules/child-book/docs"),
        child.output_rel.as_path()
    );
    assert_eq!(Path::new("."), child.book_root_rel.as_path());
    assert_eq!(
        Path::new("modules/child-book/docs"),
        child.book_src_rel.as_path()
    );
    assert_eq!(temp.path(), child.book_root_abs);
    assert_eq!(
        temp.path().join("modules/child-book/docs"),
        child.book_src_abs
    );
    assert_eq!(
        temp.path().join("modules/child-book/docs/SUMMARY.md"),
        child.summary_abs
    );
}

#[test]
fn builds_top_level_root_catalog_from_book_src() {
    let temp = TempDir::new("chunk-06a-input-catalog-top-level");
    write_file(
        temp.path(),
        "docs/SUMMARY.md",
        "# Summary\n\n- [Overview](index.md)\n",
    );
    let config_path = write_file(
        temp.path(),
        "bookshelf.toml",
        r#"
[book]
title = "Meta Root"
src = "docs"

[bookshelf]
"#,
    );

    let catalog = build_input_catalog(&config_path).expect("top-level docs fixture should build");
    assert_eq!(1, catalog.books.len());

    let root = catalog.root_book().expect("root book should exist");
    assert_eq!("docs", root.id);
    assert!(root.is_root_book);
    assert_eq!(Path::new("."), root.book_root_rel.as_path());
    assert_eq!(temp.path(), root.book_root_abs);
    assert_eq!(Path::new("docs"), root.book_src_rel.as_path());
    assert_eq!(Path::new("docs"), root.output_rel.as_path());
    assert_eq!(temp.path().join("docs/SUMMARY.md"), root.summary_abs);
}

#[test]
fn rejects_missing_canonical_summary_under_source_derived_root() {
    let temp = TempDir::new("chunk-06a-input-catalog-missing-summary");
    let config_path = write_file(
        temp.path(),
        "bookshelf.toml",
        r#"
[book]
title = "Root Book"
src = "docs"

[bookshelf]
"#,
    );

    let error = build_input_catalog(&config_path).expect_err("missing summary must fail");
    assert_eq!(
        format!(
            "book 'docs' missing canonical summary 'docs/SUMMARY.md' derived from src 'docs' at {}",
            temp.path().join("docs/SUMMARY.md").display()
        ),
        error.to_string()
    );
}

#[test]
fn rejects_summary_in_wrong_location() {
    let temp = TempDir::new("chunk-06a-input-catalog-summary-location");
    write_file(
        temp.path(),
        "src/SUMMARY.md",
        "# Summary\n\n- [Wrong Place](index.md)\n",
    );
    let config_path = write_file(
        temp.path(),
        "bookshelf.toml",
        r#"
[book]
title = "Root Book"
src = "docs"

[bookshelf]
"#,
    );

    let error = build_input_catalog(&config_path).expect_err("wrong-location summary must fail");
    assert_eq!(
        format!(
            "book 'docs' missing canonical summary 'docs/SUMMARY.md' derived from src 'docs' at {}",
            temp.path().join("docs/SUMMARY.md").display()
        ),
        error.to_string()
    );
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir()
            .join("mdbook-bookshelf")
            .join(root.file_name().unwrap_or_default())
            .join(format!("{tag}-{nanos}"));
        fs::create_dir_all(&path).expect("temp fixture directory should be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_file(dir: &Path, rel: &str, content: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(&path, content).expect("fixture file should be written");
    path
}
