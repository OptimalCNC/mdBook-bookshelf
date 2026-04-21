use crate::catalog::InputCatalog;
use anyhow::{Context, Result};
use mdbook_driver::config::Config;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProjectedBookConfig {
    pub book_id: String,
    pub config: Config,
}

pub fn project_book_configs(
    config_path: impl AsRef<Path>,
    catalog: &InputCatalog,
) -> Result<Vec<ProjectedBookConfig>> {
    let config_path = config_path.as_ref();
    let source_text = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let source: toml::Value = toml::from_str(&source_text)
        .with_context(|| format!("failed to parse TOML in {}", config_path.display()))?;

    let mut projected = Vec::with_capacity(catalog.books.len());
    for book in &catalog.books {
        let mut cfg = Config::default();

        let src_rel = book
            .book_src_abs
            .strip_prefix(&book.book_root_abs)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| book.book_src_rel.clone());
        cfg.book.src = src_rel;
        // Keep per-book display title rooted in bookshelf config, not global [book].title.
        cfg.book.title = Some(book.title.clone());

        if let Some(book_table) = source.get("book").and_then(|v| v.as_table()) {
            if let Some(language) = book_table.get("language").and_then(|v| v.as_str()) {
                cfg.book.language = Some(language.to_string());
            }
            if let Some(authors) = book_table.get("authors") {
                if let Some(authors) = authors.as_array() {
                    cfg.book.authors = authors
                        .iter()
                        .filter_map(|v| v.as_str().map(ToString::to_string))
                        .collect();
                }
            }
        }

        if let Some(output_html) = source
            .get("output")
            .and_then(|v| v.get("html"))
            .cloned()
        {
            cfg.set("output.html", output_html).with_context(|| {
                format!("failed to project [output.html] for book '{}'", book.id)
            })?;
        }
        // Intentionally minimal in CHUNK-007B: [build], [rust], and [preprocessor.*]
        // are not projected yet because this first HTML path only consumes book.src,
        // selected [book] metadata, and selected [output.html] settings.

        projected.push(ProjectedBookConfig {
            book_id: book.id.clone(),
            config: cfg,
        });
    }

    Ok(projected)
}
