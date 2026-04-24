use crate::bookshelf_ui::{BookshelfBreadcrumbPage, TransientBookshelfUiAssets};
use crate::catalog::{build_input_catalog, InputBook, InputCatalog};
use crate::config::BookshelfEntryPage;
use crate::loader::load_books_from_catalog;
use crate::navigation::build_navigation_metadata;
use crate::root_bookshelf_preprocessor::{
    ensure_reserved_bookshelf_path_is_available, inject_root_bookshelf_page,
    site_root_bookshelf_entry_path,
};
use crate::route_paths::{path_to_string, relative_path};
use crate::search::{write_site_wide_search_index, LOCAL_SHARED_SEARCH_INDEX_NAME};
use crate::site_model::{build_site_model, SitePageKind};
use crate::site_root_link_preprocessor::{SiteRootLinkMap, SiteRootLinkPreprocessor};
use crate::load_single_book_with_config_and_parsed_summary;
use anyhow::{Context, Result};
use mdbook_driver::config::Config;
use mdbook_summary::parse_summary;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn build_bookshelf(config_path: impl AsRef<Path>, dest_dir: Option<PathBuf>) -> Result<()> {
    build_bookshelf_site(config_path, dest_dir).map(|_| ())
}

pub fn build_bookshelf_site(
    config_path: impl AsRef<Path>,
    dest_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let config_path = config_path.as_ref();
    let catalog = absolutize_catalog_paths(build_input_catalog(config_path)?)
        .context("failed to resolve bookshelf catalog paths")?;
    let mdbook_config = catalog.mdbook_config.clone();
    let site_dest_dir = resolve_site_dest_dir(&catalog.config_dir, &mdbook_config, dest_dir)?;
    let config_root = catalog.config_dir.clone();
    let site_root_link_map = SiteRootLinkMap::from_catalog(&catalog)
        .context("failed to build site-root Markdown link map")?;
    let book_breadcrumbs = build_book_breadcrumbs(&catalog)
        .context("failed to build exact book/page breadcrumb metadata")?;
    let root_bookshelf_rel = PathBuf::from(site_root_bookshelf_entry_path(
        &catalog
            .root_book()
            .context("failed to resolve root book for synthetic bookshelf routing")?
            .output_rel,
    ));

    for book in &catalog.books {
        let breadcrumb_pages: &[BookshelfBreadcrumbPage] = book_breadcrumbs
            .get(&book.id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        build_catalog_book(
            book,
            &catalog,
            &mdbook_config,
            &config_root,
            &site_dest_dir,
            &root_bookshelf_rel,
            breadcrumb_pages,
            &site_root_link_map,
        )?;
    }

    write_site_wide_search_index(&catalog, &site_dest_dir)?;
    write_site_root_index(&catalog, &mdbook_config, &site_dest_dir)?;

    Ok(site_dest_dir)
}

fn build_catalog_book(
    book: &InputBook,
    catalog: &InputCatalog,
    shared_config: &Config,
    config_root: &Path,
    site_dest_dir: &Path,
    root_bookshelf_rel: &Path,
    breadcrumb_pages: &[BookshelfBreadcrumbPage],
    site_root_link_map: &SiteRootLinkMap,
) -> Result<()> {
    let summary_text = fs::read_to_string(&book.summary_abs).with_context(|| {
        format!(
            "book '{}' failed to read canonical summary at {}",
            book.id,
            book.summary_abs.display()
        )
    })?;
    let summary = parse_summary(&summary_text).with_context(|| {
        format!(
            "book '{}' failed to parse canonical summary at {}",
            book.id,
            book.summary_abs.display()
        )
    })?;

    let mut config = shared_config.clone();
    config.book = book.book_config.clone();
    config.build.build_dir = site_dest_dir.join(&book.output_rel);
    let mut ui_assets =
        TransientBookshelfUiAssets::new(&book.book_root_abs).with_context(|| {
            format!(
                "book '{}' failed to create transient bookshelf UI assets under {}",
                book.id,
                book.book_root_abs.display()
            )
        })?;
    ui_assets
        .stage_config_root_output_assets(&mut config, config_root)
        .with_context(|| {
            format!(
                "book '{}' failed to stage shared mdBook output assets from {} into {}",
                book.id,
                config_root.display(),
                book.book_root_abs.display()
            )
        })?;
    let return_target = path_to_string(&relative_path(&book.output_rel, &root_bookshelf_rel));
    let searchindex_target = LOCAL_SHARED_SEARCH_INDEX_NAME.to_string();

    ui_assets
        .inject_bookshelf_ui_assets(
            &mut config,
            &book.id,
            &return_target,
            &searchindex_target,
            breadcrumb_pages,
        )
        .with_context(|| {
            format!(
                "book '{}' failed to inject bookshelf UI assets under {}",
                book.id,
                config_root.display()
            )
        })?;

    let mut mdbook = load_single_book_with_config_and_parsed_summary(
        &book.book_root_abs,
        &book.book_src_rel,
        config,
        summary,
    )
    .with_context(|| {
        format!(
            "book '{}' failed to load mdbook from root {} and source {}",
            book.id,
            book.book_root_abs.display(),
            book.book_src_abs.display()
        )
    })?;
    ensure_reserved_bookshelf_path_is_available(&mdbook.book, &book.id).with_context(|| {
        format!(
            "book '{}' uses a reserved bookshelf content path while building {}",
            book.id,
            book.summary_abs.display()
        )
    })?;

    if book.is_root_book {
        inject_root_bookshelf_page(&mut mdbook.book, catalog).with_context(|| {
            format!(
                "book '{}' failed to inject synthetic root bookshelf page",
                book.id
            )
        })?;
    }

    mdbook.with_preprocessor(SiteRootLinkPreprocessor::new(
        book.output_rel.clone(),
        site_root_link_map.clone(),
    ));

    let html_build_dir = mdbook.build_dir_for("html");
    mdbook.build().with_context(|| {
        format!(
            "book '{}' failed to build mdbook output at {}",
            book.id,
            html_build_dir.display()
        )
    })?;

    ui_assets.cleanup().with_context(|| {
        format!(
            "book '{}' failed to clean transient bookshelf UI assets under {}",
            book.id,
            book.book_root_abs.display()
        )
    })
}

fn resolve_site_dest_dir(
    config_dir: &Path,
    projected_config: &Config,
    dest_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("failed to determine current working directory")?;

    match dest_dir {
        Some(dest_dir) => Ok(make_absolute(&cwd, dest_dir)),
        None => {
            let config_dir = make_absolute(&cwd, config_dir.to_path_buf());
            Ok(make_absolute(
                &config_dir,
                projected_config.build.build_dir.clone(),
            ))
        }
    }
}

fn make_absolute(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn absolutize_catalog_paths(mut catalog: InputCatalog) -> Result<InputCatalog> {
    let cwd = std::env::current_dir().context("failed to determine current working directory")?;
    catalog.config_path = make_absolute(&cwd, catalog.config_path);
    catalog.config_dir = make_absolute(&cwd, catalog.config_dir);

    for book in &mut catalog.books {
        absolutize_book_paths(book, &cwd);
    }

    Ok(catalog)
}

fn absolutize_book_paths(book: &mut InputBook, cwd: &Path) {
    book.book_root_abs = make_absolute(cwd, book.book_root_abs.clone());
    book.book_src_abs = make_absolute(cwd, book.book_src_abs.clone());
    book.summary_abs = make_absolute(cwd, book.summary_abs.clone());
}

fn build_book_breadcrumbs(
    catalog: &InputCatalog,
) -> Result<BTreeMap<String, Vec<BookshelfBreadcrumbPage>>> {
    let loaded =
        load_books_from_catalog(catalog).context("failed to load books for breadcrumb metadata")?;
    let site_model = build_site_model(catalog, &loaded)
        .context("failed to build site model for breadcrumb metadata")?;
    let navigation = build_navigation_metadata(&site_model)
        .context("failed to build navigation metadata for breadcrumb strings")?;
    let mut by_book = catalog
        .books
        .iter()
        .map(|book| (book.id.clone(), Vec::new()))
        .collect::<BTreeMap<_, _>>();

    for page in &site_model.pages {
        if page.kind != SitePageKind::Content {
            continue;
        }

        let html_path = page
            .source_path
            .as_ref()
            .map(|path| chapter_output_html_path(path))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "content page '{}' is missing a source path for breadcrumb output",
                    page.page_id
                )
            })?;
        let breadcrumb = navigation
            .for_page(&page.page_id)
            .and_then(|entry| entry.breadcrumb.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "content page '{}' is missing breadcrumb text in navigation metadata",
                    page.page_id
                )
            })?;

        by_book
            .entry(page.owning_book_id.clone())
            .or_default()
            .push(BookshelfBreadcrumbPage {
                html_path,
                breadcrumb,
            });
    }

    for breadcrumb_pages in by_book.values_mut() {
        breadcrumb_pages.sort();
    }

    Ok(by_book)
}

