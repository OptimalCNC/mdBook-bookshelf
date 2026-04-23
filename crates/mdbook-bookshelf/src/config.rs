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
    pub source_rel: PathBuf,
    pub mount_rel: PathBuf,
    pub book: BookConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawBookshelf {
    root_id: Option<String>,
    #[serde(default, rename = "book")]
    books: Vec<toml::Table>,
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

    let mut seen_mounts = HashSet::new();
    let root_output_rel = PathBuf::from("books").join(&root_book_id);
    let mut books = Vec::with_capacity(raw.books.len());
    for raw_book in raw.books {
        let mut child_book = parse_child_book(raw_book)?;
        validate_book_title(&child_book, "bookshelf.book.title")?;
        let source_rel = normalize_child_source_rel_path(&child_book.src)?;
        let mount_rel = derive_child_mount_rel(&source_rel)?;
        let mount_key = path_to_key(&mount_rel);

        if mount_rel == root_output_rel {
            bail!(
                "bookshelf.book.src '{}' resolves to reserved mount path '{}'",
                source_rel.display(),
                mount_rel.display()
            );
        }
        if mount_key == root_book_id {
            bail!(
                "bookshelf.book.src '{}' resolves to reserved mount key '{}' owned by bookshelf.root-id",
                source_rel.display(),
                mount_key
            );
        }
        if !seen_mounts.insert(mount_rel.clone()) {
            bail!(
                "bookshelf.book.src '{}' resolves to duplicate mount path '{}'",
                source_rel.display(),
                mount_rel.display()
            );
        }

        child_book.src = derive_local_book_src(&source_rel)?;

        books.push(BookshelfBook {
            source_rel,
            mount_rel,
            book: child_book,
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

fn parse_child_book(raw: toml::Table) -> Result<BookConfig> {
    toml::Value::Table(raw)
        .try_into::<BookConfig>()
        .with_context(|| "failed to parse mdBook book config for bookshelf.book")
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

fn normalize_book_src_rel_path(book_id: &str, raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path(book_id, raw_path, "src path", true)
}

fn normalize_child_source_rel_path(raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path("bookshelf.book", raw_path, "src path", true)
}

fn derive_child_mount_rel(source_rel: &Path) -> Result<PathBuf> {
    let mount_rel = match source_rel.file_name() {
        Some(name) if name == "docs" => source_rel
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
        _ => source_rel.to_path_buf(),
    };

    if mount_rel.as_os_str().is_empty() || mount_rel == Path::new(".") {
        bail!(
            "bookshelf.book.src must not resolve to the site root mount path: '{}'",
            source_rel.display()
        );
    }

    Ok(mount_rel)
}

fn derive_local_book_src(source_rel: &Path) -> Result<PathBuf> {
    source_rel
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("bookshelf.book.src must name a source directory"))
}

fn path_to_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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
