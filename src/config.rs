use anyhow::{Context, Result, bail};
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
    pub summary_rel: PathBuf,
    pub summary_abs: PathBuf,
    pub book_root: PathBuf,
    pub book_src: PathBuf,
}

#[derive(Debug, Default)]
struct RawConfig {
    root_book: Option<String>,
    books: Vec<RawBook>,
}

#[derive(Debug, Default)]
struct RawBook {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    summary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Other,
    Bookshelf,
    BookshelfBook,
}

pub fn load_bookshelf_config(path: impl AsRef<Path>) -> Result<BookshelfConfig> {
    let config_path = path.as_ref().to_path_buf();
    let config_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;

    parse_bookshelf_config_text(&content, config_path, config_dir)
}

fn parse_bookshelf_config_text(
    content: &str,
    config_path: PathBuf,
    config_dir: PathBuf,
) -> Result<BookshelfConfig> {
    let mut raw = RawConfig::default();
    let mut section = Section::Other;
    let mut pending_book: Option<RawBook> = None;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(parsed_section) = parse_section(trimmed) {
            if parsed_section == Section::BookshelfBook {
                if let Some(book) = pending_book.take() {
                    raw.books.push(book);
                }
                pending_book = Some(RawBook::default());
            }
            section = parsed_section;
            continue;
        }

        let (key, value) = parse_key_value(trimmed, line_number)?;
        match section {
            Section::Bookshelf => {
                if key == "root_book" {
                    raw.root_book = Some(value);
                }
            }
            Section::BookshelfBook => {
                let book = pending_book.as_mut().ok_or_else(|| {
                    anyhow::anyhow!("line {line_number}: internal parser state for [[bookshelf.book]]")
                })?;
                match key {
                    "id" => book.id = Some(value),
                    "title" => book.title = Some(value),
                    "description" => book.description = Some(value),
                    "summary" => book.summary = Some(value),
                    _ => {}
                }
            }
            Section::Other => {}
        }
    }

    if let Some(book) = pending_book {
        raw.books.push(book);
    }

    validate_and_build(raw, config_path, config_dir)
}

fn parse_section(line: &str) -> Option<Section> {
    if line == "[bookshelf]" {
        return Some(Section::Bookshelf);
    }
    if line == "[[bookshelf.book]]" {
        return Some(Section::BookshelfBook);
    }
    if line.starts_with('[') && line.ends_with(']') {
        return Some(Section::Other);
    }
    None
}

fn parse_key_value(line: &str, line_number: usize) -> Result<(&str, String)> {
    let (key, raw_value) = line
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("line {line_number}: expected key = \"value\""))?;
    let key = key.trim();
    let raw_value = raw_value.trim();

    if !raw_value.starts_with('"') || !raw_value.ends_with('"') || raw_value.len() < 2 {
        bail!("line {line_number}: value for '{key}' must be a quoted string");
    }

    let value = raw_value[1..raw_value.len() - 1].to_string();
    Ok((key, value))
}

fn validate_and_build(
    raw: RawConfig,
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
        let summary = required(raw_book.summary, "bookshelf.book.summary")?;

        if !seen_ids.insert(id.clone()) {
            bail!("duplicate bookshelf.book id: '{id}'");
        }

        let summary_rel = normalize_summary_rel_path(&id, &summary)?;
        let book_src = summary_rel.parent().unwrap_or(Path::new("")).to_path_buf();
        let summary_abs = config_dir.join(&summary_rel);

        books.push(BookshelfBook {
            id,
            title,
            description: raw_book.description,
            summary_rel,
            summary_abs,
            book_root: config_dir.clone(),
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

fn normalize_summary_rel_path(book_id: &str, raw_path: &str) -> Result<PathBuf> {
    let path = Path::new(raw_path);

    if path.is_absolute() {
        bail!("bookshelf.book '{book_id}' summary path must be relative: '{raw_path}'");
    }

    if !raw_path.ends_with("SUMMARY.md") {
        bail!("bookshelf.book '{book_id}' summary path must end with SUMMARY.md: '{raw_path}'");
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir | Component::ParentDir => {
                bail!(
                    "bookshelf.book '{book_id}' summary path must not contain '.' or '..': '{raw_path}'"
                )
            }
            Component::RootDir | Component::Prefix(_) => {
                bail!("bookshelf.book '{book_id}' summary path must be relative: '{raw_path}'")
            }
        }
    }

    if normalized.parent().is_none_or(|p| p.as_os_str().is_empty()) {
        bail!(
            "bookshelf.book '{book_id}' summary path must include a source directory before SUMMARY.md: '{raw_path}'"
        );
    }

    Ok(normalized)
}
