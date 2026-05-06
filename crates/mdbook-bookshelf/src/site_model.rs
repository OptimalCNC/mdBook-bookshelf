use crate::catalog::InputCatalog;
use crate::loader::LoadedBooks;
use anyhow::{bail, Result};
use mdbook_driver::book::BookItem;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteModel {
    pub root_book_id: String,
    pub documentation_index_page_id: String,
    pub books: Vec<SiteBook>,
    pub pages: Vec<SitePage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteBook {
    pub book_id: String,
    pub title: String,
    pub is_root_book: bool,
    pub page_ids_in_order: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SitePage {
    pub page_id: String,
    pub kind: SitePageKind,
    pub owning_book_id: Option<String>,
    pub title: String,
    pub source_path: Option<PathBuf>,
    pub order_in_book: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SitePageKind {
    SyntheticDocumentationIndex,
    Content,
}

pub fn build_site_model(catalog: &InputCatalog, loaded: &LoadedBooks) -> Result<SiteModel> {
    let root_book_id = catalog.root_book()?.id.clone();

    if root_book_id != loaded.root_book_id {
        bail!(
            "site model root book mismatch: catalog='{}' loaded='{}'",
            root_book_id,
            loaded.root_book_id
        );
    }
    if catalog.books.len() != loaded.books.len() {
        bail!(
            "site model book count mismatch: catalog={} loaded={}",
            catalog.books.len(),
            loaded.books.len()
        );
    }

    let mut books = Vec::with_capacity(catalog.books.len());
    let mut pages = Vec::new();

    let documentation_index_page_id = "documentation:index".to_string();
    pages.push(SitePage {
        page_id: documentation_index_page_id.clone(),
        kind: SitePageKind::SyntheticDocumentationIndex,
        owning_book_id: None,
        title: "Documentation".to_string(),
        source_path: None,
        order_in_book: None,
    });

    for (index, (catalog_book, loaded_book)) in catalog.books.iter().zip(&loaded.books).enumerate()
    {
        if catalog_book.id != loaded_book.book_id {
            bail!(
                "site model book order mismatch at index {}: catalog='{}' loaded='{}'",
                index,
                catalog_book.id,
                loaded_book.book_id
            );
        }

        let mut page_ids_in_order = Vec::new();
        for item in loaded_book.mdbook.iter() {
            let BookItem::Chapter(chapter) = item else {
                continue;
            };
            if chapter.path.is_none() {
                continue;
            }
            let order = page_ids_in_order.len();
            let path = chapter.path.clone().expect("checked is_some");
            let page_id = format!("{}:{:04}", loaded_book.book_id, order);
            pages.push(SitePage {
                page_id: page_id.clone(),
                kind: SitePageKind::Content,
                owning_book_id: Some(loaded_book.book_id.clone()),
                title: chapter.name.clone(),
                source_path: Some(path),
                order_in_book: Some(order),
            });
            page_ids_in_order.push(page_id);
        }

        books.push(SiteBook {
            book_id: catalog_book.id.clone(),
            title: catalog_book.title.clone(),
            is_root_book: catalog_book.is_root_book,
            page_ids_in_order,
        });
    }

    Ok(SiteModel {
        root_book_id,
        documentation_index_page_id,
        books,
        pages,
    })
}
