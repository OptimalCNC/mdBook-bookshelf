use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::reader_context::{AuthoredPageReaderContext, ReaderContextModel};
use crate::render_manifest::{RenderManifest, RenderedPageIdentity, RenderedPageManifestEntry};
use crate::search::{build_search_index, write_search_index, SearchIndexError};
use crate::sidebar::{BookSidebar, SidebarChapter, SidebarModel};
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

#[derive(Debug)]
pub enum RenderSiteError {
    BookshelfRoot(RenderBookshelfRootError),
    Io { path: PathBuf, source: io::Error },
    MissingAuthoredManifestEntry { source_path: PathBuf },
    MissingBookshelfManifestEntry { page_id: String },
    MissingAdjacentManifestEntry { source_path: PathBuf },
    MissingSidebar { book_id: String },
    SearchIndex(SearchIndexError),
}

impl fmt::Display for RenderSiteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BookshelfRoot(error) => error.fmt(f),
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to render authored page output at {}: {}",
                    path.display(),
                    source
                )
            }
            Self::MissingAuthoredManifestEntry { source_path } => write!(
                f,
                "render manifest is missing authored page output for {}",
                source_path.display()
            ),
            Self::MissingBookshelfManifestEntry { page_id } => write!(
                f,
                "render manifest is missing synthetic Bookshelf page `{}`",
                page_id
            ),
            Self::MissingAdjacentManifestEntry { source_path } => write!(
                f,
                "render manifest is missing adjacent authored page output for {}",
                source_path.display()
            ),
            Self::MissingSidebar { book_id } => {
                write!(
                    f,
                    "sidebar model is missing sidebar data for book `{}`",
                    book_id
                )
            }
            Self::SearchIndex(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for RenderSiteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BookshelfRoot(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            Self::SearchIndex(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RenderBookshelfRootError> for RenderSiteError {
    fn from(value: RenderBookshelfRootError) -> Self {
        Self::BookshelfRoot(value)
    }
}

impl From<SearchIndexError> for RenderSiteError {
    fn from(value: SearchIndexError) -> Self {
        Self::SearchIndex(value)
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

pub fn render_site(
    output_dir: impl AsRef<Path>,
    site_model: &SiteModel,
    sidebar_model: &SidebarModel,
    reader_context: &ReaderContextModel,
    render_manifest: &RenderManifest,
) -> Result<Vec<PathBuf>, RenderSiteError> {
    let output_dir = output_dir.as_ref();
    let mut written_paths = Vec::with_capacity(reader_context.authored_page_contexts.len() + 1);
    written_paths.push(render_bookshelf_root_page(
        output_dir,
        site_model,
        render_manifest,
    )?);

    for context in &reader_context.authored_page_contexts {
        let manifest_entry = render_manifest
            .authored_entry_for_source_path(&context.source_path)
            .ok_or_else(|| RenderSiteError::MissingAuthoredManifestEntry {
                source_path: context.source_path.clone(),
            })?;
        let sidebar = sidebar_model
            .sidebar_for(&context.active_book.book_id)
            .ok_or_else(|| RenderSiteError::MissingSidebar {
                book_id: context.active_book.book_id.clone(),
            })?;

        let output_path = output_dir.join(&manifest_entry.output_path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|source| RenderSiteError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let html = render_authored_page_html(
            context,
            sidebar,
            render_manifest,
            manifest_entry,
            &reader_context.bookshelf_page_context.page_id,
            site_model
                .config_path
                .parent()
                .unwrap_or_else(|| Path::new(".")),
        )?;
        fs::write(&output_path, html).map_err(|source| RenderSiteError::Io {
            path: output_path.clone(),
            source,
        })?;
        written_paths.push(output_path);
    }

    Ok(written_paths)
}

pub fn render_site_with_search_index(
    output_dir: impl AsRef<Path>,
    site_model: &SiteModel,
    sidebar_model: &SidebarModel,
    reader_context: &ReaderContextModel,
    render_manifest: &RenderManifest,
) -> Result<Vec<PathBuf>, RenderSiteError> {
    let output_dir = output_dir.as_ref();
    let mut written_paths = render_site(
        output_dir,
        site_model,
        sidebar_model,
        reader_context,
        render_manifest,
    )?;
    let search_index = build_search_index(reader_context, render_manifest)?;
    let search_index_path = write_search_index(output_dir, &search_index)?;
    written_paths.push(search_index_path);
    Ok(written_paths)
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

fn render_authored_page_html(
    context: &AuthoredPageReaderContext,
    sidebar: &BookSidebar,
    render_manifest: &RenderManifest,
    current_entry: &RenderedPageManifestEntry,
    bookshelf_page_id: &str,
    config_dir: &Path,
) -> Result<String, RenderSiteError> {
    let markdown =
        fs::read_to_string(&context.source_path).map_err(|source| RenderSiteError::Io {
            path: context.source_path.clone(),
            source,
        })?;
    let content_html = render_markdown(
        &markdown,
        &context.source_path,
        &current_entry.output_path,
        render_manifest,
        config_dir,
    );

    let bookshelf_entry = render_manifest
        .entry_for_page_id(bookshelf_page_id)
        .ok_or_else(|| RenderSiteError::MissingBookshelfManifestEntry {
            page_id: bookshelf_page_id.to_owned(),
        })?;
    let bookshelf_href = relative_href(&current_entry.output_path, &bookshelf_entry.output_path);

    let sidebar_affixes = render_sidebar_affixes(
        &sidebar.affix_entries,
        &current_entry.output_path,
        &bookshelf_entry.output_path,
    );
    let sidebar_chapters = render_sidebar_chapters(
        &sidebar.chapter_entries,
        &current_entry.output_path,
        render_manifest,
    )?;

    let previous_link = match &context.previous_page {
        Some(link) => {
            let manifest_entry = render_manifest
                .authored_entry_for_source_path(&link.source_path)
                .ok_or_else(|| RenderSiteError::MissingAdjacentManifestEntry {
                    source_path: link.source_path.clone(),
                })?;
            format!(
                "<a class=\"nav-chapter previous\" href=\"{href}\">Previous</a>",
                href = escape_html(&relative_href(
                    &current_entry.output_path,
                    &manifest_entry.output_path
                ))
            )
        }
        None => String::new(),
    };
    let next_link = match &context.next_page {
        Some(link) => {
            let manifest_entry = render_manifest
                .authored_entry_for_source_path(&link.source_path)
                .ok_or_else(|| RenderSiteError::MissingAdjacentManifestEntry {
                    source_path: link.source_path.clone(),
                })?;
            format!(
                "<a class=\"nav-chapter next\" href=\"{href}\">Next</a>",
                href = escape_html(&relative_href(
                    &current_entry.output_path,
                    &manifest_entry.output_path
                ))
            )
        }
        None => String::new(),
    };

    Ok(format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>{title}</title></head><body><div id=\"page-wrapper\" class=\"page-wrapper\"><nav id=\"sidebar\" class=\"sidebar\"><div class=\"sidebar-scrollbox\">{affixes}<ol class=\"chapter\">{chapters}</ol></div></nav><main id=\"content\" class=\"content\" role=\"main\"><div class=\"menu-bar\"><a class=\"bookshelf-return\" href=\"{bookshelf_href}\">Bookshelf</a></div><nav class=\"breadcrumbs\">{book_title} / {page_title}</nav><div class=\"page\">{content_html}</div><div class=\"nav-wrapper\">{previous_link}{next_link}</div></main></div></body></html>",
        title = escape_html(&context.page_title),
        affixes = sidebar_affixes,
        chapters = sidebar_chapters,
        bookshelf_href = escape_html(&bookshelf_href),
        book_title = escape_html(&context.breadcrumbs.book_title),
        page_title = escape_html(&context.breadcrumbs.page_title),
        content_html = content_html,
        previous_link = previous_link,
        next_link = next_link
    ))
}

fn render_sidebar_affixes(
    affix_entries: &[crate::sidebar::SidebarAffixEntry],
    current_output_path: &Path,
    bookshelf_output_path: &Path,
) -> String {
    if affix_entries.is_empty() {
        return String::new();
    }

    let mut markup = String::from("<div class=\"sidebar-affix\">");
    for entry in affix_entries {
        markup.push_str(&format!(
            "<a class=\"affix\" href=\"{href}\">{title}</a>",
            href = escape_html(&relative_href(current_output_path, bookshelf_output_path)),
            title = escape_html(&entry.title)
        ));
    }
    markup.push_str("</div>");
    markup
}

fn render_sidebar_chapters(
    chapters: &[SidebarChapter],
    current_output_path: &Path,
    render_manifest: &RenderManifest,
) -> Result<String, RenderSiteError> {
    let mut markup = String::new();

    for chapter in chapters {
        let manifest_entry = render_manifest
            .authored_entry_for_source_path(&chapter.source_path)
            .ok_or_else(|| RenderSiteError::MissingAuthoredManifestEntry {
                source_path: chapter.source_path.clone(),
            })?;
        let href = relative_href(current_output_path, &manifest_entry.output_path);
        let children =
            render_sidebar_chapters(&chapter.children, current_output_path, render_manifest)?;
        markup.push_str(&format!(
            "<li class=\"chapter-item\"><a href=\"{href}\">{title}</a>{children_markup}</li>",
            href = escape_html(&href),
            title = escape_html(&chapter.title),
            children_markup = if children.is_empty() {
                String::new()
            } else {
                format!("<ol class=\"section\">{children}</ol>")
            }
        ));
    }

    Ok(markup)
}

fn render_markdown(
    markdown: &str,
    current_source_path: &Path,
    current_output_path: &Path,
    render_manifest: &RenderManifest,
    config_dir: &Path,
) -> String {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut index = 0;
    let mut html = String::new();

    while index < lines.len() {
        let line = lines[index].trim();
        if line.is_empty() {
            index += 1;
            continue;
        }

        if let Some(text) = line.strip_prefix("# ") {
            html.push_str(&format!(
                "<h1>{}</h1>",
                render_inline_markdown(
                    text,
                    current_source_path,
                    current_output_path,
                    render_manifest,
                    config_dir,
                )
            ));
            index += 1;
            continue;
        }

        if let Some(text) = line.strip_prefix("## ") {
            html.push_str(&format!(
                "<h2>{}</h2>",
                render_inline_markdown(
                    text,
                    current_source_path,
                    current_output_path,
                    render_manifest,
                    config_dir,
                )
            ));
            index += 1;
            continue;
        }

        if line.starts_with("- ") {
            html.push_str("<ul>");
            while index < lines.len() {
                let item = lines[index].trim();
                let Some(item_text) = item.strip_prefix("- ") else {
                    break;
                };
                html.push_str(&format!(
                    "<li>{}</li>",
                    render_inline_markdown(
                        item_text,
                        current_source_path,
                        current_output_path,
                        render_manifest,
                        config_dir,
                    )
                ));
                index += 1;
            }
            html.push_str("</ul>");
            continue;
        }

        let mut paragraph = String::new();
        while index < lines.len() {
            let current = lines[index].trim();
            if current.is_empty() || current.starts_with("#") || current.starts_with("- ") {
                break;
            }
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(current);
            index += 1;
        }
        html.push_str(&format!(
            "<p>{}</p>",
            render_inline_markdown(
                &paragraph,
                current_source_path,
                current_output_path,
                render_manifest,
                config_dir,
            )
        ));
    }

    html
}

fn render_inline_markdown(
    input: &str,
    current_source_path: &Path,
    current_output_path: &Path,
    render_manifest: &RenderManifest,
    config_dir: &Path,
) -> String {
    let mut rendered = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        match chars[index] {
            '[' => {
                if let Some(close_label) = chars[index + 1..].iter().position(|ch| *ch == ']') {
                    let close_label = index + 1 + close_label;
                    if chars.get(close_label + 1) == Some(&'(') {
                        if let Some(close_target) =
                            chars[close_label + 2..].iter().position(|ch| *ch == ')')
                        {
                            let close_target = close_label + 2 + close_target;
                            let label: String = chars[index + 1..close_label].iter().collect();
                            let target: String =
                                chars[close_label + 2..close_target].iter().collect();
                            let href = resolve_inline_href(
                                &target,
                                current_source_path,
                                current_output_path,
                                render_manifest,
                                config_dir,
                            );
                            rendered.push_str(&format!(
                                "<a href=\"{href}\">{label}</a>",
                                href = escape_html(&href),
                                label = escape_html(&label)
                            ));
                            index = close_target + 1;
                            continue;
                        }
                    }
                }
                rendered.push_str(&escape_html("["));
                index += 1;
            }
            '`' => {
                if let Some(close_code) = chars[index + 1..].iter().position(|ch| *ch == '`') {
                    let close_code = index + 1 + close_code;
                    let code: String = chars[index + 1..close_code].iter().collect();
                    rendered.push_str(&format!("<code>{}</code>", escape_html(&code)));
                    index = close_code + 1;
                    continue;
                }
                rendered.push_str(&escape_html("`"));
                index += 1;
            }
            other => {
                let mut buffer = String::new();
                buffer.push(other);
                rendered.push_str(&escape_html(&buffer));
                index += 1;
            }
        }
    }

    rendered
}

fn resolve_inline_href(
    target: &str,
    current_source_path: &Path,
    current_output_path: &Path,
    render_manifest: &RenderManifest,
    config_dir: &Path,
) -> String {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with('#')
    {
        return target.to_owned();
    }

    let (path_part, suffix) = split_target_suffix(target);
    if !path_part.ends_with(".md") && !path_part.ends_with(".markdown") {
        return target.to_owned();
    }

    let candidate_source = if let Some(stripped) = path_part.strip_prefix('/') {
        config_dir.join(stripped)
    } else {
        current_source_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path_part)
    };

    let candidate_source = normalize_path(&candidate_source);

    match authored_manifest_entry_for_path(render_manifest, &candidate_source) {
        Some(entry) => format!(
            "{}{}",
            relative_href(current_output_path, &entry.output_path),
            suffix
        ),
        None => target.to_owned(),
    }
}

fn split_target_suffix(target: &str) -> (&str, &str) {
    match target.find(['#', '?']) {
        Some(index) => (&target[..index], &target[index..]),
        None => (target, ""),
    }
}

fn authored_manifest_entry_for_path<'a>(
    render_manifest: &'a RenderManifest,
    source_path: &Path,
) -> Option<&'a RenderedPageManifestEntry> {
    render_manifest.entries.iter().find(|entry| {
        matches!(
            &entry.identity,
            RenderedPageIdentity::AuthoredPage { source_path: entry_source_path }
                if entry_source_path == source_path
        )
    })
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

fn render_href(manifest_entry: &RenderedPageManifestEntry) -> String {
    match &manifest_entry.identity {
        RenderedPageIdentity::SyntheticPage { .. } => String::new(),
        RenderedPageIdentity::AuthoredPage { .. } => {
            manifest_entry.output_path.to_string_lossy().into_owned()
        }
    }
}

fn relative_href(from_output_path: &Path, to_output_path: &Path) -> String {
    let from_dir = from_output_path.parent().unwrap_or_else(|| Path::new(""));
    let from_components: Vec<Component<'_>> = from_dir.components().collect();
    let to_components: Vec<Component<'_>> = to_output_path.components().collect();

    let mut shared = 0;
    while shared < from_components.len()
        && shared < to_components.len()
        && from_components[shared] == to_components[shared]
    {
        shared += 1;
    }

    let mut relative = PathBuf::new();
    for _ in shared..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[shared..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        String::from(".")
    } else {
        relative.to_string_lossy().into_owned()
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

#[cfg(test)]
mod tests {
    use super::{normalize_path, relative_href, render_markdown, split_target_suffix};
    use crate::render_manifest::{RenderManifest, RenderedPageIdentity, RenderedPageManifestEntry};
    use std::path::{Path, PathBuf};

    #[test]
    fn computes_relative_hrefs_between_outputs() {
        assert_eq!(
            relative_href(
                Path::new("modules/parser/docs/grammar.html"),
                Path::new("index.html")
            ),
            "../../../index.html"
        );
        assert_eq!(
            relative_href(Path::new("docs/onboarding.html"), Path::new("index.html")),
            "../index.html"
        );
        assert_eq!(
            relative_href(
                Path::new("modules/parser/docs/grammar.html"),
                Path::new("modules/parser/docs/runtime.html")
            ),
            "runtime.html"
        );
    }

    #[test]
    fn renders_basic_markdown_blocks_and_inline_markup() {
        let markdown = "# Title\n\nParagraph with [link](./next.md) and `code`.\n\n- One\n- Two\n";
        let manifest = RenderManifest {
            entries: vec![RenderedPageManifestEntry {
                identity: RenderedPageIdentity::AuthoredPage {
                    source_path: PathBuf::from("/tmp/docs/next.md"),
                },
                title: "Next".to_owned(),
                route_path: PathBuf::from("docs/next.html"),
                output_path: PathBuf::from("docs/next.html"),
            }],
        };
        let html = render_markdown(
            markdown,
            Path::new("/tmp/docs/index.md"),
            Path::new("docs/index.html"),
            &manifest,
            Path::new("/tmp"),
        );

        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains(
            "<p>Paragraph with <a href=\"next.html\">link</a> and <code>code</code>.</p>"
        ));
        assert!(html.contains("<ul><li>One</li><li>Two</li></ul>"));
    }

    #[test]
    fn splits_link_targets_from_suffixes() {
        assert_eq!(
            split_target_suffix("./page.md#part"),
            ("./page.md", "#part")
        );
        assert_eq!(split_target_suffix("./page.md?x=1"), ("./page.md", "?x=1"));
        assert_eq!(split_target_suffix("./page.md"), ("./page.md", ""));
    }

    #[test]
    fn normalizes_paths_lexically_without_filesystem_access() {
        assert_eq!(
            normalize_path(Path::new("/tmp/docs/./nested/../page.md")),
            PathBuf::from("/tmp/docs/page.md")
        );
    }
}
