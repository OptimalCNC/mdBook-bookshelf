use std::path::PathBuf;

use mdbook_bookshelf::{
    build_reader_context_model, build_render_manifest, build_sidebar_model, build_site_model,
    load_input_catalog, RenderedPageIdentity,
};

#[test]
fn builds_expected_render_manifest_for_the_self_contained_example() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let config_dir = config_path.parent().unwrap();
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");
    let render_manifest =
        build_render_manifest(&site_model, &reader_context).expect("render manifest should build");

    assert_eq!(render_manifest.entries.len(), 10);

    let bookshelf = render_manifest
        .entry_for_page_id("bookshelf")
        .expect("bookshelf entry should exist");
    assert_eq!(
        bookshelf.identity,
        RenderedPageIdentity::SyntheticPage {
            page_id: "bookshelf".to_owned()
        }
    );
    assert_eq!(bookshelf.output_path, PathBuf::from("index.html"));
    assert!(bookshelf.route_path.as_os_str().is_empty());

    assert_eq!(
        authored_output(&render_manifest, config_dir.join("docs/index.md")),
        PathBuf::from("docs/index.html")
    );
    assert_eq!(
        authored_output(&render_manifest, config_dir.join("docs/architecture.md")),
        PathBuf::from("docs/architecture.html")
    );
    assert_eq!(
        authored_output(
            &render_manifest,
            config_dir.join("modules/parser/docs/grammar.md")
        ),
        PathBuf::from("modules/parser/docs/grammar.html")
    );
    assert_eq!(
        authored_output(
            &render_manifest,
            config_dir.join("modules/ui/docs/index.md")
        ),
        PathBuf::from("modules/ui/docs/index.html")
    );

    for entry in &render_manifest.entries {
        match &entry.identity {
            RenderedPageIdentity::SyntheticPage { .. } => {
                assert_eq!(entry.output_path, PathBuf::from("index.html"));
            }
            RenderedPageIdentity::AuthoredPage { .. } => {
                assert_ne!(entry.output_path, PathBuf::from("index.html"));
                assert_eq!(
                    entry
                        .output_path
                        .extension()
                        .and_then(|extension| extension.to_str()),
                    Some("html")
                );
            }
        }
    }
}

fn authored_output(
    render_manifest: &mdbook_bookshelf::RenderManifest,
    source_path: PathBuf,
) -> PathBuf {
    render_manifest
        .authored_entry_for_source_path(source_path)
        .expect("authored entry should exist")
        .output_path
        .clone()
}
