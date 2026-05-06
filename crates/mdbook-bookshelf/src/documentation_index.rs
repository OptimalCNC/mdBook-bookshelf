#![allow(dead_code)]

use crate::catalog::{InputBook, InputCatalog};
use crate::route_paths::path_to_string;
use anyhow::{Context, Result};
use mdbook_driver::config::Config;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn write_documentation_index(
    catalog: &InputCatalog,
    projected_config: &Config,
    site_dest_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(site_dest_dir).with_context(|| {
        format!(
            "failed to create site output directory {}",
            site_dest_dir.display()
        )
    })?;
    copy_configured_covers(catalog, site_dest_dir)?;
    let index_path: PathBuf = site_dest_dir.join("index.html");
    fs::write(
        &index_path,
        render_documentation_index_html(catalog, projected_config)?,
    )
    .with_context(|| {
        format!(
            "failed to write documentation index at {}",
            index_path.display()
        )
    })
}

pub(crate) fn render_documentation_index_html(
    catalog: &InputCatalog,
    projected_config: &Config,
) -> Result<String> {
    let mut used_category_ids = BTreeMap::new();
    let category_ids = catalog
        .categories
        .iter()
        .map(|category| category_id(&category.title, &mut used_category_ids))
        .collect::<Vec<_>>();
    let books_by_id = catalog
        .books
        .iter()
        .map(|book| (book.id.as_str(), book))
        .collect::<BTreeMap<_, _>>();

    let site_title = projected_config
        .book
        .title
        .as_deref()
        .or(catalog.mdbook_config.book.title.as_deref())
        .unwrap_or("Documentation");
    let language = projected_config
        .book
        .language
        .as_deref()
        .or(catalog.mdbook_config.book.language.as_deref())
        .unwrap_or("en");
    let page_title = if site_title.trim().is_empty() {
        "Documentation".to_string()
    } else {
        format!("Documentation - {}", site_title.trim())
    };

    let mut html = String::new();
    html.push_str("<!doctype html>\n<html lang=\"");
    html.push_str(&escape_html_attr(language));
    html.push_str("\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>");
    html.push_str(&escape_html_text(&page_title));
    html.push_str("</title>\n  <style>\n");
    html.push_str(INDEX_CSS);
    html.push_str("  </style>\n</head>\n<body>\n<div class=\"documentation-index\">\n");
    render_toc(&mut html, catalog, &category_ids);
    render_categories(&mut html, catalog, &category_ids, &books_by_id)?;
    html.push_str("</div>\n</body>\n</html>\n");

    Ok(html)
}

fn render_toc(html: &mut String, catalog: &InputCatalog, category_ids: &[String]) {
    html.push_str("<nav class=\"documentation-index-toc\" aria-label=\"Table of Contents\">\n");
    html.push_str("<h2>Table of Contents</h2>\n<ul>\n");
    for (category, category_id) in catalog.categories.iter().zip(category_ids) {
        html.push_str("<li><a href=\"#");
        html.push_str(&escape_html_attr(category_id));
        html.push_str("\">");
        html.push_str(&escape_html_text(&category.title));
        html.push_str("</a></li>\n");
    }
    html.push_str("</ul>\n</nav>\n");
}

fn render_categories(
    html: &mut String,
    catalog: &InputCatalog,
    category_ids: &[String],
    books_by_id: &BTreeMap<&str, &InputBook>,
) -> Result<()> {
    html.push_str("<main class=\"documentation-index-content\">\n");
    html.push_str("<h1>Documentation</h1>\n");
    for (category, category_id) in catalog.categories.iter().zip(category_ids) {
        html.push_str("<section class=\"documentation-category\" id=\"");
        html.push_str(&escape_html_attr(category_id));
        html.push_str("\">\n<h2>");
        html.push_str(&escape_html_text(&category.title));
        html.push_str("</h2>\n<div class=\"documentation-book-grid\">\n");

        for book_id in &category.book_ids {
            let book = books_by_id.get(book_id.as_str()).with_context(|| {
                format!(
                    "documentation category '{}' references unknown book '{}'",
                    category.title, book_id
                )
            })?;
            render_book_card(html, book);
        }

        html.push_str("</div>\n</section>\n");
    }
    html.push_str("</main>\n");

    Ok(())
}

