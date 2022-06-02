#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
use clap::Parser;
use serde_json::Value;
mod args;
mod chunk_generator;
mod complex;
mod event_collector;
mod game;
mod generator;
mod gravity;
mod handler;
mod positioner;
mod server;
mod server_request;
mod world;
use crate::{world::World};
use std::time::Duration;
#[async_std::main]
async fn main() {
    let args = args::Args::parse();
    let mut server = server::Server::new(&args).await;
    server.run_game().await;
}
