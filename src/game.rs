use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use serde_json::json;
use sqlx::MySql;
use sqlx::Pool;

use crate::context;
use crate::event_collector;
use crate::game_event;
use crate::game_event::GameEvent;
use crate::generator;
use crate::handler;
use crate::raws::RawTree;
use crate::server;
use crate::server::ServerRequest;
use crate::world;
pub struct Game {
    world: world::World,
    generators: Vec<Box<dyn generator::Generator>>,
    handlers: Vec<Box<dyn handler::HandlerInterface>>,
    event_collector: event_collector::EventCollector,
    conn : Pool<MySql>,
    raws : RawTree
}

impl Game {
    pub fn new(path : &str, conn : Pool<MySql>, world_id : String) -> Self {
        Game {
            world: world::World::new(conn.clone(), world_id),
            generators: Vec::new(),
            handlers: Vec::new(),
            event_collector: event_collector::EventCollector::new(),
            raws : RawTree::new(path),
            conn : conn
        }
    }
    pub async fn handle(sv: Arc<RwLock<Self>>, request: ServerRequest) {}
    pub fn add_generator(&mut self, generator: Box<dyn generator::Generator>) {
        self.generators.push(generator);
    }

    pub fn tick(&mut self) {}
    pub fn get_world(&mut self) -> &mut world::World {
        &mut self.world
    }
    pub fn start_game(gm: Arc<RwLock<Self>>) {
        std::thread::spawn(move || {
            loop {
                //borrow as writable
                let mut gmw1 = gm.write().unwrap();
                for g in &mut gmw1.generators {
                    g.update();
                }
                drop(gmw1);
                let gmr1 = gm.read().unwrap();
                let evs = gmr1.world.process(&gmr1.generators, Arc::new(context::Context { raws : &gmr1.raws}));
                drop(gmr1);
                let mut gmw2 = gm.write().unwrap();
                gmw2.event_collector.add_events(evs);
                for h in &gmw2.handlers {
                    h.handle(&gmw2.event_collector);
                }
                drop(gmw2);
                println!("Ticked!");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }
}
