use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::render_manifest::{RenderManifest, RenderedPageIdentity};
use crate::site_model::SiteModel;

#[derive(Debug)]
pub enum RenderBookshelfRootError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    MissingBookshelfManifestEntry {
        page_id: String,
    },
    MissingShelfItemManifestEntry {
        book_id: String,
        source_path: PathBuf,
    },
}

impl fmt::Display for RenderBookshelfRootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to write bookshelf root output at {}: {}",
                    path.display(),
                    source
                )
            }
            Self::MissingBookshelfManifestEntry { page_id } => {
                write!(
                    f,
                    "render manifest is missing synthetic Bookshelf page `{}`",
                    page_id
                )
            }
            Self::MissingShelfItemManifestEntry {
                book_id,
                source_path,
            } => write!(
                f,
                "render manifest is missing the root authored page {} for bookshelf book `{}`",
                source_path.display(),
                book_id
            ),
        }
    }
}

impl std::error::Error for RenderBookshelfRootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub fn render_bookshelf_root_page(
    output_dir: impl AsRef<Path>,
    site_model: &SiteModel,
    render_manifest: &RenderManifest,
) -> Result<PathBuf, RenderBookshelfRootError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).map_err(|source| RenderBookshelfRootError::Io {
        path: output_dir.to_path_buf(),
        source,
    })?;

    let bookshelf_entry = render_manifest
        .entry_for_page_id(&site_model.bookshelf_page.page_id)
        .ok_or_else(|| RenderBookshelfRootError::MissingBookshelfManifestEntry {
            page_id: site_model.bookshelf_page.page_id.clone(),
        })?;
    let output_path = output_dir.join(&bookshelf_entry.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|source| RenderBookshelfRootError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let html = render_bookshelf_html(site_model, render_manifest)?;
    fs::write(&output_path, html).map_err(|source| RenderBookshelfRootError::Io {
        path: output_path.clone(),
        source,
    })?;

    Ok(output_path)
}

fn render_bookshelf_html(
    site_model: &SiteModel,
    render_manifest: &RenderManifest,
) -> Result<String, RenderBookshelfRootError> {
    let mut shelf_items_markup = String::new();

    for shelf_item in &site_model.bookshelf_page.shelf_items {
        let manifest_entry = render_manifest
            .authored_entry_for_source_path(&shelf_item.target_source_path)
            .ok_or_else(|| RenderBookshelfRootError::MissingShelfItemManifestEntry {
                book_id: shelf_item.book_id.clone(),
                source_path: shelf_item.target_source_path.clone(),
            })?;
        let href = render_href(manifest_entry);
        let description = shelf_item
            .description
            .as_deref()
            .map(escape_html)
            .unwrap_or_default();

        shelf_items_markup.push_str(&format!(
            "<li class=\"bookshelf-item\"><a class=\"bookshelf-link\" href=\"{href}\"><span class=\"bookshelf-link-title\">{title}</span><span class=\"bookshelf-link-description\">{description}</span></a></li>",
            href = escape_html(&href),
            title = escape_html(&shelf_item.title),
            description = description
        ));
    }

    Ok(format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>{title}</title></head><body><div id=\"page-wrapper\" class=\"page-wrapper\"><main id=\"content\" class=\"content\" role=\"main\"><h1>{title}</h1><ul class=\"bookshelf-grid\">{shelf_items}</ul></main></div></body></html>",
        title = escape_html(&site_model.bookshelf_page.title),
        shelf_items = shelf_items_markup
    ))
}

fn render_href(manifest_entry: &crate::render_manifest::RenderedPageManifestEntry) -> String {
    match &manifest_entry.identity {
        RenderedPageIdentity::SyntheticPage { .. } => String::new(),
        RenderedPageIdentity::AuthoredPage { .. } => {
            manifest_entry.output_path.to_string_lossy().into_owned()
        }
    }
}

fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
