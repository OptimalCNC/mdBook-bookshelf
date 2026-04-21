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
        Some("serve") => {
            print_usage(&bin);
            bail!("subcommand 'serve' is not implemented yet")
        }
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
