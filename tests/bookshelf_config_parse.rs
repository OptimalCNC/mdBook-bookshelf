use mdbook_bookshelf::load_bookshelf_config;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parses_explicit_books_without_root_book_id() {
    let temp = TempDir::new("explicit-books-config-parse-valid");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"
description = "Site-level metadata."
src = "site-default"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
description = "Core book."
src = "docs"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
description = "Parser book."
src = "modules/parser/docs"

[[bookshelf.category]]
title = "Start Here"
books = ["core"]

[[bookshelf.category]]
title = "Reference"
books = ["parser"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("explicit books should parse");

    assert_eq!(
        "Documentation",
        config.mdbook_config.book.title.as_deref().unwrap()
    );
    assert_eq!(
        Path::new("site-default"),
        config.mdbook_config.book.src.as_path()
    );
    assert_eq!(2, config.books.len());
    assert_eq!("core", config.books[0].id);
    assert_eq!(Path::new("docs"), config.books[0].source_rel.as_path());
    assert_eq!("Core Docs", config.books[0].book.title.as_deref().unwrap());
    assert_eq!(
        "Core book.",
        config.books[0].book.description.as_deref().unwrap()
    );
    assert_eq!("parser", config.books[1].id);
    assert_eq!(
        Path::new("modules/parser/docs"),
        config.books[1].source_rel.as_path()
    );
    assert_eq!(2, config.categories.len());
    assert_eq!("Start Here", config.categories[0].title);
    assert_eq!(vec!["core"], config.categories[0].book_ids);
    assert_eq!("Reference", config.categories[1].title);
    assert_eq!(vec!["parser"], config.categories[1].book_ids);
}

#[test]
fn explicit_book_config_inherits_site_book_config_defaults() {
    let temp = TempDir::new("explicit-book-inherits-site-book-defaults");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"
authors = ["Docs Team"]
language = "zh"
src = "site-default"

[bookshelf]

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "Reference"
books = ["parser"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("fixture should parse");
    assert_eq!(vec!["Docs Team"], config.books[0].book.authors);
    assert_eq!("zh", config.books[0].book.language.as_deref().unwrap());
}

#[test]
fn rejects_root_book_id_as_unknown_bookshelf_field() {
    let temp = TempDir::new("explicit-books-config-root-book-id-rejected");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "Start Here"
books = ["core"]
"#,
    );

    expect_error_contains(&config_path, "unknown field `root-book-id`");
}

#[test]
fn rejects_invalid_documentation_index_category_config() {
    for (tag, toml_fragment, expected) in [
        (
            "missing-book-id",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["parser"]
"#,
            "missing required key bookshelf.book.id",
        ),
        (
            "empty-book-id",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = ""
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["parser"]
"#,
            "bookshelf.book.id must not be empty",
        ),
        (
            "dot-book-id",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "."
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["."]
"#,
            "bookshelf.book.id must not be '.' or '..'",
        ),
        (
            "parent-book-id",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = ".."
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = [".."]
"#,
            "bookshelf.book.id must not be '.' or '..'",
        ),
        (
            "invalid-book-id",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "par/ser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["par/ser"]
"#,
            "bookshelf.book.id may only contain ASCII letters, numbers, '-', '_', and '.'",
        ),
        (
            "duplicate-id",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.book]]
id = "core"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
            "duplicate bookshelf book id 'core'",
        ),
        (
            "missing-book-title",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "parser"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["parser"]
"#,
            "missing required key bookshelf.book.title",
        ),
        (
            "missing-book-src",
            r#"
[book]
title = "Documentation"
src = "site-default"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"

[[bookshelf.category]]
title = "Start Here"
books = ["core"]
"#,
            "missing required key bookshelf.book.src",
        ),
        (
            "unknown-category-book",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "missing"]
"#,
            "bookshelf.category 'All Docs' references unknown book id 'missing'",
        ),
        (
            "uncategorized-book",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "Start Here"
books = ["core"]
"#,
            "book id 'parser' must appear in exactly one bookshelf.category",
        ),
        (
            "duplicate-category-assignment",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "Start Here"
books = ["core"]

[[bookshelf.category]]
title = "Again"
books = ["core"]
"#,
            "book id 'core' appears in more than one bookshelf.category",
        ),
        (
            "duplicate-book-in-same-category",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "core"]
"#,
            "book id 'core' appears more than once in bookshelf.category 'All Docs'",
        ),
        (
            "no-categories",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"
"#,
            "bookshelf must define at least one [[bookshelf.category]]",
        ),
        (
            "empty-category-title",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = ""
books = ["core"]
"#,
            "bookshelf.category.title must not be empty",
        ),
        (
            "missing-category-title",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
books = ["core"]
"#,
            "missing required key bookshelf.category.title",
        ),
        (
            "missing-category-books",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
"#,
            "missing required key bookshelf.category.books",
        ),
        (
            "empty-category-books",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = []
"#,
            "bookshelf.category 'All Docs' must list at least one book",
        ),
        (
            "root-book-table-rejected",
            r#"
[book]
title = "Documentation"

[bookshelf]

[bookshelf.root-book]
cover = "assets/covers/core.png"

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
            "unknown field `root-book`",
        ),
        (
            "book-cover-rejected",
            r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"
cover = "assets/covers/parser.png"

[[bookshelf.category]]
title = "All Docs"
books = ["parser"]
"#,
            "unknown field `cover` in bookshelf.book",
        ),
        (
            "entry-page-rejected",
            r#"
[book]
title = "Documentation"

[bookshelf]
entry-page = "bookshelf"

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
            "unknown field `entry-page`",
        ),
    ] {
        let temp = TempDir::new(&format!("documentation-index-config-{tag}"));
        let config_path = write_temp_bookshelf_toml(temp.path(), toml_fragment);
        expect_error_contains(&config_path, expected);
    }
}

