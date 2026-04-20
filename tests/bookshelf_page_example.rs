use std::path::{Path, PathBuf};

use mdbook_bookshelf::{build_site_model, load_input_catalog, SiteRoot};

#[test]
fn builds_the_generated_bookshelf_page_for_the_self_contained_example() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let config_dir = config_path.parent().unwrap();

    assert_eq!(
        site_model.root_entry,
        SiteRoot::BookshelfPage {
            page_id: "bookshelf".to_owned()
        }
    );
    assert_eq!(site_model.bookshelf_page.page_id, "bookshelf");
    assert_eq!(site_model.bookshelf_page.title, "Bookshelf");
    assert_eq!(
        site_model.bookshelf_page.route_path,
        PathBuf::from("bookshelf")
    );
    assert_eq!(site_model.bookshelf_page.owner_book_id, "meta");
    assert_eq!(site_model.bookshelf_page.shelf_items.len(), 3);
    assert_eq!(site_model.authored_pages.len(), 9);

    assert_eq!(
        authored_order(&site_model, "meta", config_dir),
        vec![
            PathBuf::from("docs/index.md"),
            PathBuf::from("docs/onboarding.md"),
            PathBuf::from("docs/architecture.md"),
        ]
    );
    assert_eq!(
        authored_order(&site_model, "parser", config_dir),
        vec![
            PathBuf::from("modules/parser/docs/index.md"),
            PathBuf::from("modules/parser/docs/grammar.md"),
            PathBuf::from("modules/parser/docs/runtime.md"),
        ]
    );
    assert_eq!(
        authored_order(&site_model, "ui", config_dir),
        vec![
            PathBuf::from("modules/ui/docs/index.md"),
            PathBuf::from("modules/ui/docs/navigation.md"),
            PathBuf::from("modules/ui/docs/diagnostics.md"),
        ]
    );

    assert_eq!(
        shelf_item(&site_model, "meta", config_dir,),
        (
            "Example Core".to_owned(),
            Some("Repository-wide onboarding and architecture notes.".to_owned()),
            PathBuf::from("docs/index.md"),
        )
    );
    assert_eq!(
        shelf_item(&site_model, "parser", config_dir),
        (
            "Example Parser".to_owned(),
            Some("Parser-specific reference pages with their own reading order.".to_owned()),
            PathBuf::from("modules/parser/docs/index.md"),
        )
    );
    assert_eq!(
        shelf_item(&site_model, "ui", config_dir),
        (
            "Example UI".to_owned(),
            Some("Interface and runtime guides for the UI book.".to_owned()),
            PathBuf::from("modules/ui/docs/index.md"),
        )
    );
}

fn authored_order(
    site_model: &mdbook_bookshelf::SiteModel,
    book_id: &str,
    config_dir: &Path,
) -> Vec<PathBuf> {
    site_model
        .books
        .iter()
        .find(|book| book.id == book_id)
        .expect("book should exist")
        .authored_page_order
        .iter()
        .map(|path| {
            path.strip_prefix(config_dir)
                .expect("authored pages should resolve from the config dir")
                .to_path_buf()
        })
        .collect()
}

fn shelf_item(
    site_model: &mdbook_bookshelf::SiteModel,
    book_id: &str,
    config_dir: &Path,
) -> (String, Option<String>, PathBuf) {
    let item = site_model
        .bookshelf_page
        .shelf_items
        .iter()
        .find(|item| item.book_id == book_id)
        .expect("shelf item should exist");

    (
        item.title.clone(),
        item.description.clone(),
        item.target_source_path
            .strip_prefix(config_dir)
            .expect("shelf item target should resolve from the config dir")
            .to_path_buf(),
    )
}
