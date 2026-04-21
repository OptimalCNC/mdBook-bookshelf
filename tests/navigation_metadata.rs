use mdbook_bookshelf::{
    build_input_catalog, build_navigation_metadata, build_site_model, load_books_from_catalog,
};
use std::path::PathBuf;

#[test]
fn navigation_metadata() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("tests/fixtures/navigation/valid/bookshelf.toml");

    let catalog = build_input_catalog(&config_path).expect("catalog should build");
    let loaded = load_books_from_catalog(&catalog).expect("books should load");
    let model = build_site_model(&catalog, &loaded).expect("site model should build");
    let nav = build_navigation_metadata(&model).expect("navigation metadata should build");

    let root_first = nav.for_page("root:0000").expect("root first page nav");
    let root_last = nav.for_page("root:0001").expect("root last page nav");
    let child_first = nav.for_page("child:0000").expect("child first page nav");
    let child_last = nav.for_page("child:0001").expect("child last page nav");

    assert_eq!(None, root_first.prev_page_id);
    assert_eq!(Some("root:0001".to_string()), root_first.next_page_id);
    assert_eq!(Some("Root Book / Root Intro".to_string()), root_first.breadcrumb);
    assert_eq!("root", root_first.active_book_id);

    assert_eq!(Some("root:0000".to_string()), root_last.prev_page_id);
    assert_eq!(None, root_last.next_page_id);
    assert_eq!(Some("Root Book / Root Next".to_string()), root_last.breadcrumb);
    assert_eq!("root", root_last.active_book_id);

    assert_eq!(None, child_first.prev_page_id);
    assert_eq!(Some("child:0001".to_string()), child_first.next_page_id);
    assert_eq!(Some("Child Book / Child Intro".to_string()), child_first.breadcrumb);
    assert_eq!("child", child_first.active_book_id);

    assert_eq!(Some("child:0000".to_string()), child_last.prev_page_id);
    assert_eq!(None, child_last.next_page_id);
    assert_eq!(Some("Child Book / Child Next".to_string()), child_last.breadcrumb);
    assert_eq!("child", child_last.active_book_id);

    assert_ne!(Some("child:0000".to_string()), root_last.next_page_id);
    assert_ne!(Some("root:0001".to_string()), child_first.prev_page_id);

    assert_eq!(Some("root"), nav.resolve_active_book_id("bookshelf:root"));
    assert_eq!(Some("root"), nav.resolve_active_book_id("root:0000"));
    assert_eq!(Some("child"), nav.resolve_active_book_id("child:0001"));
}
