use std::env;
use std::path::PathBuf;
use std::process;

use mdbook_bookshelf::build_site;

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
        _ => Err(usage_error("unknown subcommand")),
    }
}

fn usage_error(message: &str) -> String {
    format!("{message}\nusage: mdbook-bookshelf build <bookshelf.toml> <out-dir>")
}
