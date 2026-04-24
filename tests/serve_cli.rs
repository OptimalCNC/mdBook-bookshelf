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
    let config_path = repo_root.join("examples/self-contained/bookshelf.toml");
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
        "http-equiv=\"refresh\" content=\"0; url=docs/index.html\"",
    );
    assert_text_contains(
        response_body(&root_response),
        "window.location.replace(\"docs/index.html\")",
    );

    let shelf_response = wait_for_response(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/docs/bookshelf.html",
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
        "/modules/parser/docs/grammar.html",
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

#[test]
fn serve_cli_defaults_config_path_to_invoking_directory_bookshelf_toml() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = repo_root.join("examples/self-contained");
    let output_dir = make_temp_dir("chunk-016-serve-default-config", &repo_root);

    let mut child = Command::new(&bin)
        .current_dir(&fixture_root)
        .arg("serve")
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

    let shelf_response = wait_for_response(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/docs/bookshelf.html",
    );
    assert_status_ok(&shelf_response);
    assert_text_contains(response_body(&shelf_response), "<h1 id=\"bookshelf\">");
    assert_text_contains(response_body(&shelf_response), "Example Core");

    shutdown_child(&mut child);
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn serve_cli_rebuilds_changed_source_and_serves_live_reload_output() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = make_temp_dir("serve-watch-fixture", &repo_root);
    let serve_output_dir = make_temp_dir("serve-watch-output", &repo_root);
    let build_output_dir = make_temp_dir("serve-watch-build-output", &repo_root);
    copy_dir_all(&repo_root.join("examples/self-contained"), &fixture_root)
        .expect("fixture should be copied to temp directory");
    let config_path = fixture_root.join("bookshelf.toml");

    let build_output = Command::new(&bin)
        .arg("build")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&build_output_dir)
        .output()
        .expect("build command should run");
    assert!(
        build_output.status.success(),
        "build command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build_output.stdout),
        String::from_utf8_lossy(&build_output.stderr)
    );
    let built_page = fs::read_to_string(build_output_dir.join("modules/parser/docs/grammar.html"))
        .expect("normal build output page should be readable");
    assert_text_not_contains(&built_page, "__livereload");

    let mut child = Command::new(&bin)
        .arg("serve")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&serve_output_dir)
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

    let parser_path = "/modules/parser/docs/grammar.html";
    let initial_response = wait_for_response(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        parser_path,
    );
    assert_status_ok(&initial_response);
    assert_text_contains(response_body(&initial_response), "__livereload");
    assert_text_contains(
        response_body(&initial_response),
        "Grammar is a parser-only concern.",
    );

    let marker = format!("Serve watch rebuild marker {}", unique_nanos());
    let grammar_path = fixture_root.join("modules/parser/docs/grammar.md");
    let mut grammar =
        fs::read_to_string(&grammar_path).expect("grammar fixture should be readable");
    grammar.push_str("\n\n");
    grammar.push_str(&marker);
    grammar.push('\n');
    fs::write(&grammar_path, grammar).expect("grammar fixture should be editable");

    let updated_response = wait_for_response_body_contains(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        parser_path,
        &marker,
    );
    assert_status_ok(&updated_response);

    shutdown_child(&mut child);
    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&serve_output_dir).expect("temp serve output directory should be removed");
    fs::remove_dir_all(&build_output_dir).expect("temp build output directory should be removed");
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

fn wait_for_response_body_contains(
    child: &mut Child,
    stderr_lines: &Receiver<String>,
    stderr_log: &mut String,
    server_address: &str,
    path: &str,
    expected: &str,
) -> String {
    let mut last_response = String::new();

    for _ in 0..200 {
        if let Some(status) = child
            .try_wait()
            .expect("serve child status should be readable")
        {
            drain_stderr(stderr_lines, stderr_log);
            panic!("serve exited early with {status}\nstderr:\n{stderr_log}");
        }

        match http_get(server_address, path) {
            Ok(response) => {
                if response_body(&response).contains(expected) {
                    return response;
                }
                last_response = response;
            }
            Err(_) => drain_stderr(stderr_lines, stderr_log),
        }

        drain_stderr(stderr_lines, stderr_log);
        thread::sleep(Duration::from_millis(100));
    }

    shutdown_child(child);
    panic!(
        "timed out waiting for HTTP response from serve process for {path} to contain {expected:?}\nstderr:\n{stderr_log}\nlast response body:\n{}",
        response_body(&last_response)
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

fn assert_text_not_contains(text: &str, unexpected: &str) {
    assert!(
        !text.contains(unexpected),
        "expected not to find {:?} in text:\n{}",
        unexpected,
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
    let path = repo_root
        .join(".tmp")
        .join(format!("{prefix}-{}", unique_nanos()));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn unique_nanos() -> u128 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    nanos
}

fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination_path)?;
        } else {
            fs::copy(entry.path(), destination_path)?;
        }
    }

    Ok(())
}
