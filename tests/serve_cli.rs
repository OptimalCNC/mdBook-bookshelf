use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BOOKSHELF_UI_SITE_FIXTURE: &str = "tests/fixtures/bookshelf-ui-site";
const PUBLIC_SELF_CONTAINED_EXAMPLE: &str = "examples/self-contained";

#[test]
fn serve_cli_serves_bookshelf_ui_fixture_site() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let config_path = repo_root
        .join(BOOKSHELF_UI_SITE_FIXTURE)
        .join("bookshelf.toml");
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
    assert_text_contains(&stderr_log, "Bookshelf");
    assert_text_contains(&stderr_log, "root: tests/fixtures/bookshelf-ui-site");
    assert_text_contains(&stderr_log, "output: .tmp/chunk-016-serve");
    assert_text_contains(&stderr_log, "sources:");
    assert_text_contains(
        &stderr_log,
        "\"Fixture Core\": tests/fixtures/bookshelf-ui-site/docs",
    );
    assert_text_not_contains(&stderr_log, ".tmp/chunk-016-serve/modules/parser/docs");

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
fn serve_cli_smoke_serves_public_self_contained_example_from_default_config_path() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = repo_root.join(PUBLIC_SELF_CONTAINED_EXAMPLE);
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
fn serve_cli_without_port_scans_default_port_range() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let config_path = repo_root
        .join(BOOKSHELF_UI_SITE_FIXTURE)
        .join("bookshelf.toml");
    let output_dir = make_temp_dir("serve-default-port-scan", &repo_root);
    let Some(_reserved_ports) = reserve_tcp_ports(3000, 3099) else {
        fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
        return;
    };
    let Ok(last_port_probe) = TcpListener::bind(("127.0.0.1", 3100)) else {
        eprintln!("skipping default port scan test because 127.0.0.1:3100 is unavailable");
        fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
        return;
    };
    drop(last_port_probe);

    let mut child = Command::new(&bin)
        .arg("serve")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .arg("--hostname")
        .arg("127.0.0.1")
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

    assert_eq!(server_address, "127.0.0.1:3100");

    let root_response = wait_for_response(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/",
    );
    assert_status_ok(&root_response);

    shutdown_child(&mut child);
    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn serve_cli_explicit_occupied_port_does_not_fall_back() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let config_path = repo_root
        .join(BOOKSHELF_UI_SITE_FIXTURE)
        .join("bookshelf.toml");
    let output_dir = make_temp_dir("serve-explicit-occupied-port", &repo_root);
    let occupied_listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("ephemeral test port should bind");
    let occupied_port = occupied_listener
        .local_addr()
        .expect("occupied listener address should be readable")
        .port();

    let mut child = Command::new(&bin)
        .arg("serve")
        .arg(&config_path)
        .arg("--dest-dir")
        .arg(&output_dir)
        .arg("--hostname")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(occupied_port.to_string())
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
    let status = wait_for_serve_exit(&mut child, &stderr_lines, &mut stderr_log);

    assert!(
        !status.success(),
        "serve with an occupied explicit port should fail\nstderr:\n{stderr_log}"
    );
    assert!(
        stderr_log.contains(&format!(
            "failed to bind HTTP listener at 127.0.0.1:{occupied_port}"
        )),
        "stderr should mention the requested port\nstderr:\n{stderr_log}"
    );

    fs::remove_dir_all(&output_dir).expect("temp output directory should be removed");
}

#[test]
fn serve_cli_rebuilds_changed_source_and_serves_live_reload_output() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = make_temp_dir("serve-watch-fixture", &repo_root);
    let serve_output_dir = make_temp_dir("serve-watch-output", &repo_root);
    let build_output_dir = make_temp_dir("serve-watch-build-output", &repo_root);
    copy_dir_all(&repo_root.join(BOOKSHELF_UI_SITE_FIXTURE), &fixture_root)
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

#[test]
fn serve_cli_rebuilds_changed_configured_html_asset() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_book"));
    let fixture_root = make_temp_dir("serve-watch-html-asset-fixture", &repo_root);
    let serve_output_dir = make_temp_dir("serve-watch-html-asset-output", &repo_root);
    copy_dir_all(
        &repo_root.join("tests/fixtures/build-cli/shared-config-root"),
        &fixture_root,
    )
    .expect("fixture should be copied to temp directory");
    let config_path = fixture_root.join("bookshelf.toml");

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
    wait_for_stderr_contains(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        "Watching for changes...",
    );
    let serving_index = stderr_log
        .find("Serving on:")
        .expect("serve stderr should include bound address");
    let watching_index = stderr_log
        .find("Watching for changes...")
        .expect("serve stderr should include watcher startup");
    assert!(
        serving_index < watching_index,
        "serve should log the bound address before watcher startup\nstderr:\n{stderr_log}"
    );

    let initial_css = wait_for_served_site_css_body_contains(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/docs/index.html",
        "border-top: 4px solid #0b7285",
    );
    assert_status_ok(&initial_css);

    let marker = format!("Serve watch CSS marker {}", unique_nanos());
    let css_path = fixture_root.join("shared/site.css");
    let mut css = fs::read_to_string(&css_path).expect("shared CSS fixture should be readable");
    css.push_str("\n/* ");
    css.push_str(&marker);
    css.push_str(" */\n");
    fs::write(&css_path, css).expect("shared CSS fixture should be editable");

    let updated_css = wait_for_served_site_css_body_contains(
        &mut child,
        &stderr_lines,
        &mut stderr_log,
        &server_address,
        "/docs/index.html",
        &marker,
    );
    assert_status_ok(&updated_css);

    shutdown_child(&mut child);
    fs::remove_dir_all(&fixture_root).expect("temp fixture directory should be removed");
    fs::remove_dir_all(&serve_output_dir).expect("temp serve output directory should be removed");
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

