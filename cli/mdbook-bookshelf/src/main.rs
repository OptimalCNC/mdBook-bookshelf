use anyhow::Result;
use clap::Command;
use std::process::ExitCode;

mod cmd;

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
    let matches = create_clap_command().get_matches();

    match matches.subcommand() {
        Some(("build", sub_matches)) => cmd::build::execute(sub_matches),
        Some(("serve", sub_matches)) => cmd::serve::execute(sub_matches),
        _ => unreachable!("clap enforces that a subcommand is present"),
    }
}

fn create_clap_command() -> Command {
    Command::new("mdbook")
        .about("Build and serve a multi-book mdBook bookshelf site")
        .version(concat!("v", env!("CARGO_PKG_VERSION")))
        .propagate_version(true)
        .arg_required_else_help(true)
        .after_help("For more information about a specific command, try `mdbook <command> --help`.")
        .subcommand(cmd::build::make_subcommand())
        .subcommand(cmd::serve::make_subcommand())
}

#[test]
fn verify_cli() {
    create_clap_command().debug_assert();
}
