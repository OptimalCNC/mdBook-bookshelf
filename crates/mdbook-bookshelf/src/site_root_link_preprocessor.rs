use crate::catalog::{InputBook, InputCatalog};
use crate::loader::load_books_from_catalog_with_progress;
use crate::route_paths::{path_to_string, relative_path};
use anyhow::{bail, Context, Result};
use mdbook_markdown::pulldown_cmark::{CowStr, Event, Tag};
use mdbook_preprocessor::book::{Book, Chapter};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PREPROCESSOR_NAME: &str = "bookshelf-site-root-links";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SiteRootLinkMap {
    pages: BTreeMap<PathBuf, PathBuf>,
}

impl SiteRootLinkMap {
    pub(crate) fn from_catalog_with_progress(
        catalog: &InputCatalog,
        before_book: impl FnMut(usize, usize, &InputBook),
    ) -> Result<Self> {
        let loaded = load_books_from_catalog_with_progress(catalog, before_book)
            .context("failed to load books for site-root Markdown link map")?;
        Self::from_catalog_and_loaded_books(catalog, &loaded)
    }

    fn from_catalog_and_loaded_books(
        catalog: &InputCatalog,
        loaded: &crate::loader::LoadedBooks,
    ) -> Result<Self> {
        let mut map = Self {
            pages: BTreeMap::new(),
        };

        for (index, (catalog_book, loaded_book)) in
            catalog.books.iter().zip(&loaded.books).enumerate()
        {
            if catalog_book.id != loaded_book.book_id {
                bail!(
                    "site-root Markdown link map book order mismatch at index {}: catalog='{}' loaded='{}'",
                    index,
                    catalog_book.id,
                    loaded_book.book_id
                );
            }

            for chapter in loaded_book.mdbook.book.chapters() {
                map.insert_chapter(catalog_book, chapter).with_context(|| {
                    format!(
                        "failed to add chapter '{}' from book '{}' to site-root Markdown link map",
                        chapter.name, catalog_book.id
                    )
                })?;
            }
        }

        Ok(map)
    }

    fn insert_chapter(&mut self, book: &InputBook, chapter: &Chapter) -> Result<()> {
        let Some(chapter_path) = chapter.path.as_deref() else {
            return Ok(());
        };
        let output_markdown_path = index_preprocessed_path(chapter_path);
        let output_html_path = normalize_site_path(
            &book
                .output_rel
                .join(output_markdown_path.with_extension("html")),
        )?;

        self.insert_page(
            normalize_site_path(&book.output_rel.join(chapter_path))?,
            output_html_path.clone(),
        )?;
        self.insert_page(
            normalize_site_path(&book.output_rel.join(&output_markdown_path))?,
            output_html_path.clone(),
        )?;

        if let Some(source_path) = chapter.source_path.as_deref() {
            self.insert_page(
                normalize_site_path(&book.output_rel.join(source_path))?,
                output_html_path,
            )?;
        }

        Ok(())
    }

    fn insert_page(&mut self, source_path: PathBuf, output_path: PathBuf) -> Result<()> {
        if let Some(existing) = self.pages.get(&source_path) {
            if existing != &output_path {
                bail!(
                    "site-root Markdown source path '{}' maps to both '{}' and '{}'",
                    path_to_string(&source_path),
                    path_to_string(existing),
                    path_to_string(&output_path)
                );
            }
            return Ok(());
        }

        self.pages.insert(source_path, output_path);
        Ok(())
    }

    fn page_output_path(&self, source_path: &Path) -> Option<&Path> {
        self.pages.get(source_path).map(PathBuf::as_path)
    }

