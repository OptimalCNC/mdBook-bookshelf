use std::path::PathBuf;

use mdbook_bookshelf::{build_sidebar_model, build_site_model, load_input_catalog};

#[test]
fn preserves_root_affix_behavior_and_authored_order_invariants() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let original_orders: Vec<_> = site_model
        .books
        .iter()
        .map(|book| (book.id.clone(), book.authored_page_order.clone()))
        .collect();

    let sidebar_model = build_sidebar_model(&site_model);

    assert_eq!(site_model.authored_pages.len(), 9);
    for (book_id, authored_page_order) in &original_orders {
        let book = site_model
            .books
            .iter()
            .find(|book| &book.id == book_id)
            .expect("book should exist");
        assert_eq!(&book.authored_page_order, authored_page_order);
    }

    let meta = sidebar_model
        .sidebar_for("meta")
        .expect("meta sidebar should exist");
    assert_eq!(meta.affix_entries.len(), 1);
    assert_eq!(meta.affix_entries[0].title, "Bookshelf");
    assert_eq!(
        meta.affix_entries[0].target_page_id,
        site_model.bookshelf_page.page_id
    );
    assert!(meta
        .chapter_entries
        .iter()
        .all(|entry| entry.title != "Bookshelf"));

    for book_id in ["parser", "ui"] {
        let sidebar = sidebar_model
            .sidebar_for(book_id)
            .expect("non-root sidebar should exist");
        assert!(sidebar.affix_entries.is_empty());
        let titles = flatten_titles(&sidebar.chapter_entries);
        assert!(titles.iter().all(|title| title != "Bookshelf"));
        assert!(titles.iter().all(|title| {
            matches!(
                (book_id, title.as_str()),
                ("parser", "Example Parser" | "Grammar" | "Runtime")
                    | ("ui", "Example UI" | "Navigation" | "Diagnostics")
            )
        }));
    }
}

#[test]
fn keeps_non_root_sidebars_isolated_even_with_similar_structures() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);

    let parser_titles = flatten_titles(
        &sidebar_model
            .sidebar_for("parser")
            .expect("parser sidebar should exist")
            .chapter_entries,
    );
    let ui_titles = flatten_titles(
        &sidebar_model
            .sidebar_for("ui")
            .expect("ui sidebar should exist")
            .chapter_entries,
    );

    assert!(parser_titles.contains(&"Example Parser".to_owned()));
    assert!(parser_titles.contains(&"Grammar".to_owned()));
    assert!(parser_titles.contains(&"Runtime".to_owned()));
    assert!(!parser_titles.contains(&"Example UI".to_owned()));
    assert!(!parser_titles.contains(&"Navigation".to_owned()));
    assert!(!parser_titles.contains(&"Diagnostics".to_owned()));

    assert!(ui_titles.contains(&"Example UI".to_owned()));
    assert!(ui_titles.contains(&"Navigation".to_owned()));
    assert!(ui_titles.contains(&"Diagnostics".to_owned()));
    assert!(!ui_titles.contains(&"Example Parser".to_owned()));
    assert!(!ui_titles.contains(&"Grammar".to_owned()));
    assert!(!ui_titles.contains(&"Runtime".to_owned()));
}

fn flatten_titles(chapters: &[mdbook_bookshelf::SidebarChapter]) -> Vec<String> {
    let mut titles = Vec::new();
    for chapter in chapters {
        titles.push(chapter.title.clone());
        titles.extend(flatten_titles(&chapter.children));
    }
    titles
}
