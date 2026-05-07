use mdbook_bookshelf::{
    build_input_catalog, build_site_model, load_books_from_catalog, SitePageKind,
};
use mdbook_driver::book::BookItem;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn site_model_build() {
    let temp = TempDir::new("documentation-index-site-model");
    let config_path = write_explicit_id_fixture(temp.path());

    let catalog = build_input_catalog(&config_path).expect("catalog should build");
    let loaded = load_books_from_catalog(&catalog).expect("books should load");
    let model = build_site_model(&catalog, &loaded).expect("site model should build");

    assert_eq!("documentation:index", model.documentation_index_page_id);
    assert_eq!(2, model.books.len());
    assert_eq!("root", model.books[0].book_id);
    assert_eq!("child", model.books[1].book_id);

    let documentation_index_pages: Vec<_> = model
        .pages
        .iter()
        .filter(|p| p.kind == SitePageKind::DocumentationIndex)
        .collect();
    assert_eq!(
        1,
        documentation_index_pages.len(),
        "must contain exactly one documentation index page"
    );
    let documentation_index = documentation_index_pages[0];
    assert_eq!("documentation:index", documentation_index.page_id);
    assert_eq!("Docs Portal", documentation_index.title);
    assert_eq!(None, documentation_index.owning_book_id);
    assert_eq!(None, documentation_index.order_in_book);
    assert!(
        model.books.iter().all(|book| !book
            .page_ids_in_order
            .contains(&documentation_index.page_id)),
        "documentation index page must not be represented as a content-book entry"
    );

    let content_pages: Vec<_> = model
        .pages
        .iter()
        .filter(|p| p.kind == SitePageKind::Content)
        .collect();
    let loaded_content_count = loaded
        .books
        .iter()
        .map(|book| {
            book.mdbook
                .iter()
                .filter(|item| matches!(item, BookItem::Chapter(ch) if ch.path.is_some()))
                .count()
        })
        .sum::<usize>();
    assert_eq!(loaded_content_count, content_pages.len());

    for book in &model.books {
        for (idx, page_id) in book.page_ids_in_order.iter().enumerate() {
            let page = model
                .pages
                .iter()
                .find(|p| &p.page_id == page_id)
                .expect("book page id should resolve");
            assert_eq!(SitePageKind::Content, page.kind);
            assert_eq!(Some(book.book_id.as_str()), page.owning_book_id.as_deref());
            assert_eq!(Some(idx), page.order_in_book);
        }
    }
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
        "# Summary\n\n- [Child Intro](intro.md)\n",
    );
    write_file(dir, "modules/child/docs/intro.md", "# Child Intro\n");
    write_file(
        dir,
        "bookshelf.toml",
        r#"
[book]
title = "Documentation"

[bookshelf]
index-title = "Docs Portal"

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
