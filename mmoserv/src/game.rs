use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use mmolib::server_response_type::ServerResponseType;
use tokio::sync::RwLock;
use tokio::task;
use serde_json::json;
use sqlx::MySql;
use sqlx::Pool;
use tokio_tungstenite::WebSocketStream;

use crate::connection;
use crate::event_collector;
use mmolib::game_event::GameEvent;
use crate::handler;
use mmolib::raws::RawTree;
use mmolib::world;
use crate::server;
use crate::server_request::ServerRequest;
use mmolib::generator;
pub struct Game {
    world: world::World,
    generators: Vec<Box<dyn generator::Generator>>,
    handlers: Vec<Box<dyn handler::HandlerInterface>>,
    event_collector: event_collector::EventCollector,
    conn: Pool<MySql>,
    active_connections : Vec<connection::Connection>
}

impl Game {
    pub fn new(path: &str, conn: Pool<MySql>, world_id: String) -> Self {
        let rt = RawTree::new(path);
        Game {
            world: world::World::new(world_id, rt),
            generators: Vec::new(),
            handlers: Vec::new(),
            event_collector: event_collector::EventCollector::new(),
            conn: conn,
            active_connections : Vec::new()
        }
    }
    pub async fn handle(gm: Arc<RwLock<Self>>, request: ServerRequest) {
        match &request.get_dat() {
            mmolib::server_request_type::ServerRequestType::Join { world_name } => {
                gm.write().await.active_connections.push(request.get_connection());
                println!("Someone else has joined game")
            },
            _ => unimplemented!()
        }
    }
    pub fn add_generator(&mut self, generator: Box<dyn generator::Generator>) {
        self.generators.push(generator);
    }

    pub fn tick(&mut self) {}
    pub fn get_world(&mut self) -> &mut world::World {
        &mut self.world
    }
    pub async fn start_game(gm: Arc<RwLock<Self>>) {
        task::spawn(async move {
            loop {
                //borrow as writable
                let mut gmw1 = gm.write().await;
                for g in &mut gmw1.generators {
                    g.update();
                }
                drop(gmw1);
                let gmr1 = gm.read().await;
                let evs = gmr1.world.process(&gmr1.generators);
                drop(gmr1);
                let mut gmw2 = gm.write().await;
                gmw2.event_collector.add_events(evs);
                for h in &gmw2.handlers {
                    h.handle(&gmw2.event_collector);
                }
                drop(gmw2);
                println!("Ticked!");
                //send tick message to all connections.
                let mut gmw2 = gm.write().await;
                for conn in &gmw2.active_connections {
                    conn.send(ServerResponseType::Ticked {});
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }
}
