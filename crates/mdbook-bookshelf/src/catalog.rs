use crate::config::{load_bookshelf_config, BookshelfConfig};
use anyhow::{bail, Context, Result};
use mdbook_driver::config::{BookConfig, Config};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct InputCatalog {
    pub config_path: PathBuf,
    pub config_dir: PathBuf,
    pub mdbook_config: Config,
    pub root_book_id: String,
    pub books: Vec<InputBook>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputBook {
    pub id: String,
    pub output_rel: PathBuf,
    pub book_config: BookConfig,
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
    let mut books = Vec::with_capacity(config.books.len() + 1);

    books.push(build_catalog_book(
        &config.config_dir,
        &config.root_book_id,
        PathBuf::from("books").join(&config.root_book_id),
        PathBuf::from("."),
        &config.mdbook_config.book,
        true,
    )?);

    for book in &config.books {
        books.push(build_catalog_book(
            &config.config_dir,
            &path_to_book_key(&book.mount_rel),
            book.mount_rel.clone(),
            book_root_from_source_rel(&book.source_rel),
            &book.book,
            false,
        )?);
    }

    let root_book_count = books.iter().filter(|book| book.is_root_book).count();
    if root_book_count != 1 {
        bail!(
            "input catalog must contain exactly one root book '{}', found {}",
            config.root_book_id,
            root_book_count
        );
    }

    Ok(InputCatalog {
        config_path: config.config_path.clone(),
        config_dir: config.config_dir.clone(),
        mdbook_config: config.mdbook_config.clone(),
        root_book_id: config.root_book_id.clone(),
        books,
    })
}

fn build_catalog_book(
    config_dir: &Path,
    id: &str,
    output_rel: PathBuf,
    book_root_rel: PathBuf,
    book_config: &BookConfig,
    is_root_book: bool,
) -> Result<InputBook> {
    let book_src_rel = book_config.src.clone();
    let book_root_abs = join_rel_dir(config_dir, &book_root_rel);
    let book_src_abs = book_root_abs.join(&book_src_rel);
    let summary_abs = book_src_abs.join("SUMMARY.md");
    let summary_rel = book_src_rel.join("SUMMARY.md");

    let metadata = std::fs::metadata(&summary_abs).with_context(|| {
        format!(
            "book '{id}' missing canonical summary '{}' derived from src '{}' at {}",
            summary_rel.display(),
            book_src_rel.display(),
            summary_abs.display()
        )
    })?;
    if !metadata.is_file() {
        bail!(
            "book '{id}' missing canonical summary '{}' derived from src '{}' at {}",
            summary_rel.display(),
            book_src_rel.display(),
            summary_abs.display()
        );
    }

    Ok(InputBook {
        id: id.to_string(),
        output_rel,
        book_config: book_config.clone(),
        title: book_config
            .title
            .clone()
            .expect("config validation requires every catalog book to have a title"),
        description: book_config.description.clone(),
        book_root_rel,
        book_root_abs,
        book_src_rel,
        book_src_abs,
        summary_abs,
        is_root_book,
    })
}

fn join_rel_dir(base: &Path, rel: &Path) -> PathBuf {
    if rel == Path::new(".") {
        base.to_path_buf()
    } else {
        base.join(rel)
    }
}

fn book_root_from_source_rel(source_rel: &Path) -> PathBuf {
    source_rel
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn path_to_book_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
