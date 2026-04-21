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

fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let bin = args
        .first()
        .cloned()
        .unwrap_or_else(|| "mdbook-bookshelf".to_string());

    if let Some(command) = args.get(1) {
        if command == "build" || command == "serve" {
            print_usage(&bin);
            anyhow::bail!(
                "subcommand '{}' is not implemented yet; the previous custom HTML pipeline was removed because it diverged from the mdBook-powered direction",
                command
            )
        }
    }

    print_usage(&bin);
    anyhow::bail!("invalid arguments")
}

fn print_usage(bin: &str) {
    eprintln!("Usage: {bin} <build|serve> ...");
}
