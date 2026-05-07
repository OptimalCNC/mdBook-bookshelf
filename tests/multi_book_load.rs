use mdbook_bookshelf::{build_input_catalog, load_books_from_catalog};
use mdbook_driver::book::BookItem;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn multi_book_load() {
    let temp = TempDir::new("multi-book-load");

    write_file(
        temp.path(),
        "valid/root-book/docs/SUMMARY.md",
        "# Summary\n\n- [Root Intro](index.md)\n",
    );
    write_file(
        temp.path(),
        "valid/root-book/docs/index.md",
        "# Root Intro\n",
    );
    write_file(
        temp.path(),
        "valid/modules/child/docs/SUMMARY.md",
        "# Summary\n\n- [Child Intro](index.md)\n",
    );
    write_file(
        temp.path(),
        "valid/modules/child/docs/index.md",
        "# Child Intro\n",
    );
    let valid_config = write_file(
        temp.path(),
        "valid/bookshelf.toml",
        r#"[book]
	title = "Documentation"

	[bookshelf]

	[[bookshelf.book]]
	id = "root"
	title = "Root Book"
	src = "root-book/docs"

	[[bookshelf.book]]
	id = "child"
title = "Child Book"
src = "modules/child/docs"

[[bookshelf.category]]
title = "Main"
books = ["root", "child"]
"#,
    );

    let valid_catalog =
        build_input_catalog(&valid_config).expect("valid fixture should build input catalog");
    let loaded = load_books_from_catalog(&valid_catalog).expect("valid fixture should load books");

    assert_eq!(2, loaded.books.len());
    assert_eq!("root", loaded.books[0].book_id);
    assert_eq!("child", loaded.books[1].book_id);
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

    write_file(
        temp.path(),
        "invalid-summary-parse/root-book/docs/SUMMARY.md",
        "# Summary\n\n- [Root Intro](index.md)\n",
    );
    write_file(
        temp.path(),
        "invalid-summary-parse/root-book/docs/index.md",
        "# Root Intro\n",
    );
    write_file(
        temp.path(),
        "invalid-summary-parse/broken-book/docs/SUMMARY.md",
        "# Summary\n\n- [Broken Chapter](chapter.md\n",
    );
    write_file(
        temp.path(),
        "invalid-summary-parse/broken-book/docs/chapter.md",
        "# Broken Chapter\n",
    );
    let invalid_summary_config = write_file(
        temp.path(),
        "invalid-summary-parse/bookshelf.toml",
        r#"[book]
	title = "Documentation"

	[bookshelf]

	[[bookshelf.book]]
	id = "root"
	title = "Root Book"
	src = "root-book/docs"

	[[bookshelf.book]]
	id = "broken"
title = "Broken Book"
src = "broken-book/docs"

[[bookshelf.category]]
title = "Main"
books = ["root", "broken"]
"#,
    );
    let invalid_catalog = build_input_catalog(&invalid_summary_config)
        .expect("invalid-summary-parse fixture should build catalog");
    let parse_err = match load_books_from_catalog(&invalid_catalog) {
        Ok(_) => panic!("must fail"),
        Err(err) => err,
    };
    assert_eq!(
        format!(
            "book 'broken' failed to parse canonical summary at {}",
            temp.path()
                .join("invalid-summary-parse/broken-book/docs/SUMMARY.md")
                .display()
        ),
        parse_err.to_string()
    );

    write_file(
        temp.path(),
        "invalid-mdbook-load/root-book/docs/SUMMARY.md",
        "# Summary\n\n- [Root Intro](index.md)\n",
    );
    write_file(
        temp.path(),
        "invalid-mdbook-load/root-book/docs/index.md",
        "# Root Intro\n",
    );
    write_file(
        temp.path(),
        "invalid-mdbook-load/missing-book/docs/SUMMARY.md",
        "# Summary\n\n- [Missing Chapter](missing.md)\n",
    );
    let invalid_load_config = write_file(
        temp.path(),
        "invalid-mdbook-load/bookshelf.toml",
        r#"[book]
	title = "Documentation"

	[bookshelf]

	[[bookshelf.book]]
	id = "root"
	title = "Root Book"
	src = "root-book/docs"

	[[bookshelf.book]]
	id = "missing"
title = "Missing Chapter Book"
src = "missing-book/docs"

[[bookshelf.category]]
title = "Main"
books = ["root", "missing"]
"#,
    );
    let invalid_load_catalog = build_input_catalog(&invalid_load_config)
        .expect("invalid-mdbook-load fixture should build catalog");
    let load_err = match load_books_from_catalog(&invalid_load_catalog) {
        Ok(_) => panic!("must fail"),
        Err(err) => err,
    };
    assert_eq!(
        format!(
            "book 'missing' failed to load mdbook from root {} and source {}",
            temp.path().join("invalid-mdbook-load").display(),
            temp.path()
                .join("invalid-mdbook-load/missing-book/docs")
                .display()
        ),
        load_err.to_string()
    );
}

#[test]
fn loads_authored_bookshelf_markdown_pages() {
    let temp = TempDir::new("authored-bookshelf-page");
    write_file(
        temp.path(),
        "docs/SUMMARY.md",
        "# Summary\n\n- [Root](index.md)\n",
    );
    write_file(temp.path(), "docs/index.md", "# Root\n");
    write_file(
        temp.path(),
        "modules/child/docs/SUMMARY.md",
        "# Summary\n\n- [Child](index.md)\n- [Guide Bookshelf](guide/bookshelf.md)\n",
    );
    write_file(temp.path(), "modules/child/docs/index.md", "# Child\n");
    write_file(
        temp.path(),
        "modules/child/docs/guide/bookshelf.md",
        "# Authored Bookshelf\n",
    );
    let config_path = write_file(
        temp.path(),
        "bookshelf.toml",
        r#"[book]
	title = "Documentation"

	[bookshelf]

	[[bookshelf.book]]
	id = "root"
	title = "Root"
	src = "docs"

	[[bookshelf.book]]
	id = "child"
title = "Child"
src = "modules/child/docs"

[[bookshelf.category]]
title = "Main"
books = ["root", "child"]
"#,
    );

    let catalog = build_input_catalog(&config_path).expect("catalog should build");
    let loaded = load_books_from_catalog(&catalog).expect("authored bookshelf.md should load");
    let child = loaded
        .books
        .iter()
        .find(|book| book.book_id == "child")
        .expect("child book should be loaded");

    assert!(
        child.mdbook.iter().any(|item| {
            matches!(
                item,
                BookItem::Chapter(chapter)
                    if chapter.path.as_deref() == Some(Path::new("guide/bookshelf.md"))
            )
        }),
        "child book should include authored guide/bookshelf.md chapter"
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
