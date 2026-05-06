use crate::catalog::{build_input_catalog, InputBook, InputCatalog};
use crate::documentation_index::{validate_documentation_index_inputs, write_documentation_index};
use crate::documentation_ui::{DocumentationAssets, DocumentationPageMetadataPreprocessor};
use crate::load_single_book_with_config_and_parsed_summary;
use crate::route_paths::{path_to_string, relative_path};
use crate::search::{write_site_wide_search_index, LOCAL_SHARED_SEARCH_INDEX_NAME};
use crate::site_root_link_preprocessor::{SiteRootLinkMap, SiteRootLinkPreprocessor};
use anyhow::{Context, Result};
use mdbook_driver::book::Book;
use mdbook_driver::config::Config;
use mdbook_summary::parse_summary;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn build_bookshelf(config_path: impl AsRef<Path>, dest_dir: Option<PathBuf>) -> Result<()> {
    build_bookshelf_site_with_options(
        config_path,
        dest_dir,
        BuildOptions::default()
            .with_layout_logging()
            .with_progress_logging(),
    )
    .map(|_| ())
}

pub fn build_bookshelf_site(
    config_path: impl AsRef<Path>,
    dest_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    build_bookshelf_site_with_options(config_path, dest_dir, BuildOptions::default())
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BuildOptions {
    live_reload_endpoint: Option<String>,
    log_layout: bool,
    log_progress: bool,
}

impl BuildOptions {
    pub(crate) fn with_live_reload_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            live_reload_endpoint: Some(endpoint.into()),
            ..Self::default()
        }
    }

    pub(crate) fn with_layout_logging(mut self) -> Self {
        self.log_layout = true;
        self
    }

    pub(crate) fn with_progress_logging(mut self) -> Self {
        self.log_progress = true;
        self
    }

    fn apply_to_config(&self, config: &mut Config) -> Result<()> {
        if let Some(endpoint) = &self.live_reload_endpoint {
            config
                .set("output.html.live-reload-endpoint", endpoint)
                .context("failed to set serve live-reload endpoint in mdBook config")?;
        }

        Ok(())
    }
}

pub(crate) fn build_bookshelf_site_with_options(
    config_path: impl AsRef<Path>,
    dest_dir: Option<PathBuf>,
    options: BuildOptions,
) -> Result<PathBuf> {
    let config_path = config_path.as_ref();
    let progress = BuildProgress::new(options.log_progress);
    let build_started = Instant::now();

    progress.start(config_path);
    let catalog = absolutize_catalog_paths(build_input_catalog(config_path)?)
        .context("failed to resolve bookshelf catalog paths")?;

    let mut mdbook_config = catalog.mdbook_config.clone();
    options.apply_to_config(&mut mdbook_config)?;
    let site_dest_dir = resolve_site_dest_dir(&catalog.config_dir, &mdbook_config, dest_dir)?;
    validate_documentation_index_inputs(&catalog).context("invalid documentation index inputs")?;
    if options.log_layout {
        progress.summary(&catalog, &site_dest_dir);
    }

    let config_root = catalog.config_dir.clone();
    let site_root_link_map = SiteRootLinkMap::from_catalog_with_progress(&catalog, |_, _, _| {})
        .context("failed to build site-root Markdown link map")?;
    let documentation_index_rel = PathBuf::from("index.html");
    let documentation_assets = DocumentationAssets::new(&config_root, &catalog.asset_dir);

    let total_books = catalog.books.len();
    for (index, book) in catalog.books.iter().enumerate() {
        let book_started = Instant::now();
        let page_count = build_catalog_book(
            book,
            &mdbook_config,
            &config_root,
            &site_dest_dir,
            &documentation_index_rel,
            &site_root_link_map,
            &documentation_assets,
        )
        .with_context(|| {
            format!(
                "failed while building book {}/{} '{}' from {}",
                index + 1,
                total_books,
                book.title,
                book.book_src_abs.display()
            )
        })?;
        progress.book_finished(
            index + 1,
            total_books,
            book,
            book_started.elapsed(),
            page_count,
        );
    }

    let index_started = Instant::now();
    write_documentation_index(&catalog, &mdbook_config, &site_dest_dir)
        .context("failed to write documentation index")?;
    progress.documentation_index_written(index_started.elapsed());

    let search_started = Instant::now();
    write_site_wide_search_index(&catalog, &site_dest_dir)
        .context("failed to write bookshelf shared search output")?;
    progress.search_index_written(search_started.elapsed());

    progress.finish(&site_dest_dir, build_started.elapsed());

    Ok(site_dest_dir)
}

