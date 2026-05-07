#![allow(dead_code)]

use crate::catalog::{InputBook, InputCatalog};
use anyhow::{Context, Result};
use mdbook_driver::config::Config;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) fn write_documentation_index(
    catalog: &InputCatalog,
    projected_config: &Config,
    site_dest_dir: &Path,
) -> Result<()> {
    let html = render_documentation_index_html(catalog, projected_config)?;
    fs::create_dir_all(site_dest_dir).with_context(|| {
        format!(
            "failed to create site output directory {}",
            site_dest_dir.display()
        )
    })?;
    let index_path: PathBuf = site_dest_dir.join("index.html");
    fs::write(&index_path, html).with_context(|| {
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
    let live_reload_endpoint = projected_config
        .get::<String>("output.html.live-reload-endpoint")
        .context("failed to read output.html.live-reload-endpoint")?;

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
    html.push_str("</div>\n");
    if let Some(endpoint) = live_reload_endpoint.as_deref() {
        render_live_reload_script(&mut html, endpoint)?;
    }
    html.push_str("</body>\n</html>\n");

    Ok(html)
}

fn render_live_reload_script(html: &mut String, endpoint: &str) -> Result<()> {
    let endpoint_json =
        serde_json::to_string(endpoint).context("failed to serialize live-reload endpoint")?;
    html.push_str(
        "<!-- Livereload script (if served using the cli tool) -->\n<script>\n\
const wsProtocol = location.protocol === 'https:' ? 'wss:' : 'ws:';\n\
const wsAddress = wsProtocol + \"//\" + location.host + \"/\" + ",
    );
    html.push_str(&endpoint_json);
    html.push_str(
        ";\n\
const socket = new WebSocket(wsAddress);\n\
socket.onmessage = function (event) {\n\
    if (event.data === \"reload\") {\n\
        socket.close();\n\
        location.reload();\n\
    }\n\
};\n\
window.onbeforeunload = function() {\n\
    socket.close();\n\
};\n\
</script>\n",
    );
    Ok(())
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

fn book_href(book: &InputBook) -> String {
    site_relative_url(&book.output_rel.join("index.html"))
}

fn site_relative_url(path: &Path) -> String {
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(percent_encode_path_component(&part.to_string_lossy())),
            Component::ParentDir => Some("..".to_string()),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => None,
        })
        .collect::<Vec<_>>();

    if components.is_empty() {
        ".".to_string()
    } else {
        components.join("/")
    }
}

fn percent_encode_path_component(component: &str) -> String {
    let mut encoded = String::new();
    for byte in component.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(*byte));
            }
            _ => {
                encoded.push('%');
                encoded.push(hex_digit(byte >> 4));
                encoded.push(hex_digit(byte & 0x0f));
            }
        }
    }
    encoded
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'A' + value - 10),
        _ => unreachable!("hex digit nibble must be in range"),
    }
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
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn renders_category_toc_sections_and_book_text_without_covers() {
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
        assert!(html.contains("Parser Book"));
        assert!(html.contains("href=\"modules/parser/docs/index.html\""));
        assert!(!html.contains("documentation-book-cover"));
        assert!(!html.contains(".mdbook/bookshelf/covers"));
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

    #[test]
    fn encodes_generated_book_urls_per_path_component() {
        let mut catalog = sample_catalog();
        catalog.books[0].output_rel = PathBuf::from("root docs")
            .join("chapter#draft?review%done")
            .join("javascript:example");

        let html = render_documentation_index_html(&catalog, &Config::default())
            .expect("index should render");

        assert!(html.contains(
            "href=\"root%20docs/chapter%23draft%3Freview%25done/javascript%3Aexample/index.html\""
        ));
        assert!(!html.contains(".mdbook/bookshelf/covers"));
    }

    #[test]
    fn write_documentation_index_writes_index_without_cover_assets() {
        let config_root = TempDir::new("documentation-index-config-root");
        let site_output = TempDir::new("documentation-index-site-output");
        let catalog = sample_catalog_at_config_root(config_root.path());

        write_documentation_index(&catalog, &Config::default(), site_output.path())
            .expect("documentation index should be written");

        let index_path = site_output.path().join("index.html");
        let index_html =
            fs::read_to_string(&index_path).expect("documentation index should be readable");
        assert!(index_path.exists());
        assert!(index_html.contains("<div class=\"documentation-index\">"));
        assert!(index_html.contains("Root Book"));
        assert!(!site_output.path().join(".mdbook/bookshelf/covers").exists());
    }

    #[test]
    fn write_documentation_index_reports_invalid_catalog_before_writing_output() {
        let config_root = TempDir::new("documentation-index-invalid-catalog-config-root");
        let site_output = TempDir::new("documentation-index-invalid-catalog-site-output");
        let mut catalog = sample_catalog_at_config_root(config_root.path());
        catalog.categories[0].book_ids.push("missing".to_string());

        let error = write_documentation_index(&catalog, &Config::default(), site_output.path())
            .expect_err("invalid catalog should fail");
        let error = format!("{error:#}");

        assert!(error.contains("references unknown book 'missing'"));
        assert!(!site_output.path().join("index.html").exists());
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
                ),
                sample_book("parser", "modules/parser/docs", "Parser Book", None),
            ],
            categories: vec![InputCategory {
                title: "Start Here".to_string(),
                book_ids: vec!["root".to_string(), "parser".to_string()],
            }],
        }
    }

    fn sample_catalog_at_config_root(config_root: &Path) -> InputCatalog {
        let mut catalog = sample_catalog();
        catalog.config_path = config_root.join("bookshelf.toml");
        catalog.config_dir = config_root.to_path_buf();
        catalog
    }

    fn sample_book(id: &str, src: &str, title: &str, description: Option<&str>) -> InputBook {
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
            book_root_rel: PathBuf::from("."),
            book_root_abs: PathBuf::from("/tmp"),
            book_src_rel: PathBuf::from(src),
            book_src_abs: PathBuf::from("/tmp").join(src),
            summary_abs: PathBuf::from("/tmp").join(src).join("SUMMARY.md"),
        }
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be valid")
                .as_nanos();
            let path = std::env::temp_dir()
                .join("mdbook-bookshelf")
                .join(format!("{tag}-{nanos}"));
            fs::create_dir_all(&path).expect("temp directory should be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
