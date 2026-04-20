use std::path::PathBuf;

use mdbook_bookshelf::{build_sidebar_model, build_site_model, load_input_catalog};

#[test]
fn builds_expected_sidebar_trees_for_the_self_contained_example() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let input_catalog = load_input_catalog(&config_path).expect("input catalog should load");
    let site_model = build_site_model(&input_catalog).expect("site model should build");
    let sidebar_model = build_sidebar_model(&site_model);
    let config_dir = config_path.parent().unwrap();

    assert_eq!(sidebar_model.sidebars.len(), 3);

    let meta = sidebar_model
        .sidebar_for("meta")
        .expect("meta sidebar should exist");
    assert_eq!(meta.affix_entries.len(), 1);
    assert_eq!(meta.affix_entries[0].title, "Bookshelf");
    assert_eq!(meta.affix_entries[0].target_page_id, "bookshelf");
    assert_eq!(
        meta.chapter_entries,
        vec![chapter(
            "Example Core",
            PathBuf::from("docs/index.md"),
            0,
            vec![
                chapter("Onboarding", PathBuf::from("docs/onboarding.md"), 1, vec![]),
                chapter(
                    "Architecture",
                    PathBuf::from("docs/architecture.md"),
                    1,
                    vec![],
                ),
            ],
        )]
    );

    let parser = sidebar_model
        .sidebar_for("parser")
        .expect("parser sidebar should exist");
    assert!(parser.affix_entries.is_empty());
    assert_eq!(
        parser.chapter_entries,
        vec![chapter(
            "Example Parser",
            PathBuf::from("modules/parser/docs/index.md"),
            0,
            vec![
                chapter(
                    "Grammar",
                    PathBuf::from("modules/parser/docs/grammar.md"),
                    1,
                    vec![],
                ),
                chapter(
                    "Runtime",
                    PathBuf::from("modules/parser/docs/runtime.md"),
                    1,
                    vec![],
                ),
            ],
        )]
    );

    let ui = sidebar_model
        .sidebar_for("ui")
        .expect("ui sidebar should exist");
    assert!(ui.affix_entries.is_empty());
    assert_eq!(
        ui.chapter_entries,
        vec![chapter(
            "Example UI",
            PathBuf::from("modules/ui/docs/index.md"),
            0,
            vec![
                chapter(
                    "Navigation",
                    PathBuf::from("modules/ui/docs/navigation.md"),
                    1,
                    vec![],
                ),
                chapter(
                    "Diagnostics",
                    PathBuf::from("modules/ui/docs/diagnostics.md"),
                    1,
                    vec![],
                ),
            ],
        )]
    );

    for sidebar in &sidebar_model.sidebars {
        for chapter in flatten_chapters(&sidebar.chapter_entries) {
            assert!(
                chapter.source_path.starts_with(config_dir),
                "sidebar chapter should stay rooted in the example config tree"
            );
        }
    }
}

fn chapter(
    title: &str,
    relative_source_path: PathBuf,
    summary_depth: usize,
    children: Vec<mdbook_bookshelf::SidebarChapter>,
) -> mdbook_bookshelf::SidebarChapter {
    mdbook_bookshelf::SidebarChapter {
        title: title.to_owned(),
        source_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("bookshelf/handoffs/examples/self-contained")
            .join(relative_source_path),
        summary_depth,
        children,
    }
}

fn flatten_chapters(
    chapters: &[mdbook_bookshelf::SidebarChapter],
) -> Vec<&mdbook_bookshelf::SidebarChapter> {
    let mut flattened = Vec::new();
    for chapter in chapters {
        flattened.push(chapter);
        flattened.extend(flatten_chapters(&chapter.children));
    }
    flattened
}
