use mdbook_bookshelf::load_single_book_with_summary;
use mdbook_driver::book::BookItem;

#[test]
fn seam_single_book_load() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let book_root = repo_root.join("bookshelf/examples/self-contained");

    let loaded_book =
        load_single_book_with_summary(&book_root, "docs").expect("single-book seam should load");

    let chapter_count = loaded_book
        .iter()
        .filter(|item| matches!(item, BookItem::Chapter(_)))
        .count();
    assert!(chapter_count > 0, "expected at least one chapter");

    let has_root_chapter = loaded_book
        .iter()
        .any(|item| matches!(item, BookItem::Chapter(ch) if ch.name == "Example Core"));
    assert!(
        has_root_chapter,
        "expected summary-defined chapter 'Example Core'"
    );
}
