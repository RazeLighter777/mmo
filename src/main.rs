#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
use clap::Parser;
use component::ComponentDataType;
use serde_json::Value;
mod args;
mod block_type;
mod chunk;
mod chunk_generator;
mod complex;
mod component;
mod registry;
mod context;
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
mod raws;
mod resource;
mod server;
mod effect;
mod world;
use crate::{entity::EntityBuilder, world::World};
use std::time::Duration;
#[async_std::main]
async fn main() {
    let args = args::Args::parse();
    let mut server = server::Server::new(&args).await;
    server.run_game().await;
}
