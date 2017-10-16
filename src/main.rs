// Copyright (C) 2018 Wargaming.net Limited. All rights reserved.

// Main entry point.
// Implements basic wrap around, like parsing command line arguments, file I/O, et cetera.

#![allow(unused_variables)]
#![allow(dead_code)]

#[macro_use]
extern crate clap;
extern crate syn;

#[macro_use]
extern crate synom;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate failure;

mod crate_parser;
mod model;

use std::fs::File;
use std::io::BufReader;
use std::io::{self, Read, Write};

use clap::{App, Arg, SubCommand};

// supported languages
arg_enum! {
    #[derive(Debug)]
    enum Lang {
        Cpp
    }
}

fn run_app() -> Result<(), failure::Error> {
    let lang_help = format!("target programming language, {:?}", Lang::variants());

    let matches = App::new("wiregen")
        .version(crate_version!())
        .about(
"Efficient bitstream protocol generator.
Uses a subset of Rust for interface definitions.")

        .subcommand(SubCommand::with_name("write")
                    .about("generate protocol implementation source code")
                    .arg(Arg::with_name("iface")
                         .index(1)
                         .required(true)
                         .help("interface definition (a single-file Rust module of limited syntax)"))
                    .arg(Arg::with_name("output")
                         .short("o")
                         .long("output")
                         .takes_value(true)
                         .required(true)
                         .help("an output file name"))
                    .arg(Arg::with_name("lang")
                         .short("l")
                         .long("lang")
                         .help(&lang_help)
                         .required(true)
                         .takes_value(true)))

        .get_matches_safe()?;

    match matches.subcommand() {
        // Only 'write' subcommand is supported so far.
        ("write", Some(write_matches)) => {
            // parse the arguments
            let lang = value_t!(write_matches, "lang", Lang)?;

            let iface = write_matches.value_of("iface").unwrap();
            let output = write_matches.value_of("output").unwrap();

            // read the interface
            let mut reader = BufReader::new(
                File::open(iface)
                    .map_err(|e| format_err!("Unable to open {}: {}", iface, e))?);
            let mut buffer = String::new();

            reader.read_to_string(&mut buffer)
                .map_err(|e| format_err!("Unable to read {}: {}", iface, e))?;

            // parse the interface
            let items = crate_parser::get_items(&buffer)
                .map_err(|e| format_err!("Cannot parse from here:\n<<<<\n{}\n>>>>\n", e))?;

            let model = model::Model::from_crate(items);

            Ok(())
        },

        _ => Err(format_err!("unknown subcommand, try --help"))
    }

}

fn main() {
    ::std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            writeln!(io::stderr(), "{}", err).unwrap();
            1
        }
    });
}
