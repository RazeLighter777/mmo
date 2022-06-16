#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![allow(unused)]
#![deny(warnings)]
use clap::Parser;
use serde_json::Value;
mod args;
mod complex;
mod connection;
mod event_collector;
mod flat_world_generator;
mod game;
mod handler;
mod server;
mod server_request;
mod sql_loaders;
mod sql_world_serializer;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = args::Args::parse();
    let mut server = server::Server::new(&args).await;
    server.run_game().await;
}
