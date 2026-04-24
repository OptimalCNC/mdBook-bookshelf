use crate::catalog::{build_input_catalog, InputCatalog};
use crate::load_single_book_with_config_and_parsed_summary;
use crate::root_bookshelf_preprocessor::ensure_reserved_bookshelf_path_is_available;
use anyhow::{Context, Result};
use mdbook_driver::MDBook;
use mdbook_summary::{parse_summary, Summary};
use std::fs;
use std::path::Path;

pub struct LoadedBooks {
    pub root_book_id: String,
    pub books: Vec<LoadedBook>,
}

pub struct LoadedBook {
    pub book_id: String,
    pub summary: Summary,
    pub mdbook: MDBook,
}

pub fn load_books_from_config(config_path: impl AsRef<Path>) -> Result<LoadedBooks> {
    let catalog = build_input_catalog(config_path)?;
    load_books_from_catalog(&catalog)
}

pub fn load_books_from_catalog(catalog: &InputCatalog) -> Result<LoadedBooks> {
    let mut books = Vec::with_capacity(catalog.books.len());
    let root_book_id = catalog.root_book()?.id.clone();

    for book in &catalog.books {
        let summary_text = fs::read_to_string(&book.summary_abs).with_context(|| {
            format!(
                "book '{}' failed to read canonical summary at {}",
                book.id,
                book.summary_abs.display()
            )
        })?;
        let summary = parse_summary(&summary_text).with_context(|| {
            format!(
                "book '{}' failed to parse canonical summary at {}",
                book.id,
                book.summary_abs.display()
            )
        })?;
        let mut config = catalog.mdbook_config.clone();
        config.book = book.book_config.clone();
        let mdbook = load_single_book_with_config_and_parsed_summary(
            &book.book_root_abs,
            &book.book_src_rel,
            config,
            summary.clone(),
        )
        .with_context(|| {
            format!(
                "book '{}' failed to load mdbook from root {} and source {}",
                book.id,
                book.book_root_abs.display(),
                book.book_src_abs.display()
            )
        })?;
        ensure_reserved_bookshelf_path_is_available(&mdbook.book, &book.id).with_context(|| {
            format!(
                "book '{}' uses a reserved bookshelf content path while loading {}",
                book.id,
                book.summary_abs.display()
            )
        })?;

        books.push(LoadedBook {
            book_id: book.id.clone(),
            summary,
            mdbook,
        });
    }

    Ok(LoadedBooks {
        root_book_id,
        books,
    })
}
