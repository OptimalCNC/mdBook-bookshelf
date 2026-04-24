use mdbook_bookshelf::{load_bookshelf_config, BookshelfEntryPage};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parses_source_derived_root_and_child_routes() {
    let temp = TempDir::new("chunk-06a-config-parse-valid");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
description = "Root book."
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "Parser Docs"
src = "modules/parser/docs"
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("fixture should parse");
    assert_eq!(BookshelfEntryPage::RootBook, config.entry_page);
    assert_eq!(
        "Core Docs",
        config.mdbook_config.book.title.as_deref().unwrap()
    );
    assert_eq!(
        "Root book.",
        config.mdbook_config.book.description.as_deref().unwrap()
    );
    assert_eq!(Path::new("docs"), config.mdbook_config.book.src.as_path());
    assert_eq!(1, config.books.len());
    assert_eq!(
        Path::new("modules/parser/docs"),
        config.books[0].source_rel.as_path()
    );
    assert_eq!(Path::new("docs"), config.books[0].book.src.as_path());
}

#[test]
fn parses_configured_bookshelf_entry_page() {
    let temp = TempDir::new("chunk-06a-config-parse-entry-page");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
entry-page = "bookshelf"
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("fixture should parse");
    assert_eq!(BookshelfEntryPage::Bookshelf, config.entry_page);
}

#[test]
fn rejects_legacy_root_id_as_unknown_field() {
    let temp = TempDir::new("chunk-06a-config-parse-legacy-root-id");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-id = "meta"
"#,
    );

    let error = load_bookshelf_config(&config_path).expect_err("legacy root-id must fail");
    assert!(format!("{error:#}").contains("unknown field `root-id`"));
}

#[test]
fn rejects_duplicate_canonical_output_roots() {
    let temp = TempDir::new("chunk-06a-config-parse-duplicate-output");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "Parser Docs"
src = "docs"
"#,
    );

    let error = load_bookshelf_config(&config_path).expect_err("duplicate output root must fail");
    assert_eq!(
        "bookshelf.book.src 'docs' resolves to duplicate canonical output root 'docs'",
        error.to_string()
    );
}

#[test]
fn rejects_overlapping_canonical_output_roots() {
    let root_child_temp = TempDir::new("chunk-06a-config-parse-overlap-root-child");
    let root_child_config_path = write_temp_bookshelf_toml(
        root_child_temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "Nested Child"
src = "docs/api"
"#,
    );

    let root_child_error =
        load_bookshelf_config(&root_child_config_path).expect_err("overlap must fail");
    assert_eq!(
        "bookshelf.book.src 'docs/api' resolves to overlapping canonical output root 'docs'",
        root_child_error.to_string()
    );

    let child_child_temp = TempDir::new("chunk-06a-config-parse-overlap-child-child");
    let child_child_config_path = write_temp_bookshelf_toml(
        child_child_temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.book]]
title = "Parser API"
src = "modules/parser/docs/reference"
"#,
    );

    let child_child_error =
        load_bookshelf_config(&child_child_config_path).expect_err("nested child overlap must fail");
    assert_eq!(
        "bookshelf.book.src 'modules/parser/docs/reference' resolves to overlapping canonical output root 'modules/parser/docs'",
        child_child_error.to_string()
    );
}

#[test]
fn rejects_site_root_output_roots() {
    let root_temp = TempDir::new("chunk-06a-config-parse-root-site-root");
    let root_config_path = write_temp_bookshelf_toml(
        root_temp.path(),
        r#"
[book]
title = "Core Docs"
src = "."

[bookshelf]
"#,
    );

    let root_error =
        load_bookshelf_config(&root_config_path).expect_err("site root book.src must fail");
    assert_eq!(
        "book.src must not resolve to the site root canonical output root: '.'",
        root_error.to_string()
    );

    let child_temp = TempDir::new("chunk-06a-config-parse-child-site-root");
    let child_config_path = write_temp_bookshelf_toml(
        child_temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "Parser Docs"
src = "."
"#,
    );

    let child_error =
        load_bookshelf_config(&child_config_path).expect_err("site root child src must fail");
    assert_eq!(
        "bookshelf.book.src must not resolve to the site root canonical output root: '.'",
        child_error.to_string()
    );
}

#[test]
fn preserves_local_book_src_for_nested_child_source() {
    let temp = TempDir::new("chunk-06a-config-parse-dotted-child-src");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]

[[bookshelf.book]]
title = "Parser Docs"
src = "./modules/parser/docs"
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("dotted child src should parse");
    assert_eq!(
        Path::new("modules/parser/docs"),
        config.books[0].source_rel.as_path()
    );
    assert_eq!(Path::new("docs"), config.books[0].book.src.as_path());
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir()
            .join("mdbook-bookshelf")
            .join(root.file_name().unwrap_or_default())
            .join(format!("{tag}-{nanos}"));
        fs::create_dir_all(&path).expect("temp fixture directory should be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_temp_bookshelf_toml(dir: &Path, content: &str) -> PathBuf {
    let file = dir.join("bookshelf.toml");
    fs::write(&file, content).expect("temp fixture file should be written");
    file
}