    #[cfg(test)]
    fn from_page_pairs(pairs: &[(&str, &str)]) -> Self {
        let pages = pairs
            .iter()
            .map(|(source, output)| (PathBuf::from(source), PathBuf::from(output)))
            .collect();
        Self { pages }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SiteRootLinkPreprocessor {
    book_output_rel: PathBuf,
    link_map: SiteRootLinkMap,
}

impl SiteRootLinkPreprocessor {
    pub(crate) fn new(book_output_rel: PathBuf, link_map: SiteRootLinkMap) -> Self {
        Self {
            book_output_rel,
            link_map,
        }
    }

    fn rewrite_chapter(&self, chapter: &mut Chapter) -> Result<()> {
        let Some(chapter_path) = chapter.path.as_deref() else {
            return Ok(());
        };
        let chapter_output_path = self
            .book_output_rel
            .join(chapter_path.with_extension("html"));
        let chapter_output_dir = chapter_output_path
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let events = mdbook_markdown::new_cmark_parser(
            &chapter.content,
            &mdbook_markdown::MarkdownOptions::default(),
        )
        .map(|event| self.rewrite_event(event, chapter_output_dir, &chapter.name))
        .collect::<Result<Vec<_>>>()?;

        let mut rewritten = String::new();
        pulldown_cmark_to_cmark::cmark(events.into_iter(), &mut rewritten)
            .context("failed to serialize rewritten Markdown events")?;
        chapter.content = rewritten;
        Ok(())
    }

    fn rewrite_event<'a>(
        &self,
        event: Event<'a>,
        chapter_output_dir: &Path,
        chapter_name: &str,
    ) -> Result<Event<'a>> {
        match event {
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let dest_url =
                    self.rewrite_destination(dest_url, chapter_output_dir, chapter_name)?;
                Ok(Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                }))
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let dest_url =
                    self.rewrite_destination(dest_url, chapter_output_dir, chapter_name)?;
                Ok(Event::Start(Tag::Image {
                    link_type,
                    dest_url,
                    title,
                    id,
                }))
            }
            other => Ok(other),
        }
    }

    fn rewrite_destination<'a>(
        &self,
        destination: CowStr<'a>,
        chapter_output_dir: &Path,
        chapter_name: &str,
    ) -> Result<CowStr<'a>> {
        let destination_text = destination.as_ref();
        if !is_site_root_destination(destination_text) {
            return Ok(destination);
        }

        let (path_part, suffix) = split_destination_suffix(destination_text);
        let normalized_path =
            normalize_site_root_destination_path(path_part).with_context(|| {
                format!(
                    "chapter '{}' contains invalid site-root Markdown link '{}'",
                    chapter_name, destination_text
                )
            })?;

        let site_root_target = if normalized_path.as_os_str().is_empty() {
            PathBuf::from("index.html")
        } else if is_markdown_page_path(&normalized_path) {
            self.link_map
                .page_output_path(&normalized_path)
                .map(Path::to_path_buf)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "chapter '{}' contains unresolved site-root Markdown page link '{}'",
                        chapter_name,
                        destination_text
                    )
                })?
        } else {
            normalized_path
        };

        let mut rewritten = path_to_string(&relative_path(chapter_output_dir, &site_root_target));
        rewritten.push_str(suffix);
        Ok(CowStr::from(rewritten))
    }
}

impl Preprocessor for SiteRootLinkPreprocessor {
    fn name(&self) -> &str {
        PREPROCESSOR_NAME
    }

    fn run(
        &self,
        _ctx: &PreprocessorContext,
        mut book: Book,
    ) -> mdbook_preprocessor::errors::Result<Book> {
        let mut error = None;
        book.for_each_chapter_mut(|chapter| {
            if error.is_some() {
                return;
            }

            if let Err(err) = self.rewrite_chapter(chapter).with_context(|| {
                format!(
                    "chapter '{}' failed to rewrite site-root Markdown links",
                    chapter.name
                )
            }) {
                error = Some(err);
            }
        });

        if let Some(error) = error {
            return Err(error);
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> mdbook_preprocessor::errors::Result<bool> {
        Ok(renderer == "html")
    }
}

fn is_site_root_destination(destination: &str) -> bool {
    destination.starts_with('/') && !destination.starts_with("//")
}

fn split_destination_suffix(destination: &str) -> (&str, &str) {
    let suffix_start = destination
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
        .unwrap_or(destination.len());
    destination.split_at(suffix_start)
}

fn normalize_site_root_destination_path(path: &str) -> Result<PathBuf> {
    let Some(relative) = path.strip_prefix('/') else {
        bail!(
            "site-root Markdown link path '{}' must start with '/'",
            path
        );
    };

    normalize_url_path_components(relative)
}

fn normalize_site_path(path: &Path) -> Result<PathBuf> {
    normalize_url_path_components(&path_to_string(path))
}

fn normalize_url_path_components(path: &str) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();

    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if !normalized.pop() {
                    bail!("site-root Markdown link path '{}' escapes site root", path);
                }
            }
            component => normalized.push(component),
        }
    }

    Ok(normalized)
}

fn is_markdown_page_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn index_preprocessed_path(path: &Path) -> PathBuf {
    if is_readme_path(path) {
        path.with_file_name("index.md")
    } else {
        path.to_path_buf()
    }
}

