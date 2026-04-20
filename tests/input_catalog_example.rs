use std::path::{Path, PathBuf};

use mdbook_bookshelf::load_input_catalog;

#[test]
fn loads_the_self_contained_example_catalog() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");

    let catalog = load_input_catalog(&config_path).expect("self-contained example should load");

    assert_eq!(catalog.root_book, "meta");
    assert_eq!(catalog.books.len(), 3);
    assert_eq!(book_ids(&catalog), vec!["meta", "parser", "ui"]);

    let config_dir = config_path.parent().unwrap();
    assert_eq!(
        relative_root(&catalog, "meta", config_dir),
        Path::new("docs/index.md")
    );
    assert_eq!(
        relative_root(&catalog, "parser", config_dir),
        Path::new("modules/parser/docs/index.md")
    );
    assert_eq!(
        relative_root(&catalog, "ui", config_dir),
        Path::new("modules/ui/docs/index.md")
    );
}

fn book_ids(catalog: &mdbook_bookshelf::InputCatalog) -> Vec<&str> {
    catalog.books.iter().map(|book| book.id.as_str()).collect()
}

fn relative_root<'a>(
    catalog: &'a mdbook_bookshelf::InputCatalog,
    book_id: &str,
    config_dir: &Path,
) -> &'a Path {
    catalog
        .books
        .iter()
        .find(|book| book.id == book_id)
        .expect("book must exist")
        .content_root
        .strip_prefix(config_dir)
        .expect("content root should resolve from config dir")
}
