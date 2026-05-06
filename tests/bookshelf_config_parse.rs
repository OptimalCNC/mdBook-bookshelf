use mdbook_bookshelf::{load_bookshelf_config, BookshelfCategory};
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
root-book-id = "core"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("fixture should parse");
    assert_eq!("core", config.root_book_id);
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
    assert_eq!("parser", config.books[0].id);
    assert_eq!(
        Path::new("modules/parser/docs"),
        config.books[0].source_rel.as_path()
    );
    assert_eq!(
        Path::new("modules/parser/docs"),
        config.books[0].book.src.as_path()
    );
    assert_eq!(Path::new(".mdbook/bookshelf"), config.asset_dir.as_path());
    assert_eq!(1, config.categories.len());
    assert_eq!("All Docs", config.categories[0].title);
    assert_eq!(vec!["core", "parser"], config.categories[0].book_ids);
}

#[test]
fn parses_documentation_index_categories_and_book_metadata() {
    let temp = TempDir::new("documentation-index-config-parse-valid");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
description = "Root book."
src = "docs"

[bookshelf]
root-book-id = "core"

[bookshelf.root-book]
cover = "assets/covers/core.png"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
description = "Parser book."
src = "modules/parser/docs"
cover = "assets/covers/parser.png"

[[bookshelf.category]]
title = "Start Here"
books = ["core"]

[[bookshelf.category]]
title = "Reference"
books = ["parser"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("fixture should parse");

    assert_eq!("core", config.root_book_id);
    assert_eq!(
        Some(Path::new("assets/covers/core.png")),
        config.root_book_cover.as_deref()
    );
    assert_eq!(Path::new("docs"), config.mdbook_config.book.src.as_path());
    assert_eq!(1, config.books.len());
    assert_eq!("parser", config.books[0].id);
    assert_eq!(
        Some(Path::new("assets/covers/parser.png")),
        config.books[0].cover.as_deref()
    );
    assert_eq!(2, config.categories.len());
    let categories: &[BookshelfCategory] = &config.categories;
    assert_eq!("Start Here", config.categories[0].title);
    assert_eq!(vec!["core"], categories[0].book_ids);
    assert_eq!("Reference", config.categories[1].title);
    assert_eq!(vec!["parser"], categories[1].book_ids);
}

#[test]
fn rejects_invalid_documentation_index_category_config() {
    for (tag, toml_fragment, expected) in [
        (
            "missing-root-id",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]

[[bookshelf.category]]
title = "Start Here"
books = ["core"]
"#,
            "missing required key bookshelf.root-book-id",
        ),
        (
            "missing-child-id",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
"#,
            "missing required key bookshelf.book.id",
        ),
        (
            "empty-root-id",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = ""

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
            "bookshelf.root-book-id must not be empty",
        ),
        (
            "invalid-root-id",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "co/re"

[[bookshelf.category]]
title = "All Docs"
books = ["co/re"]
"#,
            "bookshelf.root-book-id may only contain ASCII letters, numbers, '-', '_', and '.'",
        ),
        (
            "empty-child-id",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = ""
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
"#,
            "bookshelf.book.id must not be empty",
        ),
        (
            "invalid-child-id",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = "par/ser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "par/ser"]
"#,
            "bookshelf.book.id may only contain ASCII letters, numbers, '-', '_', and '.'",
        ),
        (
            "duplicate-id",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

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
            "unknown-category-book",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

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
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

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
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

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
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

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
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"
"#,
            "bookshelf must define at least one [[bookshelf.category]]",
        ),
        (
            "empty-category-title",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

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
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.category]]
books = ["core"]
"#,
            "missing required key bookshelf.category.title",
        ),
        (
            "missing-category-books",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.category]]
title = "All Docs"
"#,
            "missing required key bookshelf.category.books",
        ),
        (
            "empty-category-books",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.category]]
title = "All Docs"
books = []
"#,
            "bookshelf.category 'All Docs' must list at least one book",
        ),
        (
            "invalid-root-cover-parent",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[bookshelf.root-book]
cover = "../covers/core.png"

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
            "book 'core' bookshelf.root-book.cover must not contain '..'",
        ),
        (
            "invalid-child-cover-parent",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"
cover = "../covers/parser.png"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
"#,
            "book 'parser' bookshelf.book.cover must not contain '..'",
        ),
        (
            "entry-page-rejected",
            r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"
entry-page = "bookshelf"

[[bookshelf.category]]
title = "All Docs"
books = ["core"]
"#,
            "unknown field `entry-page`",
        ),
    ] {
        let temp = TempDir::new(&format!("documentation-index-config-{tag}"));
        let config_path = write_temp_bookshelf_toml(temp.path(), toml_fragment);
        let error = load_bookshelf_config(&config_path).expect_err("invalid config must fail");
        assert!(
            format!("{error:#}").contains(expected),
            "expected {expected:?}, got {error:#}"
        );
    }
}

#[test]
fn child_book_config_inherits_root_book_config_defaults() {
    let temp = TempDir::new("chunk-asset-dir-child-inherits-root-book");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
authors = ["Docs Team"]
language = "zh"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("fixture should parse");
    assert_eq!(vec!["Docs Team"], config.books[0].book.authors);
    assert_eq!("zh", config.books[0].book.language.as_deref().unwrap());
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
root-book-id = "core"

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
    let root_child_temp = TempDir::new("chunk-06a-config-parse-overlap-root-child");
    let root_child_config_path = write_temp_bookshelf_toml(
        root_child_temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = "nested"
title = "Nested Child"
src = "docs/api"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "nested"]
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
root-book-id = "core"

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
books = ["core", "parser", "parser-api"]
"#,
    );

    let child_child_error = load_bookshelf_config(&child_child_config_path)
        .expect_err("nested child overlap must fail");
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
root-book-id = "core"
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
root-book-id = "core"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "."

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
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
fn preserves_site_root_relative_book_src_for_nested_child_source() {
    let temp = TempDir::new("chunk-06a-config-parse-dotted-child-src");
    let config_path = write_temp_bookshelf_toml(
        temp.path(),
        r#"
[book]
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"

[[bookshelf.book]]
id = "parser"
title = "Parser Docs"
src = "./modules/parser/docs"

[[bookshelf.category]]
title = "All Docs"
books = ["core", "parser"]
"#,
    );

    let config = load_bookshelf_config(&config_path).expect("dotted child src should parse");
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
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"
asset-dir = ".generated/bookshelf"

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
title = "Core Docs"
src = "docs"

[bookshelf]
root-book-id = "core"
asset-dir = "{asset_dir}"
"#
            ),
        );

        let error = load_bookshelf_config(&config_path).expect_err("invalid asset-dir must fail");
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?}, got {error:#}"
        );
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
