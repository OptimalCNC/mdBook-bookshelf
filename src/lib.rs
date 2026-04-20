use std::fmt;
use std::path::{Path, PathBuf};

pub mod config;
pub mod input_catalog;
pub mod reader_context;
pub mod render_manifest;
pub mod renderer;
pub mod search;
pub mod serve;
pub mod sidebar;
pub mod site_model;

pub use config::{BookConfig, BookshelfConfig};
pub use input_catalog::{load_input_catalog, InputBook, InputCatalog, LoadInputCatalogError};
pub use reader_context::{
    build_reader_context_model, ActiveBookContext, AdjacentPageLink, AuthoredPageReaderContext,
    BookshelfPageReaderContext, BookshelfReturn, Breadcrumbs, BuildReaderContextError,
    ReaderContextModel,
};
pub use render_manifest::{
    build_render_manifest, BuildRenderManifestError, RenderManifest, RenderedPageIdentity,
    RenderedPageManifestEntry,
};
pub use renderer::{
    render_bookshelf_root_page, render_site, render_site_with_search_index,
    RenderBookshelfRootError, RenderSiteError,
};
pub use search::{
    build_search_index, write_search_index, SearchDocument, SearchIndex, SearchIndexError,
};
pub use serve::{serve_site, start_site_server, ServeSiteError, SiteServer};
pub use sidebar::{
    build_sidebar_model, BookSidebar, SidebarAffixEntry, SidebarChapter, SidebarModel,
};
pub use site_model::{
    build_site_model, AuthoredPage, BookshelfPage, BuildSiteModelError, ShelfItem, SiteBook,
    SiteModel, SiteRoot,
};

#[derive(Debug)]
pub enum BuildSiteCommandError {
    LoadInputCatalog(LoadInputCatalogError),
    BuildSiteModel(BuildSiteModelError),
    BuildReaderContext(BuildReaderContextError),
    BuildRenderManifest(BuildRenderManifestError),
    RenderSite(RenderSiteError),
}

impl fmt::Display for BuildSiteCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LoadInputCatalog(error) => error.fmt(f),
            Self::BuildSiteModel(error) => error.fmt(f),
            Self::BuildReaderContext(error) => error.fmt(f),
            Self::BuildRenderManifest(error) => error.fmt(f),
            Self::RenderSite(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for BuildSiteCommandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LoadInputCatalog(error) => Some(error),
            Self::BuildSiteModel(error) => Some(error),
            Self::BuildReaderContext(error) => Some(error),
            Self::BuildRenderManifest(error) => Some(error),
            Self::RenderSite(error) => Some(error),
        }
    }
}

impl From<LoadInputCatalogError> for BuildSiteCommandError {
    fn from(value: LoadInputCatalogError) -> Self {
        Self::LoadInputCatalog(value)
    }
}

impl From<BuildSiteModelError> for BuildSiteCommandError {
    fn from(value: BuildSiteModelError) -> Self {
        Self::BuildSiteModel(value)
    }
}

impl From<BuildReaderContextError> for BuildSiteCommandError {
    fn from(value: BuildReaderContextError) -> Self {
        Self::BuildReaderContext(value)
    }
}

impl From<BuildRenderManifestError> for BuildSiteCommandError {
    fn from(value: BuildRenderManifestError) -> Self {
        Self::BuildRenderManifest(value)
    }
}

impl From<RenderSiteError> for BuildSiteCommandError {
    fn from(value: RenderSiteError) -> Self {
        Self::RenderSite(value)
    }
}

pub fn build_site(
    config_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, BuildSiteCommandError> {
    let config_path = config_path.as_ref();
    let normalized_config_path = if config_path.exists() {
        config_path
            .canonicalize()
            .unwrap_or_else(|_| config_path.to_path_buf())
    } else {
        config_path.to_path_buf()
    };

    let input_catalog = load_input_catalog(&normalized_config_path)?;
    let site_model = build_site_model(&input_catalog)?;
    let sidebar_model = build_sidebar_model(&site_model);
    let reader_context = build_reader_context_model(&site_model, &sidebar_model)?;
    let render_manifest = build_render_manifest(&site_model, &reader_context)?;
    let written_paths = render_site_with_search_index(
        output_dir,
        &site_model,
        &sidebar_model,
        &reader_context,
        &render_manifest,
    )?;

    Ok(written_paths)
}
