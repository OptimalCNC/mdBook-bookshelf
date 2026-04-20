use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::sidebar::{SidebarChapter, SidebarModel};
use crate::site_model::{AuthoredPage, SiteBook, SiteModel};

#[derive(Debug)]
pub enum BuildReaderContextError {
    MissingBook {
        book_id: String,
    },
    MissingSidebar {
        book_id: String,
    },
    MissingAuthoredPage {
        book_id: String,
        source_path: PathBuf,
    },
    MissingSidebarChapter {
        book_id: String,
        source_path: PathBuf,
    },
    SourcePathIo {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for BuildReaderContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBook { book_id } => {
                write!(f, "site model is missing book `{}` while building reader context", book_id)
            }
            Self::MissingSidebar { book_id } => {
                write!(f, "sidebar model is missing book `{}` while building reader context", book_id)
            }
            Self::MissingAuthoredPage { book_id, source_path } => write!(
                f,
                "site model is missing authored page {} for book `{}` while building reader context",
                source_path.display(),
                book_id
            ),
            Self::MissingSidebarChapter { book_id, source_path } => write!(
                f,
                "sidebar model is missing chapter {} for book `{}` while building reader context",
                source_path.display(),
                book_id
            ),
            Self::SourcePathIo { path, source } => {
                write!(f, "failed to normalize source path {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for BuildReaderContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SourcePathIo { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReaderContextModel {
    pub authored_page_contexts: Vec<AuthoredPageReaderContext>,
    pub bookshelf_page_context: BookshelfPageReaderContext,
}

impl ReaderContextModel {
    pub fn authored_context_for_source_path(
        &self,
        source_path: impl AsRef<Path>,
    ) -> Option<&AuthoredPageReaderContext> {
        let normalized = source_path.as_ref().canonicalize().ok()?;
        self.authored_page_contexts
            .iter()
            .find(|context| context.source_path == normalized)
    }

    pub fn bookshelf_context_for_page_id(
        &self,
        page_id: &str,
    ) -> Option<&BookshelfPageReaderContext> {
        (self.bookshelf_page_context.page_id == page_id).then_some(&self.bookshelf_page_context)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveBookContext {
    pub book_id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breadcrumbs {
    pub book_title: String,
    pub page_title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjacentPageLink {
    pub title: String,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookshelfReturn {
    pub page_id: String,
    pub title: String,
    pub route_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredPageReaderContext {
    pub source_path: PathBuf,
    pub page_title: String,
    pub active_book: ActiveBookContext,
    pub breadcrumbs: Breadcrumbs,
    pub previous_page: Option<AdjacentPageLink>,
    pub next_page: Option<AdjacentPageLink>,
    pub bookshelf_return: BookshelfReturn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookshelfPageReaderContext {
    pub page_id: String,
    pub title: String,
    pub route_path: PathBuf,
    pub active_book: ActiveBookContext,
    pub breadcrumbs: Breadcrumbs,
    pub previous_page: Option<AdjacentPageLink>,
    pub next_page: Option<AdjacentPageLink>,
}

pub fn build_reader_context_model(
    site_model: &SiteModel,
    sidebar_model: &SidebarModel,
) -> Result<ReaderContextModel, BuildReaderContextError> {
    let authored_page_by_path: HashMap<_, _> = site_model
        .authored_pages
        .iter()
        .map(|page| (page.source_path.clone(), page))
        .collect();
    let mut authored_page_contexts = Vec::with_capacity(site_model.authored_pages.len());

    for book in &site_model.books {
        let sidebar = sidebar_model.sidebar_for(&book.id).ok_or_else(|| {
            BuildReaderContextError::MissingSidebar {
                book_id: book.id.clone(),
            }
        })?;

        for (index, source_path) in book.authored_page_order.iter().enumerate() {
            authored_page_by_path
                .get(source_path)
                .copied()
                .ok_or_else(|| BuildReaderContextError::MissingAuthoredPage {
                    book_id: book.id.clone(),
                    source_path: source_path.clone(),
                })?;
            let page_title =
                find_sidebar_title(&sidebar.chapter_entries, source_path).ok_or_else(|| {
                    BuildReaderContextError::MissingSidebarChapter {
                        book_id: book.id.clone(),
                        source_path: source_path.clone(),
                    }
                })?;

            authored_page_contexts.push(AuthoredPageReaderContext {
                source_path: source_path.clone(),
                page_title: page_title.clone(),
                active_book: ActiveBookContext {
                    book_id: book.id.clone(),
                    title: book.title.clone(),
                },
                breadcrumbs: Breadcrumbs {
                    book_title: book.title.clone(),
                    page_title,
                },
                previous_page: adjacent_page_link(
                    book,
                    &authored_page_by_path,
                    index.checked_sub(1),
                )?,
                next_page: adjacent_page_link(book, &authored_page_by_path, Some(index + 1))?,
                bookshelf_return: BookshelfReturn {
                    page_id: site_model.bookshelf_page.page_id.clone(),
                    title: site_model.bookshelf_page.title.clone(),
                    route_path: site_model.bookshelf_page.route_path.clone(),
                },
            });
        }
    }

    let root_book = site_model
        .books
        .iter()
        .find(|book| book.id == site_model.bookshelf_page.owner_book_id)
        .ok_or_else(|| BuildReaderContextError::MissingBook {
            book_id: site_model.bookshelf_page.owner_book_id.clone(),
        })?;

    Ok(ReaderContextModel {
        authored_page_contexts,
        bookshelf_page_context: BookshelfPageReaderContext {
            page_id: site_model.bookshelf_page.page_id.clone(),
            title: site_model.bookshelf_page.title.clone(),
            route_path: site_model.bookshelf_page.route_path.clone(),
            active_book: ActiveBookContext {
                book_id: root_book.id.clone(),
                title: root_book.title.clone(),
            },
            breadcrumbs: Breadcrumbs {
                book_title: root_book.title.clone(),
                page_title: site_model.bookshelf_page.title.clone(),
            },
            previous_page: None,
            next_page: None,
        },
    })
}

fn adjacent_page_link(
    book: &SiteBook,
    authored_page_by_path: &HashMap<PathBuf, &AuthoredPage>,
    index: Option<usize>,
) -> Result<Option<AdjacentPageLink>, BuildReaderContextError> {
    let Some(index) = index else {
        return Ok(None);
    };
    let Some(source_path) = book.authored_page_order.get(index) else {
        return Ok(None);
    };
    let page = authored_page_by_path
        .get(source_path)
        .copied()
        .ok_or_else(|| BuildReaderContextError::MissingAuthoredPage {
            book_id: book.id.clone(),
            source_path: source_path.clone(),
        })?;

    Ok(Some(AdjacentPageLink {
        title: page.title.clone(),
        source_path: page.source_path.clone(),
    }))
}

fn find_sidebar_title(chapters: &[SidebarChapter], source_path: &Path) -> Option<String> {
    for chapter in chapters {
        if chapter.source_path == source_path {
            return Some(chapter.title.clone());
        }
        if let Some(title) = find_sidebar_title(&chapter.children, source_path) {
            return Some(title);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::find_sidebar_title;
    use crate::sidebar::SidebarChapter;
    use std::path::PathBuf;

    #[test]
    fn finds_nested_sidebar_titles_by_source_path() {
        let chapters = vec![SidebarChapter {
            title: "Root".to_owned(),
            source_path: PathBuf::from("/tmp/root.md"),
            summary_depth: 0,
            children: vec![SidebarChapter {
                title: "Nested".to_owned(),
                source_path: PathBuf::from("/tmp/nested.md"),
                summary_depth: 1,
                children: vec![],
            }],
        }];

        assert_eq!(
            find_sidebar_title(&chapters, &PathBuf::from("/tmp/nested.md")),
            Some("Nested".to_owned())
        );
    }
}
