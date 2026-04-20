use crate::config::{BookshelfConfig, load_bookshelf_config};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputCatalog {
    pub config_path: PathBuf,
    pub config_dir: PathBuf,
    pub root_book_id: String,
    pub books: Vec<InputBook>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputBook {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub book_root_rel: PathBuf,
    pub book_root_abs: PathBuf,
    pub book_src_rel: PathBuf,
    pub book_src_abs: PathBuf,
    pub summary_abs: PathBuf,
    pub is_root_book: bool,
}

pub fn build_input_catalog(config_path: impl AsRef<Path>) -> Result<InputCatalog> {
    let config = load_bookshelf_config(config_path)?;
    build_input_catalog_from_config(&config)
}

fn build_input_catalog_from_config(config: &BookshelfConfig) -> Result<InputCatalog> {
    let mut books = Vec::with_capacity(config.books.len());

    for book in &config.books {
        let summary_rel = &book.summary_rel;
        let summary_parent = summary_rel.parent().ok_or_else(|| {
            anyhow::anyhow!(
                "book '{}' has invalid summary path '{}'",
                book.id,
                summary_rel.display()
            )
        })?;
        let src_dir_name = summary_parent.file_name().and_then(|v| v.to_str());

        if src_dir_name != Some("src") {
            bail!(
                "book '{}' summary must be at '<book-root>/src/SUMMARY.md': '{}'",
                book.id,
                summary_rel.display()
            );
        }

        let book_root_rel = summary_parent
            .parent()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "book '{}' summary must be at '<book-root>/src/SUMMARY.md': '{}'",
                    book.id,
                    summary_rel.display()
                )
            })?
            .to_path_buf();
        if book_root_rel.as_os_str().is_empty() {
            bail!(
                "book '{}' summary must be at '<book-root>/src/SUMMARY.md': '{}'",
                book.id,
                summary_rel.display()
            );
        }

        let book_root_abs = config.config_dir.join(&book_root_rel);
        let book_src_rel = PathBuf::from("src");
        let book_src_abs = book_root_abs.join(&book_src_rel);
        let summary_abs = book_src_abs.join("SUMMARY.md");

        let metadata = std::fs::metadata(&summary_abs).with_context(|| {
            format!(
                "book '{}' missing canonical summary at {}",
                book.id,
                summary_abs.display()
            )
        })?;
        if !metadata.is_file() {
            bail!(
                "book '{}' missing canonical summary at {}",
                book.id,
                summary_abs.display()
            );
        }

        books.push(InputBook {
            id: book.id.clone(),
            title: book.title.clone(),
            description: book.description.clone(),
            book_root_rel,
            book_root_abs,
            book_src_rel,
            book_src_abs,
            summary_abs,
            is_root_book: book.id == config.root_book,
        });
    }

    let root_book_count = books.iter().filter(|book| book.is_root_book).count();
    if root_book_count != 1 {
        bail!(
            "input catalog must contain exactly one root book '{}', found {}",
            config.root_book,
            root_book_count
        );
    }

    Ok(InputCatalog {
        config_path: config.config_path.clone(),
        config_dir: config.config_dir.clone(),
        root_book_id: config.root_book.clone(),
        books,
    })
}
