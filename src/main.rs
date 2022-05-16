#![feature(const_type_name)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
#![forbid(unsafe_code)]
use clap::Parser;
use component::ComponentDataType;
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
mod world;
use crate::{entity::EntityBuilder, world::World};
use std::time::Duration;
fn main() {
    let args = args::Args::parse();
    let server = server::Server::new(&args);
    let mut g = game::Game::new();
    let grav = Box::new(gravity::Gravity {});
    let ps = Box::new(positioner::Positioner {});
    g.add_generator(grav);
    g.add_generator(ps);
    loop {
        //g.get_world().spawn(entity::EntityBuilder::new().add(pos::Pos { x : 3., y : 3.}).add(mass::Mass { m : 36}).build());
        g.tick();
    }
}