fn render_book_card(html: &mut String, book: &InputBook) {
    html.push_str("<a class=\"documentation-book-card\" href=\"");
    html.push_str(&escape_html_attr(&book_href(book)));
    html.push_str("\" aria-label=\"");
    html.push_str(&escape_html_attr(&book.title));
    html.push_str("\">\n");

    if let Some(cover_src) = cover_src(book) {
        html.push_str("<img class=\"documentation-book-cover\" src=\"");
        html.push_str(&escape_html_attr(&cover_src));
        html.push_str("\" alt=\"Cover for ");
        html.push_str(&escape_html_attr(&book.title));
        html.push_str("\">\n");
    } else {
        html.push_str("<div class=\"documentation-book-cover-fallback\" aria-hidden=\"true\">");
        html.push_str(&escape_html_text(&book_initials(book)));
        html.push_str("</div>\n");
    }

    html.push_str("<h3>");
    html.push_str(&escape_html_text(&book.title));
    html.push_str("</h3>\n");

    if let Some(description) = book.description.as_deref().map(str::trim) {
        if !description.is_empty() {
            html.push_str("<p>");
            html.push_str(&escape_html_text(description));
            html.push_str("</p>\n");
        }
    }

    html.push_str("</a>\n");
}

fn book_initials(book: &InputBook) -> String {
    let initials = book
        .title
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>();

    if initials.is_empty() {
        book.id.chars().take(2).collect()
    } else {
        initials
    }
}

fn book_href(book: &InputBook) -> String {
    path_to_string(&book.output_rel.join("index.html"))
}

fn cover_src(book: &InputBook) -> Option<String> {
    book.cover.as_deref().map(path_to_string)
}

fn copy_configured_covers(catalog: &InputCatalog, site_dest_dir: &Path) -> Result<()> {
    for book in &catalog.books {
        let Some(cover) = &book.cover else {
            continue;
        };
        let source = catalog.config_dir.join(cover);
        let destination = site_dest_dir.join(cover);
        let parent = destination.parent().with_context(|| {
            format!(
                "cover output path {} is missing a parent",
                destination.display()
            )
        })?;
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create cover output directory {}",
                parent.display()
            )
        })?;
        fs::copy(&source, &destination).with_context(|| {
            format!(
                "failed to copy cover for book '{}' from {} to {}",
                book.id,
                source.display(),
                destination.display()
            )
        })?;
    }
    Ok(())
}

fn category_id(title: &str, used: &mut BTreeMap<String, usize>) -> String {
    let mut slug = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    let base = if slug.is_empty() { "category" } else { slug };
    let count = used.entry(base.to_string()).or_insert(0);
    *count += 1;
    if *count == 1 {
        format!("category-{base}")
    } else {
        format!("category-{base}-{count}")
    }
}

fn escape_html_attr(text: &str) -> String {
    escape_html_text(text).replace('"', "&quot;")
}

fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const INDEX_CSS: &str = r#"
:root {
    color-scheme: light dark;
}

body {
    font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    line-height: 1.5;
    margin: 0;
}

.documentation-index {
    display: grid;
    grid-template-columns: minmax(14rem, 18rem) minmax(0, 1fr);
    min-height: 100vh;
}

.documentation-index-toc {
    border-right: 1px solid #d8dee4;
    padding: 2rem 1.5rem;
}

.documentation-index-content {
    padding: 2rem clamp(1.5rem, 4vw, 4rem);
}

.documentation-book-grid {
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
}

.documentation-book-card {
    border: 1px solid #d8dee4;
    border-radius: 6px;
    color: inherit;
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
    text-decoration: none;
}

