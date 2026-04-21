use crate::build_input_catalog;
use crate::build_navigation_metadata;
use crate::build_site_model;
use crate::config_projection::project_book_configs;
use crate::load_books_from_catalog;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

pub fn build_html_site(config_path: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> Result<()> {
    let config_path = config_path.as_ref();
    let output_dir = output_dir.as_ref();

    let catalog = build_input_catalog(config_path)?;
    let projected_configs = project_book_configs(config_path, &catalog)?;
    let loaded = load_books_from_catalog(&catalog)?;
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
    let mut content_output_paths = BTreeMap::new();
    for page in &site_model.pages {
        if let Some(order) = page.order_in_book {
            let rel = format!("{}/p{:04}.html", page.owning_book_id, order);
            content_output_paths.insert(page.page_id.clone(), rel);
        }
    }

    for page in &site_model.pages {
        let Some(order) = page.order_in_book else {
            continue;
        };
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
                .map(|path| format!("<a href=\"/{}\">Prev</a>", escape_html(path)))
        });
        let next_link = nav_page.next_page_id.as_ref().and_then(|id| {
            content_output_paths
                .get(id)
                .map(|path| format!("<a href=\"/{}\">Next</a>", escape_html(path)))
        });

        let html = format!(
            "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><title>{}</title></head>\
             <body data-mdbook-default-theme=\"{}\"><h1>{}</h1><p>{}</p><p>Book {}</p><p>Order {}</p>\
             <nav>{} {}</nav></body></html>",
            escape_html(html_lang),
            escape_html(&page.title),
            escape_html(&default_theme),
            escape_html(&page.title),
            escape_html(nav_page.breadcrumb.as_deref().unwrap_or("")),
            escape_html(&page.owning_book_id),
            order,
            prev_link.unwrap_or_default(),
            next_link.unwrap_or_default(),
        );
        std::fs::write(&file_path, html)
            .with_context(|| format!("failed to write {}", file_path.display()))?;
    }

    let mut book_list_items = String::new();
    for book in &site_model.books {
        let first_link = book
            .page_ids_in_order
            .first()
            .and_then(|id| content_output_paths.get(id))
            .map(|path| format!("/{}", path))
            .unwrap_or_else(|| "/".to_string());
        book_list_items.push_str(&format!(
            "<li data-book-id=\"{}\"><a href=\"{}\">{}</a></li>",
            escape_html(&book.book_id),
            escape_html(&first_link),
            escape_html(&book.title)
        ));
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
    let root_html = format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><title>Bookshelf</title></head>\
         <body data-mdbook-default-theme=\"{}\"><h1>Bookshelf</h1><ul>{}</ul></body></html>",
        escape_html(root_lang),
        escape_html(&root_theme),
        book_list_items
    );
    std::fs::write(output_dir.join("index.html"), root_html)
        .with_context(|| format!("failed to write {}", output_dir.join("index.html").display()))?;

    Ok(())
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
