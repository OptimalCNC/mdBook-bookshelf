use mdbook_bookshelf::{
    build_input_catalog, build_navigation_metadata, build_search_documents, build_site_model,
    load_books_from_catalog,
};
use std::path::PathBuf;

#[test]
fn search_model_build() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("tests/fixtures/search-model/valid/bookshelf.toml");

    let catalog = build_input_catalog(&config_path).expect("catalog should build");
    let loaded = load_books_from_catalog(&catalog).expect("books should load");
    let site_model = build_site_model(&catalog, &loaded).expect("site model should build");
    let nav = build_navigation_metadata(&site_model).expect("navigation should build");
    let search = build_search_documents(&site_model, &nav).expect("search model should build");

    assert_eq!(4, search.documents.len(), "site-wide search should include all content pages");
    assert!(search.documents.iter().all(|doc| !doc.owning_book_label.is_empty()));
    assert!(
        search.documents
            .iter()
            .all(|doc| doc.page_id != site_model.synthetic_bookshelf_page_id),
        "synthetic bookshelf page must not be searchable"
    );

    let ids: Vec<_> = search
        .documents
        .iter()
        .map(|doc| doc.page_id.as_str())
        .collect();
    assert_eq!(vec!["root:0000", "root:0001", "child:0000", "child:0001"], ids);

    assert_eq!("Root Book", search.documents[0].owning_book_label);
    assert_eq!("Child Book", search.documents[2].owning_book_label);

    let search_again = build_search_documents(&site_model, &nav).expect("repeat should build");
    assert_eq!(search.documents, search_again.documents);
}
