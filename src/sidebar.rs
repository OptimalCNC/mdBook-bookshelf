use std::path::PathBuf;

use crate::site_model::{AuthoredPage, SiteModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarModel {
    pub sidebars: Vec<BookSidebar>,
}

impl SidebarModel {
    pub fn sidebar_for(&self, book_id: &str) -> Option<&BookSidebar> {
        self.sidebars
            .iter()
            .find(|sidebar| sidebar.book_id == book_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookSidebar {
    pub book_id: String,
    pub affix_entries: Vec<SidebarAffixEntry>,
    pub chapter_entries: Vec<SidebarChapter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarAffixEntry {
    pub title: String,
    pub target_page_id: String,
    pub target_route_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarChapter {
    pub title: String,
    pub source_path: PathBuf,
    pub summary_depth: usize,
    pub children: Vec<SidebarChapter>,
}

pub fn build_sidebar_model(site_model: &SiteModel) -> SidebarModel {
    let mut sidebars = Vec::with_capacity(site_model.books.len());

    for book in &site_model.books {
        let pages = authored_pages_for_book(site_model, &book.id);
        let mut page_index = 0;
        let chapter_entries = build_sidebar_chapters(&pages, &mut page_index, 0);

        let affix_entries = if book.id == site_model.root_book {
            vec![SidebarAffixEntry {
                title: site_model.bookshelf_page.title.clone(),
                target_page_id: site_model.bookshelf_page.page_id.clone(),
                target_route_path: site_model.bookshelf_page.route_path.clone(),
            }]
        } else {
            Vec::new()
        };

        sidebars.push(BookSidebar {
            book_id: book.id.clone(),
            affix_entries,
            chapter_entries,
        });
    }

    SidebarModel { sidebars }
}

fn authored_pages_for_book<'a>(site_model: &'a SiteModel, book_id: &str) -> Vec<&'a AuthoredPage> {
    let mut pages: Vec<_> = site_model
        .authored_pages
        .iter()
        .filter(|page| page.book_id == book_id)
        .collect();
    pages.sort_by_key(|page| page.order);
    pages
}

fn build_sidebar_chapters(
    pages: &[&AuthoredPage],
    index: &mut usize,
    depth: usize,
) -> Vec<SidebarChapter> {
    let mut chapters: Vec<SidebarChapter> = Vec::new();

    while *index < pages.len() {
        let page = pages[*index];

        if page.summary_depth < depth {
            break;
        }

        if page.summary_depth > depth {
            if let Some(last) = chapters.last_mut() {
                last.children = build_sidebar_chapters(pages, index, page.summary_depth);
                continue;
            }
        }

        let node_depth = page.summary_depth;
        let mut chapter = SidebarChapter {
            title: page.title.clone(),
            source_path: page.source_path.clone(),
            summary_depth: page.summary_depth,
            children: Vec::new(),
        };
        *index += 1;

        if *index < pages.len() && pages[*index].summary_depth > node_depth {
            chapter.children = build_sidebar_chapters(pages, index, pages[*index].summary_depth);
        }

        chapters.push(chapter);
    }

    chapters
}

#[cfg(test)]
mod tests {
    use super::{build_sidebar_chapters, SidebarChapter};
    use crate::site_model::AuthoredPage;
    use std::path::PathBuf;

    #[test]
    fn builds_nested_sidebar_chapters_from_summary_depth() {
        let pages = vec![
            AuthoredPage {
                title: "Root".to_owned(),
                source_path: PathBuf::from("root.md"),
                book_id: "book".to_owned(),
                order: 0,
                summary_depth: 0,
            },
            AuthoredPage {
                title: "Child".to_owned(),
                source_path: PathBuf::from("child.md"),
                book_id: "book".to_owned(),
                order: 1,
                summary_depth: 1,
            },
            AuthoredPage {
                title: "Sibling".to_owned(),
                source_path: PathBuf::from("sibling.md"),
                book_id: "book".to_owned(),
                order: 2,
                summary_depth: 0,
            },
        ];
        let page_refs: Vec<_> = pages.iter().collect();
        let mut index = 0;

        let chapters = build_sidebar_chapters(&page_refs, &mut index, 0);

        assert_eq!(
            chapters,
            vec![
                SidebarChapter {
                    title: "Root".to_owned(),
                    source_path: PathBuf::from("root.md"),
                    summary_depth: 0,
                    children: vec![SidebarChapter {
                        title: "Child".to_owned(),
                        source_path: PathBuf::from("child.md"),
                        summary_depth: 1,
                        children: vec![],
                    }],
                },
                SidebarChapter {
                    title: "Sibling".to_owned(),
                    source_path: PathBuf::from("sibling.md"),
                    summary_depth: 0,
                    children: vec![],
                },
            ]
        );
    }
}
