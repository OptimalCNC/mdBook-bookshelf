use super::command_prelude::*;
use anyhow::Result;
use clap::builder::NonEmptyStringValueParser;

pub fn make_subcommand() -> Command {
    Command::new("serve")
        .about("Build a bookshelf site and serve it over HTTP")
        .arg_bookshelf_config()
        .arg_dest_dir()
        .arg(
            Arg::new("hostname")
                .short('n')
                .long("hostname")
                .value_name("HOST")
                .default_value("localhost")
                .value_parser(NonEmptyStringValueParser::new())
                .help("Hostname to listen on for HTTP connections"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .default_value("3000")
                .value_parser(clap::value_parser!(u16))
                .help("Port to listen on for HTTP connections"),
        )
}

pub fn execute(args: &ArgMatches) -> Result<()> {
    let options = mdbook_bookshelf::ServeOptions {
        config_path: get_config_path(args),
        dest_dir: get_dest_dir(args),
        hostname: args
            .get_one::<String>("hostname")
            .expect("default hostname should be present")
            .clone(),
        port: *args
            .get_one::<u16>("port")
            .expect("default port should be present"),
    };

    mdbook_bookshelf::serve_bookshelf(options)
}