#[test]
fn rejects_legacy_root_id_as_unknown_field() {
    let temp = TempDir::new("chunk-06a-config-parse-legacy-root-id");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
    );

    expect_error_contains(&config_path, "unknown field `root-id`");
}

#[test]
fn rejects_duplicate_canonical_output_roots() {
    let temp = TempDir::new("chunk-06a-config-parse-duplicate-output");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
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
    let parent_child_temp = TempDir::new("chunk-06a-config-parse-overlap-parent-child");
    let parent_child_config_path = write_temp_bookshelf_toml(
        parent_child_temp.path(),
        r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.book]]
id = "nested"
title = "Nested Book"
src = "docs/api"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "nested"]
"#,
    );

    let parent_child_error =
        load_bookshelf_config(&parent_child_config_path).expect_err("overlap must fail");
    assert_eq!(
        "bookshelf.book.src 'docs/api' resolves to overlapping canonical output root 'docs'",
        parent_child_error.to_string()
    );

    let sibling_child_temp = TempDir::new("chunk-06a-config-parse-overlap-sibling-child");
    let sibling_child_config_path = write_temp_bookshelf_toml(
        sibling_child_temp.path(),
        r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.book]]
id = "parser-api"
title = "Parser API"
src = "modules/parser/docs/reference"

[[bookshelf.category]]
title = "All Docs"
books = ["parser", "parser-api"]
"#,
    );

    let sibling_child_error = load_bookshelf_config(&sibling_child_config_path)
        .expect_err("nested book overlap must fail");
    assert_eq!(
        "bookshelf.book.src 'modules/parser/docs/reference' resolves to overlapping canonical output root 'modules/parser/docs'",
        sibling_child_error.to_string()
    );
}

#[test]
fn rejects_site_root_output_roots_for_explicit_books() {
    let temp = TempDir::new("chunk-06a-config-parse-book-site-root");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"
src = "."

[bookshelf]

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "."

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
    );

    let error = load_bookshelf_config(&config_path).expect_err("site root book src must fail");
    assert_eq!(
        "bookshelf.book.src must not resolve to the site root canonical output root: '.'",
        error.to_string()
    );
}

#[test]
fn preserves_site_root_relative_book_src_for_nested_source() {
    let temp = TempDir::new("chunk-06a-config-parse-dotted-book-src");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"

[bookshelf]

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "./modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["parser"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("dotted book src should parse");
    assert_eq!(
        Path::new("modules/parser/docs"),
        config.books[0].source_rel.as_path()
    );
    assert_eq!(
        Path::new("modules/parser/docs"),
        config.books[0].book.src.as_path()
    );
}

#[test]
fn parses_custom_bookshelf_asset_dir() {
    let temp = TempDir::new("chunk-asset-dir-custom");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Documentation"

[bookshelf]
asset-dir = ".generated/bookshelf"

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("custom asset-dir should parse");
    assert_eq!(
        Path::new(".generated/bookshelf"),
        config.asset_dir.as_path()
    );
}

#[test]
fn rejects_invalid_bookshelf_asset_dirs() {
    for (tag, asset_dir, expected) in [
        (
            "empty",
            "",
            "book 'bookshelf' asset-dir path must not be empty",
        ),
        (
            "absolute",
            "/tmp/bookshelf-assets",
            "book 'bookshelf' asset-dir path must be relative",
        ),
        (
            "parent",
            "../bookshelf-assets",
            "book 'bookshelf' asset-dir path must not contain '..'",
        ),
        (
            "dot",
            ".",
            "[bookshelf].asset-dir must name a directory below the bookshelf config root",
        ),
    ] {
        let temp = TempDir::new(&format!("chunk-asset-dir-invalid-{tag}"));
        let config_path = write_temp_bookshelf_toml(
            temp.path(),
            &format!(
                r#"
[book]
title = "Documentation"

[bookshelf]
asset-dir = "{asset_dir}"
"#
            ),
        );

        expect_error_contains(&config_path, expected);
    }
}

#[test]
fn rejects_bookshelf_asset_dirs_that_overlap_book_sources() {
    for (tag, asset_dir, expected_source) in [
        ("core-equal", "docs", "docs"),
        ("core-descendant", "docs/.mdbook/bookshelf", "docs"),
        ("parser-ancestor", "modules", "modules/parser/docs"),
        (
            "parser-descendant",
            "modules/parser/docs/.mdbook/bookshelf",
            "modules/parser/docs",
        ),
    ] {
        let temp = TempDir::new(&format!("chunk-asset-dir-overlap-{tag}"));
        let config_path = write_temp_bookshelf_toml(
            temp.path(),
            &format!(
                r#"
[book]
title = "Documentation"

[bookshelf]
asset-dir = "{asset_dir}"

[[bookshelf.book]]
id = "core"
title = "Core Docs"
src = "docs"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
"#
            ),
        );

        let error = load_bookshelf_config(&config_path)
            .expect_err("asset-dir overlapping a book source must fail");
        let expected = format!(
            "[bookshelf].asset-dir '{asset_dir}' must not overlap book source directory '{expected_source}'"
        );
        assert_eq!(expected, error.to_string());
    }
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

fn expect_error_contains(config_path: &Path, expected: &str) {
    let error = load_bookshelf_config(config_path).expect_err("invalid config must fail");
    assert!(
        format!("{error:#}").contains(expected),
        "expected {expected:?}, got {error:#}"
    );
}
