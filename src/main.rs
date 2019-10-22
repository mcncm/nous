//#[cfg(feature = "yaml")]
#[macro_use]
extern crate clap;

use std::io;
use std::env;
use clap::App;

fn main() -> io::Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    match matches.subcommand_name() {
        Some("init") => {
            nous::nous_init(&env::current_dir()?)
                  .unwrap_or_else(|e| {println!("{}", e);});
        },
        Some("add") => {
            let res = matches.subcommand_matches("add").unwrap().value_of("RESOURCE");
            if let Some(uri) = res {
                nous::nous_add(uri.to_string()).
                    unwrap_or_else(|e| {println!("{}", e);});
            } else {
                eprintln!("Missing argument!");
            }
        },
        _ => {eprintln!("no subcommand!");},
    };

    Ok(())

}
