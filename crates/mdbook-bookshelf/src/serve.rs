use crate::build::{build_bookshelf_site_with_options, BuildOptions};
use crate::catalog::build_input_catalog;
use anyhow::{Context, Result};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::fs::{self, FileType};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};

const LIVE_RELOAD_ENDPOINT: &str = "__livereload";
const WATCH_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub config_path: PathBuf,
    pub dest_dir: Option<PathBuf>,
    pub hostname: String,
    pub port: u16,
}

pub fn serve_bookshelf(options: ServeOptions) -> Result<()> {
    let site_root = build_bookshelf_site_for_serve(&options.config_path, options.dest_dir.clone())
        .with_context(|| {
            format!(
                "failed to build bookshelf site before serving {}",
                options.config_path.display()
            )
        })?;
    let (reload_tx, _reload_rx) = broadcast::channel::<String>(100);

    spawn_rebuild_watcher(
        options.config_path.clone(),
        Some(site_root.clone()),
        site_root.clone(),
        reload_tx.clone(),
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .context("failed to create tokio runtime for serve")?;

    runtime.block_on(run_server(
        site_root,
        options.hostname,
        options.port,
        reload_tx,
    ))
}

fn build_bookshelf_site_for_serve(
    config_path: &Path,
    dest_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    build_bookshelf_site_with_options(
        config_path,
        dest_dir,
        BuildOptions::with_live_reload_endpoint(LIVE_RELOAD_ENDPOINT),
    )
}

async fn run_server(
    site_root: PathBuf,
    hostname: String,
    port: u16,
    reload_tx: broadcast::Sender<String>,
) -> Result<()> {
    let bind_address = format!("{hostname}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("failed to bind HTTP listener at {bind_address}"))?;
    let local_addr = listener
        .local_addr()
        .context("failed to read bound HTTP listener address")?;

    eprintln!("Serving on: http://{local_addr}");

    axum::serve(listener, build_router(site_root, reload_tx))
        .await
        .context("serve loop exited unexpectedly")
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

        match build_bookshelf_site_for_serve(&config_path, dest_dir.clone()) {
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

        let mut roots = Vec::with_capacity(catalog.books.len() + 1);
        push_watch_root(
            &mut roots,
            normalize_path(&catalog.config_path),
            &self.output_dir,
        );

        for book in catalog.books {
            push_watch_root(
                &mut roots,
                normalize_path(&book.book_src_abs),
                &self.output_dir,
            );
        }

        self.root_paths = roots;
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
        if self.is_output_path(path) {
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
                self.scan_path(&entry.path(), path_data);
            }
            return;
        }

        let normalized = normalize_path(path);
        if self.is_output_path(&normalized) {
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

fn push_watch_root(roots: &mut Vec<PathBuf>, path: PathBuf, output_dir: &Path) {
    if path == output_dir || path.starts_with(output_dir) || roots.contains(&path) {
        return;
    }

    roots.push(path);
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
