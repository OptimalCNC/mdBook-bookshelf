use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("build failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 6 || args[1] != "build" || args[2] != "--config" || args[4] != "--dest" {
        print_usage(&args.first().cloned().unwrap_or_else(|| "mdbook-bookshelf".to_string()));
        anyhow::bail!("invalid arguments")
    }

    let config = std::fs::canonicalize(&args[3])
        .map_err(|e| anyhow::anyhow!("failed to resolve config path '{}': {e}", &args[3]))?;
    let dest = PathBuf::from(&args[5]);

    mdbook_bookshelf::build_html_site(&config, &dest)?;

    let resolved = std::fs::canonicalize(&dest).unwrap_or(dest);
    println!("Built site: {}", resolved.display());

    Ok(())
}

fn print_usage(bin: &str) {
    eprintln!("Usage: {bin} build --config <path> --dest <dir>");
}
