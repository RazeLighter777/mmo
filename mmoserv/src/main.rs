#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![allow(unused)]
#![deny(warnings)]
use clap::Parser;
use serde_json::Value;
use tracing::{info, subscriber};
mod args;
mod complex;
mod connection;
mod flat_world_generator;
mod game;
mod server;
mod server_request;
mod sql_loaders;
use std::time::Duration;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let args = args::Args::parse();
    let mut server = server::Server::new(&args).await;
    server.run_game().await;
}
