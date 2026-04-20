use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mdbook_bookshelf::{
    build_reader_context_model, build_render_manifest, build_sidebar_model, build_site_model,
    load_input_catalog, render_site,
};

#[test]
fn renders_representative_authored_pages_with_navigation_chrome() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");
    let render_manifest =
        build_render_manifest(&site_model, &reader_context).expect("render manifest should build");
    let output_dir = temp_root().join(unique_dir_name("authored-page-html-example"));

    let written_paths = render_site(
        &output_dir,
        &site_model,
        &sidebar_model,
        &reader_context,
        &render_manifest,
    )
    .expect("site render should succeed");
    assert_eq!(written_paths.len(), 10);

    let grammar_html = fs::read_to_string(output_dir.join("modules/parser/docs/grammar.html"))
        .expect("grammar html should exist");
    assert!(grammar_html.contains("Example Parser / Grammar"));
    assert!(grammar_html.contains("class=\"bookshelf-return\" href=\"../../../index.html\""));
    assert!(grammar_html.contains(">Example Parser<"));
    assert!(grammar_html.contains(">Grammar<"));
    assert!(grammar_html.contains(">Runtime<"));
    assert!(!grammar_html.contains(">Example UI<"));
    assert!(!grammar_html.contains(">Example Core<"));
    assert!(grammar_html.contains("class=\"nav-chapter previous\" href=\"index.html\""));
    assert!(grammar_html.contains("class=\"nav-chapter next\" href=\"runtime.html\""));
    assert!(grammar_html.contains("<h1>Grammar</h1>"));
    assert!(grammar_html.contains("parser-only concern"));
    assert!(grammar_html.contains("class=\"page-wrapper\""));
    assert!(grammar_html.contains("class=\"sidebar\""));
    assert!(grammar_html.contains("class=\"content\""));
    assert!(grammar_html.contains("class=\"nav-wrapper\""));

    let onboarding_html = fs::read_to_string(output_dir.join("docs/onboarding.html"))
        .expect("onboarding html should exist");
    let onboarding_sidebar = sidebar_region(&onboarding_html);
    assert!(onboarding_html.contains("class=\"sidebar-affix\""));
    assert!(onboarding_html.contains("class=\"affix\" href=\"../index.html\">Bookshelf</a>"));
    assert!(onboarding_sidebar.contains(">Example Core<"));
    assert!(onboarding_sidebar.contains(">Onboarding<"));
    assert!(onboarding_sidebar.contains(">Architecture<"));
    assert!(!onboarding_sidebar.contains(">Example Parser<"));
    assert!(!onboarding_sidebar.contains(">Example UI<"));
    assert!(onboarding_html.contains("<h1>Onboarding</h1>"));
    assert!(
        onboarding_html.contains("Readers start here before they jump into parser or UI details.")
    );
    assert!(onboarding_html.contains("href=\"../modules/parser/docs/index.html\""));
    assert!(!onboarding_html.contains("/modules/parser/docs/index.md"));

    let parser_index_html = fs::read_to_string(output_dir.join("modules/parser/docs/index.html"))
        .expect("parser index html should exist");
    assert!(parser_index_html.contains("href=\"grammar.html\""));
    assert!(parser_index_html.contains("href=\"runtime.html\""));
    assert!(!parser_index_html.contains("./grammar.md"));
    assert!(!parser_index_html.contains("./runtime.md"));

    let runtime_html = fs::read_to_string(output_dir.join("modules/parser/docs/runtime.html"))
        .expect("runtime html should exist");
    assert!(runtime_html.contains("href=\"../../ui/docs/index.html\""));
    assert!(!runtime_html.contains("/modules/ui/docs/index.md"));
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
