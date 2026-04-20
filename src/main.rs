use std::env;
use std::path::PathBuf;
use std::process;

use mdbook_bookshelf::{build_site, start_site_server};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage_error("missing subcommand"));
    };

    match command.as_str() {
        "build" => {
            let Some(config_path) = args.next() else {
                return Err(usage_error("missing required <bookshelf.toml> path"));
            };
            let Some(output_dir) = args.next() else {
                return Err(usage_error("missing required <out-dir> path"));
            };
            if args.next().is_some() {
                return Err(usage_error("unexpected extra arguments"));
            }

            build_site(PathBuf::from(config_path), PathBuf::from(output_dir))
                .map(|_| ())
                .map_err(|error| format!("build failed: {error}"))
        }
        "serve" => {
            let Some(config_path) = args.next() else {
                return Err(usage_error("missing required <bookshelf.toml> path"));
            };
            let mut bind_addr = String::from("127.0.0.1:3000");

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--bind" => {
                        let Some(value) = args.next() else {
                            return Err(usage_error("missing required <bind-addr> after --bind"));
                        };
                        bind_addr = value;
                    }
                    _ => return Err(usage_error("unexpected extra arguments")),
                }
            }

            let server = start_site_server(PathBuf::from(config_path), &bind_addr)
                .map_err(|error| format!("serve failed: {error}"))?;
            println!("listening on http://{}", server.local_addr());
            server
                .serve_forever()
                .map_err(|error| format!("serve failed: {error}"))
        }
        _ => Err(usage_error("unknown subcommand")),
    }
}

fn usage_error(message: &str) -> String {
    format!(
        "{message}\nusage: mdbook-bookshelf build <bookshelf.toml> <out-dir>\n       mdbook-bookshelf serve <bookshelf.toml> [--bind <addr>]"
    )
}
