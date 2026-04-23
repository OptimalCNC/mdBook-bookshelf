use mdbook_bookshelf::build_input_catalog;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn input_catalog_build() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = repo_root.join("tests/fixtures/input-catalog");

    let valid_config = fixtures.join("valid/bookshelf.toml");
    let catalog = build_input_catalog(&valid_config).expect("valid fixture should build catalog");
    assert_eq!("root", catalog.root_book_id);
    assert_eq!(2, catalog.books.len());

    assert_eq!("root", catalog.books[0].id);
    assert!(catalog.books[0].is_root_book);
    assert_eq!(
        Path::new("root-book"),
        catalog.books[0].book_root_rel.as_path()
    );
    assert_eq!(
        Path::new("root-book/docs"),
        catalog.books[0].book_src_rel.as_path()
    );
    assert_eq!(
        fixtures.join("valid/root-book/docs/SUMMARY.md"),
        catalog.books[0].summary_abs
    );

    assert_eq!("child", catalog.books[1].id);
    assert!(!catalog.books[1].is_root_book);
    assert_eq!(
        Path::new("modules/child-book"),
        catalog.books[1].book_root_rel.as_path()
    );
    assert_eq!(
        Path::new("modules/child-book/docs"),
        catalog.books[1].book_src_rel.as_path()
    );
    assert_eq!(
        fixtures.join("valid/modules/child-book/docs/SUMMARY.md"),
        catalog.books[1].summary_abs
    );

    let top_level_config = fixtures.join("valid-top-level/bookshelf.toml");
    let top_level_catalog =
        build_input_catalog(&top_level_config).expect("top-level docs fixture should build");
    assert_eq!("meta", top_level_catalog.root_book_id);
    assert_eq!(1, top_level_catalog.books.len());
    assert_eq!(
        Path::new("."),
        top_level_catalog.books[0].book_root_rel.as_path()
    );
    assert_eq!(
        fixtures.join("valid-top-level"),
        top_level_catalog.books[0].book_root_abs
    );
    assert_eq!(
        Path::new("docs"),
        top_level_catalog.books[0].book_src_rel.as_path()
    );
    assert_eq!(
        fixtures.join("valid-top-level/docs/SUMMARY.md"),
        top_level_catalog.books[0].summary_abs
    );
    assert!(top_level_catalog.books[0].is_root_book);

    let missing_config = fixtures.join("invalid-missing-summary/bookshelf.toml");
    let missing_err = build_input_catalog(&missing_config).expect_err("must fail");
    assert_eq!(
        format!(
            "book 'root' missing canonical summary 'docs/SUMMARY.md' derived from src 'docs' at {}",
            fixtures
                .join("invalid-missing-summary/docs/SUMMARY.md")
                .display()
        ),
        missing_err.to_string()
    );

    let invalid_location_config = fixtures.join("invalid-summary-location/bookshelf.toml");
    assert!(
        fs::metadata(fixtures.join("invalid-summary-location/src/SUMMARY.md")).is_ok(),
        "invalid-summary-location fixture should include wrong-location summary file"
    );
    let location_err = build_input_catalog(&invalid_location_config).expect_err("must fail");
    assert_eq!(
        format!(
            "book 'root' missing canonical summary 'docs/SUMMARY.md' derived from src 'docs' at {}",
            fixtures
                .join("invalid-summary-location/docs/SUMMARY.md")
                .display()
        ),
        location_err.to_string()
    );
}
