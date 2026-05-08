use crate::catalog::{InputBook, InputCatalog};
use crate::documentation_ui::{DocumentationAssets, DocumentationPageMetadataPreprocessor};
use crate::load_single_book_with_config_and_parsed_summary;
use crate::search::SHARED_SEARCH_INDEX_NAME;
use anyhow::{Context, Result};
use mdbook_driver::config::Config;
use mdbook_summary::parse_summary;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

const DOCUMENTATION_INDEX_SOURCE_DIR: &str = "documentation-index";
const SUMMARY_MD: &str = "SUMMARY.md";
const INDEX_MD: &str = "index.md";

#[derive(Debug, Clone)]
pub(crate) struct DocumentationIndexSources {
    source_rel: PathBuf,
    summary: String,
}

pub(crate) fn write_documentation_index_sources(
    catalog: &InputCatalog,
) -> Result<DocumentationIndexSources> {
    let source_rel = documentation_index_source_rel(catalog);
    let source_abs = catalog.config_dir.join(&source_rel);
    let summary = render_documentation_index_summary(&catalog.index_title);
    let index = render_documentation_index_markdown(catalog)?;

    write_generated_file(&source_abs.join(SUMMARY_MD), &summary)?;
    write_generated_file(&source_abs.join(INDEX_MD), &index)?;

    Ok(DocumentationIndexSources {
        source_rel,
        summary,
    })
}

pub(crate) fn build_documentation_index_book(
    catalog: &InputCatalog,
    shared_config: &Config,
    site_dest_dir: &Path,
    documentation_assets: &DocumentationAssets,
    sources: &DocumentationIndexSources,
) -> Result<usize> {
    let summary = parse_summary(&sources.summary).with_context(|| {
        format!(
            "failed to parse generated documentation index summary at {}",
            catalog
                .config_dir
                .join(&sources.source_rel)
                .join(SUMMARY_MD)
                .display()
        )
    })?;

    let mut config = shared_config.clone();
    config.build.build_dir = site_dest_dir.to_path_buf();
    config
        .set("output.html.no-section-label", true)
        .context("failed to disable documentation index sidebar section labels")?;
    adapt_documentation_index_input_404(&mut config, &catalog.config_dir, &sources.source_rel)
        .context("failed to adapt documentation index 404 input")?;

    documentation_assets
        .append_documentation_index_assets(&mut config)
        .context("failed to inject documentation index runtime assets")?;

    let mut mdbook = load_single_book_with_config_and_parsed_summary(
        &catalog.config_dir,
        &sources.source_rel,
        config,
        summary,
    )
    .with_context(|| {
        format!(
            "failed to load generated documentation index mdBook from {}",
            catalog.config_dir.join(&sources.source_rel).display()
        )
    })?;
    let page_count = mdbook.book.chapters().count();

    mdbook.with_preprocessor(
        DocumentationPageMetadataPreprocessor::without_documentation_index_target(
            PathBuf::new(),
            PathBuf::from(SHARED_SEARCH_INDEX_NAME),
        ),
    );

    let html_build_dir = mdbook.build_dir_for("html");
    mdbook.build().with_context(|| {
        format!(
            "generated documentation index failed to build mdBook output at {}",
            html_build_dir.display()
        )
    })?;

    Ok(page_count)
}

fn adapt_documentation_index_input_404(
    config: &mut Config,
    config_root: &Path,
    source_rel: &Path,
) -> Result<()> {
    let Some(input_404) = config
        .html_config()
        .and_then(|html_config| html_config.input_404)
    else {
        return Ok(());
    };
    if input_404.is_empty() {
        return Ok(());
    }

    let input_404_path = Path::new(&input_404);
    let generated_source_input = if input_404_path.is_absolute() {
        input_404_path.to_path_buf()
    } else {
        config_root.join(source_rel).join(input_404_path)
    };
    if generated_source_input.is_file() {
        return Ok(());
    }

    let config_root_input = if input_404_path.is_absolute() {
        input_404_path.to_path_buf()
    } else {
        config_root.join(input_404_path)
    };
    if config_root_input.is_file() {
        config
            .set(
                "output.html.input-404",
                config_root_input.to_string_lossy().as_ref(),
            )
            .context("failed to point documentation index 404 input at config-root file")?;
    } else {
        config
            .set("output.html.input-404", "")
            .context("failed to disable missing documentation index 404 input")?;
    }

    Ok(())
}

fn documentation_index_source_rel(catalog: &InputCatalog) -> PathBuf {
    catalog.asset_dir.join(DOCUMENTATION_INDEX_SOURCE_DIR)
}

fn render_documentation_index_summary(index_title: &str) -> String {
    format!(
        "# Summary\n\n- [{}]({INDEX_MD})\n",
        escape_markdown_link_text(index_title)
    )
}

