use anyhow::{bail, Context, Result};
use mdbook_driver::config::{BookConfig, Config};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct BookshelfConfig {
    pub config_path: PathBuf,
    pub config_dir: PathBuf,
    pub mdbook_config: Config,
    pub root_book_id: String,
    pub books: Vec<BookshelfBook>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BookshelfBook {
    pub id: String,
    pub root: PathBuf,
    pub book: BookConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawBookshelf {
    root_id: Option<String>,
    #[serde(default, rename = "book")]
    books: Vec<toml::Table>,
}

struct RawChildBook {
    id: String,
    root: PathBuf,
    book: BookConfig,
}

pub fn load_bookshelf_config(path: impl AsRef<Path>) -> Result<BookshelfConfig> {
    let config_path = path.as_ref().to_path_buf();
    let config_dir = config_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;

    let mut toml_root: toml::Table = toml::from_str(&content)
        .with_context(|| format!("failed to parse TOML in {}", config_path.display()))?;
    let raw_bookshelf = parse_bookshelf_table(&mut toml_root, &config_path)?;
    let mdbook_config = parse_mdbook_config(toml_root, &config_path)?;

    validate_and_build(raw_bookshelf, mdbook_config, config_path, config_dir)
}

fn validate_and_build(
    raw: RawBookshelf,
    mut mdbook_config: Config,
    config_path: PathBuf,
    config_dir: PathBuf,
) -> Result<BookshelfConfig> {
    let root_book_id = normalize_book_id(
        required_non_empty(raw.root_id, "bookshelf.root-id")?,
        "bookshelf.root-id",
    )?;
    validate_book_title(&mdbook_config.book, "book.title")?;
    mdbook_config.book.src = normalize_book_src_rel_path(&root_book_id, &mdbook_config.book.src)?;

    let mut seen_ids = HashSet::new();
    seen_ids.insert(root_book_id.clone());
    let mut books = Vec::with_capacity(raw.books.len());
    for raw_book in raw.books {
        let mut child = parse_child_book(raw_book)?;
        child.id = normalize_book_id(child.id, "bookshelf.book.id")?;
        let id = child.id.clone();

        if !seen_ids.insert(id.clone()) {
            if id == root_book_id {
                bail!("duplicate bookshelf.book id: '{id}' collides with bookshelf.root-id");
            }
            bail!("duplicate bookshelf.book id: '{id}'");
        }

        validate_book_title(&child.book, &format!("bookshelf.book '{id}' title"))?;
        child.root = normalize_book_root_rel_path(&id, &child.root)?;
        child.book.src = normalize_book_src_rel_path(&id, &child.book.src)?;

        books.push(BookshelfBook {
            id: child.id,
            root: child.root,
            book: child.book,
        });
    }

    Ok(BookshelfConfig {
        config_path,
        config_dir,
        mdbook_config,
        root_book_id,
        books,
    })
}

fn parse_bookshelf_table(toml_root: &mut toml::Table, config_path: &Path) -> Result<RawBookshelf> {
    let bookshelf = toml_root
        .remove("bookshelf")
        .ok_or_else(|| anyhow::anyhow!("missing required table [bookshelf]"))?;

    bookshelf.try_into().with_context(|| {
        format!(
            "failed to parse bookshelf config in {}",
            config_path.display()
        )
    })
}

fn parse_mdbook_config(toml_root: toml::Table, config_path: &Path) -> Result<Config> {
    let projected = toml::to_string(&toml_root).with_context(|| {
        format!(
            "failed to serialize mdBook config from {} after removing [bookshelf]",
            config_path.display()
        )
    })?;

    Config::from_str(&projected).with_context(|| {
        format!(
            "failed to parse mdBook config from {} after removing [bookshelf]",
            config_path.display()
        )
    })
}

fn parse_child_book(mut raw: toml::Table) -> Result<RawChildBook> {
    let id = remove_required_non_empty_string(&mut raw, "id", "bookshelf.book.id")?;
    let root = remove_required_path(&mut raw, "root", &format!("bookshelf.book '{id}' root"))?;
    let book = toml::Value::Table(raw)
        .try_into::<BookConfig>()
        .with_context(|| format!("failed to parse mdBook book config for bookshelf.book '{id}'"))?;

    Ok(RawChildBook { id, root, book })
}

fn remove_required_non_empty_string(
    raw: &mut toml::Table,
    key: &str,
    display_key: &str,
) -> Result<String> {
    match raw.remove(key) {
        Some(toml::Value::String(value)) if !value.is_empty() => Ok(value),
        Some(toml::Value::String(_)) => bail!("{display_key} must not be empty"),
        Some(_) => bail!("{display_key} must be a string"),
        None => bail!("missing required key {display_key}"),
    }
}

fn remove_required_path(raw: &mut toml::Table, key: &str, display_key: &str) -> Result<PathBuf> {
    match raw.remove(key) {
        Some(value) => value
            .try_into::<PathBuf>()
            .with_context(|| format!("{display_key} must be a string path")),
        None => bail!("missing required key {display_key}"),
    }
}

fn required_non_empty(value: Option<String>, key: &str) -> Result<String> {
    match value {
        Some(value) if !value.is_empty() => Ok(value),
        Some(_) => bail!("{key} must not be empty"),
        None => bail!("missing required key {key}"),
    }
}

fn validate_book_title(book: &BookConfig, key: &str) -> Result<()> {
    match book.title.as_deref() {
        Some(title) if !title.is_empty() => Ok(()),
        Some(_) => bail!("{key} must not be empty"),
        None => bail!("missing required key {key}"),
    }
}

fn normalize_book_id(raw_id: String, key: &str) -> Result<String> {
    let path = Path::new(&raw_id);
    let mut components = path.components();
    let Some(Component::Normal(part)) = components.next() else {
        bail!("{key} must be a single safe path segment: '{raw_id}'");
    };

    if components.next().is_some() || part != raw_id.as_str() {
        bail!("{key} must be a single safe path segment: '{raw_id}'");
    }

    Ok(raw_id)
}

fn normalize_book_root_rel_path(book_id: &str, raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path(book_id, raw_path, "root path", false)
}

fn normalize_book_src_rel_path(book_id: &str, raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path(book_id, raw_path, "src path", true)
}

fn normalize_rel_dir_path(
    book_id: &str,
    raw_path: &Path,
    noun: &str,
    reject_summary_file: bool,
) -> Result<PathBuf> {
    let raw_display = raw_path.display();
    let mut saw_curdir = false;

    if raw_path.as_os_str().is_empty() {
        bail!("book '{book_id}' {noun} must not be empty");
    }

    if raw_path.is_absolute() {
        bail!("book '{book_id}' {noun} must be relative: '{raw_display}'");
    }

    let mut normalized = PathBuf::new();
    for component in raw_path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => saw_curdir = true,
            Component::ParentDir => {
                bail!("book '{book_id}' {noun} must not contain '..': '{raw_display}'")
            }
            Component::RootDir | Component::Prefix(_) => {
                bail!("book '{book_id}' {noun} must be relative: '{raw_display}'")
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        if saw_curdir {
            normalized.push(".");
        } else {
            bail!("book '{book_id}' {noun} must not be empty");
        }
    }

    if reject_summary_file
        && normalized
            .file_name()
            .is_some_and(|name| name == "SUMMARY.md")
    {
        bail!(
            "book '{book_id}' {noun} must name a source directory, not SUMMARY.md: '{raw_display}'"
        );
    }

    Ok(normalized)
}
