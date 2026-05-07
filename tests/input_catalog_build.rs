use mdbook_bookshelf::build_input_catalog;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn builds_catalog_with_explicit_ids_and_categories() {
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
title = "Documentation"
src = "site-default"

[bookshelf]

[[bookshelf.book]]
id = "root"
title = "Root Book"
src = "root-book/docs"

[[bookshelf.book]]
id = "child"
title = "Child Book"
src = "modules/child-book/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root", "child"]
"#,
    );

    let catalog = build_input_catalog(&config_path).expect("valid fixture should build catalog");
    assert_eq!(2, catalog.books.len());
    assert_eq!(Path::new(".mdbook/bookshelf"), catalog.asset_dir.as_path());

    let root = &catalog.books[0];
    assert_eq!("root", root.id);
    assert_eq!(Path::new("root-book/docs"), root.output_rel.as_path());
    assert_eq!(Path::new("."), root.book_root_rel.as_path());
    assert_eq!(Path::new("root-book/docs"), root.book_src_rel.as_path());
    assert_eq!(
        temp.path().join("root-book/docs/SUMMARY.md"),
        root.summary_abs
    );

    let child = &catalog.books[1];
    assert_eq!("child", child.id);
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
    assert_eq!(1, catalog.categories.len());
    assert_eq!("All Docs", catalog.categories[0].title);
    assert_eq!(vec!["root", "child"], catalog.categories[0].book_ids);
}

#[test]
fn builds_single_explicit_book_catalog_from_book_src() {
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
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "root"
title = "Meta Root"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root"]
"#,
    );

    let catalog = build_input_catalog(&config_path).expect("top-level docs fixture should build");
    assert_eq!(1, catalog.books.len());

    let root = &catalog.books[0];
    assert_eq!("root", root.id);
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
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "root"
title = "Root Book"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root"]
"#,
    );

    let error = build_input_catalog(&config_path).expect_err("missing summary must fail");
    assert_eq!(
        format!(
            "book 'root' missing canonical summary 'docs/SUMMARY.md' derived from src 'docs' at {}",
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
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "root"
title = "Root Book"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root"]
"#,
    );

    let error = build_input_catalog(&config_path).expect_err("wrong-location summary must fail");
    assert_eq!(
        format!(
            "book 'root' missing canonical summary 'docs/SUMMARY.md' derived from src 'docs' at {}",
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
