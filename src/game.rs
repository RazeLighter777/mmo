use std::collections::HashMap;

use crate::event_collector;
use crate::game_event;
use crate::game_event::GameEvent;
use crate::generator;
use crate::handler;
use crate::server;
use crate::world;
pub struct Game {
    world: world::World,
    generators: Vec<Box<dyn generator::Generator>>,
    handlers: Vec<Box<dyn handler::HandlerInterface>>,
    event_collector: event_collector::EventCollector,
}

impl Game {
    pub fn new() -> Self {
        Game {
            world: world::World::new(),
            generators: Vec::new(),
            handlers: Vec::new(),
            event_collector: event_collector::EventCollector::new(),
        }
    }
    pub fn add_generator(&mut self, generator: Box<dyn generator::Generator>) {
        self.generators.push(generator);
    }

    pub fn tick(&mut self) {
        for g in &mut self.generators {
            g.update();
        }
        self.event_collector
            .add_events(self.world.process(&self.generators));
        for h in &self.handlers {
            h.handle(&self.event_collector);
        }
    }
    pub fn get_world(&mut self) -> &mut world::World {
        &mut self.world
    }
}
