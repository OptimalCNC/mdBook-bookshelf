use mdbook_bookshelf::{
    build_input_catalog, build_navigation_metadata, build_site_model, load_books_from_catalog,
    SiteBook, SiteModel, SitePage, SitePageKind,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn navigation_metadata() {
    let temp = TempDir::new("documentation-index-navigation");
    let config_path = write_explicit_id_fixture(temp.path());

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
    assert_eq!("root", root_first.active_book_id);

    assert_eq!(Some("root:0000".to_string()), root_last.prev_page_id);
    assert_eq!(None, root_last.next_page_id);
    assert_eq!("root", root_last.active_book_id);

    assert_eq!(None, child_first.prev_page_id);
    assert_eq!(Some("child:0001".to_string()), child_first.next_page_id);
    assert_eq!("child", child_first.active_book_id);

    assert_eq!(Some("child:0000".to_string()), child_last.prev_page_id);
    assert_eq!(None, child_last.next_page_id);
    assert_eq!("child", child_last.active_book_id);

    assert_ne!(Some("child:0000".to_string()), root_last.next_page_id);
    assert_ne!(Some("root:0001".to_string()), child_first.prev_page_id);

    assert!(nav.for_page("documentation:index").is_none());
    assert_eq!(None, nav.resolve_active_book_id("documentation:index"));
    assert_eq!(Some("root"), nav.resolve_active_book_id("root:0000"));
    assert_eq!(Some("child"), nav.resolve_active_book_id("child:0001"));
}

#[test]
fn navigation_metadata_ownership_mismatch_formats_missing_owner() {
    let model = SiteModel {
        documentation_index_page_id: "documentation:index".to_string(),
        books: vec![SiteBook {
            book_id: "root".to_string(),
            title: "Root Book".to_string(),
            page_ids_in_order: vec!["root:0000".to_string()],
        }],
        pages: vec![
            SitePage {
                page_id: "documentation:index".to_string(),
                kind: SitePageKind::DocumentationIndex,
                owning_book_id: None,
                title: "Documentation".to_string(),
                source_path: None,
                order_in_book: None,
            },
            SitePage {
                page_id: "root:0000".to_string(),
                kind: SitePageKind::Content,
                owning_book_id: None,
                title: "Root Intro".to_string(),
                source_path: Some(PathBuf::from("index.md")),
                order_in_book: Some(0),
            },
        ],
    };

    let error = build_navigation_metadata(&model)
        .expect_err("ownership mismatch should reject content page without owning book")
        .to_string();

    assert_eq!(
        "navigation metadata ownership mismatch for page 'root:0000': page owns no owning book; book owns 'root'",
        error
    );
    assert!(!error.contains("None"));
    assert!(!error.contains("Some("));
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir()
            .join("mdbook-bookshelf")
            .join(root.file_name().unwrap_or_default())
            .join(format!("{tag}-{nanos}"));
        fs::create_dir_all(&path).expect("temp fixture directory should be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_explicit_id_fixture(dir: &Path) -> PathBuf {
    write_file(
        dir,
        "root-book/docs/SUMMARY.md",
        "# Summary\n\n- [Root Intro](index.md)\n- [Root Next](next.md)\n",
    );
    write_file(dir, "root-book/docs/index.md", "# Root Intro\n");
    write_file(dir, "root-book/docs/next.md", "# Root Next\n");
    write_file(
        dir,
        "modules/child/docs/SUMMARY.md",
        "# Summary\n\n- [Child Intro](intro.md)\n- [Child Next](next.md)\n",
    );
    write_file(dir, "modules/child/docs/intro.md", "# Child Intro\n");
    write_file(dir, "modules/child/docs/next.md", "# Child Next\n");
    write_file(
        dir,
        "bookshelf.toml",
        r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "root"
title = "Root Book"
src = "root-book/docs"

[[bookshelf.book]]
id = "child"
title = "Child Book"
src = "modules/child/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["root", "child"]
"#,
    )
}

fn write_file(dir: &Path, rel: &str, content: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory should be created");
    }
    fs::write(&path, content).expect("fixture file should be written");
    path
}