fn wait_for_serve_exit(
    child: &mut Child,
    stderr_lines: &Receiver<String>,
    stderr_log: &mut String,
) -> std::process::ExitStatus {
    for _ in 0..200 {
        if let Some(status) = child
            .try_wait()
            .expect("serve child status should be readable")
        {
            drain_stderr(stderr_lines, stderr_log);
            return status;
        }

        if let Some(address) = drain_stderr_for_serving_address(stderr_lines, stderr_log) {
            shutdown_child(child);
            panic!("serve started unexpectedly at {address}\nstderr:\n{stderr_log}");
        }

        thread::sleep(Duration::from_millis(50));
    }

    shutdown_child(child);
    panic!("timed out waiting for serve process to exit\nstderr:\n{stderr_log}");
}

fn wait_for_stderr_contains(
    child: &mut Child,
    stderr_lines: &Receiver<String>,
    stderr_log: &mut String,
    expected: &str,
) {
    for _ in 0..100 {
        if stderr_log.contains(expected) {
            return;
        }

        if let Some(status) = child
            .try_wait()
            .expect("serve child status should be readable")
        {
            drain_stderr(stderr_lines, stderr_log);
            panic!("serve exited early with {status}\nstderr:\n{stderr_log}");
        }

        drain_stderr(stderr_lines, stderr_log);
        thread::sleep(Duration::from_millis(50));
    }

    shutdown_child(child);
    panic!("timed out waiting for stderr to contain {expected:?}\nstderr:\n{stderr_log}");
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

fn wait_for_served_site_css_body_contains(
    child: &mut Child,
    stderr_lines: &Receiver<String>,
    stderr_log: &mut String,
    server_address: &str,
    page_path: &str,
    expected: &str,
) -> String {
    let mut last_page_response = String::new();
    let mut last_css_path = String::new();
    let mut last_css_response = String::new();

    for _ in 0..250 {
        if let Some(status) = child
            .try_wait()
            .expect("serve child status should be readable")
        {
            drain_stderr(stderr_lines, stderr_log);
            panic!("serve exited early with {status}\nstderr:\n{stderr_log}");
        }

        match http_get(server_address, page_path) {
            Ok(page_response) => {
                if let Some(href) = extract_site_css_href(response_body(&page_response)) {
                    let css_path = resolve_page_relative_path(page_path, &href);
                    last_css_path = css_path.clone();

                    match http_get(server_address, &css_path) {
                        Ok(css_response) => {
                            if response_body(&css_response).contains(expected) {
                                return css_response;
                            }
                            last_css_response = css_response;
                        }
                        Err(_) => drain_stderr(stderr_lines, stderr_log),
                    }
                }
                last_page_response = page_response;
            }
            Err(_) => drain_stderr(stderr_lines, stderr_log),
        }

        drain_stderr(stderr_lines, stderr_log);
        thread::sleep(Duration::from_millis(100));
    }

    shutdown_child(child);
    panic!(
        "timed out waiting for served site CSS referenced from {page_path} to contain {expected:?}\nstderr:\n{stderr_log}\nlast css path: {last_css_path}\nlast css body:\n{}\nlast page body:\n{}",
        response_body(&last_css_response),
        response_body(&last_page_response)
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

fn extract_site_css_href(page_html: &str) -> Option<String> {
    page_html
        .split("href=\"")
        .skip(1)
        .filter_map(|part| part.split('"').next())
        .find(|href| href.contains("shared/site") && href.ends_with(".css"))
        .map(str::to_string)
}

fn resolve_page_relative_path(page_path: &str, href: &str) -> String {
    if href.starts_with('/') {
        return href.to_string();
    }

    let parent = page_path
        .rsplit_once('/')
        .map(|(parent, _)| {
            if parent.is_empty() {
                "/".to_string()
            } else {
                format!("{parent}/")
            }
        })
        .unwrap_or_else(|| "/".to_string());

    format!("{parent}{href}")
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

fn reserve_tcp_ports(start: u16, end: u16) -> Option<Vec<TcpListener>> {
    let mut listeners = Vec::new();

    for port in start..=end {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => listeners.push(listener),
            Err(err) => {
                eprintln!(
                    "skipping default port scan test because 127.0.0.1:{port} is unavailable: {err}"
                );
                return None;
            }
        }
    }

    Some(listeners)
}
