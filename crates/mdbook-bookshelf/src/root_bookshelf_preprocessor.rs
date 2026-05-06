use crate::catalog::{InputBook, InputCatalog};
use crate::route_paths::{path_to_string, relative_path};
use anyhow::{bail, Result};
use mdbook_driver::book::{Book, BookItem, Chapter};
use std::path::{Path, PathBuf};

pub const ROOT_BOOKSHELF_CHAPTER_NAME: &str = "Bookshelf";
pub const ROOT_BOOKSHELF_CHAPTER_PATH: &str = "bookshelf.md";
pub const ROOT_BOOKSHELF_HTML_PATH: &str = "bookshelf.html";

pub fn inject_root_bookshelf_page(book: &mut Book, catalog: &InputCatalog) -> Result<()> {
    ensure_reserved_bookshelf_path_is_available(book, "root book")?;
    book.items
        .insert(0, BookItem::Chapter(build_root_bookshelf_chapter(catalog)?));
    Ok(())
}

pub fn site_root_bookshelf_entry_path(root_output_rel: impl AsRef<Path>) -> String {
    path_to_string(&root_output_rel.as_ref().join(ROOT_BOOKSHELF_HTML_PATH))
}

pub fn ensure_reserved_bookshelf_path_is_available(book: &Book, book_id: &str) -> Result<()> {
    if book.chapters().any(|chapter| {
        chapter
            .path
            .as_deref()
            .and_then(Path::file_name)
            .is_some_and(|name| name == ROOT_BOOKSHELF_CHAPTER_PATH)
    }) {
        bail!(
            "book '{}' already contains reserved bookshelf path '{}'",
            book_id,
            ROOT_BOOKSHELF_CHAPTER_PATH
        );
    }

    Ok(())
}

fn build_root_bookshelf_chapter(catalog: &InputCatalog) -> Result<Chapter> {
    Ok(Chapter {
        name: ROOT_BOOKSHELF_CHAPTER_NAME.to_string(),
        content: render_root_bookshelf_markdown(catalog)?,
        number: None,
        sub_items: Vec::new(),
        path: Some(PathBuf::from(ROOT_BOOKSHELF_CHAPTER_PATH)),
        source_path: None,
        parent_names: Vec::new(),
    })
}

fn render_root_bookshelf_markdown(catalog: &InputCatalog) -> Result<String> {
    let mut markdown = String::from(
        "# Bookshelf\n\n<div class=\"bookshelf\">\n<ul class=\"bookshelf-list\" role=\"list\">\n",
    );
    let root_output_rel = catalog.root_book()?.output_rel.clone();

    for book in &catalog.books {
        markdown.push_str("<li>\n<a class=\"bookshelf-book\" href=\"");
        markdown.push_str(&escape_html_attr(&bookshelf_book_href(
            book,
            &root_output_rel,
        )));
        markdown.push_str("\">\n<span class=\"bookshelf-book-title\">");
        markdown.push_str(&escape_html_text(&book.title));
        markdown.push_str("</span>\n");

        if let Some(description) = book.description.as_deref().map(str::trim) {
            if !description.is_empty() {
                markdown.push_str("<span class=\"bookshelf-book-description\">");
                markdown.push_str(&escape_html_text(description));
                markdown.push_str("</span>\n");
            }
        }

        markdown.push_str("</a>\n</li>\n");
    }

    markdown.push_str("</ul>\n</div>\n");

    Ok(markdown)
}

fn bookshelf_book_href(book: &InputBook, root_output_rel: &Path) -> String {
    let target = book.output_rel.join("index.html");
    path_to_string(&relative_path(root_output_rel, &target))
}

fn escape_html_attr(text: &str) -> String {
    escape_html_text(text).replace('"', "&quot;")
}

fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::InputCategory;
    use mdbook_driver::book::BookItem;

    #[test]
    fn inject_root_bookshelf_page_prepends_a_synthetic_chapter() {
        let mut book = Book::new();
        book.push_item(Chapter::new(
            "Sample Core",
            "# Sample Core".to_string(),
            "index.md",
            Vec::new(),
        ));
        book.push_item(Chapter::new(
            "Architecture",
            "# Architecture".to_string(),
            "architecture.md",
            Vec::new(),
        ));

        let catalog = sample_catalog();

        inject_root_bookshelf_page(&mut book, &catalog).expect("injection should succeed");

        let synthetic = match &book.items[0] {
            BookItem::Chapter(chapter) => chapter,
            other => panic!("expected chapter, got {other:?}"),
        };
        assert_eq!(synthetic.name, ROOT_BOOKSHELF_CHAPTER_NAME);
        assert_eq!(
            synthetic.path.as_deref(),
            Some(Path::new(ROOT_BOOKSHELF_CHAPTER_PATH))
        );
        assert_eq!(synthetic.source_path, None);
        assert_eq!(synthetic.number, None);
        assert!(synthetic.content.contains("class=\"bookshelf-list\""));
        assert!(synthetic
            .content
            .contains("<a class=\"bookshelf-book\" href=\"index.html\">"));
        assert!(synthetic
            .content
            .contains("<span class=\"bookshelf-book-title\">Sample Core</span>"));
        assert!(synthetic.content.contains("Repository-wide onboarding"));
        assert!(synthetic
            .content
            .contains("<a class=\"bookshelf-book\" href=\"../modules/parser/docs/index.html\">"));

        let first_authored = match &book.items[1] {
            BookItem::Chapter(chapter) => chapter,
            other => panic!("expected chapter, got {other:?}"),
        };
        assert_eq!(first_authored.name, "Sample Core");
    }

    #[test]
    fn inject_root_bookshelf_page_rejects_reserved_path_collisions() {
        let mut book = Book::new();
        book.push_item(Chapter::new(
            "Authored Bookshelf",
            "# Existing".to_string(),
            ROOT_BOOKSHELF_CHAPTER_PATH,
            Vec::new(),
        ));

        let error =
            inject_root_bookshelf_page(&mut book, &sample_catalog()).expect_err("must fail");
        assert!(error.to_string().contains("reserved bookshelf path"));
    }

    fn sample_catalog() -> InputCatalog {
        InputCatalog {
            config_path: PathBuf::from("bookshelf.toml"),
            config_dir: PathBuf::from("."),
            mdbook_config: mdbook_driver::config::Config::default(),
            asset_dir: PathBuf::from(".mdbook/bookshelf"),
            books: vec![
                sample_book(
                    "docs",
                    "Sample Core",
                    Some("Repository-wide onboarding and architecture notes."),
                    true,
                ),
                sample_book(
                    "modules/parser/docs",
                    "Sample Parser",
                    Some("Parser-specific reference pages with their own reading order."),
                    false,
                ),
            ],
            categories: vec![InputCategory {
                title: "All Docs".to_string(),
                book_ids: vec!["docs".to_string(), "modules/parser/docs".to_string()],
            }],
        }
    }

    fn sample_book(
        id: &str,
        title: &str,
        description: Option<&str>,
        is_root_book: bool,
    ) -> InputBook {
        let mut book_config = mdbook_driver::config::BookConfig::default();
        book_config.title = Some(title.to_string());
        book_config.description = description.map(str::to_string);
        book_config.src = PathBuf::from(id);

        InputBook {
            id: id.to_string(),
            output_rel: PathBuf::from(id),
            book_config,
            title: title.to_string(),
            description: description.map(str::to_string),
            cover: None,
            book_root_rel: PathBuf::from("."),
            book_root_abs: PathBuf::from("/tmp"),
            book_src_rel: PathBuf::from(id),
            book_src_abs: PathBuf::from("/tmp").join(id),
            summary_abs: PathBuf::from("/tmp").join(id).join("SUMMARY.md"),
            is_root_book,
        }
    }
}