fn build_catalog_book(
    book: &InputBook,
    shared_config: &Config,
    config_root: &Path,
    site_dest_dir: &Path,
    documentation_index_rel: &Path,
    site_root_link_map: &SiteRootLinkMap,
    documentation_assets: &DocumentationAssets,
) -> Result<usize> {
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

    documentation_assets
        .inject_documentation_runtime_assets(&mut config)
        .with_context(|| {
            format!(
                "book '{}' failed to inject documentation UI assets under {}",
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
    let page_count = count_book_pages(&mdbook.book);

    mdbook.with_preprocessor(SiteRootLinkPreprocessor::new(
        book.output_rel.clone(),
        site_root_link_map.clone(),
    ));
    mdbook.with_preprocessor(DocumentationPageMetadataPreprocessor::new(
        book.output_rel.clone(),
        documentation_index_rel.to_path_buf(),
        book.output_rel.join(LOCAL_SHARED_SEARCH_INDEX_NAME),
    ));

    let html_build_dir = mdbook.build_dir_for("html");
    mdbook.build().with_context(|| {
        format!(
            "book '{}' failed to build mdbook output at {}",
            book.id,
            html_build_dir.display()
        )
    })?;

    Ok(page_count)
}

fn count_book_pages(book: &Book) -> usize {
    book.chapters().count()
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

struct BuildProgress {
    enabled: bool,
    style: LogStyle,
    cwd: Option<PathBuf>,
}

impl BuildProgress {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            style: LogStyle::stderr(),
            cwd: std::env::current_dir().ok(),
        }
    }

    fn start(&self, config_path: &Path) {
        if !self.enabled {
            return;
        }

        eprintln!("{}", self.style.heading("Build"));
        eprintln!(
            "  {} {}",
            self.style.label("config:"),
            self.format_path(config_path)
        );
    }

    fn summary(&self, catalog: &InputCatalog, site_dest_dir: &Path) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "  {} {}",
            self.style.label("root:"),
            self.format_path(&catalog.config_dir)
        );
        eprintln!(
            "  {} {}",
            self.style.label("output:"),
            self.format_output_path(site_dest_dir)
        );
        eprintln!("  {} {}", self.style.label("books:"), catalog.books.len());
    }

    fn book_finished(
        &self,
        index: usize,
        total: usize,
        book: &InputBook,
        elapsed: Duration,
        page_count: usize,
    ) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "  [{index}/{total}] {}: {} ({}, {})",
            self.style.source_title(&book.title, book.is_root_book),
            self.format_path(&book.book_src_abs),
            format_duration(elapsed),
            format_page_count(page_count)
        );
    }

    fn search_index_written(&self, elapsed: Duration) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "  {} {}",
            self.style.label("search index:"),
            format_duration(elapsed)
        );
    }

    fn documentation_index_written(&self, elapsed: Duration) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "  {} {}",
            self.style.label("documentation index:"),
            format_duration(elapsed)
        );
    }

    fn finish(&self, site_dest_dir: &Path, elapsed: Duration) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "  {} {} ({})",
            self.style.success("Finished:"),
            self.format_output_path(site_dest_dir),
            format_duration(elapsed)
        );
    }

    fn format_path(&self, path: &Path) -> String {
        if !path.is_absolute() {
            return path_to_string(path);
        }

        self.cwd
            .as_ref()
            .map(|cwd| log_path(path, cwd))
            .unwrap_or_else(|| path.display().to_string())
    }

    fn format_output_path(&self, path: &Path) -> String {
        if !path.is_absolute() {
            return path_to_string(path);
        }

        let Some(cwd) = &self.cwd else {
            return path.display().to_string();
        };
        let path = normalize_log_path(path);
        let cwd = normalize_log_path(cwd);

        match path.strip_prefix(&cwd) {
            Ok(relative) => path_to_string(relative),
            Err(_) => path_to_string(&path),
        }
    }
}

struct LogStyle {
    enabled: bool,
}

impl LogStyle {
    fn stderr() -> Self {
        Self {
            enabled: std::io::stderr().is_terminal(),
        }
    }

    fn heading(&self, text: &str) -> String {
        self.paint("1", text)
    }

    fn label(&self, text: &str) -> String {
        self.paint("36", text)
    }

    fn source_title(&self, title: &str, is_root_book: bool) -> String {
        if is_root_book {
            self.paint("1;32", title)
        } else {
            self.paint("36", title)
        }
    }

    fn success(&self, text: &str) -> String {
        self.paint("32", text)
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

fn log_path(path: &Path, cwd: &Path) -> String {
    path_to_string(&relative_path(cwd, path))
}

fn normalize_log_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

pub(crate) fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1_000 {
        return format!("{millis}ms");
    }

    let seconds = duration.as_secs_f64();
    if seconds < 10.0 {
        format!("{seconds:.1}s")
    } else {
        format!("{}s", duration.as_secs())
    }
}

fn format_page_count(page_count: usize) -> String {
    match page_count {
        1 => "1 page".to_string(),
        count => format!("{count} pages"),
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
