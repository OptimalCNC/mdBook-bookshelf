use crate::site_model::{SiteModel, SitePageKind};
use anyhow::{bail, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationMetadata {
    pub by_page_id: BTreeMap<String, PageNavigation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageNavigation {
    pub page_id: String,
    pub owning_book_id: String,
    pub prev_page_id: Option<String>,
    pub next_page_id: Option<String>,
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
            if page.owning_book_id.as_deref() != Some(book.book_id.as_str()) {
                bail!(
                    "navigation metadata ownership mismatch for page '{}': page owns {}; book owns '{}'",
                    page_id,
                    format_optional_book_id(page.owning_book_id.as_deref()),
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
            by_page_id.insert(
                page.page_id.clone(),
                PageNavigation {
                    page_id: page.page_id.clone(),
                    owning_book_id: book.book_id.clone(),
                    prev_page_id,
                    next_page_id,
                    active_book_id: book.book_id.clone(),
                },
            );
        }
    }

    Ok(NavigationMetadata { by_page_id })
}

fn format_optional_book_id(book_id: Option<&str>) -> String {
    match book_id {
        Some(book_id) => format!("'{book_id}'"),
        None => "no owning book".to_string(),
    }
}
