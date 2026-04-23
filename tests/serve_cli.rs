use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn serve_cli_serves_built_site() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let config_path = repo_root.join("bookshelf/handoffs/examples/self-contained/bookshelf.toml");
    let output_dir = make_temp_dir("chunk-016-serve", &repo_root);

    let mut child = Command::new(&bin)
        .arg("serve")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .arg("--hostname")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("0")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("serve command should spawn");

    let stderr = child
        .stderr
        .take()
        .expect("serve child stderr should be piped");
    let stderr_lines = spawn_stderr_reader(stderr);
    let mut stderr_log = String::new();
    let server_address = wait_for_serving_address(&mut child, &stderr_lines, &mut stderr_log);

    let root_response = wait_for_response(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/",
    );
    assert_status_ok(&root_response);
    assert_text_contains(
        response_body(&root_response),
        "http-equiv=\"refresh\" content=\"0; url=books/meta/bookshelf.html\"",
    );
    assert_text_contains(
        response_body(&root_response),
        "window.location.replace(\"books/meta/bookshelf.html\")",
    );

    let shelf_response = wait_for_response(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/books/meta/bookshelf.html",
    );
    assert_status_ok(&shelf_response);
    assert_text_contains(response_body(&shelf_response), "<h1 id=\"bookshelf\">");
    assert_text_contains(
        response_body(&shelf_response),
        "Choose a book to enter its root page.",
    );

    let parser_response = wait_for_response(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/books/parser/grammar.html",
    );
    assert_status_ok(&parser_response);
    assert_text_contains(response_body(&parser_response), "<h1 id=\"grammar\">");
    assert_text_contains(
        response_body(&parser_response),
        "Grammar is a parser-only concern.",
    );

    shutdown_child(&mut child);
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

fn wait_for_serving_address(
    child: &mut Child,
    stderr_lines: &Receiver<String>,
    stderr_log: &mut String,
) -> String {
    for _ in 0..100 {
        if let Some(status) = child
            .try_wait()
            .expect("serve child status should be readable")
        {
            drain_stderr(stderr_lines, stderr_log);
            panic!("serve exited early with {status}\nstderr:\n{stderr_log}");
        }

        if let Some(address) = drain_stderr_for_serving_address(stderr_lines, stderr_log) {
            return address;
        }

        thread::sleep(Duration::from_millis(50));
    }

    shutdown_child(child);
    panic!("timed out waiting for serve process to report a bound address\nstderr:\n{stderr_log}");
}

fn wait_for_response(
    child: &mut Child,
    stderr_lines: &Receiver<String>,
    stderr_log: &mut String,
    server_address: &str,
    path: &str,
) -> String {
    for _ in 0..100 {
        if let Some(status) = child
            .try_wait()
            .expect("serve child status should be readable")
        {
            drain_stderr(stderr_lines, stderr_log);
            panic!("serve exited early with {status}\nstderr:\n{stderr_log}");
        }

        match http_get(server_address, path) {
            Ok(response) => return response,
            Err(_) => {
                drain_stderr(stderr_lines, stderr_log);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    shutdown_child(child);
    panic!(
        "timed out waiting for HTTP response from serve process for {path}\nstderr:\n{stderr_log}"
    );
}

fn http_get(server_address: &str, path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect(server_address)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(
        format!("GET {path} HTTP/1.1\r\nHost: {server_address}\r\nConnection: close\r\n\r\n")
            .as_bytes(),
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

fn spawn_stderr_reader(stderr: ChildStderr) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let _ = sender.send(line);
                }
                Err(_) => break,
            }
        }
    });

    receiver
}

fn drain_stderr_for_serving_address(
    stderr_lines: &Receiver<String>,
    stderr_log: &mut String,
) -> Option<String> {
    let mut address = None;

    while let Ok(line) = stderr_lines.try_recv() {
        if !stderr_log.is_empty() {
            stderr_log.push('\n');
        }
        stderr_log.push_str(&line);

        if let Some(bound) = line.strip_prefix("Serving on: http://") {
            address = Some(bound.trim().to_string());
        }
    }

    address
}

fn drain_stderr(stderr_lines: &Receiver<String>, stderr_log: &mut String) {
    while let Ok(line) = stderr_lines.try_recv() {
        if !stderr_log.is_empty() {
            stderr_log.push('\n');
        }
        stderr_log.push_str(&line);
    }
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
