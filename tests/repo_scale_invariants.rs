use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn repo_scale_output_preserves_canonical_routes_affix_separation_and_search_hrefs() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = env!("CARGO_BIN_EXE_mdbook-bookshelf");
    let relative_config_path = PathBuf::from("tests/fixtures/repo-scale/bookshelf.toml");
    let output_dir = temp_root().join(unique_dir_name("repo-scale-invariants"));

    let status = Command::new(binary)
        .current_dir(&repo_root)
        .arg("build")
        .arg(&relative_config_path)
        .arg(&output_dir)
        .status()
        .expect("build command should run");

    assert!(status.success());
    assert!(!output_dir.join("bookshelf/index.html").exists());

    let files = collect_files(&output_dir);
    assert_eq!(
        files,
        vec![
            PathBuf::from("docs/index.html"),
            PathBuf::from("index.html"),
            PathBuf::from("modules/hmi/docs/index.html"),
            PathBuf::from("modules/parser/docs/acceptance-reference.html"),
            PathBuf::from("modules/parser/docs/index.html"),
            PathBuf::from("searchindex.json"),
        ]
    );

    let metanc_html =
        fs::read_to_string(output_dir.join("docs/index.html")).expect("MetaNC page should exist");
    assert!(metanc_html.contains("class=\"sidebar-affix\""));
    let sidebar = sidebar_region(&metanc_html);
    assert!(sidebar.contains(">MetaNC<"));
    assert!(!sidebar.contains(">G-code Parser<"));
    assert!(!sidebar.contains(">HMI<"));

    let parser_html =
        fs::read_to_string(output_dir.join("modules/parser/docs/acceptance-reference.html"))
            .expect("parser acceptance page should exist");
    assert!(!parser_html.contains("class=\"nav-chapter next\""));

    let search_json =
        fs::read_to_string(output_dir.join("searchindex.json")).expect("search index should exist");
    assert!(!search_json.contains("\"title\":\"Bookshelf\""));
    assert!(!search_json.contains(".md\""));
    assert!(!search_json.contains("\"href\":\"/\""));
    assert!(!search_json.contains("bookshelf/"));
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

fn collect_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_recursive(root, root, &mut files);
    files.sort();
    files
}

fn collect_files_recursive(root: &PathBuf, current: &PathBuf, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(current).expect("directory should be readable") {
        let entry = entry.expect("directory entry should exist");
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("file should be under root")
                    .to_path_buf(),
            );
        }
    }
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
