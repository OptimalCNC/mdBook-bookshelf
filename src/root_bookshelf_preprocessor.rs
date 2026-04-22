use crate::catalog::{InputBook, InputCatalog};
use anyhow::{bail, Result};
use mdbook_driver::book::{Book, Chapter};
use std::path::{Path, PathBuf};

pub const ROOT_BOOKSHELF_CHAPTER_NAME: &str = "Bookshelf";
pub const ROOT_BOOKSHELF_CHAPTER_PATH: &str = "bookshelf.md";
pub const ROOT_BOOKSHELF_HTML_PATH: &str = "bookshelf.html";

pub fn inject_root_bookshelf_page(book: &mut Book, catalog: &InputCatalog) -> Result<()> {
    ensure_reserved_bookshelf_path_is_available(book)?;
    book.items
        .insert(0, build_root_bookshelf_chapter(catalog).into());
    Ok(())
}

pub fn site_root_bookshelf_entry_path(root_book_id: &str) -> String {
    format!("books/{root_book_id}/{ROOT_BOOKSHELF_HTML_PATH}")
}

fn ensure_reserved_bookshelf_path_is_available(book: &Book) -> Result<()> {
    let reserved_path = Path::new(ROOT_BOOKSHELF_CHAPTER_PATH);
    if book
        .chapters()
        .any(|chapter| chapter.path.as_deref() == Some(reserved_path))
    {
        bail!(
            "root book already contains reserved synthetic bookshelf path '{}'",
            ROOT_BOOKSHELF_CHAPTER_PATH
        );
    }

    Ok(())
}

fn build_root_bookshelf_chapter(catalog: &InputCatalog) -> Chapter {
    Chapter {
        name: ROOT_BOOKSHELF_CHAPTER_NAME.to_string(),
        content: render_root_bookshelf_markdown(catalog),
        number: None,
        sub_items: Vec::new(),
        path: Some(PathBuf::from(ROOT_BOOKSHELF_CHAPTER_PATH)),
        source_path: None,
        parent_names: Vec::new(),
    }
}

fn render_root_bookshelf_markdown(catalog: &InputCatalog) -> String {
    let mut markdown = String::from("# Bookshelf\n\nChoose a book to enter its root page.\n");

    for book in &catalog.books {
        markdown.push_str("\n- [");
        markdown.push_str(&escape_markdown_text(&book.title));
        markdown.push_str("](<");
        markdown.push_str(&bookshelf_book_href(book, &catalog.root_book_id));
        markdown.push_str(">)");

        if let Some(description) = book.description.as_deref() {
            markdown.push_str(": ");
            markdown.push_str(description.trim());
        }

        markdown.push('\n');
    }

    markdown
}

fn bookshelf_book_href(book: &InputBook, root_book_id: &str) -> String {
    if book.id == root_book_id {
        "index.html".to_string()
    } else {
        format!("../{}/index.html", book.id)
    }
}

fn escape_markdown_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_root_bookshelf_page_adds_a_synthetic_first_chapter() {
        let mut book = Book::new();
        book.push_item(Chapter::new(
            "Example Core",
            "# Example Core".to_string(),
            "index.md",
            Vec::new(),
        ));

        let catalog = sample_catalog();

        inject_root_bookshelf_page(&mut book, &catalog).expect("injection should succeed");

        let synthetic = match &book.items[0] {
            mdbook_driver::book::BookItem::Chapter(chapter) => chapter,
            other => panic!("expected chapter, got {other:?}"),
        };
        assert_eq!(synthetic.name, ROOT_BOOKSHELF_CHAPTER_NAME);
        assert_eq!(
            synthetic.path.as_deref(),
            Some(Path::new(ROOT_BOOKSHELF_CHAPTER_PATH))
        );
        assert_eq!(synthetic.source_path, None);
        assert_eq!(synthetic.number, None);
        assert!(synthetic.content.contains("[Example Core](<index.html>)"));
        assert!(synthetic.content.contains("Repository-wide onboarding"));
        assert!(synthetic
            .content
            .contains("[Example Parser](<../parser/index.html>)"));
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
        assert!(error
            .to_string()
            .contains("reserved synthetic bookshelf path"));
    }

    fn sample_catalog() -> InputCatalog {
        InputCatalog {
            config_path: PathBuf::from("bookshelf.toml"),
            config_dir: PathBuf::from("."),
            root_book_id: "meta".to_string(),
            books: vec![
                sample_book(
                    "meta",
                    "Example Core",
                    Some("Repository-wide onboarding and architecture notes."),
                    true,
                ),
                sample_book(
                    "parser",
                    "Example Parser",
                    Some("Parser-specific reference pages with their own reading order."),
                    false,
                ),
            ],
        }
    }

    fn sample_book(
        id: &str,
        title: &str,
        description: Option<&str>,
        is_root_book: bool,
    ) -> InputBook {
        InputBook {
            id: id.to_string(),
            title: title.to_string(),
            description: description.map(str::to_string),
            book_root_rel: PathBuf::from("."),
            book_root_abs: PathBuf::from("/tmp"),
            book_src_rel: PathBuf::from("docs"),
            book_src_abs: PathBuf::from("/tmp/docs"),
            summary_abs: PathBuf::from("/tmp/docs/SUMMARY.md"),
            is_root_book,
        }
    }
}