pub(crate) fn render_documentation_index_markdown(catalog: &InputCatalog) -> Result<String> {
    let books_by_id = catalog
        .books
        .iter()
        .map(|book| (book.id.as_str(), book))
        .collect::<BTreeMap<_, _>>();

    let mut markdown = String::new();
    markdown.push_str("# ");
    markdown.push_str(&catalog.index_title);
    markdown.push_str("\n\n");

    for category in &catalog.categories {
        markdown.push_str("## ");
        markdown.push_str(&category.title);
        markdown.push_str(
            "\n\n<div class=\"bookshelf\">\n<ul class=\"bookshelf-list\" role=\"list\">\n",
        );

        for book_id in &category.book_ids {
            let book = books_by_id.get(book_id.as_str()).with_context(|| {
                format!(
                    "documentation category '{}' references unknown book '{}'",
                    category.title, book_id
                )
            })?;
            render_book_card(&mut markdown, book);
        }

        markdown.push_str("</ul>\n</div>\n\n");
    }

    Ok(markdown)
}

fn render_book_card(markdown: &mut String, book: &InputBook) {
    markdown.push_str("<li>\n<a class=\"bookshelf-book\" href=\"");
    markdown.push_str(&escape_html_attr(&book_href(book)));
    markdown.push_str("\">\n<span class=\"bookshelf-book-title\">");
    markdown.push_str(&escape_html_text(&book.title));
    markdown.push_str("</span>\n");

    if let Some(description) = book.description.as_deref().map(str::trim) {
        if !description.is_empty() {
            markdown.push_str("<span class=\"bookshelf-book-description\">");
            markdown.push_str(&escape_html_text(description));
            markdown.push_str("</span>\n");
        }
    }

    markdown.push_str("</a>\n</li>\n");
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

fn escape_markdown_link_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if matches!(ch, '\\' | '[' | ']') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

fn escape_html_attr(text: &str) -> String {
    escape_html_text(text).replace('"', "&quot;")
}

fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn write_generated_file(path: &Path, contents: &str) -> Result<()> {
    let parent = path.parent().with_context(|| {
        format!(
            "generated documentation index path {} is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create generated documentation index directory {}",
            parent.display()
        )
    })?;
    match fs::read(path) {
        Ok(existing) if existing == contents.as_bytes() => return Ok(()),
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", path.display()));
        }
    }

    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{InputBook, InputCatalog, InputCategory};
    use mdbook_driver::config::{BookConfig, Config};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn renders_markdown_summary_categories_books_and_descriptions() {
        let catalog = sample_catalog();
        let summary = render_documentation_index_summary(&catalog.index_title);
        let index =
            render_documentation_index_markdown(&catalog).expect("index markdown should render");

        assert_eq!(summary, "# Summary\n\n- [Documentation](index.md)\n");
        assert_eq!(
            index,
            "\
# Documentation

## Start Here

<div class=\"bookshelf\">
<ul class=\"bookshelf-list\" role=\"list\">
<li>
<a class=\"bookshelf-book\" href=\"root-book/docs/index.html\">
<span class=\"bookshelf-book-title\">Root Book</span>
<span class=\"bookshelf-book-description\">Repository-wide docs.</span>
</a>
</li>
<li>
<a class=\"bookshelf-book\" href=\"modules/parser/docs/index.html\">
<span class=\"bookshelf-book-title\">Parser Book</span>
</a>
</li>
</ul>
</div>

"
        );
    }

    #[test]
    fn renders_configured_index_title() {
        let mut catalog = sample_catalog();
        catalog.index_title = "Docs Portal".to_string();
        let summary = render_documentation_index_summary(&catalog.index_title);
        let index =
            render_documentation_index_markdown(&catalog).expect("index markdown should render");

        assert_eq!(summary, "# Summary\n\n- [Docs Portal](index.md)\n");
        assert!(index.starts_with("# Docs Portal\n\n"));
        assert!(!index.contains("# Documentation\n\n"));
    }

    #[test]
    fn escapes_markdown_link_text() {
        let mut catalog = sample_catalog();
        catalog.index_title = "Docs [Portal]".to_string();
        catalog.categories[0].title = "Start & <Here>".to_string();
        catalog.books[0].title = "Root [Book]".to_string();
        catalog.books[0].description = Some("A <trusted> & useful book.".to_string());
        let summary = render_documentation_index_summary(&catalog.index_title);
        let index =
            render_documentation_index_markdown(&catalog).expect("index markdown should render");

        assert!(summary.contains("[Docs \\[Portal\\]](index.md)"));
        assert!(index.contains("# Docs [Portal]"));
        assert!(index.contains("## Start & <Here>"));
        assert!(index.contains("<span class=\"bookshelf-book-title\">Root [Book]</span>"));
        assert!(index.contains("A &lt;trusted&gt; &amp; useful book."));
    }

    #[test]
    fn encodes_generated_book_urls_per_path_component() {
        let mut catalog = sample_catalog();
        catalog.books[0].output_rel = PathBuf::from("root docs")
            .join("chapter#draft?review%done")
            .join("javascript:example");

        let index =
            render_documentation_index_markdown(&catalog).expect("index markdown should render");

        assert!(index.contains(
            "href=\"root%20docs/chapter%23draft%3Freview%25done/javascript%3Aexample/index.html\""
        ));
    }

    #[test]
    fn write_documentation_index_sources_writes_managed_source_files() {
        let config_root = TempDir::new("documentation-index-config-root");
        let catalog = sample_catalog_at_config_root(config_root.path());

        write_documentation_index_sources(&catalog)
            .expect("documentation index sources should be written");

        let source_dir = config_root
            .path()
            .join(".mdbook/bookshelf/documentation-index");
        assert_file_contents(
            source_dir.join("SUMMARY.md"),
            "# Summary\n\n- [Documentation](index.md)\n",
        );
        assert_file_contains(source_dir.join("index.md"), "# Documentation");
        assert_file_contains(
            source_dir.join("index.md"),
            "<a class=\"bookshelf-book\" href=\"root-book/docs/index.html\">",
        );
    }

    #[test]
    fn write_documentation_index_sources_keeps_unchanged_existing_files() {
        let config_root = TempDir::new("documentation-index-stable-source-write");
        let catalog = sample_catalog_at_config_root(config_root.path());
        write_documentation_index_sources(&catalog)
            .expect("initial documentation index sources should be written");

        let source_dir = config_root
            .path()
            .join(".mdbook/bookshelf/documentation-index");
        let summary_path = source_dir.join("SUMMARY.md");
        let index_path = source_dir.join("index.md");
        make_readonly(&summary_path);
        make_readonly(&index_path);

        let result = write_documentation_index_sources(&catalog);
        make_writable(&summary_path);
        make_writable(&index_path);
        result.expect("unchanged generated source files should not be rewritten");

        assert_file_contents(summary_path, "# Summary\n\n- [Documentation](index.md)\n");
    }

    #[test]
    fn write_documentation_index_sources_reports_invalid_catalog_before_writing_output() {
        let config_root = TempDir::new("documentation-index-invalid-catalog-config-root");
        let mut catalog = sample_catalog_at_config_root(config_root.path());
        catalog.categories[0].book_ids.push("missing".to_string());

        let error = write_documentation_index_sources(&catalog)
            .expect_err("invalid catalog should fail before writing");
        let error = format!("{error:#}");

        assert!(error.contains("references unknown book 'missing'"));
        assert!(!config_root
            .path()
            .join(".mdbook/bookshelf/documentation-index/SUMMARY.md")
            .exists());
        assert!(!config_root
            .path()
            .join(".mdbook/bookshelf/documentation-index/index.md")
            .exists());
    }

    #[test]
    fn generated_index_disables_missing_relative_custom_404() {
        let config_root = TempDir::new("documentation-index-missing-custom-404");
        let mut config = Config::default();
        config
            .set("output.html.input-404", "missing.md")
            .expect("custom 404 should configure");

        adapt_documentation_index_input_404(
            &mut config,
            config_root.path(),
            Path::new(".mdbook/bookshelf/documentation-index"),
        )
        .expect("custom 404 should adapt");

        assert_eq!(
            config
                .html_config()
                .expect("html config should exist")
                .input_404
                .as_deref(),
            Some("")
        );
    }

    #[test]
    fn generated_index_can_use_config_root_custom_404() {
        let config_root = TempDir::new("documentation-index-config-root-custom-404");
        fs::write(config_root.path().join("missing.md"), "# Missing\n")
            .expect("config-root 404 should be written");
        let mut config = Config::default();
        config
            .set("output.html.input-404", "missing.md")
            .expect("custom 404 should configure");

        adapt_documentation_index_input_404(
            &mut config,
            config_root.path(),
            Path::new(".mdbook/bookshelf/documentation-index"),
        )
        .expect("custom 404 should adapt");

        let expected = config_root
            .path()
            .join("missing.md")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            config
                .html_config()
                .expect("html config should exist")
                .input_404
                .as_deref(),
            Some(expected.as_str())
        );
    }

    fn sample_catalog() -> InputCatalog {
        let mut mdbook_config = Config::default();
        mdbook_config.book.title = Some("Root Book".to_string());
        mdbook_config.book.language = Some("en".to_string());

        InputCatalog {
            config_path: PathBuf::from("bookshelf.toml"),
            config_dir: PathBuf::from("."),
            mdbook_config,
            index_title: "Documentation".to_string(),
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

    fn assert_file_contents(path: PathBuf, expected: &str) {
        let actual = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert_eq!(actual, expected);
    }

    fn assert_file_contains(path: PathBuf, expected: &str) {
        let actual = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert!(
            actual.contains(expected),
            "{path:?} did not contain {expected:?}"
        );
    }

    fn make_readonly(path: &Path) {
        let mut permissions = fs::metadata(path)
            .unwrap_or_else(|err| panic!("failed to read {} metadata: {err}", path.display()))
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions)
            .unwrap_or_else(|err| panic!("failed to make {} read-only: {err}", path.display()));
    }

    fn make_writable(path: &Path) {
        let mut permissions = fs::metadata(path)
            .unwrap_or_else(|err| panic!("failed to read {} metadata: {err}", path.display()))
            .permissions();
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions)
            .unwrap_or_else(|err| panic!("failed to make {} writable: {err}", path.display()));
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
