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
    pub root_book_id: String,
    pub root_book_cover: Option<PathBuf>,
    pub asset_dir: PathBuf,
    pub books: Vec<BookshelfBook>,
    pub categories: Vec<BookshelfCategory>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BookshelfBook {
    pub id: String,
    pub source_rel: PathBuf,
    pub book: BookConfig,
    pub cover: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookshelfCategory {
    pub title: String,
    pub book_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawBookshelf {
    root_book_id: String,
    #[serde(default)]
    root_book: RawBookshelfRootBook,
    #[serde(default = "default_bookshelf_asset_dir")]
    asset_dir: PathBuf,
    #[serde(default, rename = "book")]
    books: Vec<toml::Table>,
    #[serde(default, rename = "category")]
    categories: Vec<RawBookshelfCategory>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawBookshelfRootBook {
    #[serde(default)]
    cover: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawBookshelfCategory {
    title: String,
    books: Vec<String>,
}

#[derive(Debug)]
struct ParsedChildBook {
    id: String,
    cover: Option<PathBuf>,
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
    validate_book_title(&mdbook_config.book, "book.title")?;
    let root_source_rel = normalize_root_source_rel_path(&mdbook_config.book.src)?;
    validate_canonical_output_root(&root_source_rel, "book.src")?;
    mdbook_config.book.src = root_source_rel.clone();
    let root_book_id = raw.root_book_id;
    validate_book_id(&root_book_id, "bookshelf.root-book-id")?;
    let root_book_cover =
        normalize_optional_cover_path(raw.root_book.cover, "bookshelf.root-book.cover")?;
    let asset_dir = normalize_bookshelf_asset_dir_path(&raw.asset_dir)?;

    let mut seen_book_ids = std::collections::BTreeSet::new();
    seen_book_ids.insert(root_book_id.clone());
    let mut all_book_ids = vec![root_book_id.clone()];
    let mut seen_output_roots = vec![root_source_rel.clone()];
    let mut books = Vec::with_capacity(raw.books.len());
    for raw_book in raw.books {
        let mut parsed = parse_child_book(raw_book, &mdbook_config.book)?;
        validate_book_id(&parsed.id, "bookshelf.book.id")?;
        if !seen_book_ids.insert(parsed.id.clone()) {
            bail!("duplicate bookshelf book id '{}'", parsed.id);
        }
        all_book_ids.push(parsed.id.clone());

        validate_book_title(&parsed.book, "bookshelf.book.title")?;
        let source_rel = normalize_child_source_rel_path(&parsed.book.src)?;
        validate_canonical_output_root(&source_rel, "bookshelf.book.src")?;
        ensure_output_root_available(&source_rel, &seen_output_roots)?;
        seen_output_roots.push(source_rel.clone());

        parsed.book.src = source_rel.clone();
        let cover = normalize_optional_cover_path(parsed.cover, "bookshelf.book.cover")?;

        books.push(BookshelfBook {
            id: parsed.id,
            source_rel,
            book: parsed.book,
            cover,
        });
    }

    let categories = validate_categories(raw.categories, &all_book_ids)?;

    Ok(BookshelfConfig {
        config_path,
        config_dir,
        mdbook_config,
        root_book_id,
        root_book_cover,
        asset_dir,
        books,
        categories,
    })
}

fn parse_bookshelf_table(toml_root: &mut toml::Table, config_path: &Path) -> Result<RawBookshelf> {
    let bookshelf = toml_root
        .remove("bookshelf")
        .ok_or_else(|| anyhow::anyhow!("missing required table [bookshelf]"))?;

    match bookshelf.try_into() {
        Ok(raw) => Ok(raw),
        Err(error) if error.to_string().contains("missing field `root-book-id`") => {
            bail!("missing required key bookshelf.root-book-id")
        }
        Err(error) => Err(error).with_context(|| {
            format!(
                "failed to parse bookshelf config in {}",
                config_path.display()
            )
        }),
    }
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

fn parse_child_book(raw: toml::Table, root_book: &BookConfig) -> Result<ParsedChildBook> {
    let mut id = None;
    let mut cover = None;
    let mut book = root_book.clone();

    for (key, value) in raw {
        match key.as_str() {
            "id" => {
                id = Some(parse_toml_value(value, "bookshelf.book.id")?);
            }
            "cover" => {
                cover = Some(parse_toml_value(value, "bookshelf.book.cover")?);
            }
            "title" => {
                book.title = Some(parse_toml_value(value, "bookshelf.book.title")?);
            }
            "authors" => {
                book.authors = parse_toml_value(value, "bookshelf.book.authors")?;
            }
            "description" => {
                book.description = Some(parse_toml_value(value, "bookshelf.book.description")?);
            }
            "src" => {
                book.src = parse_toml_value(value, "bookshelf.book.src")?;
            }
            "language" => {
                book.language = Some(parse_toml_value(value, "bookshelf.book.language")?);
            }
            "text-direction" => {
                book.text_direction =
                    Some(parse_toml_value(value, "bookshelf.book.text-direction")?);
            }
            unknown => {
                bail!("unknown field `{unknown}` in bookshelf.book");
            }
        }
    }

    let id = id.ok_or_else(|| anyhow::anyhow!("missing required key bookshelf.book.id"))?;
    Ok(ParsedChildBook { id, cover, book })
}

fn parse_toml_value<T>(value: toml::Value, key: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    value
        .try_into()
        .with_context(|| format!("failed to parse {key}"))
}

fn validate_book_title(book: &BookConfig, key: &str) -> Result<()> {
    match book.title.as_deref() {
        Some(title) if !title.is_empty() => Ok(()),
        Some(_) => bail!("{key} must not be empty"),
        None => bail!("missing required key {key}"),
    }
}

fn validate_book_id(id: &str, key: &str) -> Result<()> {
    if id.is_empty() {
        bail!("{key} must not be empty");
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        bail!("{key} may only contain ASCII letters, numbers, '-', '_', and '.'");
    }
    Ok(())
}

fn normalize_optional_cover_path(raw_path: Option<PathBuf>, key: &str) -> Result<Option<PathBuf>> {
    raw_path
        .map(|path| normalize_rel_file_path("bookshelf", &path, key))
        .transpose()
}

fn normalize_rel_file_path(book_id: &str, raw_path: &Path, noun: &str) -> Result<PathBuf> {
    let normalized = normalize_rel_dir_path(book_id, raw_path, noun, false)?;
    if normalized == Path::new(".") {
        bail!("book '{book_id}' {noun} must name a file below the bookshelf config root");
    }
    Ok(normalized)
}

fn validate_categories(
    categories: Vec<RawBookshelfCategory>,
    all_book_ids: &[String],
) -> Result<Vec<BookshelfCategory>> {
    if categories.is_empty() {
        bail!("bookshelf must define at least one [[bookshelf.category]]");
    }

    let known: std::collections::BTreeSet<_> = all_book_ids.iter().cloned().collect();
    let mut assigned = std::collections::BTreeSet::new();
    let mut output = Vec::with_capacity(categories.len());

    for raw_category in categories {
        let title = raw_category.title.trim().to_string();
        if title.is_empty() {
            bail!("bookshelf.category.title must not be empty");
        }
        if raw_category.books.is_empty() {
            bail!("bookshelf.category '{title}' must list at least one book");
        }

        let mut book_ids = Vec::with_capacity(raw_category.books.len());
        for book_id in raw_category.books {
            if !known.contains(&book_id) {
                bail!("bookshelf.category '{title}' references unknown book id '{book_id}'");
            }
            if !assigned.insert(book_id.clone()) {
                bail!("book id '{book_id}' appears in more than one bookshelf.category");
            }
            book_ids.push(book_id);
        }
        output.push(BookshelfCategory { title, book_ids });
    }

    for book_id in all_book_ids {
        if !assigned.contains(book_id) {
            bail!("book id '{book_id}' must appear in exactly one bookshelf.category");
        }
    }

    Ok(output)
}

fn normalize_root_source_rel_path(raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path("root book", raw_path, "src path", true)
}

fn normalize_child_source_rel_path(raw_path: &Path) -> Result<PathBuf> {
    normalize_rel_dir_path("bookshelf.book", raw_path, "src path", true)
}

fn normalize_bookshelf_asset_dir_path(raw_path: &Path) -> Result<PathBuf> {
    let normalized = normalize_rel_dir_path("bookshelf", raw_path, "asset-dir path", false)?;
    if normalized == Path::new(".") {
        bail!("[bookshelf].asset-dir must name a directory below the bookshelf config root");
    }

    Ok(normalized)
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

fn default_bookshelf_asset_dir() -> PathBuf {
    PathBuf::from(".mdbook/bookshelf")
}
