#![feature(const_type_name)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]

use component::ComponentDataType;
mod entity;
mod component;
mod gravity;
mod pos;
mod hashing;
mod mass;
mod world;
mod generator;
mod game;

use crate::{world::World, entity::EntityBuilder};

fn main() {
    let mut g = game::Game::new();
    let grav = Box::new(gravity::Gravity {

    });
    g.add_generator(grav);
    for x in 0..10 {
        g.get_world().spawn(entity::EntityBuilder::new().add(pos::Pos { x : 3., y : 3.}).add(mass::Mass { m : 3}).build());
        g.tick();
    }
}
