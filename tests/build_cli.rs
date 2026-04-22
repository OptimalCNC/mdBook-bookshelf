use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const BOOKSHELF_RETURN_CSS_PATH: &str = ".mdbook-bookshelf/shared/bookshelf-return.css";

#[test]
fn build_cli_emits_root_book_bookshelf_page_and_preserves_authored_root_index() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook-bookshelf"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = make_temp_dir("chunk-014-build-cli", &repo_root);

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let root_entry_html = assert_read_to_string(output_dir.join("index.html"));
    assert_text_contains(
        &root_entry_html,
        "http-equiv=\"refresh\" content=\"0; url=books/meta/bookshelf.html\"",
    );
    assert_text_contains(
        &root_entry_html,
        "window.location.replace(\"books/meta/bookshelf.html\")",
    );
    assert_text_contains(&root_entry_html, "href=\"books/meta/bookshelf.html\"");
    assert_text_not_contains(&root_entry_html, "bookshelf-card__link");
    assert_text_not_contains(&root_entry_html, "Example Core");

    let bookshelf_html = assert_read_to_string(output_dir.join("books/meta/bookshelf.html"));
    assert_text_contains(&bookshelf_html, "<h1 id=\"bookshelf\">");
    assert_text_contains(&bookshelf_html, "Choose a book to enter its root page.");
    assert_text_contains(&bookshelf_html, "Example Core");
    assert_text_contains(
        &bookshelf_html,
        "Repository-wide onboarding and architecture notes.",
    );
    assert_text_contains(&bookshelf_html, "Example Parser");
    assert_text_contains(
        &bookshelf_html,
        "Parser-specific reference pages with their own reading order.",
    );
    assert_text_contains(&bookshelf_html, "Example UI");
    assert_text_contains(
        &bookshelf_html,
        "Interface and runtime guides for the UI book.",
    );
    assert_text_contains(&bookshelf_html, "href=\"index.html\">Example Core</a>");
    assert_text_contains(
        &bookshelf_html,
        "href=\"../parser/index.html\">Example Parser</a>",
    );
    assert_text_contains(&bookshelf_html, "href=\"../ui/index.html\">Example UI</a>");
    assert_text_not_contains(&bookshelf_html, "href=\"bookshelf.html\">Example Core</a>");

    let root_index_html = assert_read_to_string(output_dir.join("books/meta/index.html"));
    assert_text_contains(
        &root_index_html,
        "<title>Example Core - Bookshelf Example</title>",
    );
    assert_text_contains(
        &root_index_html,
        "Repository-wide onboarding and architecture guidance for the example project.",
    );
    assert_text_contains(&root_index_html, "href=\"onboarding.html\"");
    assert_text_not_contains(&root_index_html, "Choose a book to enter its root page.");
    assert_text_not_contains(
        &root_index_html,
        "Parser-specific reference pages with their own reading order.",
    );
    assert_text_contains(
        &root_index_html,
        &format!("href=\"{BOOKSHELF_RETURN_CSS_PATH}\""),
    );
    assert_text_contains(
        &root_index_html,
        "src=\".mdbook-bookshelf/books/meta/bookshelf-return.js\"",
    );

    let parser_index_html = assert_read_to_string(output_dir.join("books/parser/index.html"));
    assert_text_contains(
        &parser_index_html,
        &format!("href=\"{BOOKSHELF_RETURN_CSS_PATH}\""),
    );
    assert_text_contains(
        &parser_index_html,
        "src=\".mdbook-bookshelf/books/parser/bookshelf-return.js\"",
    );

    assert_exists(output_dir.join("books/meta/index.html"));
    assert_exists(output_dir.join("books/meta/bookshelf.html"));
    assert_exists(output_dir.join("books/meta/onboarding.html"));
    assert_exists(output_dir.join("books/meta/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/meta"), "book-", ".js");
    let root_return_script =
        output_dir.join("books/meta/.mdbook-bookshelf/books/meta/bookshelf-return.js");
    assert_exists(root_return_script.clone());
    assert_file_contains(
        root_return_script,
        "const bookshelfTarget = \"bookshelf.html\";",
    );
    assert_exists(output_dir.join("books/parser/index.html"));
    assert_exists(output_dir.join("books/parser/grammar.html"));
    assert_exists(output_dir.join("books/parser/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/parser"), "book-", ".js");
    let parser_return_script =
        output_dir.join("books/parser/.mdbook-bookshelf/books/parser/bookshelf-return.js");
    assert_exists(parser_return_script.clone());
    assert_file_contains(
        parser_return_script.clone(),
        "const bookshelfTarget = \"../meta/bookshelf.html\";",
    );
    assert_file_contains(
        parser_return_script.clone(),
        "window.location.pathname.split(\"/\").pop()",
    );
    assert_file_contains(
        parser_return_script.clone(),
        "currentPage === \"bookshelf.html\"",
    );
    assert_file_contains(
        parser_return_script,
        "document.querySelector(\"#mdbook-menu-bar .right-buttons\")",
    );
    assert_exists(output_dir.join("books/meta/.mdbook-bookshelf/shared/bookshelf-return.css"));
    assert_exists(output_dir.join("books/parser/.mdbook-bookshelf/shared/bookshelf-return.css"));

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_resolves_relative_mdbook_paths_from_bookshelf_config_dir() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook-bookshelf"));
    let fixture_root = repo_root.join("tests/fixtures/build-cli/shared-config-root");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-014-shared-config-root", &repo_root);

    let output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .output()
        .expect("build command should run");

    if !output.status.success() {
        panic!(
            "build command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let root_css =
        assert_has_file_with_prefix(&output_dir.join("books/root/shared"), "site-", ".css");
    let child_css =
        assert_has_file_with_prefix(&output_dir.join("books/child/shared"), "site-", ".css");
    assert_eq!(root_css, child_css);

    assert_file_contains(
        output_dir.join("books/root/index.html"),
        &format!("shared/{root_css}"),
    );
    assert_file_contains(
        output_dir.join("books/child/index.html"),
        &format!("shared/{child_css}"),
    );
    assert_file_contains(
        output_dir.join("books/root/index.html"),
        BOOKSHELF_RETURN_CSS_PATH,
    );
    assert_file_contains(
        output_dir.join("books/child/index.html"),
        BOOKSHELF_RETURN_CSS_PATH,
    );

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

fn assert_exists(path: PathBuf) {
    assert!(path.exists(), "expected {} to exist", path.display());
}

fn assert_has_file_with_prefix(dir: &Path, prefix: &str, suffix: &str) -> String {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()));
    let matched = entries.filter_map(|entry| entry.ok()).find_map(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        (name.starts_with(prefix) && name.ends_with(suffix)).then(|| name.into_owned())
    });

    assert!(
        matched.is_some(),
        "expected {} to contain a file matching {}*{}",
        dir.display(),
        prefix,
        suffix
    );

    matched.expect("matching file should exist")
}

fn assert_file_contains(path: PathBuf, needle: &str) {
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    assert!(
        content.contains(needle),
        "expected {} to contain {:?}",
        path.display(),
        needle
    );
}

fn assert_read_to_string(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn assert_text_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected text to contain {:?}",
        needle
    );
}

fn assert_text_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "expected text not to contain {:?}",
        needle
    );
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
    fs::create_dir_all(&dir).expect("temp output directory should be created");
    dir
}
