use crate::build::{build_bookshelf_site_with_options, BuildOptions};
use crate::catalog::{build_input_catalog, InputCatalog};
use anyhow::{bail, Context, Result};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::fs::{self, FileType};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};

const LIVE_RELOAD_ENDPOINT: &str = "__livereload";
const WATCH_POLL_INTERVAL: Duration = Duration::from_secs(1);
const GENERATED_BOOKSHELF_ASSET_DIR: &str = ".mdbook-bookshelf";
const DEFAULT_SERVE_PORT_START: u16 = 3000;
const DEFAULT_SERVE_PORT_END: u16 = 3100;

#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub config_path: PathBuf,
    pub dest_dir: Option<PathBuf>,
    pub hostname: String,
    pub port: Option<u16>,
}

pub fn serve_bookshelf(options: ServeOptions) -> Result<()> {
    let site_root =
        build_bookshelf_site_for_serve(&options.config_path, options.dest_dir.clone(), true)
            .with_context(|| {
                format!(
                    "failed to build bookshelf site before serving {}",
                    options.config_path.display()
                )
            })?;
    let (reload_tx, _reload_rx) = broadcast::channel::<String>(100);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .context("failed to create tokio runtime for serve")?;

    runtime.block_on(run_server(
        site_root,
        options.config_path,
        options.hostname,
        options.port,
        reload_tx,
    ))
}

fn build_bookshelf_site_for_serve(
    config_path: &Path,
    dest_dir: Option<PathBuf>,
    log_layout: bool,
) -> Result<PathBuf> {
    let mut options = BuildOptions::with_live_reload_endpoint(LIVE_RELOAD_ENDPOINT);
    if log_layout {
        options = options.with_layout_logging();
    }

    build_bookshelf_site_with_options(config_path, dest_dir, options)
}

async fn run_server(
    site_root: PathBuf,
    config_path: PathBuf,
    hostname: String,
    port: Option<u16>,
    reload_tx: broadcast::Sender<String>,
) -> Result<()> {
    let listener = bind_http_listener(&hostname, port).await?;
    let local_addr = listener
        .local_addr()
        .context("failed to read bound HTTP listener address")?;

    eprintln!("Serving on: http://{local_addr}");

    spawn_rebuild_watcher(
        config_path,
        Some(site_root.clone()),
        site_root.clone(),
        reload_tx.clone(),
    );

    axum::serve(listener, build_router(site_root, reload_tx))
        .await
        .context("serve loop exited unexpectedly")
}

async fn bind_http_listener(hostname: &str, port: Option<u16>) -> Result<tokio::net::TcpListener> {
    match port {
        Some(port) => bind_http_listener_at(hostname, port).await,
        None => bind_first_available_http_listener(hostname).await,
    }
}

async fn bind_http_listener_at(hostname: &str, port: u16) -> Result<tokio::net::TcpListener> {
    let bind_address = format!("{hostname}:{port}");

    tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("failed to bind HTTP listener at {bind_address}"))
}

async fn bind_first_available_http_listener(hostname: &str) -> Result<tokio::net::TcpListener> {
    let mut last_addr_in_use = None;

    for port in DEFAULT_SERVE_PORT_START..=DEFAULT_SERVE_PORT_END {
        let bind_address = format!("{hostname}:{port}");

        match tokio::net::TcpListener::bind(&bind_address).await {
            Ok(listener) => return Ok(listener),
            Err(err) if err.kind() == ErrorKind::AddrInUse => {
                last_addr_in_use = Some(err);
            }
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to bind HTTP listener at {bind_address}"));
            }
        }
    }

    if let Some(err) = last_addr_in_use {
        return Err(err).with_context(|| {
            format!(
                "failed to bind HTTP listener on {hostname} using ports {DEFAULT_SERVE_PORT_START}-{DEFAULT_SERVE_PORT_END}"
            )
        });
    }

    bail!("failed to bind HTTP listener: no default ports configured for {hostname}")
}

fn build_router(site_root: PathBuf, reload_tx: broadcast::Sender<String>) -> Router {
    let site_404 = site_root.join("404.html");
    let websocket_handler = {
        let reload_tx = reload_tx.clone();
        move |ws: WebSocketUpgrade| {
            let reload_tx = reload_tx.clone();
            async move { ws.on_upgrade(move |socket| websocket_connection(socket, reload_tx)) }
        }
    };

    if site_404.is_file() {
        Router::new()
            .route(&format!("/{LIVE_RELOAD_ENDPOINT}"), get(websocket_handler))
            .fallback_service(ServeDir::new(site_root).not_found_service(ServeFile::new(site_404)))
    } else {
        Router::new()
            .route(&format!("/{LIVE_RELOAD_ENDPOINT}"), get(websocket_handler))
            .fallback_service(ServeDir::new(site_root))
    }
}

