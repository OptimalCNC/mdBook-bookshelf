use mdbook_bookshelf::{build_input_catalog, load_books_from_catalog};
use mdbook_driver::book::BookItem;
use std::path::PathBuf;

#[test]
fn multi_book_load() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = repo_root.join("tests/fixtures/multi-book-load");

    let valid_catalog = build_input_catalog(fixtures.join("valid/bookshelf.toml"))
        .expect("valid fixture should build input catalog");
    let loaded = load_books_from_catalog(&valid_catalog).expect("valid fixture should load books");

    assert_eq!("root", loaded.root_book_id);
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

    let invalid_catalog =
        build_input_catalog(fixtures.join("invalid-summary-parse/bookshelf.toml"))
            .expect("invalid-summary-parse fixture should build catalog");
    let parse_err = match load_books_from_catalog(&invalid_catalog) {
        Ok(_) => panic!("must fail"),
        Err(err) => err,
    };
    assert_eq!(
        format!(
            "book 'broken' failed to parse canonical summary at {}",
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
            "book 'missing' failed to load mdbook from root {} and source {}",
            fixtures.join("invalid-mdbook-load/missing-book").display(),
            fixtures
                .join("invalid-mdbook-load/missing-book/docs")
                .display()
        ),
        load_err.to_string()
    );
}
