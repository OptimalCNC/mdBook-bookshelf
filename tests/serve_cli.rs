use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn serve_cli_serves_built_site() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_mdbook-bookshelf"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = make_temp_dir("chunk-016-serve", &repo_root);
    let port = reserve_loopback_port();

    let mut child = Command::new(&bin)
        .arg("serve")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .arg("--hostname")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("serve command should spawn");

    let root_response = wait_for_response(&mut child, port, "/");
    assert_status_ok(&root_response);
    assert_text_contains(
        response_body(&root_response),
        "http-equiv=\"refresh\" content=\"0; url=books/meta/bookshelf.html\"",
    );
    assert_text_contains(
        response_body(&root_response),
        "window.location.replace(\"books/meta/bookshelf.html\")",
    );

    let shelf_response = wait_for_response(&mut child, port, "/books/meta/bookshelf.html");
    assert_status_ok(&shelf_response);
    assert_text_contains(response_body(&shelf_response), "<h1 id=\"bookshelf\">");
    assert_text_contains(
        response_body(&shelf_response),
        "Choose a book to enter its root page.",
    );

    let parser_response = wait_for_response(&mut child, port, "/books/parser/grammar.html");
    assert_status_ok(&parser_response);
    assert_text_contains(response_body(&parser_response), "<h1 id=\"grammar\">");
    assert_text_contains(
        response_body(&parser_response),
        "Grammar is a parser-only concern.",
    );

    shutdown_child(&mut child);
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

fn wait_for_response(child: &mut Child, port: u16, path: &str) -> String {
    for _ in 0..100 {
        if let Some(status) = child
            .try_wait()
            .expect("serve child status should be readable")
        {
            let stderr = read_child_stderr(child);
            panic!("serve exited early with {status}\nstderr:\n{stderr}");
        }

        match http_get(port, path) {
            Ok(response) => return response,
            Err(_) => thread::sleep(Duration::from_millis(50)),
        }
    }

    shutdown_child(child);
    panic!("timed out waiting for HTTP response from serve process for {path}");
}

fn http_get(port: u16, path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").as_bytes(),
    )?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

fn response_body(response: &str) -> &str {
    response.split("\r\n\r\n").nth(1).unwrap_or("")
}

fn assert_status_ok(response: &str) {
    let status_line = response.lines().next().unwrap_or_default();
    assert!(
        status_line.starts_with("HTTP/1.1 200") || status_line.starts_with("HTTP/1.0 200"),
        "expected 200 response, got: {status_line}\nfull response:\n{response}"
    );
}

fn assert_text_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "expected to find {:?} in response body:\n{}",
        expected,
        text
    );
}

fn shutdown_child(child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn read_child_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    stderr
}

fn reserve_loopback_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("loopback listener should bind")
        .local_addr()
        .expect("loopback listener should report local address")
        .port()
}

fn make_temp_dir(prefix: &str, repo_root: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = repo_root.join(".tmp").join(format!("{prefix}-{nanos}"));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}
