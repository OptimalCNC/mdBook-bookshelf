use super::command_prelude::*;
use anyhow::Result;

pub fn make_subcommand() -> Command {
    Command::new("build")
        .about("Build a bookshelf site")
        .arg_bookshelf_config()
        .arg_dest_dir()
}

pub fn execute(args: &ArgMatches) -> Result<()> {
    let config_path = get_config_path(args);
    let dest_dir = get_dest_dir(args);

    mdbook_bookshelf::build_bookshelf(config_path, dest_dir)
}
