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
                assert_eq!(Path::new("docs"), config.books[0].book_src.as_path());
                assert_eq!(
                    config.config_dir.join("docs/SUMMARY.md"),
                    config.books[0].summary_abs
                );
            }
        }
    }

    let empty_catalog_path = write_temp_bookshelf_toml(
        &root,
        "chunk-002-empty-catalog",
        "[bookshelf]\nroot_book = \"meta\"\n",
    );
    let empty_catalog_error = load_bookshelf_config(&empty_catalog_path).expect_err("must fail");
    assert_eq!(
        "bookshelf.book catalog cannot be empty",
        empty_catalog_error.to_string()
    );
}

struct Case {
    path: PathBuf,
    expected_error: Option<&'static str>,
}

fn write_temp_bookshelf_toml(root: &Path, tag: &str, content: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be valid")
        .as_nanos();
    let dir = root.join(".tmp").join(format!("{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp fixture directory should be created");
    let file = dir.join("bookshelf.toml");
    fs::write(&file, content).expect("temp fixture file should be written");
    file
}