fn is_readme_path(path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| stem.eq_ignore_ascii_case("readme"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_site_root_links_and_images_relative_to_current_page() {
        let rewritten = rewrite_sample_chapter(
            "onboarding.md",
            "docs",
            r#"# Onboarding

[Root](/)
[Core](/docs/index.md)
[Same book](/docs/architecture.md)
[Nested](/docs/deep/topic.md)
[Cross book](/modules/parser/docs/index.md?mode=fast#grammar)
![Diagram](/assets/diagram.svg#icon)
"#,
        )
        .expect("chapter should rewrite");

        assert!(rewritten.contains("[Root](../index.html)"));
        assert!(rewritten.contains("[Core](index.html)"));
        assert!(rewritten.contains("[Same book](architecture.html)"));
        assert!(rewritten.contains("[Nested](deep/topic.html)"));
        assert!(
            rewritten.contains("[Cross book](../modules/parser/docs/index.html?mode=fast#grammar)")
        );
        assert!(rewritten.contains("![Diagram](../assets/diagram.svg#icon)"));
    }

    #[test]
    fn rewrites_nested_cross_book_page_links() {
        let rewritten = rewrite_sample_chapter(
            "runtime.md",
            "modules/parser/docs",
            "[Sample UI](/modules/ui/docs/index.md)\n",
        )
        .expect("chapter should rewrite");

        assert!(rewritten.contains("[Sample UI](../../ui/docs/index.html)"));
    }

    #[test]
    fn preserves_query_and_fragment_for_root_and_page_links() {
        let rewritten = rewrite_sample_chapter(
            "runtime.md",
            "modules/parser/docs",
            "[Root](/?print=true#top)\n[Core](/docs/index.md?from=parser#intro)\n",
        )
        .expect("chapter should rewrite");

        assert!(rewritten.contains("[Root](../../../index.html?print=true#top)"));
        assert!(rewritten.contains("[Core](../../../docs/index.html?from=parser#intro)"));
    }

    #[test]
    fn leaves_external_protocol_fragment_and_local_links_unchanged() {
        let rewritten = rewrite_sample_chapter(
            "index.md",
            "docs",
            r#"[External](https://example.com/docs/index.md)
[Mail](mailto:team@example.com)
[Protocol](//cdn.example.com/app.js)
[Fragment](#section)
[Local](./onboarding.md)
[Parent](../README.md)
"#,
        )
        .expect("chapter should rewrite");

        assert!(rewritten.contains("[External](https://example.com/docs/index.md)"));
        assert!(rewritten.contains("[Mail](mailto:team@example.com)"));
        assert!(rewritten.contains("[Protocol](//cdn.example.com/app.js)"));
        assert!(rewritten.contains("[Fragment](#section)"));
        assert!(rewritten.contains("[Local](./onboarding.md)"));
        assert!(rewritten.contains("[Parent](../README.md)"));
    }

    #[test]
    fn rejects_unresolved_site_root_markdown_page_links() {
        let error = rewrite_sample_chapter("index.md", "docs", "[Missing](/missing.md)\n")
            .expect_err("unresolved .md page link should fail");
        let message = error.to_string();

        assert!(message.contains("chapter 'Sample'"));
        assert!(message.contains("unresolved site-root Markdown page link '/missing.md'"));
    }

    #[test]
    fn rejects_site_root_paths_that_escape_the_site_root() {
        let error = rewrite_sample_chapter("index.md", "docs", "[Bad](/../secret.md)\n")
            .expect_err("escaping path should fail");
        let message = format!("{error:#}");

        assert!(message.contains("chapter 'Sample'"));
        assert!(message.contains("escapes site root"));
    }

    #[test]
    fn readme_sources_are_available_as_index_markdown_targets() {
        let mut map = SiteRootLinkMap {
            pages: BTreeMap::new(),
        };
        let book = sample_book("docs");
        let chapter = Chapter::new("Read Me", "# Read Me".to_string(), "README.md", Vec::new());

        map.insert_chapter(&book, &chapter)
            .expect("README chapter should be inserted");

        assert_eq!(
            map.page_output_path(Path::new("docs/README.md")),
            Some(Path::new("docs/index.html"))
        );
        assert_eq!(
            map.page_output_path(Path::new("docs/index.md")),
            Some(Path::new("docs/index.html"))
        );
    }

    fn rewrite_sample_chapter(
        chapter_path: &str,
        book_output_rel: &str,
        content: &str,
    ) -> Result<String> {
        let preprocessor = SiteRootLinkPreprocessor::new(
            PathBuf::from(book_output_rel),
            SiteRootLinkMap::from_page_pairs(&[
                ("docs/index.md", "docs/index.html"),
                ("docs/architecture.md", "docs/architecture.html"),
                ("docs/deep/topic.md", "docs/deep/topic.html"),
                (
                    "modules/parser/docs/index.md",
                    "modules/parser/docs/index.html",
                ),
                ("modules/ui/docs/index.md", "modules/ui/docs/index.html"),
            ]),
        );
        let mut chapter = Chapter::new("Sample", content.to_string(), chapter_path, Vec::new());

        preprocessor.rewrite_chapter(&mut chapter)?;
        Ok(chapter.content)
    }

    fn sample_book(output_rel: &str) -> InputBook {
        InputBook {
            id: output_rel.to_string(),
            output_rel: PathBuf::from(output_rel),
            book_config: mdbook_preprocessor::config::BookConfig::default(),
            title: output_rel.to_string(),
            description: None,
            book_root_rel: PathBuf::from("."),
            book_root_abs: PathBuf::from("/tmp"),
            book_src_rel: PathBuf::from(output_rel),
            book_src_abs: PathBuf::from("/tmp").join(output_rel),
            summary_abs: PathBuf::from("/tmp").join(output_rel).join("SUMMARY.md"),
        }
    }
}
