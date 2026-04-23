use mdbook_bookshelf::{build_input_catalog, load_books_from_catalog};
use mdbook_driver::book::BookItem;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn multi_book_load() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = repo_root.join("tests/fixtures/multi-book-load");

    let valid_catalog = build_input_catalog(fixtures.join("valid/bookshelf.toml"))
        .expect("valid fixture should build input catalog");
    let loaded = load_books_from_catalog(&valid_catalog).expect("valid fixture should load books");

    assert_eq!("root-book/docs", loaded.root_book_id);
    assert_eq!(2, loaded.books.len());
    assert_eq!("root-book/docs", loaded.books[0].book_id);
    assert_eq!("modules/child/docs", loaded.books[1].book_id);
    assert!(
        !loaded.books[0].summary.numbered_chapters.is_empty(),
        "root summary should contain chapter entries"
    );
    assert!(
        !loaded.books[1].summary.numbered_chapters.is_empty(),
        "child summary should contain chapter entries"
    );

    let root_chapter_count = loaded.books[0]
        .mdbook
        .iter()
        .filter(|item| matches!(item, BookItem::Chapter(_)))
        .count();
    let child_chapter_count = loaded.books[1]
        .mdbook
        .iter()
        .filter(|item| matches!(item, BookItem::Chapter(_)))
        .count();
    assert!(root_chapter_count > 0, "root mdbook should load chapters");
    assert!(child_chapter_count > 0, "child mdbook should load chapters");

    let invalid_catalog =
        build_input_catalog(fixtures.join("invalid-summary-parse/bookshelf.toml"))
            .expect("invalid-summary-parse fixture should build catalog");
    let parse_err = match load_books_from_catalog(&invalid_catalog) {
        Ok(_) => panic!("must fail"),
        Err(err) => err,
    };
    assert_eq!(
        format!(
            "book 'broken-book/docs' failed to parse canonical summary at {}",
            fixtures
                .join("invalid-summary-parse/broken-book/docs/SUMMARY.md")
                .display()
        ),
        parse_err.to_string()
    );

    let invalid_load_catalog =
        build_input_catalog(fixtures.join("invalid-mdbook-load/bookshelf.toml"))
            .expect("invalid-mdbook-load fixture should build catalog");
    let load_err = match load_books_from_catalog(&invalid_load_catalog) {
        Ok(_) => panic!("must fail"),
        Err(err) => err,
    };
    assert_eq!(
        format!(
            "book 'missing-book/docs' failed to load mdbook from root {} and source {}",
            fixtures.join("invalid-mdbook-load/missing-book").display(),
            fixtures
                .join("invalid-mdbook-load/missing-book/docs")
                .display()
        ),
        load_err.to_string()
    );
}

#[test]
fn rejects_nested_authored_bookshelf_page_in_child_book() {
    let temp = TempDir::new("chunk-06d-authored-nested-child-bookshelf");
    write_file(
        temp.path(),
        "docs/SUMMARY.md",
        "# Summary\n\n- [Root](index.md)\n",
    );
    write_file(temp.path(), "docs/index.md", "# Root\n");
    write_file(
        temp.path(),
        "modules/child/docs/SUMMARY.md",
        "# Summary\n\n- [Child](index.md)\n- [Guide](guide/bookshelf.md)\n",
    );
    write_file(temp.path(), "modules/child/docs/index.md", "# Child\n");
    write_file(
        temp.path(),
        "modules/child/docs/guide/bookshelf.md",
        "# Nested Child Bookshelf\n",
    );
    let config_path = write_file(
        temp.path(),
        "bookshelf.toml",
        r#"
[book]
title = "Root"
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "Child"
src = "modules/child/docs"
"#,
    );

    let catalog = build_input_catalog(&config_path).expect("catalog should build");
    let error = match load_books_from_catalog(&catalog) {
        Ok(_) => panic!("reserved nested child bookshelf page must fail"),
        Err(error) => error,
    };
    let error_text = format!("{error:#}");
    assert!(
        error_text.contains(
            "book 'modules/child/docs' already contains reserved bookshelf path 'bookshelf.md'"
        ),
        "unexpected error: {error_text}"
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
