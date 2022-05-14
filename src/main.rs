#![feature(const_type_name)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
#![forbid(unsafe_code)]
use component::ComponentDataType;
mod entity;
mod component;
mod gravity;
mod event_collector;
mod game_event;
mod handler;
mod pos;
mod hashing;
mod mass;
mod world;
mod generator;
mod game;
mod positioner;
use crate::{world::World, entity::EntityBuilder};
fn main() {
    let mut g = game::Game::new();
    let grav = Box::new(gravity::Gravity {});
    let ps = Box::new(positioner::Positioner {});
    g.add_generator(grav);
    g.add_generator(ps);
    for x in 0..10 {
        g.get_world().spawn(entity::EntityBuilder::new().add(pos::Pos { x : 3., y : 3.}).add(mass::Mass { m : 36}).build());
        g.tick();
    }
}
