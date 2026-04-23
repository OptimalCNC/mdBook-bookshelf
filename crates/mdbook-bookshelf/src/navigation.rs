use crate::site_model::{SiteModel, SitePageKind};
use anyhow::{bail, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationMetadata {
    pub root_book_id: String,
    pub by_page_id: BTreeMap<String, PageNavigation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageNavigation {
    pub page_id: String,
    pub owning_book_id: String,
    pub prev_page_id: Option<String>,
    pub next_page_id: Option<String>,
    pub breadcrumb: Option<String>,
    pub active_book_id: String,
}

impl NavigationMetadata {
    pub fn for_page(&self, page_id: &str) -> Option<&PageNavigation> {
        self.by_page_id.get(page_id)
    }

    pub fn resolve_active_book_id(&self, page_id: &str) -> Option<&str> {
        self.by_page_id
            .get(page_id)
            .map(|entry| entry.active_book_id.as_str())
    }
}

pub fn build_navigation_metadata(site_model: &SiteModel) -> Result<NavigationMetadata> {
    let mut by_page_id = BTreeMap::new();

    let pages_by_id: BTreeMap<_, _> = site_model
        .pages
        .iter()
        .map(|page| (page.page_id.as_str(), page))
        .collect();

    let synthetic = pages_by_id
        .get(site_model.synthetic_bookshelf_page_id.as_str())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "navigation metadata missing synthetic bookshelf page '{}'",
                site_model.synthetic_bookshelf_page_id
            )
        })?;
    by_page_id.insert(
        synthetic.page_id.clone(),
        PageNavigation {
            page_id: synthetic.page_id.clone(),
            owning_book_id: synthetic.owning_book_id.clone(),
            prev_page_id: None,
            next_page_id: None,
            breadcrumb: None,
            active_book_id: site_model.root_book_id.clone(),
        },
    );

    for book in &site_model.books {
        for (idx, page_id) in book.page_ids_in_order.iter().enumerate() {
            let page = pages_by_id.get(page_id.as_str()).ok_or_else(|| {
                anyhow::anyhow!(
                    "navigation metadata missing site page '{}' referenced by book '{}'",
                    page_id,
                    book.book_id
                )
            })?;
            if page.kind != SitePageKind::Content {
                bail!(
                    "navigation metadata expected content page '{}' for book '{}'",
                    page_id,
                    book.book_id
                );
            }
            if page.owning_book_id != book.book_id {
                bail!(
                    "navigation metadata ownership mismatch for page '{}': page owns '{}' book owns '{}'",
                    page_id,
                    page.owning_book_id,
                    book.book_id
                );
            }

            let prev_page_id = if idx == 0 {
                None
            } else {
                Some(book.page_ids_in_order[idx - 1].clone())
            };
            let next_page_id = if idx + 1 >= book.page_ids_in_order.len() {
                None
            } else {
                Some(book.page_ids_in_order[idx + 1].clone())
            };
            let breadcrumb = format!("{} / {}", book.title, page.title);

            by_page_id.insert(
                page.page_id.clone(),
                PageNavigation {
                    page_id: page.page_id.clone(),
                    owning_book_id: page.owning_book_id.clone(),
                    prev_page_id,
                    next_page_id,
                    breadcrumb: Some(breadcrumb),
                    active_book_id: book.book_id.clone(),
                },
            );
        }
    }

    Ok(NavigationMetadata {
        root_book_id: site_model.root_book_id.clone(),
        by_page_id,
    })
}
