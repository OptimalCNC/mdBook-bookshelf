use std::path::PathBuf;

use mdbook_bookshelf::{
    build_reader_context_model, build_render_manifest, build_sidebar_model, build_site_model,
    load_input_catalog, BuildRenderManifestError,
};

#[test]
fn preserves_root_ownership_and_unique_output_paths() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");

    let render_manifest =
        build_render_manifest(&site_model, &reader_context).expect("render manifest should build");

    let outputs: Vec<_> = render_manifest
        .entries
        .iter()
        .map(|entry| entry.output_path.clone())
        .collect();
    let mut unique_outputs = outputs.clone();
    unique_outputs.sort();
    unique_outputs.dedup();

    assert_eq!(outputs.len(), unique_outputs.len());
    assert_eq!(
        render_manifest
            .entry_for_page_id("bookshelf")
            .expect("bookshelf entry should exist")
            .output_path,
        PathBuf::from("index.html")
    );
    assert!(render_manifest
        .entry_for_page_id("bookshelf")
        .expect("bookshelf entry should exist")
        .route_path
        .as_os_str()
        .is_empty());
    assert!(site_model.bookshelf_page.route_path.as_os_str().is_empty());
    assert!(reader_context
        .bookshelf_page_context
        .route_path
        .as_os_str()
        .is_empty());
    for context in &reader_context.authored_page_contexts {
        assert!(context.bookshelf_return.route_path.as_os_str().is_empty());
        assert_eq!(
            context.bookshelf_return.route_path,
            render_manifest
                .entry_for_page_id(&context.bookshelf_return.page_id)
                .expect("bookshelf manifest entry should exist")
                .route_path
        );
    }
}

#[test]
fn rejects_render_output_collisions_explicitly() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let mut reader_context = build_reader_context_model(&site_model, &sidebar_model)
        .expect("reader context should build");

    reader_context.authored_page_contexts[0].source_path =
        config_path.parent().unwrap().join("index.md");

    let error = build_render_manifest(&site_model, &reader_context).unwrap_err();
    assert!(matches!(
        error,
        BuildRenderManifestError::OutputPathCollision { ref output_path, .. }
            if output_path == &PathBuf::from("index.html")
    ));
}
