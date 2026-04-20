use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[test]
fn serve_cli_preserves_canonical_routes_and_rejects_aliases() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = env!("CARGO_BIN_EXE_mdbook-bookshelf");
    let mut child = Command::new(binary)
        .current_dir(&repo_root)
        .arg("serve")
        .arg("bookshelf/handoffs/examples/self-contained/bookshelf.toml")
        .arg("--bind")
        .arg("127.0.0.1:0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("serve command should spawn");

    let addr = read_listen_addr(&mut child);

    let bookshelf_alias = http_get(&addr, "/bookshelf/");
    assert!(!bookshelf_alias.status_line.starts_with("HTTP/1.1 200"));

    let bookshelf_index_alias = http_get(&addr, "/bookshelf/index.html");
    assert!(!bookshelf_index_alias
        .status_line
        .starts_with("HTTP/1.1 200"));

    let markdown_route = http_get(&addr, "/docs/onboarding.md");
    assert!(!markdown_route.status_line.starts_with("HTTP/1.1 200"));

    let authored = http_get(&addr, "/docs/onboarding.html");
    assert!(authored.status_line.starts_with("HTTP/1.1 200"));
    assert!(authored.body.contains("Onboarding"));

    stop_child(&mut child);
}

fn read_listen_addr(child: &mut Child) -> String {
    let stdout = child.stdout.as_mut().expect("stdout should be piped");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .expect("stdout should be readable");
    if bytes == 0 {
        let stderr = read_child_stderr(child);
        panic!("serve process exited before reporting its listen address: {stderr}");
    }

    line.trim()
        .strip_prefix("listening on http://")
        .expect("listen address line should have expected prefix")
        .to_owned()
}

fn read_child_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(stream) = child.stderr.as_mut() {
        let _ = stream.read_to_string(&mut stderr);
    }
    stderr
}

fn stop_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

struct HttpResponse {
    status_line: String,
    body: String,
}

fn http_get(addr: &str, path: &str) -> HttpResponse {
    let mut stream = TcpStream::connect(addr).expect("http connection should succeed");
    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request should be writable");

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("response should be readable");

    let mut sections = response.split("\r\n\r\n");
    let headers = sections.next().expect("response should contain headers");
    let body = sections.next().unwrap_or_default().to_owned();
    let status_line = headers
        .lines()
        .next()
        .expect("response should contain a status line")
        .to_owned();

    HttpResponse { status_line, body }
}