.documentation-book-cover,
.documentation-book-cover-fallback {
    aspect-ratio: 4 / 5;
    border: 1px solid #d8dee4;
    border-radius: 4px;
    display: grid;
    object-fit: cover;
    place-items: center;
    width: 100%;
}

@media (max-width: 760px) {
    .documentation-index {
        display: block;
    }

    .documentation-index-toc {
        border-right: 0;
        border-bottom: 1px solid #d8dee4;
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{InputBook, InputCatalog, InputCategory};
    use mdbook_driver::config::{BookConfig, Config};
    use std::path::PathBuf;

    #[test]
    fn renders_category_toc_sections_books_and_cover_fallbacks() {
        let catalog = sample_catalog();
        let html = render_documentation_index_html(&catalog, &Config::default())
            .expect("index should render");

        assert!(html.contains("<title>Documentation - Root Book</title>"));
        assert!(html.contains("Table of Contents"));
        assert!(html.contains("href=\"#category-start-here\""));
        assert!(
            html.contains("<section class=\"documentation-category\" id=\"category-start-here\">")
        );
        assert!(html.contains("Root Book"));
        assert!(html.contains("Repository-wide docs."));
        assert!(html.contains("href=\"root-book/docs/index.html\""));
        assert!(html.contains("src=\"assets/covers/root.png\""));
        assert!(html.contains("Parser Book"));
        assert!(html.contains("href=\"modules/parser/docs/index.html\""));
        assert!(html.contains("documentation-book-cover-fallback"));
    }

    #[test]
    fn escapes_index_text_and_attributes() {
        let mut catalog = sample_catalog();
        catalog.categories[0].title = "Start & <Here>".to_string();
        catalog.books[0].title = "Root \"Book\"".to_string();
        catalog.books[0].description = Some("A <trusted> & useful book.".to_string());
        let html = render_documentation_index_html(&catalog, &Config::default())
            .expect("index should render");

        assert!(html.contains("Start &amp; &lt;Here&gt;"));
        assert!(html.contains("Root &quot;Book&quot;"));
        assert!(html.contains("A &lt;trusted&gt; &amp; useful book."));
    }

    fn sample_catalog() -> InputCatalog {
        let mut mdbook_config = Config::default();
        mdbook_config.book.title = Some("Root Book".to_string());
        mdbook_config.book.language = Some("en".to_string());

        InputCatalog {
            config_path: PathBuf::from("bookshelf.toml"),
            config_dir: PathBuf::from("."),
            mdbook_config,
            asset_dir: PathBuf::from(".mdbook/bookshelf"),
            books: vec![
                sample_book(
                    "root",
                    "root-book/docs",
                    "Root Book",
                    Some("Repository-wide docs."),
                    Some("assets/covers/root.png"),
                    true,
                ),
                sample_book(
                    "parser",
                    "modules/parser/docs",
                    "Parser Book",
                    None,
                    None,
                    false,
                ),
            ],
            categories: vec![InputCategory {
                title: "Start Here".to_string(),
                book_ids: vec!["root".to_string(), "parser".to_string()],
            }],
        }
    }

    fn sample_book(
        id: &str,
        src: &str,
        title: &str,
        description: Option<&str>,
        cover: Option<&str>,
        is_root_book: bool,
    ) -> InputBook {
        let mut book_config = BookConfig::default();
        book_config.title = Some(title.to_string());
        book_config.description = description.map(str::to_string);
        book_config.src = PathBuf::from(src);

        InputBook {
            id: id.to_string(),
            output_rel: PathBuf::from(src),
            book_config,
            title: title.to_string(),
            description: description.map(str::to_string),
            cover: cover.map(PathBuf::from),
            book_root_rel: PathBuf::from("."),
            book_root_abs: PathBuf::from("/tmp"),
            book_src_rel: PathBuf::from(src),
            book_src_abs: PathBuf::from("/tmp").join(src),
            summary_abs: PathBuf::from("/tmp").join(src).join("SUMMARY.md"),
            is_root_book,
        }
    }
}
