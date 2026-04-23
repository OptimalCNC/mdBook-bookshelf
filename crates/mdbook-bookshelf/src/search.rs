use crate::catalog::InputCatalog;
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const SHARED_SEARCH_INDEX_NAME: &str = "searchindex.js";

const SEARCH_INDEX_JS_PREFIX: &str = "window.search = Object.assign(window.search, JSON.parse('";
const SEARCH_INDEX_JS_SUFFIX: &str = "'));";

pub(crate) fn write_site_wide_search_index(
    catalog: &InputCatalog,
    site_dest_dir: &Path,
) -> Result<()> {
    let payload = compose_shared_search_index(catalog, site_dest_dir)
        .context("failed to compose site-wide search payload from per-book search indexes")?;
    let output_path = site_dest_dir.join(SHARED_SEARCH_INDEX_NAME);
    let contents =
        render_search_index_js(&payload).context("failed to render site-wide search index")?;

    fs::write(&output_path, contents).with_context(|| {
        format!(
            "failed to write site-wide shared search index at {}",
            output_path.display()
        )
    })
}

fn compose_shared_search_index(
    catalog: &InputCatalog,
    site_dest_dir: &Path,
) -> Result<SearchPayload> {
    let mut shared: Option<SearchPayload> = None;

    for book in &catalog.books {
        let searchindex_path = find_emitted_searchindex_path(site_dest_dir, &book.id)?;
        let payload = parse_search_index_file(&searchindex_path).with_context(|| {
            format!(
                "failed to decode emitted search index for book '{}' at {}",
                book.id,
                searchindex_path.display()
            )
        })?;

        if let Some(existing) = shared.as_mut() {
            ensure_payload_compatibility(existing, &payload, &book.id)?;
            ingest_book_payload(existing, &book.id, payload)?;
        } else {
            let mut initial = SearchPayload::empty_from(&payload);
            ingest_book_payload(&mut initial, &book.id, payload)?;
            shared = Some(initial);
        }
    }

    shared
        .ok_or_else(|| anyhow!("cannot compose a shared search index without any configured books"))
}

fn find_emitted_searchindex_path(site_dest_dir: &Path, book_id: &str) -> Result<PathBuf> {
    let book_dir = site_dest_dir.join("books").join(book_id);
    let mut matches = fs::read_dir(&book_dir)
        .with_context(|| format!("failed to read built book directory {}", book_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("searchindex-") && name.ends_with(".js"))
        })
        .collect::<Vec<_>>();
    matches.sort();

    match matches.as_slice() {
        [path] => Ok(path.clone()),
        [] => bail!(
            "expected {} to contain exactly one emitted searchindex-*.js file, but found none",
            book_dir.display()
        ),
        _ => bail!(
            "expected {} to contain exactly one emitted searchindex-*.js file, but found {}",
            book_dir.display(),
            matches.len()
        ),
    }
}

fn parse_search_index_file(path: &Path) -> Result<SearchPayload> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let trimmed = source.trim();
    let encoded_json = trimmed
        .strip_prefix(SEARCH_INDEX_JS_PREFIX)
        .and_then(|rest| rest.strip_suffix(SEARCH_INDEX_JS_SUFFIX))
        .ok_or_else(|| {
            anyhow!(
                "search index at {} does not match the stock mdBook JSON.parse wrapper",
                path.display()
            )
        })?;
    let decoded_json =
        unescape_js_single_quoted(encoded_json).context("failed to decode wrapped search JSON")?;

    serde_json::from_str(&decoded_json).with_context(|| {
        format!(
            "failed to parse decoded search payload from {}",
            path.display()
        )
    })
}

fn render_search_index_js(payload: &SearchPayload) -> Result<String> {
    let json =
        serde_json::to_string(payload).context("failed to serialize shared search payload")?;
    Ok(format!(
        "window.search = Object.assign(window.search, {});\n",
        json
    ))
}

