use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const BREADCRUMB_RUNTIME_HARNESS: &str = r##"
const fs = require("node:fs");

const [scriptPath, pageHref, pathToRoot] = process.argv.slice(1);
const pageUrl = new URL(pageHref);
let existingBreadcrumb = null;

const main = {
  prepended: [],
  prepend(node) {
    this.prepended.unshift(node);
    if (node.id) {
      existingBreadcrumb = node;
    }
  },
};

globalThis.document = {
  querySelector(selector) {
    return selector === "#mdbook-content main" ? main : null;
  },
  getElementById(id) {
    return existingBreadcrumb && existingBreadcrumb.id === id ? existingBreadcrumb : null;
  },
  createElement(tagName) {
    return {
      tagName: String(tagName).toUpperCase(),
      id: "",
      className: "",
      textContent: "",
      attributes: {},
      setAttribute(name, value) {
        this.attributes[String(name)] = String(value);
      },
    };
  },
};

globalThis.window = {
  location: {
    href: pageHref,
    pathname: pageUrl.pathname,
  },
};
globalThis.path_to_root = pathToRoot;

const script = fs.readFileSync(scriptPath, "utf8");
eval(script);

const node = main.prepended[0];
const lines = [`count=${main.prepended.length}`];
if (node) {
  lines.push(`tag=${node.tagName}`);
  lines.push(`id=${node.id}`);
  lines.push(`class=${node.className}`);
  lines.push(`text=${node.textContent}`);
  lines.push(`data=${node.attributes["data-bookshelf-breadcrumb"] || ""}`);
  lines.push(`aria=${node.attributes["aria-label"] || ""}`);
}
process.stdout.write(lines.join("\n"));
"##;

#[test]
fn build_cli_emits_bookshelf_ui_assets_without_fixture_residue() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook"));
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
    assert_text_contains(&root_index_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&root_index_html, "bookshelf-breadcrumb.js");
    assert_text_contains(&root_index_html, "bookshelf-return.css");
    assert_text_contains(&root_index_html, "bookshelf-return.js");
    assert_text_not_contains(&root_index_html, "Choose a book to enter its root page.");
    assert_text_not_contains(
        &root_index_html,
        "Parser-specific reference pages with their own reading order.",
    );

    let parser_index_html = assert_read_to_string(output_dir.join("books/parser/index.html"));
    assert_text_contains(&parser_index_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&parser_index_html, "bookshelf-breadcrumb.js");
    assert_text_contains(&parser_index_html, "bookshelf-return.css");
    assert_text_contains(&parser_index_html, "bookshelf-return.js");
    let architecture_html = assert_read_to_string(output_dir.join("books/meta/architecture.html"));
    assert_text_contains(&architecture_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&architecture_html, "bookshelf-breadcrumb.js");
    let grammar_html = assert_read_to_string(output_dir.join("books/parser/grammar.html"));
    assert_text_contains(&grammar_html, "bookshelf-breadcrumb.css");
    assert_text_contains(&grammar_html, "bookshelf-breadcrumb.js");

    assert_exists(output_dir.join("books/meta/index.html"));
    assert_exists(output_dir.join("books/meta/bookshelf.html"));
    assert_exists(output_dir.join("books/meta/architecture.html"));
    assert_exists(output_dir.join("books/meta/onboarding.html"));
    assert_exists(output_dir.join("books/meta/toc.html"));
    assert_has_file_with_prefix(&output_dir.join("books/meta"), "book-", ".js");
    let root_return_script =
        assert_single_file_named_recursive(&output_dir.join("books/meta"), "bookshelf-return.js");
    let root_return_css =
        assert_single_file_named_recursive(&output_dir.join("books/meta"), "bookshelf-return.css");
    let root_breadcrumb_script = assert_single_file_named_recursive(
        &output_dir.join("books/meta"),
        "bookshelf-breadcrumb.js",
    );
    let root_breadcrumb_css = assert_single_file_named_recursive(
        &output_dir.join("books/meta"),
        "bookshelf-breadcrumb.css",
    );
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
    assert_runtime_breadcrumb(
        &root_breadcrumb_script,
        "https://example.test/books/meta/architecture.html",
        "Example Core / Architecture",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/bookshelf.html",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/print.html",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/404.html",
    );
    assert_runtime_breadcrumb_absent(
        &root_breadcrumb_script,
        "https://example.test/books/meta/toc.html",
    );
    assert_file_contains(root_breadcrumb_css, ".bookshelf-breadcrumb");
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
    let parser_breadcrumb_script = assert_single_file_named_recursive(
        &output_dir.join("books/parser"),
        "bookshelf-breadcrumb.js",
    );
    let parser_breadcrumb_css = assert_single_file_named_recursive(
        &output_dir.join("books/parser"),
        "bookshelf-breadcrumb.css",
    );
    assert_file_contains(
        parser_return_script.clone(),
        "const bookshelfTarget = \"../meta/bookshelf.html\";",
    );
    assert_file_contains(parser_return_script, "link.rel = \"up\";");
    assert_file_contains(parser_return_css, ".bookshelf-return-link");
    assert_runtime_breadcrumb(
        &parser_breadcrumb_script,
        "https://example.test/books/parser/grammar.html",
        "Example Parser / Grammar",
    );
    assert_file_contains(parser_breadcrumb_css, ".bookshelf-breadcrumb");
    assert_text_not_contains(&bookshelf_html, "data-bookshelf-breadcrumb");
    assert!(!fixture_root.join(".mdbook-bookshelf").exists());
    assert_eq!(collect_tree_entries(&fixture_root), fixture_entries_before);

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn build_cli_resolves_relative_mdbook_paths_from_bookshelf_config_dir() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook"));
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

