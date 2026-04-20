use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::reader_context::ReaderContextModel;
use crate::render_manifest::RenderManifest;

#[derive(Debug)]
pub enum SearchIndexError {
    MissingManifestEntry { source_path: PathBuf },
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for SearchIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingManifestEntry { source_path } => write!(
                f,
                "render manifest is missing the authored output for search document {}",
                source_path.display()
            ),
            Self::Io { path, source } => {
                write!(
                    f,
                    "failed to build or write search index from {}: {}",
                    path.display(),
                    source
                )
            }
        }
    }
}

impl std::error::Error for SearchIndexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchIndex {
    pub documents: Vec<SearchDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDocument {
    pub title: String,
    pub href: String,
    pub book_label: String,
    pub body: String,
}

impl SearchIndex {
    pub fn to_json_string(&self) -> String {
        let documents = self
            .documents
            .iter()
            .map(|document| {
                format!(
                    "{{\"title\":\"{title}\",\"href\":\"{href}\",\"book_label\":\"{book_label}\",\"body\":\"{body}\"}}",
                    title = escape_json(&document.title),
                    href = escape_json(&document.href),
                    book_label = escape_json(&document.book_label),
                    body = escape_json(&document.body)
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!("{{\"docs\":[{documents}]}}")
    }
}

pub fn build_search_index(
    reader_context: &ReaderContextModel,
    render_manifest: &RenderManifest,
) -> Result<SearchIndex, SearchIndexError> {
    let mut documents = Vec::with_capacity(reader_context.authored_page_contexts.len());

    for context in &reader_context.authored_page_contexts {
        let manifest_entry = render_manifest
            .authored_entry_for_source_path(&context.source_path)
            .ok_or_else(|| SearchIndexError::MissingManifestEntry {
                source_path: context.source_path.clone(),
            })?;
        let markdown =
            fs::read_to_string(&context.source_path).map_err(|source| SearchIndexError::Io {
                path: context.source_path.clone(),
                source,
            })?;

        documents.push(SearchDocument {
            title: context.page_title.clone(),
            href: manifest_entry.output_path.to_string_lossy().into_owned(),
            book_label: context.active_book.title.clone(),
            body: extract_search_body(&markdown),
        });
    }

    Ok(SearchIndex { documents })
}

pub fn write_search_index(
    output_dir: impl AsRef<Path>,
    search_index: &SearchIndex,
) -> Result<PathBuf, SearchIndexError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).map_err(|source| SearchIndexError::Io {
        path: output_dir.to_path_buf(),
        source,
    })?;

    let output_path = output_dir.join("searchindex.json");
    fs::write(&output_path, search_index.to_json_string()).map_err(|source| {
        SearchIndexError::Io {
            path: output_path.clone(),
            source,
        }
    })?;

    Ok(output_path)
}

fn extract_search_body(markdown: &str) -> String {
    let mut body = String::new();
    let lines: Vec<&str> = markdown.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let line = trimmed
            .strip_prefix("# ")
            .or_else(|| trimmed.strip_prefix("## "))
            .or_else(|| trimmed.strip_prefix("- "))
            .unwrap_or(trimmed);

        let plain = extract_inline_text(line);
        if !body.is_empty() {
            body.push(' ');
        }
        body.push_str(&plain);
    }

    body
}

fn extract_inline_text(input: &str) -> String {
    let mut text = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        match chars[index] {
            '[' => {
                if let Some(close_label) = chars[index + 1..].iter().position(|ch| *ch == ']') {
                    let close_label = index + 1 + close_label;
                    if chars.get(close_label + 1) == Some(&'(') {
                        if let Some(close_target) =
                            chars[close_label + 2..].iter().position(|ch| *ch == ')')
                        {
                            let close_target = close_label + 2 + close_target;
                            let label: String = chars[index + 1..close_label].iter().collect();
                            text.push_str(&label);
                            index = close_target + 1;
                            continue;
                        }
                    }
                }
                text.push('[');
                index += 1;
            }
            '`' => {
                if let Some(close_code) = chars[index + 1..].iter().position(|ch| *ch == '`') {
                    let close_code = index + 1 + close_code;
                    let code: String = chars[index + 1..close_code].iter().collect();
                    text.push_str(&code);
                    index = close_code + 1;
                    continue;
                }
                text.push('`');
                index += 1;
            }
            ch => {
                text.push(ch);
                index += 1;
            }
        }
    }

    text
}

fn escape_json(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{extract_inline_text, extract_search_body};

    #[test]
    fn extracts_search_body_text_from_markdown() {
        let markdown = "# Title\n\nText with [link](./page.md) and `code`.\n\n- Item\n";
        let body = extract_search_body(markdown);

        assert_eq!(body, "Title Text with link and code. Item");
    }

    #[test]
    fn extracts_inline_text_from_links() {
        assert_eq!(
            extract_inline_text("Jump to [Example UI](/modules/ui/docs/index.md)."),
            "Jump to Example UI."
        );
    }
}