fn chapter_output_html_path(path: &Path) -> String {
    path.with_extension("html")
        .to_string_lossy()
        .replace('\\', "/")
}

fn write_site_root_index(
    catalog: &crate::catalog::InputCatalog,
    projected_config: &Config,
    site_dest_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(site_dest_dir).with_context(|| {
        format!(
            "failed to create site output directory {}",
            site_dest_dir.display()
        )
    })?;

    let index_path = site_dest_dir.join("index.html");
    fs::write(
        &index_path,
        render_site_root_index(catalog, projected_config)?,
    )
    .with_context(|| {
        format!(
            "failed to write site-root entry file at {}",
            index_path.display()
        )
    })
}

fn render_site_root_index(
    catalog: &crate::catalog::InputCatalog,
    projected_config: &Config,
) -> Result<String> {
    let lang = projected_config.book.language.as_deref().unwrap_or("en");
    let target = site_root_entry_path(catalog)?;

    Ok(format!(
        "<!DOCTYPE html>\n\
<html lang=\"{}\">\n\
<head>\n\
  <meta charset=\"utf-8\">\n\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
  <title>Redirecting</title>\n\
  <meta http-equiv=\"refresh\" content=\"0; url={}\">\n\
  <script>window.location.replace(\"{}\");</script>\n\
</head>\n\
<body>\n\
  <p><a href=\"{}\">Continue</a></p>\n\
</body>\n\
</html>\n",
        escape_html_attr(lang),
        escape_html_attr(&target),
        escape_html_attr(&target),
        escape_html_attr(&target),
    ))
}

fn site_root_entry_path(catalog: &InputCatalog) -> Result<String> {
    let root_output_rel = &catalog
        .root_book()
        .context("failed to resolve root book for site-root redirect")?
        .output_rel;

    match catalog.entry_page {
        BookshelfEntryPage::RootBook => Ok(path_to_string(&root_output_rel.join("index.html"))),
        BookshelfEntryPage::Bookshelf => Ok(site_root_bookshelf_entry_path(root_output_rel)),
    }
}

fn escape_html_attr(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
