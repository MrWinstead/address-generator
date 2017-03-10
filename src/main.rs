#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]


extern crate byteorder;
extern crate csv;
extern crate clap;
extern crate rand;
extern crate rocket;
extern crate rustc_serialize;
extern crate serde;

use clap::{Arg,App};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod address_generator;

fn get_file_contents(filename: &str) -> String {
    let source_csv_path = Path::new(filename);
    let mut file: File = match File::open(&source_csv_path) {
        Err(why) => panic!("Could not open source file {}: {}", source_csv_path.display(),
                            why.description()),
        Ok(file) => file,
    };
    let mut csv_contents = String::new();
    match file.read_to_string(&mut csv_contents) {
        Err(why) => panic!("Could not read contents of {}: {}", source_csv_path.display(),
                            why.description()),
        Ok(_) => println!("Read {} byes from {}", csv_contents.len(), source_csv_path.display()),
    };

    csv_contents
}

fn main() {
    let cmdline_arguments = App::new("IPv4 Address Generator Service")
                                .arg(Arg::with_name("source_csv")
                                        .short("s")
                                        .long("source")
                                        .default_value("country_ip.csv")
                                        .required(true))
                                .get_matches();
    let csv_contents = get_file_contents(cmdline_arguments.value_of("source_csv").unwrap());
    let mut server = rocket::ignite();
    server = address_generator::urls::root::mount_routes(server, "/");
    server = address_generator::urls::addresses::mount_routes(server, "/address/");
    server = address_generator::urls::addresses::add_state(server, &csv_contents);
    server.launch();
}
