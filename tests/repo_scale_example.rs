use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn repo_scale_fixture_builds_with_expected_root_parser_hmi_and_search_behavior() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = env!("CARGO_BIN_EXE_mdbook-bookshelf");
    let relative_config_path = PathBuf::from("tests/fixtures/repo-scale/bookshelf.toml");
    let output_dir = temp_root().join(unique_dir_name("repo-scale-example"));

    let status = Command::new(binary)
        .current_dir(&repo_root)
        .arg("build")
        .arg(&relative_config_path)
        .arg(&output_dir)
        .status()
        .expect("build command should run");

    assert!(status.success());

    let root_html =
        fs::read_to_string(output_dir.join("index.html")).expect("root shelf page should exist");
    assert!(root_html.contains("MetaNC"));
    assert!(root_html.contains("G-code Parser"));
    assert!(root_html.contains("HMI"));
    assert!(root_html.contains("href=\"docs/index.html\""));
    assert!(root_html.contains("href=\"modules/parser/docs/index.html\""));
    assert!(root_html.contains("href=\"modules/hmi/docs/index.html\""));

    let metanc_html =
        fs::read_to_string(output_dir.join("docs/index.html")).expect("MetaNC page should exist");
    let metanc_sidebar = sidebar_region(&metanc_html);
    assert!(metanc_html.contains("class=\"sidebar-affix\""));
    assert!(metanc_html.contains("class=\"affix\" href=\"../index.html\">Bookshelf</a>"));
    assert!(metanc_sidebar.contains(">MetaNC<"));
    assert!(!metanc_sidebar.contains(">G-code Parser<"));
    assert!(!metanc_sidebar.contains(">HMI<"));

    let parser_html =
        fs::read_to_string(output_dir.join("modules/parser/docs/acceptance-reference.html"))
            .expect("parser acceptance reference page should exist");
    let parser_sidebar = sidebar_region(&parser_html);
    assert!(parser_html.contains("G-code Parser / Acceptance Reference"));
    assert!(parser_html.contains("class=\"bookshelf-return\" href=\"../../../index.html\""));
    assert!(parser_sidebar.contains(">G-code Parser<"));
    assert!(parser_sidebar.contains(">Acceptance Reference<"));
    assert!(!parser_sidebar.contains(">MetaNC<"));
    assert!(!parser_sidebar.contains(">HMI<"));
    assert!(parser_html.contains("class=\"nav-chapter previous\" href=\"index.html\""));
    assert!(!parser_html.contains("class=\"nav-chapter next\""));

    let hmi_html = fs::read_to_string(output_dir.join("modules/hmi/docs/index.html"))
        .expect("HMI page should exist");
    let hmi_sidebar = sidebar_region(&hmi_html);
    assert!(hmi_html.contains("class=\"bookshelf-return\" href=\"../../../index.html\""));
    assert!(hmi_sidebar.contains(">HMI<"));
    assert!(!hmi_sidebar.contains(">MetaNC<"));
    assert!(!hmi_sidebar.contains(">G-code Parser<"));

    let search_json =
        fs::read_to_string(output_dir.join("searchindex.json")).expect("search index should exist");
    assert!(search_json.contains("\"book_label\":\"MetaNC\""));
    assert!(search_json.contains("\"book_label\":\"G-code Parser\""));
    assert!(search_json.contains("\"book_label\":\"HMI\""));
    assert!(search_json.contains("\"href\":\"modules/parser/docs/acceptance-reference.html\""));
    assert!(search_json.contains("\"href\":\"modules/hmi/docs/index.html\""));
}

fn sidebar_region(html: &str) -> &str {
    let start = html
        .find("<nav id=\"sidebar\"")
        .expect("sidebar should exist");
    let end = html[start..]
        .find("</nav>")
        .map(|offset| start + offset)
        .expect("sidebar closing tag should exist");
    &html[start..end]
}

fn temp_root() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
}

fn unique_dir_name(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{timestamp}-{counter}")
}