async fn websocket_connection(mut socket: WebSocket, reload_tx: broadcast::Sender<String>) {
    let mut rx = reload_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(message) => {
                let _ = socket.send(Message::Text(message.into())).await;
                break;
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

fn spawn_rebuild_watcher(
    config_path: PathBuf,
    dest_dir: Option<PathBuf>,
    output_dir: PathBuf,
    reload_tx: broadcast::Sender<String>,
) {
    thread::spawn(move || {
        watch_for_rebuilds(config_path, dest_dir, output_dir, reload_tx);
    });
}

fn watch_for_rebuilds(
    config_path: PathBuf,
    dest_dir: Option<PathBuf>,
    output_dir: PathBuf,
    reload_tx: broadcast::Sender<String>,
) {
    let mut watcher = PollWatcher::new(output_dir);

    match watcher.set_roots_from_config(&config_path) {
        Ok(()) => {
            watcher.scan();
            eprintln!("Watching for changes...");
        }
        Err(err) => {
            eprintln!(
                "failed to initialize filesystem watcher for {}: {err:?}",
                config_path.display()
            );
        }
    }

    loop {
        thread::sleep(WATCH_POLL_INTERVAL);

        let changed_paths = watcher.scan();
        if changed_paths.is_empty() {
            continue;
        }

        eprintln!("Files changed: {changed_paths:?}");

        match build_bookshelf_site_for_serve(&config_path, dest_dir.clone(), false) {
            Ok(_) => {
                if let Err(err) = watcher.set_roots_from_config(&config_path) {
                    eprintln!(
                        "failed to refresh filesystem watcher roots for {}: {err:?}",
                        config_path.display()
                    );
                } else {
                    watcher.scan();
                }

                let _ = reload_tx.send("reload".to_string());
            }
            Err(err) => {
                eprintln!("failed to rebuild bookshelf site after change: {err:?}");
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct PathData {
    file_type: FileType,
    modified: SystemTime,
    len: u64,
}

#[derive(Debug)]
struct PollWatcher {
    root_paths: Vec<PathBuf>,
    path_data: HashMap<PathBuf, PathData>,
    output_dir: PathBuf,
}

impl PollWatcher {
    fn new(output_dir: PathBuf) -> Self {
        Self {
            root_paths: Vec::new(),
            path_data: HashMap::new(),
            output_dir: normalize_path(&output_dir),
        }
    }

    fn set_roots_from_config(&mut self, config_path: &Path) -> Result<()> {
        let catalog = build_input_catalog(config_path).with_context(|| {
            format!("failed to load watch roots from {}", config_path.display())
        })?;

        self.root_paths = collect_watch_roots(&catalog, &self.output_dir);
        Ok(())
    }

    fn scan(&mut self) -> Vec<PathBuf> {
        let mut new_path_data = HashMap::new();

        for root in &self.root_paths {
            self.scan_path(root, &mut new_path_data);
        }

        let mut changed_paths = Vec::new();
        for (path, new_data) in &new_path_data {
            match self.path_data.get(path) {
                Some(old_data) if old_data == new_data => {}
                _ => changed_paths.push(path.clone()),
            }
        }

        for old_path in self.path_data.keys() {
            if !new_path_data.contains_key(old_path) {
                changed_paths.push(old_path.clone());
            }
        }

        self.path_data = new_path_data;
        changed_paths
    }

    fn scan_path(&self, path: &Path, path_data: &mut HashMap<PathBuf, PathData>) {
        if self.is_output_path(path) || is_generated_bookshelf_asset_path(path) {
            return;
        }

        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return,
        };

        if metadata.is_dir() {
            let entries = match fs::read_dir(path) {
                Ok(entries) => entries,
                Err(_) => return,
            };

            for entry in entries.flatten() {
                let entry_path = entry.path();
                if is_generated_bookshelf_asset_path(&entry_path) {
                    continue;
                }
                self.scan_path(&entry_path, path_data);
            }
            return;
        }

        let normalized = normalize_path(path);
        if self.is_output_path(&normalized) || is_generated_bookshelf_asset_path(&normalized) {
            return;
        }

        path_data.insert(
            normalized,
            PathData {
                file_type: metadata.file_type(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                len: metadata.len(),
            },
        );
    }

    fn is_output_path(&self, path: &Path) -> bool {
        let normalized = normalize_path(path);
        normalized == self.output_dir || normalized.starts_with(&self.output_dir)
    }
}

fn collect_watch_roots(catalog: &InputCatalog, output_dir: &Path) -> Vec<PathBuf> {
    let html_config = catalog.mdbook_config.html_config();
    let mut roots = Vec::new();

    push_watch_root(&mut roots, catalog.config_path.clone(), output_dir);

    for book in &catalog.books {
        push_watch_root(&mut roots, book.book_src_abs.clone(), output_dir);
    }

    if let Some(theme) = html_config
        .as_ref()
        .and_then(|config| config.theme.as_ref())
    {
        push_watch_root(
            &mut roots,
            resolve_config_dir_path(&catalog.config_dir, theme),
            output_dir,
        );
    } else {
        for book in &catalog.books {
            push_watch_root(&mut roots, book.book_root_abs.join("theme"), output_dir);
        }
    }

    for extra_dir in &catalog.mdbook_config.build.extra_watch_dirs {
        push_watch_root(
            &mut roots,
            resolve_config_dir_path(&catalog.config_dir, extra_dir),
            output_dir,
        );
    }

    if let Some(html_config) = html_config {
        for asset in html_config
            .additional_css
            .iter()
            .chain(html_config.additional_js.iter())
        {
            push_watch_root(
                &mut roots,
                resolve_config_dir_path(&catalog.config_dir, asset),
                output_dir,
            );
        }
    }

    roots
}

fn resolve_config_dir_path(config_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    }
}

fn push_watch_root(roots: &mut Vec<PathBuf>, path: PathBuf, output_dir: &Path) {
    let path = normalize_path(&path);

    if path == output_dir || path.starts_with(output_dir) || roots.contains(&path) {
        return;
    }

    roots.push(path);
}

fn is_generated_bookshelf_asset_path(path: &Path) -> bool {
    path.components().any(|component| {
        component.as_os_str() == std::ffi::OsStr::new(GENERATED_BOOKSHELF_ASSET_DIR)
    })
}

fn normalize_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    absolute.canonicalize().unwrap_or(absolute)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn watch_roots_include_shared_config_stock_categories() {
        let fixture_root = make_temp_dir("watch-roots-shared-categories");
        let absolute_script = fixture_root.join("absolute-script.js");
        write_minimal_bookshelf_fixture(
            &fixture_root,
            &format!(
                r#"
[build]
extra-watch-dirs = ["shared/watch"]

[output.html]
theme = "shared/theme"
additional-css = ["assets/site.css"]
additional-js = ["scripts/site.js", "{}"]
"#,
                toml_path(&absolute_script)
            ),
        );
        fs::create_dir_all(fixture_root.join("shared/watch"))
            .expect("extra watch dir should be created");
        fs::create_dir_all(fixture_root.join("shared/theme")).expect("theme dir should be created");
        fs::create_dir_all(fixture_root.join("assets")).expect("asset dir should be created");
        fs::create_dir_all(fixture_root.join("scripts")).expect("script dir should be created");
        fs::write(fixture_root.join("assets/site.css"), "body {}\n")
            .expect("css asset should be written");
        fs::write(
            fixture_root.join("scripts/site.js"),
            "console.log('shared');\n",
        )
        .expect("js asset should be written");
        fs::write(&absolute_script, "console.log('absolute');\n")
            .expect("absolute js asset should be written");

        let output_dir = normalize_path(&fixture_root.join("book"));
        let mut watcher = PollWatcher::new(output_dir);
        watcher
            .set_roots_from_config(&fixture_root.join("bookshelf.toml"))
            .expect("watch roots should be discovered");

        assert_eq!(
            normalized_paths(&[
                fixture_root.join("bookshelf.toml"),
                fixture_root.join("docs"),
                fixture_root.join("modules/child/docs"),
                fixture_root.join("shared/theme"),
                fixture_root.join("shared/watch"),
                fixture_root.join("assets/site.css"),
                fixture_root.join("scripts/site.js"),
                absolute_script,
            ]),
            watcher.root_paths
        );

        fs::remove_dir_all(fixture_root).expect("fixture root should be removed");
    }

    #[test]
    fn watch_roots_include_default_book_theme_dirs_without_shared_theme() {
        let fixture_root = make_temp_dir("watch-roots-default-themes");
        write_minimal_bookshelf_fixture(&fixture_root, "");
        fs::create_dir_all(fixture_root.join("theme")).expect("root theme should be created");
        fs::create_dir_all(fixture_root.join("modules/child/theme"))
            .expect("child theme should be created");

        let output_dir = normalize_path(&fixture_root.join("book"));
        let mut watcher = PollWatcher::new(output_dir);
        watcher
            .set_roots_from_config(&fixture_root.join("bookshelf.toml"))
            .expect("watch roots should be discovered");

        assert_eq!(
            normalized_paths(&[
                fixture_root.join("bookshelf.toml"),
                fixture_root.join("docs"),
                fixture_root.join("modules/child/docs"),
                fixture_root.join("theme"),
                fixture_root.join("modules/child/theme"),
            ]),
            watcher.root_paths
        );

        fs::remove_dir_all(fixture_root).expect("fixture root should be removed");
    }

    #[test]
    fn watch_roots_exclude_output_and_generated_bookshelf_asset_trees() {
        let fixture_root = make_temp_dir("watch-roots-generated-exclusion");
        write_minimal_bookshelf_fixture(
            &fixture_root,
            r#"
[build]
extra-watch-dirs = ["."]
"#,
        );
        let output_dir = fixture_root.join("book");
        fs::create_dir_all(&output_dir).expect("output dir should be created");

        let mut watcher = PollWatcher::new(output_dir.clone());
        watcher
            .set_roots_from_config(&fixture_root.join("bookshelf.toml"))
            .expect("watch roots should be discovered");
        watcher.scan();

        let normal_changed = fixture_root.join("shared/changed.txt");
        fs::create_dir_all(
            normal_changed
                .parent()
                .expect("normal path should have parent"),
        )
        .expect("normal change parent should be created");
        fs::write(&normal_changed, "changed\n").expect("normal change should be written");

        let generated_changed = fixture_root
            .join(GENERATED_BOOKSHELF_ASSET_DIR)
            .join("build-test/shared/generated.css");
        fs::create_dir_all(
            generated_changed
                .parent()
                .expect("generated path should have parent"),
        )
        .expect("generated change parent should be created");
        fs::write(&generated_changed, "generated\n").expect("generated change should be written");

        let output_changed = output_dir.join("generated.html");
        fs::write(&output_changed, "<p>generated</p>\n").expect("output change should be written");

        let changed_paths = watcher.scan();

        assert!(
            changed_paths.contains(&normalize_path(&normal_changed)),
            "ordinary changes under broad roots should be detected: {changed_paths:?}"
        );
        assert!(
            !changed_paths
                .iter()
                .any(|path| path.starts_with(normalize_path(&generated_changed))),
            "generated bookshelf asset changes should be excluded: {changed_paths:?}"
        );
        assert!(
            !changed_paths
                .iter()
                .any(|path| path.starts_with(normalize_path(&output_changed))),
            "serve output changes should be excluded: {changed_paths:?}"
        );

        fs::remove_dir_all(fixture_root).expect("fixture root should be removed");
    }

    fn write_minimal_bookshelf_fixture(fixture_root: &Path, extra_config: &str) {
        fs::create_dir_all(fixture_root.join("docs")).expect("root docs should be created");
        fs::create_dir_all(fixture_root.join("modules/child/docs"))
            .expect("child docs should be created");
        fs::write(
            fixture_root.join("docs/SUMMARY.md"),
            "# Summary\n\n- [Root](index.md)\n",
        )
        .expect("root summary should be written");
        fs::write(fixture_root.join("docs/index.md"), "# Root\n")
            .expect("root index should be written");
        fs::write(
            fixture_root.join("modules/child/docs/SUMMARY.md"),
            "# Summary\n\n- [Child](index.md)\n",
        )
        .expect("child summary should be written");
        fs::write(
            fixture_root.join("modules/child/docs/index.md"),
            "# Child\n",
        )
        .expect("child index should be written");
        fs::write(
            fixture_root.join("bookshelf.toml"),
            format!(
                r#"[book]
title = "Root"
src = "docs"
language = "en"

{extra_config}
[[bookshelf.book]]
title = "Child"
src = "modules/child/docs"
"#
            ),
        )
        .expect("bookshelf config should be written");
    }

    fn normalized_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
        paths.iter().map(|path| normalize_path(path)).collect()
    }

    fn make_temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir()
            .join("mdbook-bookshelf")
            .join(format!("{tag}-{nanos}"));
        fs::create_dir_all(&dir).expect("temp directory should be created");
        dir
    }

    fn toml_path(path: &Path) -> String {
        path.to_string_lossy().replace('\\', "\\\\")
    }
}
