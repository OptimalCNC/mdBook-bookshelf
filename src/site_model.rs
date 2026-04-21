use crate::catalog::InputCatalog;
use crate::loader::LoadedBooks;
use anyhow::{Result, bail};
use mdbook_driver::book::BookItem;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteModel {
    pub root_book_id: String,
    pub synthetic_bookshelf_page_id: String,
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
    pub owning_book_id: String,
    pub title: String,
    pub route: String,
    pub source_path: Option<PathBuf>,
    pub order_in_book: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SitePageKind {
    SyntheticBookshelf,
    Content,
}

pub fn build_site_model(catalog: &InputCatalog, loaded: &LoadedBooks) -> Result<SiteModel> {
    if catalog.root_book_id != loaded.root_book_id {
        bail!(
            "site model root book mismatch: catalog='{}' loaded='{}'",
            catalog.root_book_id,
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

    let synthetic_bookshelf_page_id = "bookshelf:root".to_string();
    pages.push(SitePage {
        page_id: synthetic_bookshelf_page_id.clone(),
        kind: SitePageKind::SyntheticBookshelf,
        owning_book_id: catalog.root_book_id.clone(),
        title: "Bookshelf".to_string(),
        route: "/".to_string(),
        source_path: None,
        order_in_book: None,
    });

    for (index, (catalog_book, loaded_book)) in catalog.books.iter().zip(&loaded.books).enumerate() {
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
            let route = format!(
                "/{}/{}",
                loaded_book.book_id,
                path.with_extension("").to_string_lossy().replace('\\', "/")
            );
            pages.push(SitePage {
                page_id: page_id.clone(),
                kind: SitePageKind::Content,
                owning_book_id: loaded_book.book_id.clone(),
                title: chapter.name.clone(),
                route,
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
        root_book_id: catalog.root_book_id.clone(),
        synthetic_bookshelf_page_id,
        books,
        pages,
    })
}
