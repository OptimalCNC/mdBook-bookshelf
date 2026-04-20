use std::fs;
use std::path::{Path, PathBuf};

use crate::input_catalog::LoadInputCatalogError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookshelfConfig {
    pub path: PathBuf,
    pub root_book: String,
    pub books: Vec<BookConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookConfig {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Other,
    Bookshelf,
    BookshelfBook,
}

#[derive(Debug, Default)]
struct BookConfigBuilder {
    line: usize,
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    summary: Option<String>,
}

impl BookConfigBuilder {
    fn new(line: usize) -> Self {
        Self {
            line,
            ..Self::default()
        }
    }

    fn finish(self, config_path: &Path) -> Result<BookConfig, LoadInputCatalogError> {
        Ok(BookConfig {
            id: self.id.ok_or_else(|| {
                invalid_config(
                    config_path,
                    self.line,
                    "missing `id` in `[[bookshelf.book]]`",
                )
            })?,
            title: self.title.ok_or_else(|| {
                invalid_config(
                    config_path,
                    self.line,
                    "missing `title` in `[[bookshelf.book]]`",
                )
            })?,
            description: self.description,
            summary: self.summary.unwrap_or_default(),
        })
    }
}

pub(crate) fn load_bookshelf_config(
    config_path: &Path,
) -> Result<BookshelfConfig, LoadInputCatalogError> {
    if !config_path.exists() {
        return Err(LoadInputCatalogError::MissingConfig {
            path: config_path.to_path_buf(),
        });
    }

    let contents =
        fs::read_to_string(config_path).map_err(|source| LoadInputCatalogError::ConfigIo {
            path: config_path.to_path_buf(),
            source,
        })?;

    parse_bookshelf_config(config_path, &contents)
}

fn parse_bookshelf_config(
    config_path: &Path,
    contents: &str,
) -> Result<BookshelfConfig, LoadInputCatalogError> {
    let mut section = Section::Other;
    let mut saw_bookshelf = false;
    let mut root_book: Option<String> = None;
    let mut books = Vec::new();
    let mut current_book: Option<BookConfigBuilder> = None;

    for (index, raw_line) in contents.lines().enumerate() {
        let line_no = index + 1;
        let line = strip_comment(raw_line).trim();

        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') {
            if let Some(book) = current_book.take() {
                books.push(book.finish(config_path)?);
            }

            section = match line {
                "[bookshelf]" => {
                    saw_bookshelf = true;
                    Section::Bookshelf
                }
                "[[bookshelf.book]]" => {
                    saw_bookshelf = true;
                    current_book = Some(BookConfigBuilder::new(line_no));
                    Section::BookshelfBook
                }
                _ => Section::Other,
            };

            continue;
        }

        let (key, raw_value) = parse_key_value(config_path, line_no, line)?;
        let value = parse_string_value(config_path, line_no, raw_value)?;

        match section {
            Section::Bookshelf => {
                if key == "root_book" {
                    root_book = Some(value);
                }
            }
            Section::BookshelfBook => {
                let book = current_book.as_mut().ok_or_else(|| {
                    invalid_config(
                        config_path,
                        line_no,
                        "encountered `bookshelf.book` value before starting a book section",
                    )
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

    if let Some(book) = current_book.take() {
        books.push(book.finish(config_path)?);
    }

    if !saw_bookshelf {
        return Err(LoadInputCatalogError::MissingBookshelfSection {
            path: config_path.to_path_buf(),
        });
    }

    let root_book = match root_book.map(|value| value.trim().to_owned()) {
        Some(value) if !value.is_empty() => value,
        _ => {
            return Err(LoadInputCatalogError::MissingRootBook {
                path: config_path.to_path_buf(),
            })
        }
    };

    Ok(BookshelfConfig {
        path: config_path.to_path_buf(),
        root_book,
        books,
    })
}

fn parse_key_value<'a>(
    config_path: &Path,
    line_no: usize,
    line: &'a str,
) -> Result<(&'a str, &'a str), LoadInputCatalogError> {
    let (key, value) = line.split_once('=').ok_or_else(|| {
        invalid_config(
            config_path,
            line_no,
            "expected a `key = \"value\"` assignment",
        )
    })?;

    Ok((key.trim(), value.trim()))
}

fn parse_string_value(
    config_path: &Path,
    line_no: usize,
    raw_value: &str,
) -> Result<String, LoadInputCatalogError> {
    if !(raw_value.starts_with('"') && raw_value.ends_with('"') && raw_value.len() >= 2) {
        return Err(invalid_config(
            config_path,
            line_no,
            "expected a quoted string value",
        ));
    }

    unescape_basic_string(config_path, line_no, &raw_value[1..raw_value.len() - 1])
}

fn unescape_basic_string(
    config_path: &Path,
    line_no: usize,
    value: &str,
) -> Result<String, LoadInputCatalogError> {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        let escaped = chars.next().ok_or_else(|| {
            invalid_config(
                config_path,
                line_no,
                "unterminated escape sequence in string value",
            )
        })?;

        match escaped {
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            other => {
                return Err(invalid_config(
                    config_path,
                    line_no,
                    format!("unsupported escape sequence `\\{other}`"),
                ))
            }
        }
    }

    Ok(result)
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(index) => &line[..index],
        None => line,
    }
}

fn invalid_config(
    config_path: &Path,
    line: usize,
    message: impl Into<String>,
) -> LoadInputCatalogError {
    LoadInputCatalogError::InvalidConfig {
        path: config_path.to_path_buf(),
        line,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_bookshelf_config;
    use crate::input_catalog::LoadInputCatalogError;
    use std::path::Path;

    #[test]
    fn parses_bookshelf_config_with_non_bookshelf_sections() {
        let config = r#"
[book]
title = "Example"

[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
summary = "docs/SUMMARY.md"
"#;

        let parsed = parse_bookshelf_config(Path::new("bookshelf.toml"), config).unwrap();
        assert_eq!(parsed.root_book, "meta");
        assert_eq!(parsed.books.len(), 1);
        assert_eq!(parsed.books[0].id, "meta");
    }

    #[test]
    fn requires_bookshelf_section() {
        let error = parse_bookshelf_config(Path::new("bookshelf.toml"), "").unwrap_err();
        assert!(matches!(
            error,
            LoadInputCatalogError::MissingBookshelfSection { .. }
        ));
    }
}
