use mdbook_bookshelf::{
    build_input_catalog, build_site_model, load_books_from_catalog, SitePageKind,
};
use mdbook_driver::book::BookItem;
use std::path::PathBuf;

#[test]
fn site_model_build() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("tests/fixtures/site-model/valid/bookshelf.toml");

    let catalog = build_input_catalog(&config_path).expect("catalog should build");
    let loaded = load_books_from_catalog(&catalog).expect("books should load");
    let model = build_site_model(&catalog, &loaded).expect("site model should build");

    assert_eq!("root", model.root_book_id);
    assert_eq!("bookshelf:root", model.synthetic_bookshelf_page_id);
    assert_eq!(2, model.books.len());
    assert_eq!("root", model.books[0].book_id);
    assert_eq!("modules/child", model.books[1].book_id);
    assert!(model.books[0].is_root_book);
    assert!(!model.books[1].is_root_book);

    let synthetic_pages: Vec<_> = model
        .pages
        .iter()
        .filter(|p| p.kind == SitePageKind::SyntheticBookshelf)
        .collect();
    assert_eq!(
        1,
        synthetic_pages.len(),
        "must contain exactly one synthetic page"
    );
    let synthetic = synthetic_pages[0];
    assert_eq!("bookshelf:root", synthetic.page_id);
    assert_eq!("root", synthetic.owning_book_id);
    assert_eq!(None, synthetic.order_in_book);
    assert!(
        model
            .books
            .iter()
            .all(|book| !book.page_ids_in_order.contains(&synthetic.page_id)),
        "synthetic page must not be represented as a content-book entry"
    );

    let content_pages: Vec<_> = model
        .pages
        .iter()
        .filter(|p| p.kind == SitePageKind::Content)
        .collect();
    let loaded_content_count = loaded
        .books
        .iter()
        .map(|book| {
            book.mdbook
                .iter()
                .filter(|item| matches!(item, BookItem::Chapter(ch) if ch.path.is_some()))
                .count()
        })
        .sum::<usize>();
    assert_eq!(loaded_content_count, content_pages.len());

    for book in &model.books {
        for (idx, page_id) in book.page_ids_in_order.iter().enumerate() {
            let page = model
                .pages
                .iter()
                .find(|p| &p.page_id == page_id)
                .expect("book page id should resolve");
            assert_eq!(SitePageKind::Content, page.kind);
            assert_eq!(book.book_id, page.owning_book_id);
            assert_eq!(Some(idx), page.order_in_book);
        }
    }
}
