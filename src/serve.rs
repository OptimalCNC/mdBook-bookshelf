use crate::build::build_bookshelf_site;
use anyhow::{Context, Result};
use axum::Router;
use std::path::PathBuf;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Debug, Clone)]
pub struct ServeOptions {
    pub config_path: PathBuf,
    pub dest_dir: Option<PathBuf>,
    pub hostname: String,
    pub port: u16,
}

pub fn serve_bookshelf(options: ServeOptions) -> Result<()> {
    let site_root = build_bookshelf_site(&options.config_path, options.dest_dir.clone())
        .with_context(|| {
            format!(
                "failed to build bookshelf site before serving {}",
                options.config_path.display()
            )
        })?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .context("failed to create tokio runtime for serve")?;

    runtime.block_on(run_server(site_root, options.hostname, options.port))
}

async fn run_server(site_root: PathBuf, hostname: String, port: u16) -> Result<()> {
    let bind_address = format!("{hostname}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("failed to bind HTTP listener at {bind_address}"))?;
    let local_addr = listener
        .local_addr()
        .context("failed to read bound HTTP listener address")?;

    eprintln!("Serving on: http://{local_addr}");

    axum::serve(listener, build_router(site_root))
        .await
        .context("serve loop exited unexpectedly")
}

fn build_router(site_root: PathBuf) -> Router {
    let site_404 = site_root.join("404.html");

    if site_404.is_file() {
        Router::new()
            .fallback_service(ServeDir::new(site_root).not_found_service(ServeFile::new(site_404)))
    } else {
        Router::new().fallback_service(ServeDir::new(site_root))
    }
}
