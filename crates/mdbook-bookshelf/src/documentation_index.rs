#![allow(dead_code)]

use crate::catalog::{InputBook, InputCatalog};
use anyhow::{bail, Context, Result};
use mdbook_driver::config::Config;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

const GENERATED_COVER_ROOT: &str = ".mdbook/bookshelf/covers";

pub(crate) fn write_documentation_index(
    catalog: &InputCatalog,
    projected_config: &Config,
    site_dest_dir: &Path,
) -> Result<()> {
    let html = render_documentation_index_html(catalog, projected_config)?;
    validate_configured_covers(catalog)?;
    fs::create_dir_all(site_dest_dir).with_context(|| {
        format!(
            "failed to create site output directory {}",
            site_dest_dir.display()
        )
    })?;
    copy_configured_covers(catalog, site_dest_dir)?;
    let index_path: PathBuf = site_dest_dir.join("index.html");
    fs::write(&index_path, html).with_context(|| {
        format!(
            "failed to write documentation index at {}",
            index_path.display()
        )
    })
}

pub(crate) fn validate_documentation_index_inputs(catalog: &InputCatalog) -> Result<()> {
    validate_configured_covers(catalog)
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
    site_relative_url(&book.output_rel.join("index.html"))
}

fn cover_src(book: &InputBook) -> Option<String> {
    book.cover
        .as_ref()
        .map(|_| site_relative_url(&generated_cover_output_rel(book)))
}

