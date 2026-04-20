use std::path::{Path, PathBuf};

use mdbook_bookshelf::{build_site_model, load_input_catalog};

#[test]
fn builds_the_self_contained_example_site_model() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");

    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let config_dir = config_path.parent().unwrap();

    assert_eq!(site_model.root_book, "meta");
    assert_eq!(site_model.authored_pages.len(), 9);
    assert_eq!(
        book_page_order(&site_model, "meta", config_dir),
        vec![
            PathBuf::from("docs/index.md"),
            PathBuf::from("docs/onboarding.md"),
            PathBuf::from("docs/architecture.md"),
        ]
    );
    assert_eq!(
        book_page_order(&site_model, "parser", config_dir),
        vec![
            PathBuf::from("modules/parser/docs/index.md"),
            PathBuf::from("modules/parser/docs/grammar.md"),
            PathBuf::from("modules/parser/docs/runtime.md"),
        ]
    );
    assert_eq!(
        book_page_order(&site_model, "ui", config_dir),
        vec![
            PathBuf::from("modules/ui/docs/index.md"),
            PathBuf::from("modules/ui/docs/navigation.md"),
            PathBuf::from("modules/ui/docs/diagnostics.md"),
        ]
    );

    for book_id in ["meta", "parser", "ui"] {
        let book_pages = pages_for_book(&site_model, book_id);
        assert_eq!(book_pages.len(), 3);
        assert_eq!(book_pages[0].order, 0);
        assert_eq!(book_pages[1].order, 1);
        assert_eq!(book_pages[2].order, 2);
        assert_eq!(book_pages[0].summary_depth, 0);
        assert_eq!(book_pages[1].summary_depth, 1);
        assert_eq!(book_pages[2].summary_depth, 1);
        for page in book_pages {
            assert_eq!(
                page.source_path
                    .strip_prefix(config_dir)
                    .expect("page should live under the example root"),
                site_model
                    .books
                    .iter()
                    .find(|book| book.id == book_id)
                    .expect("book must exist")
                    .authored_page_order[page.order]
                    .strip_prefix(config_dir)
                    .expect("book page should live under the example root"),
            );
        }
    }
}

fn book_page_order(
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
                .expect("book page should resolve from the config dir")
                .to_path_buf()
        })
        .collect()
}

fn pages_for_book<'a>(
    site_model: &'a mdbook_bookshelf::SiteModel,
    book_id: &str,
) -> Vec<&'a mdbook_bookshelf::AuthoredPage> {
    site_model
        .authored_pages
        .iter()
        .filter(|page| page.book_id == book_id)
        .collect()
}
