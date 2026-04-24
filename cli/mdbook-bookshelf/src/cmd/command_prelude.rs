pub use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;

const DEFAULT_BOOKSHELF_CONFIG: &str = "bookshelf.toml";

pub trait CommandExt: Sized {
    fn with_arg(self, arg: Arg) -> Self;

    fn arg_bookshelf_config(self) -> Self {
        self.with_arg(
            Arg::new("config")
                .value_name("BOOKSHELF_TOML")
                .help("Path to the bookshelf.toml file")
                .default_value(DEFAULT_BOOKSHELF_CONFIG)
                .value_parser(clap::value_parser!(PathBuf)),
        )
    }

    fn arg_dest_dir(self) -> Self {
        self.with_arg(
            Arg::new("dest-dir")
                .short('d')
                .long("dest-dir")
                .value_name("DIR")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Write site output to this directory"),
        )
    }
}

impl CommandExt for Command {
    fn with_arg(self, arg: Arg) -> Self {
        self.arg(arg)
    }
}

pub fn get_config_path(args: &ArgMatches) -> PathBuf {
    args.get_one::<PathBuf>("config")
        .expect("config path default should be present")
        .clone()
}

pub fn get_dest_dir(args: &ArgMatches) -> Option<PathBuf> {
    args.get_one::<PathBuf>("dest-dir").cloned()
}
