use crate::navigation::NavigationMetadata;
use crate::site_model::{SiteModel, SitePageKind};
use anyhow::{Result, bail};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDocuments {
    pub documents: Vec<SearchDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDocument {
    pub document_id: String,
    pub page_id: String,
    pub title: String,
    pub content_text: String,
    pub owning_book_id: String,
    pub owning_book_label: String,
    pub breadcrumb: String,
}

pub fn build_search_documents(
    site_model: &SiteModel,
    navigation: &NavigationMetadata,
) -> Result<SearchDocuments> {
    let pages_by_id: BTreeMap<_, _> = site_model
        .pages
        .iter()
        .map(|page| (page.page_id.as_str(), page))
        .collect();
    let books_by_id: BTreeMap<_, _> = site_model
        .books
        .iter()
        .map(|book| (book.book_id.as_str(), book))
        .collect();

    let mut documents = Vec::new();
    for book in &site_model.books {
        for page_id in &book.page_ids_in_order {
            let page = pages_by_id.get(page_id.as_str()).ok_or_else(|| {
                anyhow::anyhow!(
                    "search model missing site page '{}' referenced by book '{}'",
                    page_id,
                    book.book_id
                )
            })?;
            if page.kind != SitePageKind::Content {
                bail!(
                    "search model expected content page '{}' for book '{}'",
                    page_id,
                    book.book_id
                );
            }

            let nav = navigation.for_page(page_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "search model missing navigation metadata for page '{}'",
                    page_id
                )
            })?;
            let breadcrumb = nav.breadcrumb.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "search model missing breadcrumb for content page '{}'",
                    page_id
                )
            })?;

            let owning_book = books_by_id.get(page.owning_book_id.as_str()).ok_or_else(|| {
                anyhow::anyhow!(
                    "search model missing owning book '{}' for page '{}'",
                    page.owning_book_id,
                    page_id
                )
            })?;

            documents.push(SearchDocument {
                document_id: format!("search:{}", page.page_id),
                page_id: page.page_id.clone(),
                title: page.title.clone(),
                content_text: page.title.clone(),
                owning_book_id: page.owning_book_id.clone(),
                owning_book_label: owning_book.title.clone(),
                breadcrumb,
            });
        }
    }

    Ok(SearchDocuments { documents })
}
