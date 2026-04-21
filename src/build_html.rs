use crate::build_input_catalog;
use crate::build_navigation_metadata;
use crate::build_site_model;
use crate::config_projection::project_book_configs;
use crate::load_books_from_catalog;
use crate::load_single_book_with_config_and_parsed_summary;
use crate::render_html::{
    ContentPageRenderInput, RootBookLink, SidebarItem, render_content_page, render_root_page,
};
use anyhow::{Context, Result};
use mdbook_driver::book::BookItem;
use mdbook_driver::builtin_renderers::CmdRenderer;
use std::collections::BTreeMap;
use std::path::Path;

pub fn build_html_site(config_path: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> Result<()> {
    let config_path = config_path.as_ref();
    let output_dir = output_dir.as_ref();

    let catalog = build_input_catalog(config_path)?;
    let mut projected_configs = project_book_configs(config_path, &catalog)?;
    let preprocessor_table = load_preprocessor_table(config_path)?;
    if let Some(preprocessor) = preprocessor_table {
        for projected in &mut projected_configs {
            projected
                .config
                .set("preprocessor", preprocessor.clone())
                .with_context(|| {
                    format!(
                        "book '{}' failed to project global [preprocessor.*] config",
                        projected.book_id
                    )
                })?;
        }
    }
    let mut loaded = load_books_from_catalog(&catalog)?;
    apply_projected_configs_to_loaded_books(&projected_configs, &mut loaded)?;
    preprocess_loaded_books(&projected_configs, &mut loaded)?;
    let preprocessed_content_by_page_id = extract_page_content(&loaded);
    let site_model = build_site_model(&catalog, &loaded)?;
    let nav = build_navigation_metadata(&site_model)?;

    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir)
            .with_context(|| format!("failed to clean {}", output_dir.display()))?;
    }
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let projected_by_book: BTreeMap<_, _> = projected_configs
        .iter()
        .map(|p| (p.book_id.as_str(), &p.config))
        .collect();
    let books_by_id: BTreeMap<_, _> = site_model
        .books
        .iter()
        .map(|book| (book.book_id.as_str(), book))
        .collect();
    let pages_by_id: BTreeMap<_, _> = site_model
        .pages
        .iter()
        .map(|page| (page.page_id.as_str(), page))
        .collect();
    let mut content_output_paths = BTreeMap::new();
    for page in &site_model.pages {
        if let Some(order) = page.order_in_book {
            let rel = format!("{}/p{:04}.html", page.owning_book_id, order);
            content_output_paths.insert(page.page_id.clone(), rel);
        }
    }

    for page in &site_model.pages {
        if page.order_in_book.is_none() {
            continue;
        }
        let rel_path = content_output_paths
            .get(&page.page_id)
            .expect("content page should have deterministic output path");
        let file_path = output_dir.join(rel_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let nav_page = nav
            .for_page(&page.page_id)
            .ok_or_else(|| anyhow::anyhow!("missing navigation metadata for '{}'", page.page_id))?;
        let active_book = books_by_id
            .get(page.owning_book_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing active book '{}'", page.owning_book_id))?;
        let projected_cfg = projected_by_book
            .get(page.owning_book_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing projected config for '{}'", page.owning_book_id))?;
        let html_lang = projected_cfg.book.language.as_deref().unwrap_or("en");
        let default_theme = projected_cfg
            .get::<String>("output.html.default-theme")
            .ok()
            .flatten()
            .unwrap_or_else(|| "light".to_string());

        let prev_link = nav_page.prev_page_id.as_ref().and_then(|id| {
            content_output_paths
                .get(id)
                .map(|path| relative_href(rel_path, path))
        });
        let next_link = nav_page.next_page_id.as_ref().and_then(|id| {
            content_output_paths
                .get(id)
                .map(|path| relative_href(rel_path, path))
        });

        let mut sidebar_items = Vec::new();
        for book_page_id in &active_book.page_ids_in_order {
            let sidebar_rel = content_output_paths
                .get(book_page_id)
                .ok_or_else(|| anyhow::anyhow!("missing output path for '{}'", book_page_id))?;
            let sidebar_page = pages_by_id
                .get(book_page_id.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing site page '{}'", book_page_id))?;
            sidebar_items.push(SidebarItem {
                page_id: book_page_id.clone(),
                href: relative_href(rel_path, sidebar_rel),
                title: sidebar_page.title.clone(),
                is_active: *book_page_id == page.page_id,
            });
        }

        let html = render_content_page(&ContentPageRenderInput {
            html_lang: html_lang.to_string(),
            default_theme,
            page_title: page.title.clone(),
            page_id: page.page_id.clone(),
            owning_book_id: page.owning_book_id.clone(),
            breadcrumb: nav_page.breadcrumb.clone().unwrap_or_default(),
            bookshelf_href: relative_href(rel_path, "index.html"),
            sidebar_items,
            prev_href: prev_link,
            next_href: next_link,
        });
        let chapter_content = preprocessed_content_by_page_id
            .get(&page.page_id)
            .cloned()
            .unwrap_or_default();
        let chapter_html = format!(
            "<article class=\"chapter-content\">{}</article>",
            escape_html(&chapter_content)
        );
        let html = html.replacen("<nav class=\"pager\">", &(chapter_html + "<nav class=\"pager\">"), 1);
        std::fs::write(&file_path, html)
            .with_context(|| format!("failed to write {}", file_path.display()))?;
    }

    let mut book_links = Vec::new();
    for book in &site_model.books {
        let first_link = book
            .page_ids_in_order
            .first()
            .and_then(|id| content_output_paths.get(id))
            .map(|path| format!("/{}", path))
            .unwrap_or_else(|| "/".to_string());
        book_links.push(RootBookLink {
            book_id: book.book_id.clone(),
            href: first_link,
            title: book.title.clone(),
        });
    }
    let root_projected_cfg = projected_by_book
        .get(site_model.root_book_id.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing projected config for root book"))?;
    let root_lang = root_projected_cfg.book.language.as_deref().unwrap_or("en");
    let root_theme = root_projected_cfg
        .get::<String>("output.html.default-theme")
        .ok()
        .flatten()
        .unwrap_or_else(|| "light".to_string());
    let root_html = render_root_page(root_lang, &root_theme, &book_links);
    std::fs::write(output_dir.join("index.html"), root_html)
        .with_context(|| format!("failed to write {}", output_dir.join("index.html").display()))?;

    Ok(())
}

fn preprocess_loaded_books(
    projected_configs: &[crate::config_projection::ProjectedBookConfig],
    loaded: &mut crate::loader::LoadedBooks,
) -> Result<()> {
    let projected_by_book: BTreeMap<_, _> = projected_configs
        .iter()
        .map(|p| (p.book_id.as_str(), &p.config))
        .collect();

    let renderer = CmdRenderer::new("html".to_string(), "true".to_string());
    for book in &mut loaded.books {
        let projected_cfg = projected_by_book
            .get(book.book_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing projected config for '{}'", book.book_id))?;
        book.mdbook.config = (*projected_cfg).clone();
        let (preprocessed_book, _) = book
            .mdbook
            .preprocess_book(&renderer)
            .with_context(|| {
                format!(
                    "book '{}' failed to preprocess for renderer '{}'",
                    book.book_id,
                    "html"
                )
            })?;
        book.mdbook.book = preprocessed_book;
    }
    Ok(())
}

fn apply_projected_configs_to_loaded_books(
    projected_configs: &[crate::config_projection::ProjectedBookConfig],
    loaded: &mut crate::loader::LoadedBooks,
) -> Result<()> {
    let projected_by_book: BTreeMap<_, _> = projected_configs
        .iter()
        .map(|p| (p.book_id.as_str(), &p.config))
        .collect();

    for book in &mut loaded.books {
        let projected_cfg = projected_by_book
            .get(book.book_id.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing projected config for '{}'", book.book_id))?;
        let mdbook = load_single_book_with_config_and_parsed_summary(
            &book.mdbook.root,
            &book.mdbook.config.book.src,
            (*projected_cfg).clone(),
            book.summary.clone(),
        )
        .with_context(|| {
            format!(
                "book '{}' failed to reload mdbook with projected config",
                book.book_id
            )
        })?;
        book.mdbook = mdbook;
    }

    Ok(())
}

fn load_preprocessor_table(config_path: &Path) -> Result<Option<toml::Value>> {
    let source_text = std::fs::read_to_string(config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    let source: toml::Value = toml::from_str(&source_text)
        .with_context(|| format!("failed to parse TOML in {}", config_path.display()))?;
    Ok(source.get("preprocessor").cloned())
}

fn extract_page_content(loaded: &crate::loader::LoadedBooks) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for book in &loaded.books {
        let mut order = 0usize;
        for item in book.mdbook.iter() {
            let BookItem::Chapter(chapter) = item else {
                continue;
            };
            if chapter.path.is_none() {
                continue;
            }
            let page_id = format!("{}:{:04}", book.book_id, order);
            out.insert(page_id, chapter.content.clone());
            order += 1;
        }
    }
    out
}

fn relative_href(from_file: &str, to_file: &str) -> String {
    let from_dir = Path::new(from_file).parent().unwrap_or_else(|| Path::new(""));
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = Path::new(to_file).components().collect();

    let mut common = 0;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }

    let mut parts: Vec<String> = Vec::new();
    for _ in common..from_components.len() {
        parts.push("..".to_string());
    }
    for component in &to_components[common..] {
        parts.push(component.as_os_str().to_string_lossy().to_string());
    }

    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
