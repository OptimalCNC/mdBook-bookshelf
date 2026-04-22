use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn build_cli_emits_bookshelf_return_assets_without_fixture_residue() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook-bookshelf"));
    let fixture_root = repo_root.join("bookshelf/handoffs/examples/self-contained");
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = make_temp_dir("chunk-011-build-cli", &repo_root);
    let fixture_entries_before = without_bookshelf_ui_entries(collect_tree_entries(&fixture_root));

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
    assert_text_contains(&root_index_html, "bookshelf-return.css");
    assert_text_contains(&root_index_html, "bookshelf-return.js");
    assert_text_not_contains(&root_index_html, "Choose a book to enter its root page.");
    assert_text_not_contains(
        &root_index_html,
        "Parser-specific reference pages with their own reading order.",
    );

    let parser_index_html = assert_read_to_string(output_dir.join("books/parser/index.html"));
    assert_text_contains(&parser_index_html, "bookshelf-return.css");
    assert_text_contains(&parser_index_html, "bookshelf-return.js");

    assert_exists(output_dir.join("books/meta/index.html"));
    assert_exists(output_dir.join("books/meta/bookshelf.html"));
    assert_exists(output_dir.join("books/meta/onboarding.html"));
    assert_exists(output_dir.join("books/meta/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/meta"), "book-", ".js");
    let root_return_script =
        assert_single_file_named_recursive(&output_dir.join("books/meta"), "bookshelf-return.js");
    let root_return_css =
        assert_single_file_named_recursive(&output_dir.join("books/meta"), "bookshelf-return.css");
    assert_file_contains(
        root_return_script.clone(),
        "const bookshelfTarget = \"bookshelf.html\";",
    );
    assert_file_contains(
        root_return_script.clone(),
        "link.textContent = \"Bookshelf\";",
    );
    assert_file_contains(
        root_return_script.clone(),
        "document.querySelector(\"#mdbook-menu-bar .right-buttons\")",
    );
    assert_file_contains(root_return_script, "currentPage === \"bookshelf.html\"");
    assert_file_contains(root_return_css, ".bookshelf-return-link");
    assert_exists(output_dir.join("books/parser/index.html"));
    assert_exists(output_dir.join("books/parser/grammar.html"));
    assert_exists(output_dir.join("books/parser/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/parser"), "book-", ".js");
    let parser_return_script =
        assert_single_file_named_recursive(&output_dir.join("books/parser"), "bookshelf-return.js");
    let parser_return_css = assert_single_file_named_recursive(
        &output_dir.join("books/parser"),
        "bookshelf-return.css",
    );
    assert_file_contains(
        parser_return_script.clone(),
        "const bookshelfTarget = \"../meta/bookshelf.html\";",
    );
    assert_file_contains(parser_return_script, "link.rel = \"up\";");
    assert_file_contains(parser_return_css, ".bookshelf-return-link");
    assert!(!fixture_root.join(".mdbook-bookshelf").exists());
    assert_eq!(collect_tree_entries(&fixture_root), fixture_entries_before);

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_resolves_relative_mdbook_paths_from_bookshelf_config_dir() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook-bookshelf"));
    let fixture_root = repo_root.join("tests/fixtures/build-cli/shared-config-root");
    let config_path = fixture_root.join("bookshelf.toml");
    let output_dir = make_temp_dir("chunk-011-shared-config-root", &repo_root);

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

fn assert_single_file_named_recursive(dir: &Path, name: &str) -> PathBuf {
    let mut matches = Vec::new();
    collect_files_named_recursive(dir, name, &mut matches);
    matches.sort();
    assert_eq!(
        matches.len(),
        1,
        "expected {} to contain exactly one file named {}",
        dir.display(),
        name
    );
    matches.pop().expect("matching file should exist")
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

fn collect_files_named_recursive(dir: &Path, name: &str, matches: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files_named_recursive(&path, name, matches);
        } else if path.file_name().is_some_and(|file_name| file_name == name) {
            matches.push(path);
        }
    }
}

fn collect_tree_entries(root: &Path) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    collect_tree_entries_recursive(root, root, &mut entries);
    entries.sort();
    entries
}

fn collect_tree_entries_recursive(root: &Path, dir: &Path, entries: &mut Vec<PathBuf>) {
    let mut dir_entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();
    dir_entries.sort_by_key(|entry| entry.path());

    for entry in dir_entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or_else(|err| panic!("failed to strip prefix {}: {err}", root.display()));
        entries.push(relative.to_path_buf());
        if path.is_dir() {
            collect_tree_entries_recursive(root, &path, entries);
        }
    }
}

fn without_bookshelf_ui_entries(entries: Vec<PathBuf>) -> Vec<PathBuf> {
    entries
        .into_iter()
        .filter(|entry| !entry.starts_with(".mdbook-bookshelf"))
        .collect()
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
