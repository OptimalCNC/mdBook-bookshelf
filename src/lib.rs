pub mod config;
pub mod input_catalog;
pub mod reader_context;
pub mod render_manifest;
pub mod renderer;
pub mod search;
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
pub use sidebar::{
    build_sidebar_model, BookSidebar, SidebarAffixEntry, SidebarChapter, SidebarModel,
};
pub use site_model::{
    build_site_model, AuthoredPage, BookshelfPage, BuildSiteModelError, ShelfItem, SiteBook,
    SiteModel, SiteRoot,
};
