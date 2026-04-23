use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("command failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args();
    let bin = args
        .next()
        .unwrap_or_else(|| "mdbook-bookshelf".to_string());

    match args.next().as_deref() {
        Some("build") => run_build(&bin, args.collect()),
        Some("serve") => run_serve(&bin, args.collect()),
        _ => {
            print_usage(&bin);
            bail!("invalid arguments")
        }
    }
}

fn print_usage(bin: &str) {
    eprintln!("Usage: {bin} <build|serve> ...");
}

fn run_build(bin: &str, args: Vec<String>) -> Result<()> {
    let (config_path, dest_dir) = match parse_build_args(args) {
        Ok(parsed) => parsed,
        Err(err) => {
            print_build_usage(bin);
            return Err(err);
        }
    };

    mdbook_bookshelf::build_bookshelf(config_path, dest_dir)
}

fn run_serve(bin: &str, args: Vec<String>) -> Result<()> {
    let options = match parse_serve_args(args) {
        Ok(parsed) => parsed,
        Err(err) => {
            print_serve_usage(bin);
            return Err(err);
        }
    };

    mdbook_bookshelf::serve_bookshelf(options)
}

fn parse_build_args(args: Vec<String>) -> Result<(PathBuf, Option<PathBuf>)> {
    let mut config_path = None;
    let mut dest_dir = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dest-dir" | "-d" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for {arg}"))?;
                if dest_dir.replace(PathBuf::from(value)).is_some() {
                    bail!("--dest-dir may only be provided once");
                }
            }
            _ if arg.starts_with('-') => bail!("unknown build flag '{arg}'"),
            _ => {
                if config_path.replace(PathBuf::from(arg)).is_some() {
                    bail!("build accepts exactly one <bookshelf.toml> path");
                }
            }
        }
    }

    let config_path =
        config_path.ok_or_else(|| anyhow::anyhow!("missing required <bookshelf.toml> path"))?;
    Ok((config_path, dest_dir))
}

fn print_build_usage(bin: &str) {
    eprintln!("Usage: {bin} build <bookshelf.toml> [--dest-dir <dir>]");
}

fn parse_serve_args(args: Vec<String>) -> Result<mdbook_bookshelf::ServeOptions> {
    let mut config_path = None;
    let mut dest_dir = None;
    let mut hostname = "localhost".to_string();
    let mut port: u16 = 3000;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dest-dir" | "-d" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for {arg}"))?;
                if dest_dir.replace(PathBuf::from(value)).is_some() {
                    bail!("--dest-dir may only be provided once");
                }
            }
            "--hostname" | "-n" => {
                hostname = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for {arg}"))?;
            }
            "--port" | "-p" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("missing value for {arg}"))?;
                port = value
                    .parse::<u16>()
                    .map_err(|_| anyhow::anyhow!("invalid port '{value}'"))?;
            }
            _ if arg.starts_with('-') => bail!("unknown serve flag '{arg}'"),
            _ => {
                if config_path.replace(PathBuf::from(arg)).is_some() {
                    bail!("serve accepts exactly one <bookshelf.toml> path");
                }
            }
        }
    }

    let config_path =
        config_path.ok_or_else(|| anyhow::anyhow!("missing required <bookshelf.toml> path"))?;

    Ok(mdbook_bookshelf::ServeOptions {
        config_path,
        dest_dir,
        hostname,
        port,
    })
}

fn print_serve_usage(bin: &str) {
    eprintln!(
        "Usage: {bin} serve <bookshelf.toml> [--dest-dir <dir>] [--hostname <host>] [--port <port>]"
    );
}
