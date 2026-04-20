use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::input_catalog::{InputBook, InputCatalog};

#[derive(Debug)]
pub enum BuildSiteModelError {
    SummaryIo {
        book_id: String,
        summary_path: PathBuf,
        source: io::Error,
    },
    MissingPageTarget {
        book_id: String,
        summary_path: PathBuf,
        line: usize,
        target: String,
        source_path: PathBuf,
    },
    DuplicatePageOwnership {
        source_path: PathBuf,
        first_book_id: String,
        second_book_id: String,
    },
}

impl fmt::Display for BuildSiteModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SummaryIo {
                book_id,
                summary_path,
                source,
            } => write!(
                f,
                "failed to read canonical SUMMARY.md for `{}` at {}: {}",
                book_id,
                summary_path.display(),
                source
            ),
            Self::MissingPageTarget {
                book_id,
                summary_path,
                line,
                target,
                source_path,
            } => write!(
                f,
                "canonical SUMMARY.md for `{}` at {} references missing page target `{}` on line {} ({})",
                book_id,
                summary_path.display(),
                target,
                line,
                source_path.display()
            ),
            Self::DuplicatePageOwnership {
                source_path,
                first_book_id,
                second_book_id,
            } => write!(
                f,
                "authored page {} is claimed by both `{}` and `{}`",
                source_path.display(),
                first_book_id,
                second_book_id
            ),
        }
    }
}

impl std::error::Error for BuildSiteModelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SummaryIo { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteModel {
    pub config_path: PathBuf,
    pub root_book: String,
    pub books: Vec<SiteBook>,
    pub authored_pages: Vec<AuthoredPage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteBook {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub summary_path: PathBuf,
    pub content_root: PathBuf,
    pub authored_page_order: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredPage {
    pub title: String,
    pub source_path: PathBuf,
    pub book_id: String,
    pub order: usize,
    pub summary_depth: usize,
}

pub fn build_site_model(input_catalog: &InputCatalog) -> Result<SiteModel, BuildSiteModelError> {
    let mut ownership: HashMap<PathBuf, String> = HashMap::new();
    let mut books = Vec::with_capacity(input_catalog.books.len());
    let mut authored_pages = Vec::new();

    for book in &input_catalog.books {
        let summary_contents = fs::read_to_string(&book.summary_path).map_err(|source| {
            BuildSiteModelError::SummaryIo {
                book_id: book.id.clone(),
                summary_path: book.summary_path.clone(),
                source,
            }
        })?;

        let summary_entries = parse_summary_entries(&summary_contents);
        let mut authored_page_order = Vec::with_capacity(summary_entries.len());

        for (order, entry) in summary_entries.into_iter().enumerate() {
            let source_path = resolve_summary_target(book, &entry)?;
            if let Some(existing_book_id) = ownership.get(&source_path) {
                if existing_book_id != &book.id {
                    return Err(BuildSiteModelError::DuplicatePageOwnership {
                        source_path,
                        first_book_id: existing_book_id.clone(),
                        second_book_id: book.id.clone(),
                    });
                }
            } else {
                ownership.insert(source_path.clone(), book.id.clone());
            }

            authored_page_order.push(source_path.clone());
            authored_pages.push(AuthoredPage {
                title: entry.title,
                source_path,
                book_id: book.id.clone(),
                order,
                summary_depth: entry.summary_depth,
            });
        }

        books.push(SiteBook {
            id: book.id.clone(),
            title: book.title.clone(),
            description: book.description.clone(),
            summary_path: book.summary_path.clone(),
            content_root: book.content_root.clone(),
            authored_page_order,
        });
    }

    Ok(SiteModel {
        config_path: input_catalog.config_path.clone(),
        root_book: input_catalog.root_book.clone(),
        books,
        authored_pages,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SummaryEntry {
    line: usize,
    title: String,
    target: String,
    summary_depth: usize,
}

fn parse_summary_entries(summary_contents: &str) -> Vec<SummaryEntry> {
    let mut entries = Vec::new();

    for (index, line) in summary_contents.lines().enumerate() {
        let line_number = index + 1;
        let summary_depth = summary_depth(line);

        for (title, target) in extract_links(line) {
            entries.push(SummaryEntry {
                line: line_number,
                title,
                target,
                summary_depth,
            });
        }
    }

    entries
}

fn resolve_summary_target(
    book: &InputBook,
    entry: &SummaryEntry,
) -> Result<PathBuf, BuildSiteModelError> {
    let summary_dir = book.summary_path.parent().unwrap_or_else(|| Path::new("."));
    let source_path = resolve_path(summary_dir, strip_fragment(&entry.target));

    if !source_path.is_file() {
        return Err(BuildSiteModelError::MissingPageTarget {
            book_id: book.id.clone(),
            summary_path: book.summary_path.clone(),
            line: entry.line,
            target: entry.target.clone(),
            source_path,
        });
    }

    let source_path =
        source_path
            .canonicalize()
            .map_err(|_| BuildSiteModelError::MissingPageTarget {
                book_id: book.id.clone(),
                summary_path: book.summary_path.clone(),
                line: entry.line,
                target: entry.target.clone(),
                source_path: resolve_path(summary_dir, strip_fragment(&entry.target)),
            })?;

    Ok(source_path)
}

fn extract_links(line: &str) -> Vec<(String, String)> {
    let mut links = Vec::new();
    let bytes = line.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let Some(open_label_rel) = line[index..].find('[') else {
            break;
        };
        let open_label = index + open_label_rel;

        let Some(close_label_rel) = line[open_label + 1..].find(']') else {
            break;
        };
        let close_label = open_label + 1 + close_label_rel;

        let open_target = close_label + 1;
        if bytes.get(open_target) != Some(&b'(') {
            index = close_label + 1;
            continue;
        }

        let Some(close_target_rel) = line[open_target + 1..].find(')') else {
            break;
        };
        let close_target = open_target + 1 + close_target_rel;

        let title = line[open_label + 1..close_label].trim().to_owned();
        let target = line[open_target + 1..close_target].trim().to_owned();
        if !title.is_empty() && !target.is_empty() {
            links.push((title, target));
        }

        index = close_target + 1;
    }

    links
}

fn summary_depth(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count() / 2
}

fn strip_fragment(target: &str) -> &str {
    target.split(['#', '?']).next().unwrap_or(target).trim()
}

fn resolve_path(base_dir: &Path, raw_path: &str) -> PathBuf {
    let path = Path::new(raw_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::parse_summary_entries;

    #[test]
    fn preserves_summary_order_and_depth() {
        let summary =
            "# Summary\n\n- [Book](index.md)\n  - [Nested](nested.md)\n- [Last](last.md)\n";
        let entries = parse_summary_entries(summary);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].title, "Book");
        assert_eq!(entries[0].summary_depth, 0);
        assert_eq!(entries[1].title, "Nested");
        assert_eq!(entries[1].summary_depth, 1);
        assert_eq!(entries[2].title, "Last");
        assert_eq!(entries[2].summary_depth, 0);
    }
}