fn generated_cover_output_rel(book: &InputBook) -> PathBuf {
    let mut output = PathBuf::from(GENERATED_COVER_ROOT).join(&book.id);

    if let Some(cover) = &book.cover {
        for component in cover.components() {
            if let Component::Normal(part) = component {
                output.push(part);
            }
        }
    }

    output
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

fn copy_configured_covers(catalog: &InputCatalog, site_dest_dir: &Path) -> Result<()> {
    for book in &catalog.books {
        let Some(cover) = &book.cover else {
            continue;
        };
        let source = catalog.config_dir.join(cover);
        let destination = site_dest_dir.join(generated_cover_output_rel(book));
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

fn validate_configured_covers(catalog: &InputCatalog) -> Result<()> {
    for book in &catalog.books {
        let Some(cover) = &book.cover else {
            continue;
        };

        if cover_is_html(cover) {
            bail!(
                "book '{}' documentation index cover '{}' must not be an HTML file",
                book.id,
                cover.display()
            );
        }
    }

    Ok(())
}

fn cover_is_html(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
        })
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
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

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
        assert!(html.contains("src=\".mdbook/bookshelf/covers/root/assets/covers/root.png\""));
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

    #[test]
    fn encodes_generated_book_and_cover_urls_per_path_component() {
        let mut catalog = sample_catalog();
        catalog.books[0].output_rel = PathBuf::from("root docs")
            .join("chapter#draft?review%done")
            .join("javascript:example");
        catalog.books[0].cover = Some(
            PathBuf::from("assets with spaces")
                .join("covers#draft")
                .join("root?100%.png"),
        );

        let html = render_documentation_index_html(&catalog, &Config::default())
            .expect("index should render");

        assert!(html.contains(
            "href=\"root%20docs/chapter%23draft%3Freview%25done/javascript%3Aexample/index.html\""
        ));
        assert!(html.contains(
            "src=\".mdbook/bookshelf/covers/root/assets%20with%20spaces/covers%23draft/root%3F100%25.png\""
        ));
    }

    #[test]
    fn write_documentation_index_writes_index_and_copies_configured_cover() {
        let config_root = TempDir::new("documentation-index-config-root");
        let site_output = TempDir::new("documentation-index-site-output");
        let cover_path = PathBuf::from("assets")
            .join("nested covers")
            .join("root cover.bin");
        let expected_cover_bytes = b"cover bytes";
        write_bytes(config_root.path(), &cover_path, expected_cover_bytes);
        let catalog = sample_catalog_at_config_root(config_root.path(), Some(cover_path.clone()));

        write_documentation_index(&catalog, &Config::default(), site_output.path())
            .expect("documentation index should be written");

        let index_path = site_output.path().join("index.html");
        let index_html =
            fs::read_to_string(&index_path).expect("documentation index should be readable");
        assert!(index_path.exists());
        assert!(index_html.contains("<div class=\"documentation-index\">"));
        assert!(index_html.contains("Root Book"));
        assert!(index_html.contains(
            "src=\".mdbook/bookshelf/covers/root/assets/nested%20covers/root%20cover.bin\""
        ));

        let copied_cover = site_output
            .path()
            .join(".mdbook/bookshelf/covers/root")
            .join(&cover_path);
        let copied_cover_bytes = fs::read(&copied_cover).unwrap_or_else(|err| {
            panic!(
                "failed to read copied cover {}: {err}",
                copied_cover.display()
            )
        });
        assert_eq!(copied_cover_bytes, expected_cover_bytes);
    }

    #[test]
    fn write_documentation_index_reports_missing_configured_cover_with_book_id() {
        let config_root = TempDir::new("documentation-index-missing-cover-config-root");
        let site_output = TempDir::new("documentation-index-missing-cover-site-output");
        let cover_path = PathBuf::from("assets")
            .join("nested covers")
            .join("missing cover.png");
        let catalog = sample_catalog_at_config_root(config_root.path(), Some(cover_path));

        let error = write_documentation_index(&catalog, &Config::default(), site_output.path())
            .expect_err("missing configured cover should fail");
        let error = format!("{error:#}");

        assert!(error.contains("failed to copy cover for book"));
        assert!(error.contains("root"));
    }

    #[test]
    fn write_documentation_index_rejects_html_root_cover_before_output_side_effects() {
        let config_root = TempDir::new("documentation-index-html-cover-config-root");
        let site_output = config_root.path().join("site-output");
        let cover_path = PathBuf::from("docs").join("index.html");
        let catalog = sample_catalog_at_config_root(config_root.path(), Some(cover_path));

        let error = write_documentation_index(&catalog, &Config::default(), &site_output)
            .expect_err("HTML cover should be rejected");
        let error = format!("{error:#}");

        assert!(
            error.contains(
                "book 'root' documentation index cover 'docs/index.html' must not be an HTML file"
            ),
            "unexpected error: {error}"
        );
        assert!(
            !site_output.exists(),
            "validation should fail before creating site output"
        );
    }

    #[test]
    fn write_documentation_index_rejects_htm_child_cover_with_book_id() {
        let config_root = TempDir::new("documentation-index-htm-cover-config-root");
        let site_output = config_root.path().join("site-output");
        let mut catalog = sample_catalog_at_config_root(config_root.path(), None);
        catalog.books[1].cover = Some(PathBuf::from("covers").join("parser.htm"));

        let error = write_documentation_index(&catalog, &Config::default(), &site_output)
            .expect_err("HTM cover should be rejected");
        let error = format!("{error:#}");

        assert!(
            error.contains(
                "book 'parser' documentation index cover 'covers/parser.htm' must not be an HTML file"
            ),
            "unexpected error: {error}"
        );
        assert!(
            !site_output.exists(),
            "validation should fail before creating site output"
        );
    }

    #[test]
    fn write_documentation_index_renders_before_copying_configured_covers() {
        let config_root = TempDir::new("documentation-index-invalid-catalog-config-root");
        let site_output = TempDir::new("documentation-index-invalid-catalog-site-output");
        let cover_path = PathBuf::from("assets")
            .join("nested covers")
            .join("root cover.bin");
        write_bytes(config_root.path(), &cover_path, b"cover bytes");
        let mut catalog =
            sample_catalog_at_config_root(config_root.path(), Some(cover_path.clone()));
        catalog.categories[0].book_ids.push("missing".to_string());

        let error = write_documentation_index(&catalog, &Config::default(), site_output.path())
            .expect_err("invalid catalog should fail before copying covers");
        let error = format!("{error:#}");

        assert!(error.contains("references unknown book 'missing'"));
        assert!(!site_output
            .path()
            .join(".mdbook/bookshelf/covers/root")
            .join(&cover_path)
            .exists());
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

    fn sample_catalog_at_config_root(
        config_root: &Path,
        root_cover: Option<PathBuf>,
    ) -> InputCatalog {
        let mut catalog = sample_catalog();
        catalog.config_path = config_root.join("bookshelf.toml");
        catalog.config_dir = config_root.to_path_buf();
        catalog.books[0].cover = root_cover;
        catalog
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

    fn write_bytes(root: &Path, rel: &Path, bytes: &[u8]) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }
        fs::write(&path, bytes)
            .unwrap_or_else(|err| panic!("failed to write fixture {}: {err}", path.display()));
    }
}
