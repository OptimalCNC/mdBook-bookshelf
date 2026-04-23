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
            expected_error: Some("duplicate bookshelf.book id: 'meta'"),
        },
        Case {
            path: fixtures.join("invalid-missing-root/bookshelf.toml"),
            expected_error: Some(
                "bookshelf.root_book 'meta' is not declared in [[bookshelf.book]]",
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
                assert_eq!("meta", config.root_book);
                assert_eq!(2, config.books.len());
                assert_eq!("meta", config.books[0].id);
                assert_eq!(
                    "Root \"book\".",
                    config.books[0].description.as_deref().unwrap()
                );
                assert_eq!(Path::new("docs"), config.books[0].book_src.as_path());
                assert_eq!(
                    config.config_dir.join("docs/SUMMARY.md"),
                    config.books[0].summary_abs
                );
            }
        }
    }

    let empty_catalog_dir = make_temp_dir("chunk-003-empty-catalog", &root);
    let empty_catalog_path =
        write_temp_bookshelf_toml(&empty_catalog_dir, "[bookshelf]\nroot_book = \"meta\"\n");
    let empty_catalog_error = load_bookshelf_config(&empty_catalog_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book catalog cannot be empty",
        empty_catalog_error.to_string()
    );
    fs::remove_dir_all(&empty_catalog_dir).expect("temp fixture directory should be removed");

    let quoted_semantics_dir = make_temp_dir("chunk-003-quoted-semantics", &root);
    let quoted_semantics_path = write_temp_bookshelf_toml(
        &quoted_semantics_dir,
        r#"
[book]
authors = ["A", "B"]

[output.html]
no-section-label = false

[bookshelf]
root_book = "meta" # comment

[[bookshelf.book]]
id = "meta"
title = "Core #1 = Intro"
src = "docs"
"#,
    );
    let quoted_semantics = load_bookshelf_config(&quoted_semantics_path).expect("must parse");
    assert_eq!("Core #1 = Intro", quoted_semantics.books[0].title);
    fs::remove_dir_all(&quoted_semantics_dir).expect("temp fixture directory should be removed");

    let invalid_parent_src_dir = make_temp_dir("chunk-003-invalid-parent-src", &root);
    let invalid_parent_src_path = write_temp_bookshelf_toml(
        &invalid_parent_src_dir,
        r#"
[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
src = "../docs"
"#,
    );
    let invalid_parent_src_error =
        load_bookshelf_config(&invalid_parent_src_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book 'meta' src path must not contain '..': '../docs'",
        invalid_parent_src_error.to_string()
    );
    fs::remove_dir_all(&invalid_parent_src_dir).expect("temp fixture directory should be removed");

    let empty_src_dir = make_temp_dir("chunk-003-empty-src", &root);
    let empty_src_path = write_temp_bookshelf_toml(
        &empty_src_dir,
        r#"
[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
src = ""
"#,
    );
    let empty_src_error = load_bookshelf_config(&empty_src_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book 'meta' src path must not be empty",
        empty_src_error.to_string()
    );
    fs::remove_dir_all(&empty_src_dir).expect("temp fixture directory should be removed");

    let file_like_src_dir = make_temp_dir("chunk-003-file-like-src", &root);
    let file_like_src_path = write_temp_bookshelf_toml(
        &file_like_src_dir,
        r#"
[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
src = "docs/SUMMARY.md"
"#,
    );
    let file_like_src_error = load_bookshelf_config(&file_like_src_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book 'meta' src path must name a source directory, not SUMMARY.md: 'docs/SUMMARY.md'",
        file_like_src_error.to_string()
    );
    fs::remove_dir_all(&file_like_src_dir).expect("temp fixture directory should be removed");

    let removed_summary_dir = make_temp_dir("chunk-003-removed-summary", &root);
    let removed_summary_path = write_temp_bookshelf_toml(
        &removed_summary_dir,
        r#"
[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
src = "docs"
summary = "conflicting/docs/SUMMARY.md"
"#,
    );
    let removed_summary_error =
        load_bookshelf_config(&removed_summary_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book 'meta' uses removed key 'summary'; use 'src' with the book source directory instead",
        removed_summary_error.to_string()
    );
    fs::remove_dir_all(&removed_summary_dir).expect("temp fixture directory should be removed");

    let current_dir_src_dir = make_temp_dir("chunk-003-current-dir-src", &root);
    let current_dir_src_path = write_temp_bookshelf_toml(
        &current_dir_src_dir,
        r#"
[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
src = "."
"#,
    );
    let current_dir_src =
        load_bookshelf_config(&current_dir_src_path).expect("must accept current-dir src");
    assert_eq!(Path::new("."), current_dir_src.books[0].book_src.as_path());
    assert_eq!(
        current_dir_src.config_dir.join(".").join("SUMMARY.md"),
        current_dir_src.books[0].summary_abs
    );
    fs::remove_dir_all(&current_dir_src_dir).expect("temp fixture directory should be removed");

    let dotted_src_dir = make_temp_dir("chunk-003-dotted-src", &root);
    let dotted_src_path = write_temp_bookshelf_toml(
        &dotted_src_dir,
        r#"
[bookshelf]
root_book = "meta"

[[bookshelf.book]]
id = "meta"
title = "Meta"
src = "./docs"
"#,
    );
    let dotted_src =
        load_bookshelf_config(&dotted_src_path).expect("must accept dotted relative src");
    assert_eq!(Path::new("docs"), dotted_src.books[0].book_src.as_path());
    assert_eq!(
        dotted_src.config_dir.join("docs/SUMMARY.md"),
        dotted_src.books[0].summary_abs
    );
    fs::remove_dir_all(&dotted_src_dir).expect("temp fixture directory should be removed");
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
