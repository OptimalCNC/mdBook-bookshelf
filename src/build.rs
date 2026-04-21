use crate::catalog::{build_input_catalog, InputBook};
use anyhow::{Context, Result};
use mdbook_driver::{config::Config, MDBook};
use mdbook_summary::parse_summary;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn build_bookshelf(config_path: impl AsRef<Path>, dest_dir: Option<PathBuf>) -> Result<()> {
    let config_path = config_path.as_ref();
    let catalog = build_input_catalog(config_path)?;
    let projected_config = project_mdbook_config(config_path)?;
    let site_dest_dir = resolve_site_dest_dir(&catalog.config_dir, &projected_config, dest_dir)?;

    for book in &catalog.books {
        build_catalog_book(book, &projected_config, &site_dest_dir)?;
    }

    Ok(())
}

pub fn project_mdbook_config(config_path: impl AsRef<Path>) -> Result<Config> {
    let config_path = config_path.as_ref();
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let mut toml_root: toml::Table = toml::from_str(&raw)
        .with_context(|| format!("failed to parse TOML in {}", config_path.display()))?;
    toml_root.remove("bookshelf");

    let projected = toml::to_string(&toml_root).with_context(|| {
        format!(
            "failed to serialize projected mdBook config from {}",
            config_path.display()
        )
    })?;

    Config::from_str(&projected).with_context(|| {
        format!(
            "failed to parse projected mdBook config from {} after removing [bookshelf]",
            config_path.display()
        )
    })
}

fn build_catalog_book(
    book: &InputBook,
    projected_config: &Config,
    site_dest_dir: &Path,
) -> Result<()> {
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

    let mut config = projected_config.clone();
    config.book.src = book_src_from_root(book)?;
    config.build.build_dir = site_dest_dir.join("books").join(&book.id);

    let mdbook = MDBook::load_with_config_and_summary(book.book_root_abs.clone(), config, summary)
        .with_context(|| {
            format!(
                "book '{}' failed to load mdbook from root {} and source {}",
                book.id,
                book.book_root_abs.display(),
                book.book_src_abs.display()
            )
        })?;

    let html_build_dir = mdbook.build_dir_for("html");
    mdbook.build().with_context(|| {
        format!(
            "book '{}' failed to build mdbook output at {}",
            book.id,
            html_build_dir.display()
        )
    })
}

fn book_src_from_root(book: &InputBook) -> Result<PathBuf> {
    book.book_src_abs
        .strip_prefix(&book.book_root_abs)
        .map(Path::to_path_buf)
        .with_context(|| {
            format!(
                "book '{}' source {} is not under root {}",
                book.id,
                book.book_src_abs.display(),
                book.book_root_abs.display()
            )
        })
}

fn resolve_site_dest_dir(
    config_dir: &Path,
    projected_config: &Config,
    dest_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("failed to determine current working directory")?;

    match dest_dir {
        Some(dest_dir) => Ok(make_absolute(&cwd, dest_dir)),
        None => {
            let config_dir = make_absolute(&cwd, config_dir.to_path_buf());
            Ok(make_absolute(
                &config_dir,
                projected_config.build.build_dir.clone(),
            ))
        }
    }
}

fn make_absolute(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}
