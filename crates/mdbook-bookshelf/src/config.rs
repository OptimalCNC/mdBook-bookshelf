use anyhow::{bail, Context, Result};
use mdbook_driver::config::{BookConfig, Config};
use serde::Deserialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct BookshelfConfig {
    pub config_path: PathBuf,
    pub config_dir: PathBuf,
    pub mdbook_config: Config,
    pub books: Vec<BookshelfBook>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BookshelfBook {
    pub source_rel: PathBuf,
    pub book: BookConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawBookshelf {
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
    validate_book_title(&mdbook_config.book, "book.title")?;
    let root_source_rel = normalize_root_source_rel_path(&mdbook_config.book.src)?;
    validate_canonical_output_root(&root_source_rel, "book.src")?;
    mdbook_config.book.src = root_source_rel.clone();

    let mut seen_output_roots = vec![root_source_rel.clone()];
    let mut books = Vec::with_capacity(raw.books.len());
    for raw_book in raw.books {
        let mut child_book = parse_child_book(raw_book)?;
        validate_book_title(&child_book, "bookshelf.book.title")?;
        let source_rel = normalize_child_source_rel_path(&child_book.src)?;
        validate_canonical_output_root(&source_rel, "bookshelf.book.src")?;
        ensure_output_root_available(&source_rel, &seen_output_roots)?;
        seen_output_roots.push(source_rel.clone());

        child_book.src = derive_local_book_src(&source_rel)?;

        books.push(BookshelfBook {
            source_rel,
            book: child_book,
        });
    }

    Ok(BookshelfConfig {
        config_path,
        config_dir,
        mdbook_config,
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

fn validate_book_title(book: &BookConfig, key: &str) -> Result<()> {
    match book.title.as_deref() {
        Some(title) if !title.is_empty() => Ok(()),
        Some(_) => bail!("{key} must not be empty"),
        None => bail!("missing required key {key}"),
    }
}

fn normalize_root_source_rel_path(raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path("root book", raw_path, "src path", true)
}

fn normalize_child_source_rel_path(raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path("bookshelf.book", raw_path, "src path", true)
}

fn derive_local_book_src(source_rel: &Path) -> Result<PathBuf> {
    source_rel
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("bookshelf.book.src must name a source directory"))
}

fn validate_canonical_output_root(output_rel: &Path, key: &str) -> Result<()> {
    if output_rel == Path::new(".") {
        bail!(
            "{key} must not resolve to the site root canonical output root: '{}'",
            output_rel.display()
        );
    }

    Ok(())
}

fn ensure_output_root_available(candidate: &Path, existing_roots: &[PathBuf]) -> Result<()> {
    for existing in existing_roots {
        if existing == candidate {
            bail!(
                "bookshelf.book.src '{}' resolves to duplicate canonical output root '{}'",
                candidate.display(),
                candidate.display()
            );
        }

        if is_ancestor_or_descendant(existing, candidate) {
            bail!(
                "bookshelf.book.src '{}' resolves to overlapping canonical output root '{}'",
                candidate.display(),
                existing.display()
            );
        }
    }

    Ok(())
}

fn is_ancestor_or_descendant(left: &Path, right: &Path) -> bool {
    is_ancestor(left, right) || is_ancestor(right, left)
}

fn is_ancestor(ancestor: &Path, path: &Path) -> bool {
    ancestor != path && path.starts_with(ancestor)
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
