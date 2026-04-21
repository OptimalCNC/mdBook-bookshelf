pub mod build;
pub mod catalog;
pub mod config;
pub mod loader;
pub mod navigation;
pub mod site_model;

pub use build::{build_bookshelf, project_mdbook_config};
pub use catalog::{build_input_catalog, InputBook, InputCatalog};
pub use config::{load_bookshelf_config, BookshelfBook, BookshelfConfig};
pub use loader::{load_books_from_catalog, load_books_from_config, LoadedBook, LoadedBooks};
pub use navigation::{build_navigation_metadata, NavigationMetadata, PageNavigation};
pub use site_model::{build_site_model, SiteBook, SiteModel, SitePage, SitePageKind};

use anyhow::{bail, Context, Result};
use mdbook_driver::{config::Config, MDBook};
use mdbook_summary::{parse_summary, Summary};
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
    load_single_book_with_config_and_parsed_summary(book_root, book_src, Config::default(), summary)
}

pub fn load_single_book_with_config_and_parsed_summary(
    book_root: impl AsRef<Path>,
    book_src: impl AsRef<Path>,
    mut config: Config,
    summary: Summary,
) -> Result<MDBook> {
    let book_root = book_root.as_ref();
    let src_relative = normalize_src_path(book_root, book_src.as_ref())?;

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
