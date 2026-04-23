use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookshelfConfig {
    pub config_path: PathBuf,
    pub config_dir: PathBuf,
    pub root_book: String,
    pub books: Vec<BookshelfBook>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookshelfBook {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub summary_abs: PathBuf,
    pub book_src: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
struct RawTopLevel {
    #[serde(default)]
    bookshelf: RawBookshelf,
}

#[derive(Debug, Deserialize, Default)]
struct RawBookshelf {
    root_book: Option<String>,
    #[serde(default, rename = "book")]
    books: Vec<RawBook>,
}

#[derive(Debug, Deserialize, Default)]
struct RawBook {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    src: Option<String>,
    summary: Option<String>,
}

pub fn load_bookshelf_config(path: impl AsRef<Path>) -> Result<BookshelfConfig> {
    let config_path = path.as_ref().to_path_buf();
    let config_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;

    let raw: RawTopLevel = toml::from_str(&content)
        .with_context(|| format!("failed to parse TOML in {}", config_path.display()))?;
    validate_and_build(raw.bookshelf, config_path, config_dir)
}

fn validate_and_build(
    raw: RawBookshelf,
    config_path: PathBuf,
    config_dir: PathBuf,
) -> Result<BookshelfConfig> {
    let root_book = raw
        .root_book
        .ok_or_else(|| anyhow::anyhow!("missing required key bookshelf.root_book"))?;

    if raw.books.is_empty() {
        bail!("bookshelf.book catalog cannot be empty");
    }

    let mut seen_ids = HashSet::new();
    let mut books = Vec::with_capacity(raw.books.len());
    for raw_book in raw.books {
        let id = required(raw_book.id, "bookshelf.book.id")?;
        let title = required(raw_book.title, "bookshelf.book.title")?;

        if raw_book.summary.is_some() {
            bail!(
                "bookshelf.book '{id}' uses removed key 'summary'; use 'src' with the book source directory instead"
            );
        }

        let book_src = required(raw_book.src, "bookshelf.book.src")?;

        if !seen_ids.insert(id.clone()) {
            bail!("duplicate bookshelf.book id: '{id}'");
        }

        let book_src = normalize_book_src_rel_path(&id, &book_src)?;
        let summary_abs = derive_summary_abs(&config_dir, &book_src);

        books.push(BookshelfBook {
            id,
            title,
            description: raw_book.description,
            summary_abs,
            book_src,
        });
    }

    if !books.iter().any(|book| book.id == root_book) {
        bail!("bookshelf.root_book '{root_book}' is not declared in [[bookshelf.book]]");
    }

    Ok(BookshelfConfig {
        config_path,
        config_dir,
        root_book,
        books,
    })
}

fn required(value: Option<String>, key: &str) -> Result<String> {
    value.ok_or_else(|| anyhow::anyhow!("missing required key {key}"))
}

fn normalize_book_src_rel_path(book_id: &str, raw_path: &str) -> Result<PathBuf> {
    let path = Path::new(raw_path);
    let mut saw_curdir = false;

    if raw_path.is_empty() {
        bail!("bookshelf.book '{book_id}' src path must not be empty");
    }

    if path.is_absolute() {
        bail!("bookshelf.book '{book_id}' src path must be relative: '{raw_path}'");
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => saw_curdir = true,
            Component::ParentDir => {
                bail!("bookshelf.book '{book_id}' src path must not contain '..': '{raw_path}'")
            }
            Component::RootDir | Component::Prefix(_) => {
                bail!("bookshelf.book '{book_id}' src path must be relative: '{raw_path}'")
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        if saw_curdir {
            normalized.push(".");
        } else {
            bail!("bookshelf.book '{book_id}' src path must not be empty");
        }
    }

    if normalized
        .file_name()
        .is_some_and(|name| name == "SUMMARY.md")
    {
        bail!(
            "bookshelf.book '{book_id}' src path must name a source directory, not SUMMARY.md: '{raw_path}'"
        );
    }

    Ok(normalized)
}

fn derive_summary_abs(config_dir: &Path, book_src: &Path) -> PathBuf {
    config_dir.join(book_src).join("SUMMARY.md")
}
