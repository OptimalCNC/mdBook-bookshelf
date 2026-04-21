pub mod config;
pub mod catalog;
pub mod loader;

pub use catalog::{InputBook, InputCatalog, build_input_catalog};
pub use config::{BookshelfBook, BookshelfConfig, load_bookshelf_config};
pub use loader::{LoadedBook, LoadedBooks, load_books_from_catalog, load_books_from_config};

use anyhow::{Context, Result, bail};
use mdbook_driver::{MDBook, config::Config};
use mdbook_summary::{Summary, parse_summary};
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_single_book_with_summary(
    book_root: impl AsRef<Path>,
    book_src: impl AsRef<Path>,
) -> Result<MDBook> {
    let book_root = book_root.as_ref();
    let src_relative = normalize_src_path(book_root, book_src.as_ref())?;
    let summary_path = book_root.join(&src_relative).join("SUMMARY.md");

    let summary_text = fs::read_to_string(&summary_path)
        .with_context(|| format!("failed to read {}", summary_path.display()))?;
    let summary = parse_summary(&summary_text)
        .with_context(|| format!("failed to parse {}", summary_path.display()))?;

    load_single_book_with_parsed_summary(book_root, src_relative, summary)
}

pub fn load_single_book_with_parsed_summary(
    book_root: impl AsRef<Path>,
    book_src: impl AsRef<Path>,
    summary: Summary,
) -> Result<MDBook> {
    let book_root = book_root.as_ref();
    let src_relative = normalize_src_path(book_root, book_src.as_ref())?;

    let mut config = Config::default();
    config.book.src = src_relative;

    MDBook::load_with_config_and_summary(book_root.to_path_buf(), config, summary)
        .context("failed to load book with parsed summary")
}

fn normalize_src_path(book_root: &Path, book_src: &Path) -> Result<PathBuf> {
    if book_src.is_absolute() {
        return book_src
            .strip_prefix(book_root)
            .map(Path::to_path_buf)
            .with_context(|| {
                format!(
                    "book src path {} must be under root {}",
                    book_src.display(),
                    book_root.display()
                )
            });
    }

    if book_src.as_os_str().is_empty() {
        bail!("book src path cannot be empty");
    }

    Ok(book_src.to_path_buf())
}
