use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::build_site;
use crate::BuildSiteCommandError;

#[derive(Debug)]
pub enum ServeSiteError {
    Build(BuildSiteCommandError),
    Bind {
        bind_addr: String,
        source: io::Error,
    },
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Accept {
        source: io::Error,
    },
}

impl fmt::Display for ServeSiteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(error) => error.fmt(f),
            Self::Bind { bind_addr, source } => {
                write!(f, "failed to bind HTTP server at {}: {}", bind_addr, source)
            }
            Self::Io { path, source } => {
                write!(f, "failed to serve file {}: {}", path.display(), source)
            }
            Self::Accept { source } => write!(f, "failed to accept HTTP connection: {}", source),
        }
    }
}

impl std::error::Error for ServeSiteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Build(error) => Some(error),
            Self::Bind { source, .. } => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::Accept { source } => Some(source),
        }
    }
}

impl From<BuildSiteCommandError> for ServeSiteError {
    fn from(value: BuildSiteCommandError) -> Self {
        Self::Build(value)
    }
}

pub struct SiteServer {
    listener: TcpListener,
    local_addr: SocketAddr,
    output_dir: PathBuf,
}

impl SiteServer {
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn serve_forever(self) -> Result<(), ServeSiteError> {
        let output_dir = self.output_dir;
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => handle_connection(stream, &output_dir)?,
                Err(source) => return Err(ServeSiteError::Accept { source }),
            }
        }

        Ok(())
    }
}

pub fn start_site_server(
    config_path: impl AsRef<Path>,
    bind_addr: &str,
) -> Result<SiteServer, ServeSiteError> {
    let output_dir = std::env::temp_dir().join(unique_serve_dir_name());
    build_site(config_path, &output_dir)?;

    let listener = TcpListener::bind(bind_addr).map_err(|source| ServeSiteError::Bind {
        bind_addr: bind_addr.to_owned(),
        source,
    })?;
    let local_addr = listener
        .local_addr()
        .map_err(|source| ServeSiteError::Bind {
            bind_addr: bind_addr.to_owned(),
            source,
        })?;

    Ok(SiteServer {
        listener,
        local_addr,
        output_dir,
    })
}

pub fn serve_site(config_path: impl AsRef<Path>, bind_addr: &str) -> Result<(), ServeSiteError> {
    let server = start_site_server(config_path, bind_addr)?;
    server.serve_forever()
}

fn handle_connection(mut stream: TcpStream, output_dir: &Path) -> Result<(), ServeSiteError> {
    let mut buffer = [0_u8; 4096];
    let bytes_read = stream
        .read(&mut buffer)
        .map_err(|source| ServeSiteError::Io {
            path: output_dir.to_path_buf(),
            source,
        })?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut parts = request
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or("/");

    if method != "GET" {
        write_response(
            &mut stream,
            405,
            "text/plain; charset=utf-8",
            b"method not allowed",
        )?;
        return Ok(());
    }

    let Some(file_path) = map_request_to_output_path(path) else {
        write_response(&mut stream, 404, "text/plain; charset=utf-8", b"not found")?;
        return Ok(());
    };
    let disk_path = output_dir.join(&file_path);
    if !disk_path.is_file() {
        write_response(&mut stream, 404, "text/plain; charset=utf-8", b"not found")?;
        return Ok(());
    }

    let body = fs::read(&disk_path).map_err(|source| ServeSiteError::Io {
        path: disk_path.clone(),
        source,
    })?;
    write_response(&mut stream, 200, content_type(&disk_path), &body)?;
    Ok(())
}

fn map_request_to_output_path(path: &str) -> Option<PathBuf> {
    let path = path.split('?').next().unwrap_or(path);
    match path {
        "/" => Some(PathBuf::from("index.html")),
        _ if !path.starts_with('/') => None,
        _ => {
            let relative = path.trim_start_matches('/');
            if relative.is_empty() || relative.ends_with('/') {
                return None;
            }
            if relative
                .split('/')
                .any(|segment| segment == ".." || segment.is_empty())
            {
                return None;
            }
            Some(PathBuf::from(relative))
        }
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), ServeSiteError> {
    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    let headers = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Length: {length}\r\nContent-Type: {content_type}\r\nConnection: close\r\n\r\n",
        status = status,
        status_text = status_text,
        length = body.len(),
        content_type = content_type
    );
    stream
        .write_all(headers.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|source| ServeSiteError::Io {
            path: PathBuf::from("<socket>"),
            source,
        })
}

fn unique_serve_dir_name() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("mdbook-bookshelf-serve-{timestamp}-{counter}")
}
