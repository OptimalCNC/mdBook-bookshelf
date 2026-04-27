use crate::bookshelf_ui::{BookshelfAssets, BookshelfPageMetadataPreprocessor};
use crate::catalog::{build_input_catalog, InputBook, InputCatalog};
use crate::config::BookshelfEntryPage;
use crate::load_single_book_with_config_and_parsed_summary;
use crate::root_bookshelf_preprocessor::{
    ensure_reserved_bookshelf_path_is_available, inject_root_bookshelf_page,
    site_root_bookshelf_entry_path,
};
use crate::route_paths::{path_to_string, relative_path};
use crate::search::{write_site_wide_search_index, LOCAL_SHARED_SEARCH_INDEX_NAME};
use crate::site_root_link_preprocessor::{SiteRootLinkMap, SiteRootLinkPreprocessor};
use anyhow::{Context, Result};
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
    if options.log_layout {
        progress.summary(&catalog, &site_dest_dir);
    }

    let config_root = catalog.config_dir.clone();
    let site_root_link_map = SiteRootLinkMap::from_catalog_with_progress(&catalog, |_, _, _| {})
        .context("failed to build site-root Markdown link map")?;
    let root_bookshelf_rel = PathBuf::from(site_root_bookshelf_entry_path(
        &catalog
            .root_book()
            .context("failed to resolve root book for synthetic bookshelf routing")?
            .output_rel,
    ));
    let bookshelf_assets = BookshelfAssets::new(&config_root, &catalog.asset_dir);

    let total_books = catalog.books.len();
    for (index, book) in catalog.books.iter().enumerate() {
        let book_started = Instant::now();
        build_catalog_book(
            book,
            &catalog,
            &mdbook_config,
            &config_root,
            &site_dest_dir,
            &root_bookshelf_rel,
            &site_root_link_map,
            &bookshelf_assets,
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
        progress.book_finished(index + 1, total_books, book, book_started.elapsed());
    }

    let search_started = Instant::now();
    write_site_wide_search_index(&catalog, &site_dest_dir)
        .context("failed to write bookshelf shared search output")?;
    progress.search_index_written(search_started.elapsed());

    write_site_root_index(&catalog, &mdbook_config, &site_dest_dir)
        .context("failed to write bookshelf site-root redirect")?;
    progress.finish(&site_dest_dir, build_started.elapsed());

    Ok(site_dest_dir)
}

fn build_catalog_book(
    book: &InputBook,
    catalog: &InputCatalog,
    shared_config: &Config,
    config_root: &Path,
    site_dest_dir: &Path,
    root_bookshelf_rel: &Path,
    site_root_link_map: &SiteRootLinkMap,
    bookshelf_assets: &BookshelfAssets,
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

    bookshelf_assets
        .inject_bookshelf_runtime_assets(&mut config)
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
    mdbook.with_preprocessor(BookshelfPageMetadataPreprocessor::new(
        book.output_rel.clone(),
        root_bookshelf_rel.to_path_buf(),
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
    if book.is_root_book {
        patch_root_bookshelf_toc_index_alias(&html_build_dir).with_context(|| {
            format!(
                "book '{}' failed to patch root bookshelf sidebar script under {}",
                book.id,
                html_build_dir.display()
            )
        })?;
    }

    Ok(())
}

fn patch_root_bookshelf_toc_index_alias(book_output_dir: &Path) -> Result<()> {
    let mut patched = 0usize;

    for entry in fs::read_dir(book_output_dir).with_context(|| {
        format!(
            "failed to read root book output directory {}",
            book_output_dir.display()
        )
    })? {
        let entry = entry.with_context(|| {
            format!(
                "failed to read an entry in root book output directory {}",
                book_output_dir.display()
            )
        })?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !(file_name.starts_with("toc-") && file_name.ends_with(".js")) {
            continue;
        }

        let source = fs::read_to_string(&path).with_context(|| {
            format!("failed to read root book sidebar script {}", path.display())
        })?;
        let patched_source = patch_root_bookshelf_toc_index_alias_source(&source)?;
        fs::write(&path, patched_source).with_context(|| {
            format!(
                "failed to write root book sidebar script {}",
                path.display()
            )
        })?;
        patched += 1;
    }

    if patched == 0 {
        anyhow::bail!(
            "failed to find generated toc-*.js in root book output directory {}",
            book_output_dir.display()
        );
    }

    Ok(())
}

fn patch_root_bookshelf_toc_index_alias_source(source: &str) -> Result<String> {
    // mdBook treats the first sidebar link as an index.html alias. Once the
    // synthetic Bookshelf link is first, that alias must stay with the real
    // root index page so page headings attach to the authored chapter.
    let mdbook_index_alias = "|| i === 0\n                && path_to_root === ''\n                && current_page.endsWith('/index.html')";
    let bookshelf_aware_index_alias = "|| i === 0\n                && href !== \"bookshelf.html\"\n                && path_to_root === ''\n                && current_page.endsWith('/index.html')";

    if source.contains(bookshelf_aware_index_alias) {
        return Ok(source.to_string());
    }

    if !source.contains(mdbook_index_alias) {
        anyhow::bail!("generated toc script is missing mdBook's first-chapter index alias");
    }

    Ok(source.replacen(mdbook_index_alias, bookshelf_aware_index_alias, 1))
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

    fn book_finished(&self, index: usize, total: usize, book: &InputBook, elapsed: Duration) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "  [{index}/{total}] {}: {} ({})",
            self.style.source_title(&book.title, book.is_root_book),
            self.format_path(&book.book_src_abs),
            format_duration(elapsed)
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