fn ensure_payload_compatibility(
    shared: &SearchPayload,
    candidate: &SearchPayload,
    book_id: &str,
) -> Result<()> {
    if shared.results_options != candidate.results_options {
        bail!(
            "book '{}' emitted different results_options for the stock search contract",
            book_id
        );
    }
    if shared.search_options != candidate.search_options {
        bail!(
            "book '{}' emitted different search_options for the stock search contract",
            book_id
        );
    }
    if shared.index.fields != candidate.index.fields {
        bail!("book '{}' emitted different indexed field order", book_id);
    }
    if shared.index.lang != candidate.index.lang {
        bail!(
            "book '{}' emitted different elasticlunr language metadata",
            book_id
        );
    }
    if shared.index.pipeline != candidate.index.pipeline {
        bail!(
            "book '{}' emitted different elasticlunr pipeline metadata",
            book_id
        );
    }
    if shared.index.ref_field != candidate.index.ref_field {
        bail!(
            "book '{}' emitted different elasticlunr ref field metadata",
            book_id
        );
    }
    if shared.index.version != candidate.index.version {
        bail!(
            "book '{}' emitted different elasticlunr version metadata",
            book_id
        );
    }
    if shared.index.document_store.save != candidate.index.document_store.save {
        bail!(
            "book '{}' emitted different document-store save metadata",
            book_id
        );
    }

    Ok(())
}

fn ingest_book_payload(
    shared: &mut SearchPayload,
    book_id: &str,
    payload: SearchPayload,
) -> Result<()> {
    let SearchPayload {
        doc_urls, index, ..
    } = payload;
    let SavedIndex {
        mut document_store,
        index: field_indexes,
        ..
    } = index;

    if doc_urls.len() != document_store.docs.len() {
        bail!(
            "book '{}' emitted {} doc_urls but {} indexed docs",
            book_id,
            doc_urls.len(),
            document_store.docs.len()
        );
    }
    if doc_urls.len() != document_store.doc_info.len() {
        bail!(
            "book '{}' emitted {} doc_urls but {} docInfo entries",
            book_id,
            doc_urls.len(),
            document_store.doc_info.len()
        );
    }

    let mut id_map = BTreeMap::new();
    for (old_offset, doc_url) in doc_urls.into_iter().enumerate() {
        let old_id = old_offset.to_string();
        let new_id = shared.doc_urls.len().to_string();
        let mut doc = document_store.docs.remove(&old_id).ok_or_else(|| {
            anyhow!(
                "book '{}' is missing indexed doc '{}' while composing the shared index",
                book_id,
                old_id
            )
        })?;
        let doc_info = document_store.doc_info.remove(&old_id).ok_or_else(|| {
            anyhow!(
                "book '{}' is missing docInfo '{}' while composing the shared index",
                book_id,
                old_id
            )
        })?;

        doc.insert("id".to_string(), Value::String(new_id.clone()));
        shared.doc_urls.push(translate_doc_url(book_id, &doc_url));
        shared.index.document_store.docs.insert(new_id.clone(), doc);
        shared
            .index
            .document_store
            .doc_info
            .insert(new_id.clone(), doc_info);
        id_map.insert(old_id, new_id);
    }

    if !document_store.docs.is_empty() || !document_store.doc_info.is_empty() {
        bail!(
            "book '{}' left unconsumed search documents after remapping ids",
            book_id
        );
    }

    for (field, mut node) in field_indexes {
        remap_inverted_index_doc_refs(&mut node, &id_map).with_context(|| {
            format!(
                "failed to rewrite saved elasticlunr doc refs for field '{}' in book '{}'",
                field, book_id
            )
        })?;

        if let Some(existing) = shared.index.index.get_mut(&field) {
            merge_inverted_index_nodes(existing, node).with_context(|| {
                format!(
                    "failed to merge saved elasticlunr index for field '{}' in book '{}'",
                    field, book_id
                )
            })?;
        } else {
            shared.index.index.insert(field, node);
        }
    }

    shared.index.document_store.length = shared.index.document_store.docs.len() as u64;
    Ok(())
}

