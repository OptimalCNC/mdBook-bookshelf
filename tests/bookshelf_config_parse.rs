use mdbook_bookshelf::load_bookshelf_config;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn bookshelf_config_parse() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/fixtures/bookshelf-config");

    let cases = [
        Case {
            path: fixtures.join("valid/bookshelf.toml"),
            expected_error: None,
        },
        Case {
            path: fixtures.join("invalid-duplicate-id/bookshelf.toml"),
            expected_error: Some(
                "bookshelf.book.src 'modules/parser' resolves to duplicate mount path 'modules/parser'",
            ),
        },
        Case {
            path: fixtures.join("invalid-missing-root/bookshelf.toml"),
            expected_error: Some(
                "bookshelf.book.src 'books/meta/docs' resolves to reserved mount path 'books/meta'",
            ),
        },
    ];

    for case in cases {
        let result = load_bookshelf_config(&case.path);
        match case.expected_error {
            Some(expected) => {
                let err = result.expect_err("fixture should fail");
                assert_eq!(expected, err.to_string());
            }
            None => {
                let config = result.expect("fixture should parse");
                assert_eq!("meta", config.root_book_id);
                assert_eq!(
                    "Core Docs",
                    config.mdbook_config.book.title.as_deref().unwrap()
                );
                assert_eq!(
                    "Root \"book\".",
                    config.mdbook_config.book.description.as_deref().unwrap()
                );
                assert_eq!(Path::new("docs"), config.mdbook_config.book.src.as_path());
                assert_eq!(1, config.books.len());
                assert_eq!(
                    Path::new("modules/parser/docs"),
                    config.books[0].source_rel.as_path()
                );
                assert_eq!(Path::new("modules/parser"), config.books[0].mount_rel.as_path());
                assert_eq!(Path::new("docs"), config.books[0].book.src.as_path());
            }
        }
    }

    let missing_root_title_dir = make_temp_dir("chunk-023-missing-root-title", &root);
    let missing_root_title_path = write_temp_bookshelf_toml(
        &missing_root_title_dir,
        r#"
[book]
src = "docs"

[bookshelf]
root-id = "meta"
"#,
    );
    let empty_catalog_error =
        load_bookshelf_config(&missing_root_title_path).expect_err("must fail");
    assert_eq!(
        "missing required key book.title",
        empty_catalog_error.to_string()
    );
    fs::remove_dir_all(&missing_root_title_dir).expect("temp fixture directory should be removed");

    let quoted_semantics_dir = make_temp_dir("chunk-023-quoted-semantics", &root);
    let quoted_semantics_path = write_temp_bookshelf_toml(
        &quoted_semantics_dir,
        r#"
[book]
title = "Meta"
src = "docs"
authors = ["A", "B"]

[output.html]
no-section-label = false

[bookshelf]
root-id = "meta" # comment

[[bookshelf.book]]
title = "Core #1 = Intro"
src = "modules/parser/docs"
"#,
    );
    let quoted_semantics = load_bookshelf_config(&quoted_semantics_path).expect("must parse");
    assert_eq!(
        "Core #1 = Intro",
        quoted_semantics.books[0].book.title.as_deref().unwrap()
    );
    fs::remove_dir_all(&quoted_semantics_dir).expect("temp fixture directory should be removed");

    let invalid_parent_src_dir = make_temp_dir("chunk-023-invalid-parent-src", &root);
    let invalid_parent_src_path = write_temp_bookshelf_toml(
        &invalid_parent_src_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Parser"
src = "../modules/parser/docs"
"#,
    );
    let invalid_parent_src_error =
        load_bookshelf_config(&invalid_parent_src_path).expect_err("must fail");
    assert_eq!(
        "book 'bookshelf.book' src path must not contain '..': '../modules/parser/docs'",
        invalid_parent_src_error.to_string()
    );
    fs::remove_dir_all(&invalid_parent_src_dir).expect("temp fixture directory should be removed");

    let empty_src_dir = make_temp_dir("chunk-023-empty-src", &root);
    let empty_src_path = write_temp_bookshelf_toml(
        &empty_src_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Parser"
src = ""
"#,
    );
    let empty_src_error = load_bookshelf_config(&empty_src_path).expect_err("must fail");
    assert_eq!(
        "book 'bookshelf.book' src path must not be empty",
        empty_src_error.to_string()
    );
    fs::remove_dir_all(&empty_src_dir).expect("temp fixture directory should be removed");

    let file_like_src_dir = make_temp_dir("chunk-023-file-like-src", &root);
    let file_like_src_path = write_temp_bookshelf_toml(
        &file_like_src_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Parser"
src = "modules/parser/docs/SUMMARY.md"
"#,
    );
    let file_like_src_error = load_bookshelf_config(&file_like_src_path).expect_err("must fail");
    assert_eq!(
        "book 'bookshelf.book' src path must name a source directory, not SUMMARY.md: 'modules/parser/docs/SUMMARY.md'",
        file_like_src_error.to_string()
    );
    fs::remove_dir_all(&file_like_src_dir).expect("temp fixture directory should be removed");

    let removed_summary_dir = make_temp_dir("chunk-023-removed-summary", &root);
    let removed_summary_path = write_temp_bookshelf_toml(
        &removed_summary_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Parser"
src = "modules/parser/docs"
summary = "conflicting/docs/SUMMARY.md"
"#,
    );
    let removed_summary_error =
        load_bookshelf_config(&removed_summary_path).expect_err("must fail");
    assert_eq!(
        "failed to parse mdBook book config for bookshelf.book",
        removed_summary_error.to_string()
    );
    fs::remove_dir_all(&removed_summary_dir).expect("temp fixture directory should be removed");

    let legacy_root_dir = make_temp_dir("chunk-023-legacy-root", &root);
    let legacy_root_path = write_temp_bookshelf_toml(
        &legacy_root_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Parser"
root = "modules/parser"
src = "modules/parser/docs"
"#,
    );
    let legacy_root_error = load_bookshelf_config(&legacy_root_path).expect_err("must fail");
    assert_eq!(
        "failed to parse mdBook book config for bookshelf.book",
        legacy_root_error.to_string()
    );
    fs::remove_dir_all(&legacy_root_dir).expect("temp fixture directory should be removed");

    let legacy_id_dir = make_temp_dir("chunk-023-legacy-id", &root);
    let legacy_id_path = write_temp_bookshelf_toml(
        &legacy_id_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
id = "parser"
title = "Parser"
src = "modules/parser/docs"
"#,
    );
    let legacy_id_error = load_bookshelf_config(&legacy_id_path).expect_err("must fail");
    assert_eq!(
        "failed to parse mdBook book config for bookshelf.book",
        legacy_id_error.to_string()
    );
    fs::remove_dir_all(&legacy_id_dir).expect("temp fixture directory should be removed");

    let current_dir_src_dir = make_temp_dir("chunk-023-current-dir-src", &root);
    let current_dir_src_path = write_temp_bookshelf_toml(
        &current_dir_src_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Parser"
src = "."
"#,
    );
    let current_dir_src_error =
        load_bookshelf_config(&current_dir_src_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book.src must not resolve to the site root mount path: '.'",
        current_dir_src_error.to_string()
    );
    fs::remove_dir_all(&current_dir_src_dir).expect("temp fixture directory should be removed");

    let dotted_src_dir = make_temp_dir("chunk-023-dotted-src", &root);
    let dotted_src_path = write_temp_bookshelf_toml(
        &dotted_src_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Parser"
src = "./modules/parser/docs"
"#,
    );
    let dotted_src =
        load_bookshelf_config(&dotted_src_path).expect("must accept dotted relative src");
    assert_eq!(
        Path::new("modules/parser/docs"),
        dotted_src.books[0].source_rel.as_path()
    );
    assert_eq!(Path::new("modules/parser"), dotted_src.books[0].mount_rel.as_path());
    assert_eq!(Path::new("docs"), dotted_src.books[0].book.src.as_path());
    fs::remove_dir_all(&dotted_src_dir).expect("temp fixture directory should be removed");

    let invalid_root_id_dir = make_temp_dir("chunk-023-invalid-root-id", &root);
    let invalid_root_id_path = write_temp_bookshelf_toml(
        &invalid_root_id_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "../../escape"
"#,
    );
    let invalid_root_id_error =
        load_bookshelf_config(&invalid_root_id_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.root-id must be a single safe path segment: '../../escape'",
        invalid_root_id_error.to_string()
    );
    fs::remove_dir_all(&invalid_root_id_dir).expect("temp fixture directory should be removed");

    let reserved_root_key_dir = make_temp_dir("chunk-023-reserved-root-key", &root);
    let reserved_root_key_path = write_temp_bookshelf_toml(
        &reserved_root_key_dir,
        r#"
[book]
title = "Meta"
src = "docs"

[bookshelf]
root-id = "meta"

[[bookshelf.book]]
title = "Child"
src = "meta/docs"
"#,
    );
    let reserved_root_key_error =
        load_bookshelf_config(&reserved_root_key_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book.src 'meta/docs' resolves to reserved mount key 'meta' owned by bookshelf.root-id",
        reserved_root_key_error.to_string()
    );
    fs::remove_dir_all(&reserved_root_key_dir).expect("temp fixture directory should be removed");
}

struct Case {
    path: PathBuf,
    expected_error: Option<&'static str>,
}

fn make_temp_dir(tag: &str, root: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let dir = std::env::temp_dir()
        .join("mdbook-bookshelf")
        .join(root.file_name().unwrap_or_default())
        .join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp fixture directory should be created");
    dir
}

fn write_temp_bookshelf_toml(dir: &Path, content: &str) -> PathBuf {
    let file = dir.join("bookshelf.toml");
    fs::write(&file, content).expect("temp fixture file should be written");
    file
}
