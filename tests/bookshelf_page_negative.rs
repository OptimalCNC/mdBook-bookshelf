use std::path::Path;
use std::path::PathBuf;

use mdbook_bookshelf::{build_site_model, load_input_catalog, BuildSiteModelError};

#[test]
fn rejects_books_that_cannot_link_to_their_root_authored_page() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let mut input_catalog = load_input_catalog(&config_path).expect("input catalog should load");

    let broken_book = input_catalog
        .books
        .iter_mut()
        .find(|book| book.id == "parser")
        .expect("parser book should exist");
    broken_book.content_root = config_path
        .parent()
        .unwrap()
        .join("modules/parser/docs/missing.md");

    let error = build_site_model(&input_catalog).unwrap_err();
    assert!(matches!(
        error,
        BuildSiteModelError::MissingBookshelfRootLink {
            ref book_id,
            ref content_root,
        } if book_id == "parser"
            && content_root.ends_with(Path::new("modules/parser/docs/missing.md"))
    ));
}
