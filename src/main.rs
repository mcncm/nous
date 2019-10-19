//#[cfg(feature = "yaml")]
#[macro_use]
extern crate clap;

use nous::*;
use std::io;
use std::env;
use std::path::PathBuf;
use clap::App;

fn main() -> io::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand_name() {
        Some("init") => {},
        Some("add") => {},
        _ => {eprintln!("no subcommand!");},
    };

    Ok(())

}
