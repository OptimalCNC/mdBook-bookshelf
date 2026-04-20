use std::path::{Path, PathBuf};

use mdbook_bookshelf::{
    build_reader_context_model, build_sidebar_model, build_site_model, load_input_catalog,
};

#[test]
fn builds_expected_reader_context_for_representative_pages() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let config_dir = config_path.parent().unwrap();
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");

    assert_eq!(reader_context.authored_page_contexts.len(), 9);

    let parser_grammar = reader_context
        .authored_context_for_source_path(config_dir.join("modules/parser/docs/grammar.md"))
        .expect("parser grammar context should exist");
    assert_eq!(parser_grammar.active_book.book_id, "parser");
    assert_eq!(parser_grammar.breadcrumbs.book_title, "Example Parser");
    assert_eq!(parser_grammar.breadcrumbs.page_title, "Grammar");
    assert_eq!(
        relative_path(
            parser_grammar
                .previous_page
                .as_ref()
                .map(|link| &link.source_path),
            config_dir
        ),
        Some(PathBuf::from("modules/parser/docs/index.md"))
    );
    assert_eq!(
        relative_path(
            parser_grammar
                .next_page
                .as_ref()
                .map(|link| &link.source_path),
            config_dir
        ),
        Some(PathBuf::from("modules/parser/docs/runtime.md"))
    );
    assert_eq!(parser_grammar.bookshelf_return.page_id, "bookshelf");

    let architecture = reader_context
        .authored_context_for_source_path(config_dir.join("docs/architecture.md"))
        .expect("architecture context should exist");
    assert_eq!(architecture.active_book.book_id, "meta");
    assert_eq!(architecture.breadcrumbs.book_title, "Example Core");
    assert_eq!(architecture.breadcrumbs.page_title, "Architecture");
    assert_eq!(
        relative_path(
            architecture
                .previous_page
                .as_ref()
                .map(|link| &link.source_path),
            config_dir
        ),
        Some(PathBuf::from("docs/onboarding.md"))
    );
    assert!(architecture.next_page.is_none());

    let ui_index = reader_context
        .authored_context_for_source_path(config_dir.join("modules/ui/docs/index.md"))
        .expect("ui index context should exist");
    assert_eq!(ui_index.active_book.book_id, "ui");
    assert_eq!(ui_index.breadcrumbs.book_title, "Example UI");
    assert_eq!(ui_index.breadcrumbs.page_title, "Example UI");
    assert!(ui_index.previous_page.is_none());
    assert_eq!(
        relative_path(
            ui_index.next_page.as_ref().map(|link| &link.source_path),
            config_dir
        ),
        Some(PathBuf::from("modules/ui/docs/navigation.md"))
    );

    let bookshelf = reader_context
        .bookshelf_context_for_page_id("bookshelf")
        .expect("bookshelf context should exist");
    assert_eq!(bookshelf.active_book.book_id, "meta");
    assert_eq!(bookshelf.breadcrumbs.book_title, "Example Core");
    assert_eq!(bookshelf.breadcrumbs.page_title, "Bookshelf");
    assert!(bookshelf.previous_page.is_none());
    assert!(bookshelf.next_page.is_none());
}

fn relative_path(path: Option<&PathBuf>, config_dir: &Path) -> Option<PathBuf> {
    path.map(|path| {
        path.strip_prefix(config_dir)
            .expect("path should resolve from the config dir")
            .to_path_buf()
    })
}
