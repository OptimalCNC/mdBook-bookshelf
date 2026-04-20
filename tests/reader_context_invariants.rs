use std::path::PathBuf;

use mdbook_bookshelf::{
    build_reader_context_model, build_sidebar_model, build_site_model, load_input_catalog,
};

#[test]
fn keeps_prev_next_book_local_and_preserves_existing_models() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let config_dir = config_path.parent().unwrap();
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let original_authored_orders: Vec<_> = site_model
        .books
        .iter()
        .map(|book| (book.id.clone(), book.authored_page_order.clone()))
        .collect();
    let original_sidebars = sidebar_model.sidebars.clone();

    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");

    for context in &reader_context.authored_page_contexts {
        if let Some(previous) = &context.previous_page {
            let previous_context = reader_context
                .authored_context_for_source_path(previous.source_path.clone())
                .expect("previous context should exist");
            assert_eq!(
                previous_context.active_book.book_id,
                context.active_book.book_id
            );
        }
        if let Some(next) = &context.next_page {
            let next_context = reader_context
                .authored_context_for_source_path(next.source_path.clone())
                .expect("next context should exist");
            assert_eq!(
                next_context.active_book.book_id,
                context.active_book.book_id
            );
        }
        assert_eq!(context.bookshelf_return.page_id, "bookshelf");
    }

    for (book_id, authored_page_order) in &original_authored_orders {
        let book = site_model
            .books
            .iter()
            .find(|book| &book.id == book_id)
            .expect("book should exist");
        assert_eq!(&book.authored_page_order, authored_page_order);
    }
    assert_eq!(sidebar_model.sidebars, original_sidebars);

    let deep_link = reader_context
        .authored_context_for_source_path(config_dir.join("modules/parser/docs/./grammar.md"))
        .expect("deep-link parser context should exist");
    assert_eq!(deep_link.active_book.book_id, "parser");
    assert_eq!(deep_link.breadcrumbs.book_title, "Example Parser");
    assert_eq!(deep_link.breadcrumbs.page_title, "Grammar");
}