fn translate_doc_url(book_id: &str, doc_url: &str) -> String {
    let normalized = doc_url.replace('\\', "/");
    let relative = normalized.strip_prefix("./").unwrap_or(normalized.as_str());
    format!("../{book_id}/{relative}")
}

fn remap_inverted_index_doc_refs(
    node: &mut Value,
    id_map: &BTreeMap<String, String>,
) -> Result<()> {
    let object = node
        .as_object_mut()
        .ok_or_else(|| anyhow!("saved elasticlunr inverted-index node must be a JSON object"))?;

    if let Some(docs_value) = object.get_mut("docs") {
        let docs = docs_value.as_object_mut().ok_or_else(|| {
            anyhow!("saved elasticlunr inverted-index docs table must be a JSON object")
        })?;
        let mut remapped = Map::with_capacity(docs.len());
        for (old_id, value) in std::mem::take(docs) {
            let new_id = id_map.get(&old_id).ok_or_else(|| {
                anyhow!(
                    "saved elasticlunr inverted-index docs table referenced unknown doc id '{}'",
                    old_id
                )
            })?;
            remapped.insert(new_id.clone(), value);
        }
        *docs = remapped;
    }

    for (key, child) in object.iter_mut() {
        if key == "df" || key == "docs" {
            continue;
        }
        remap_inverted_index_doc_refs(child, id_map)?;
    }

    Ok(())
}

fn merge_inverted_index_nodes(target: &mut Value, source: Value) -> Result<()> {
    let target_object = target
        .as_object_mut()
        .ok_or_else(|| anyhow!("saved elasticlunr target node must be a JSON object"))?;
    let source_object = source
        .as_object()
        .ok_or_else(|| anyhow!("saved elasticlunr source node must be a JSON object"))?;

    if let Some(source_df) = source_object.get("df") {
        let merged_df = target_object.get("df").and_then(Value::as_u64).unwrap_or(0)
            + source_df
                .as_u64()
                .ok_or_else(|| anyhow!("saved elasticlunr df value must be an unsigned integer"))?;
        target_object.insert("df".to_string(), Value::from(merged_df));
    }

    if let Some(source_docs) = source_object.get("docs") {
        let target_docs = target_object
            .entry("docs".to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| anyhow!("saved elasticlunr target docs table must be a JSON object"))?;
        let source_docs = source_docs
            .as_object()
            .ok_or_else(|| anyhow!("saved elasticlunr source docs table must be a JSON object"))?;

        for (doc_id, value) in source_docs {
            if target_docs.insert(doc_id.clone(), value.clone()).is_some() {
                bail!(
                    "saved elasticlunr merge encountered a duplicate remapped doc id '{}'",
                    doc_id
                );
            }
        }
    }

    for (key, source_child) in source_object {
        if key == "df" || key == "docs" {
            continue;
        }
        if let Some(target_child) = target_object.get_mut(key) {
            merge_inverted_index_nodes(target_child, source_child.clone())?;
        } else {
            target_object.insert(key.clone(), source_child.clone());
        }
    }

    Ok(())
}

fn unescape_js_single_quoted(text: &str) -> Result<String> {
    let mut decoded = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        let escaped = chars
            .next()
            .ok_or_else(|| anyhow!("unterminated escape sequence in wrapped search payload"))?;
        match escaped {
            '\\' => decoded.push('\\'),
            '\'' => decoded.push('\''),
            '"' => decoded.push('"'),
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            'b' => decoded.push('\u{0008}'),
            'f' => decoded.push('\u{000C}'),
            'u' => {
                let codepoint = take_hex_digits(&mut chars, 4)?;
                let value = u32::from_str_radix(&codepoint, 16)
                    .with_context(|| format!("invalid unicode escape sequence \\u{codepoint}"))?;
                let decoded_char = char::from_u32(value).ok_or_else(|| {
                    anyhow!("unicode escape sequence \\u{codepoint} is not a valid scalar value")
                })?;
                decoded.push(decoded_char);
            }
            'x' => {
                let codepoint = take_hex_digits(&mut chars, 2)?;
                let value = u8::from_str_radix(&codepoint, 16)
                    .with_context(|| format!("invalid hex escape sequence \\x{codepoint}"))?;
                decoded.push(value as char);
            }
            other => decoded.push(other),
        }
    }

    Ok(decoded)
}

