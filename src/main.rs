#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate serde;
#[macro_use] extern crate serde_derive;


extern crate byteorder;
extern crate csv;
extern crate rand;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate rustc_serialize;

mod address_generator;

fn main() {
    let mut server = rocket::ignite();
    server = address_generator::urls::root::mount_routes(server, "/");
    server = address_generator::urls::addresses::mount_routes(server, "/address/");
    server = address_generator::urls::addresses::add_state(server);
    server.launch();
}
