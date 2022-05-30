#![feature(const_type_name)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
use clap::Parser;
use component::ComponentDataType;
use serde_json::Value;
mod args;
mod component;
mod entity;
mod event_collector;
mod game;
mod game_event;
mod generator;
mod gravity;
mod handler;
mod hashing;
mod mass;
mod pos;
mod positioner;
mod server;
mod context;
mod world;
mod resource;
mod block_type;
mod chunk;
mod raws;
use crate::{entity::EntityBuilder, world::World};
use std::time::Duration;
fn main() {
    let t  = raws::RawTree::new("./raws");
    println!("{}",t.search(&vec!["one".to_owned(), "poo".to_owned()]).unwrap().dat().get("path").unwrap().as_str().unwrap());
    let args = args::Args::parse();
    let mut server = server::Server::new(&args);
    server.run_game();
}