fn take_hex_digits(chars: &mut std::str::Chars<'_>, count: usize) -> Result<String> {
    let mut digits = String::with_capacity(count);
    for _ in 0..count {
        let digit = chars
            .next()
            .ok_or_else(|| anyhow!("unterminated hexadecimal escape sequence"))?;
        digits.push(digit);
    }
    Ok(digits)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SearchPayload {
    doc_urls: Vec<String>,
    index: SavedIndex,
    results_options: Value,
    search_options: Value,
}

impl SearchPayload {
    fn empty_from(template: &SearchPayload) -> Self {
        Self {
            doc_urls: Vec::new(),
            index: SavedIndex {
                document_store: DocumentStore {
                    doc_info: BTreeMap::new(),
                    docs: BTreeMap::new(),
                    length: 0,
                    save: template.index.document_store.save,
                },
                fields: template.index.fields.clone(),
                index: BTreeMap::new(),
                lang: template.index.lang.clone(),
                pipeline: template.index.pipeline.clone(),
                ref_field: template.index.ref_field.clone(),
                version: template.index.version.clone(),
            },
            results_options: template.results_options.clone(),
            search_options: template.search_options.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SavedIndex {
    #[serde(rename = "documentStore")]
    document_store: DocumentStore,
    fields: Vec<String>,
    index: BTreeMap<String, Value>,
    lang: String,
    pipeline: Vec<String>,
    #[serde(rename = "ref")]
    ref_field: String,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct DocumentStore {
    #[serde(rename = "docInfo")]
    doc_info: BTreeMap<String, Value>,
    docs: BTreeMap<String, Map<String, Value>>,
    length: u64,
    save: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_doc_url_targets_book_siblings() {
        assert_eq!(
            translate_doc_url("parser", "grammar.html#grammar"),
            "../parser/grammar.html#grammar"
        );
        assert_eq!(
            translate_doc_url("meta", "./architecture.html#architecture"),
            "../meta/architecture.html#architecture"
        );
    }

    #[test]
    fn unescape_js_single_quoted_decodes_stock_wrapper_payloads() {
        let decoded =
            unescape_js_single_quoted(r#"{"title":"Don\'t","body":"line\nbreak","tab":"\t"}"#)
                .expect("payload should decode");

        assert_eq!(
            decoded,
            "{\"title\":\"Don't\",\"body\":\"line\nbreak\",\"tab\":\"\t\"}"
        );
    }

    #[test]
    fn merge_inverted_index_nodes_sums_df_and_unions_docs() {
        let mut target = serde_json::json!({
            "df": 1,
            "docs": {
                "0": { "tf": 1.0 }
            },
            "a": {
                "df": 1,
                "docs": {
                    "0": { "tf": 1.0 }
                }
            }
        });
        let source = serde_json::json!({
            "df": 2,
            "docs": {
                "1": { "tf": 1.0 },
                "2": { "tf": 1.0 }
            },
            "a": {
                "df": 1,
                "docs": {
                    "2": { "tf": 1.0 }
                }
            }
        });

        merge_inverted_index_nodes(&mut target, source).expect("nodes should merge");

        assert_eq!(target["df"], 3);
        assert!(target["docs"]["0"].is_object());
        assert!(target["docs"]["1"].is_object());
        assert!(target["docs"]["2"].is_object());
        assert_eq!(target["a"]["df"], 2);
        assert!(target["a"]["docs"]["0"].is_object());
        assert!(target["a"]["docs"]["2"].is_object());
    }
}
