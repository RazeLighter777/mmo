use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::event_collector;
use crate::game_event;
use crate::game_event::GameEvent;
use crate::generator;
use crate::handler;
use crate::server;
use crate::server::ServerRequest;
use crate::world;
pub struct Game {
    world: world::World,
    generators: Vec<Box<dyn generator::Generator>>,
    handlers: Vec<Box<dyn handler::HandlerInterface>>,
    event_collector: event_collector::EventCollector,
    pending_reqs: Arc<Mutex<Vec<ServerRequest>>>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            world: world::World::new(),
            generators: Vec::new(),
            handlers: Vec::new(),
            event_collector: event_collector::EventCollector::new(),
            pending_reqs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn handle(&mut self, req: server::ServerRequest) {
        //TODO: pass response to other things . . .
        let mut ul = self.pending_reqs.lock().unwrap();
        ul.push(req);
    }
    pub fn add_generator(&mut self, generator: Box<dyn generator::Generator>) {
        self.generators.push(generator);
    }

    pub fn tick(&mut self) {
        //handle inputs
        self.handle_queued_requests();
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
    fn handle_queued_requests(&mut self) {
        let mut pgl = self.pending_reqs.lock();
        let mut pr = pgl.unwrap();
        for i in pr.iter_mut() {
            i.handle(server::ServerResponse {  })
        }
    }
}
