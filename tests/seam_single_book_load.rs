use mdbook_bookshelf::load_single_book_with_summary;
use mdbook_driver::book::BookItem;

const BOOKSHELF_UI_SITE_FIXTURE: &str = "tests/fixtures/bookshelf-ui-site";

#[test]
fn seam_single_book_load_reads_fixture_summary() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let book_root = repo_root.join(BOOKSHELF_UI_SITE_FIXTURE);

    let loaded_book =
        load_single_book_with_summary(&book_root, "docs").expect("single-book seam should load");

    let chapter_count = loaded_book
        .iter()
        .filter(|item| matches!(item, BookItem::Chapter(_)))
        .count();
    assert!(chapter_count > 0, "expected at least one chapter");

    let has_root_chapter = loaded_book
        .iter()
        .any(|item| matches!(item, BookItem::Chapter(ch) if ch.name == "Fixture Core"));
    assert!(
        has_root_chapter,
        "expected summary-defined chapter 'Fixture Core'"
    );
}
