use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::{load_bookshelf_config, BookConfig};

#[derive(Debug)]
pub enum LoadInputCatalogError {
    MissingConfig {
        path: PathBuf,
    },
    ConfigIo {
        path: PathBuf,
        source: io::Error,
    },
    MissingBookshelfSection {
        path: PathBuf,
    },
    MissingRootBook {
        path: PathBuf,
    },
    InvalidConfig {
        path: PathBuf,
        line: usize,
        message: String,
    },
    DuplicateBookId {
        book_id: String,
    },
    MissingSummary {
        book_id: String,
    },
    SummaryIo {
        book_id: String,
        summary_path: PathBuf,
        source: io::Error,
    },
    EmptySummary {
        book_id: String,
        summary_path: PathBuf,
    },
    RootBookNotConfigured {
        root_book_id: String,
    },
    AuthoredBookshelfEntry {
        book_id: String,
        summary_path: PathBuf,
        line: usize,
    },
}

impl fmt::Display for LoadInputCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConfig { path } => write!(f, "missing bookshelf config at {}", path.display()),
            Self::ConfigIo { path, source } => {
                write!(f, "failed to read bookshelf config {}: {}", path.display(), source)
            }
            Self::MissingBookshelfSection { path } => {
                write!(f, "missing `[bookshelf]` section in {}", path.display())
            }
            Self::MissingRootBook { path } => {
                write!(f, "missing `bookshelf.root_book` in {}", path.display())
            }
            Self::InvalidConfig {
                path,
                line,
                message,
            } => write!(f, "invalid config {}:{}: {}", path.display(), line, message),
            Self::DuplicateBookId { book_id } => {
                write!(f, "duplicate bookshelf book id `{book_id}`")
            }
            Self::MissingSummary { book_id } => {
                write!(f, "missing `summary` for bookshelf book `{book_id}`")
            }
            Self::SummaryIo {
                book_id,
                summary_path,
                source,
            } => write!(
                f,
                "failed to read SUMMARY.md for `{}` at {}: {}",
                book_id,
                summary_path.display(),
                source
            ),
            Self::EmptySummary {
                book_id,
                summary_path,
            } => write!(
                f,
                "canonical SUMMARY.md for `{}` at {} is empty or has no chapter links",
                book_id,
                summary_path.display()
            ),
            Self::RootBookNotConfigured { root_book_id } => {
                write!(
                    f,
                    "configured root_book `{}` is not present in `[[bookshelf.book]]`",
                    root_book_id
                )
            }
            Self::AuthoredBookshelfEntry {
                book_id,
                summary_path,
                line,
            } => write!(
                f,
                "canonical SUMMARY.md for `{}` at {} authors a forbidden `Bookshelf` entry on line {}",
                book_id,
                summary_path.display(),
                line
            ),
        }
    }
}

impl std::error::Error for LoadInputCatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConfigIo { source, .. } => Some(source),
            Self::SummaryIo { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputCatalog {
    pub config_path: PathBuf,
    pub root_book: String,
    pub books: Vec<InputBook>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputBook {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub summary_path: PathBuf,
    pub content_root: PathBuf,
}

pub fn load_input_catalog(
    config_path: impl AsRef<Path>,
) -> Result<InputCatalog, LoadInputCatalogError> {
    let config_path = config_path.as_ref();
    let config = load_bookshelf_config(config_path)?;
    let config_dir = config.path.parent().unwrap_or_else(|| Path::new("."));

    let mut seen_ids = HashSet::new();
    let mut books = Vec::with_capacity(config.books.len());

    for book in config.books {
        if !seen_ids.insert(book.id.clone()) {
            return Err(LoadInputCatalogError::DuplicateBookId { book_id: book.id });
        }

        books.push(load_book_input(config_dir, book)?);
    }

    if !books.iter().any(|book| book.id == config.root_book) {
        return Err(LoadInputCatalogError::RootBookNotConfigured {
            root_book_id: config.root_book,
        });
    }

    Ok(InputCatalog {
        config_path: config.path,
        root_book: config.root_book,
        books,
    })
}

fn load_book_input(
    config_dir: &Path,
    book: BookConfig,
) -> Result<InputBook, LoadInputCatalogError> {
    let summary = book.summary.trim();
    if summary.is_empty() {
        return Err(LoadInputCatalogError::MissingSummary { book_id: book.id });
    }

    let summary_path = resolve_path(config_dir, summary);
    let summary_contents =
        fs::read_to_string(&summary_path).map_err(|source| LoadInputCatalogError::SummaryIo {
            book_id: book.id.clone(),
            summary_path: summary_path.clone(),
            source,
        })?;
    let content_root = parse_summary_root(&book.id, &summary_path, &summary_contents)?;

    Ok(InputBook {
        id: book.id,
        title: book.title,
        description: book.description,
        summary_path,
        content_root,
    })
}

fn parse_summary_root(
    book_id: &str,
    summary_path: &Path,
    summary_contents: &str,
) -> Result<PathBuf, LoadInputCatalogError> {
    let mut first_link_target: Option<String> = None;

    for (index, line) in summary_contents.lines().enumerate() {
        for (title, target) in extract_links(line) {
            if is_bookshelf_entry(&title, &target) {
                return Err(LoadInputCatalogError::AuthoredBookshelfEntry {
                    book_id: book_id.to_owned(),
                    summary_path: summary_path.to_path_buf(),
                    line: index + 1,
                });
            }

            if first_link_target.is_none() {
                first_link_target = Some(target);
            }
        }
    }

    let first_link_target =
        first_link_target.ok_or_else(|| LoadInputCatalogError::EmptySummary {
            book_id: book_id.to_owned(),
            summary_path: summary_path.to_path_buf(),
        })?;

    let summary_dir = summary_path.parent().unwrap_or_else(|| Path::new("."));
    Ok(resolve_path(
        summary_dir,
        strip_fragment(&first_link_target),
    ))
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

fn is_bookshelf_entry(title: &str, target: &str) -> bool {
    if title.trim() == "Bookshelf" {
        return true;
    }

    let target_path = Path::new(strip_fragment(target));
    matches!(
        target_path.file_stem().and_then(|stem| stem.to_str()),
        Some("bookshelf")
    )
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
    use super::{parse_summary_root, LoadInputCatalogError};
    use std::path::Path;

    #[test]
    fn derives_root_from_first_summary_link() {
        let summary = "# Summary\n\n- [Book](index.md)\n  - [Next](next.md)\n";
        let root = parse_summary_root("book", Path::new("/tmp/docs/SUMMARY.md"), summary).unwrap();
        assert_eq!(root, Path::new("/tmp/docs/index.md"));
    }

    #[test]
    fn rejects_authored_bookshelf_entries() {
        let summary = "# Summary\n\n- [Book](index.md)\n- [Bookshelf](bookshelf.md)\n";
        let error =
            parse_summary_root("book", Path::new("/tmp/docs/SUMMARY.md"), summary).unwrap_err();
        assert!(matches!(
            error,
            LoadInputCatalogError::AuthoredBookshelfEntry { .. }
        ));
    }
}