fn assert_runtime_breadcrumb(script_path: &Path, page_href: &str, expected_text: &str) {
    let result = run_breadcrumb_runtime(script_path, page_href, "");
    assert_eq!(
        result.count, 1,
        "expected exactly one breadcrumb node for {page_href}"
    );
    assert_eq!(result.tag.as_deref(), Some("NAV"));
    assert_eq!(result.id.as_deref(), Some("bookshelf-breadcrumb"));
    assert_eq!(result.class_name.as_deref(), Some("bookshelf-breadcrumb"));
    assert_eq!(result.text.as_deref(), Some(expected_text));
    assert_eq!(result.data_marker.as_deref(), Some("true"));
    assert_eq!(result.aria_label.as_deref(), Some("Breadcrumb"));
}

fn assert_runtime_breadcrumb_absent(script_path: &Path, page_href: &str) {
    let result = run_breadcrumb_runtime(script_path, page_href, "");
    assert_eq!(
        result.count, 0,
        "expected breadcrumb runtime to no-op for {page_href}"
    );
    assert_eq!(result.tag, None);
    assert_eq!(result.id, None);
    assert_eq!(result.class_name, None);
    assert_eq!(result.text, None);
}

fn run_breadcrumb_runtime(
    script_path: &Path,
    page_href: &str,
    path_to_root: &str,
) -> BreadcrumbRuntimeResult {
    let output = Command::new("node")
        .arg("-e")
        .arg(BREADCRUMB_RUNTIME_HARNESS)
        .arg(script_path)
        .arg(page_href)
        .arg(path_to_root)
        .output()
        .expect("node breadcrumb harness should run");

    if !output.status.success() {
        panic!(
            "node breadcrumb harness failed for {}\nstdout:\n{}\nstderr:\n{}",
            script_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_breadcrumb_runtime_output(&String::from_utf8_lossy(&output.stdout))
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

#[derive(Debug, Default, PartialEq, Eq)]
struct BreadcrumbRuntimeResult {
    count: usize,
    tag: Option<String>,
    id: Option<String>,
    class_name: Option<String>,
    text: Option<String>,
    data_marker: Option<String>,
    aria_label: Option<String>,
}

fn parse_breadcrumb_runtime_output(output: &str) -> BreadcrumbRuntimeResult {
    let mut result = BreadcrumbRuntimeResult::default();

    for line in output.lines() {
        if let Some(value) = line.strip_prefix("count=") {
            result.count = value
                .parse::<usize>()
                .unwrap_or_else(|err| panic!("invalid breadcrumb count {:?}: {err}", value));
        } else if let Some(value) = line.strip_prefix("tag=") {
            result.tag = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("id=") {
            result.id = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("class=") {
            result.class_name = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("text=") {
            result.text = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("data=") {
            result.data_marker = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("aria=") {
            result.aria_label = Some(value.to_string());
        }
    }

    if result.count == 0 {
        result.tag = None;
        result.id = None;
        result.class_name = None;
        result.text = None;
        result.data_marker = None;
        result.aria_label = None;
    }

    result
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
