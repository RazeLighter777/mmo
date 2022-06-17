use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use mmolib::entity;
use mmolib::server_response_type::ServerResponseType;
use serde_json::json;
use sqlx::MySql;
use sqlx::Pool;
use tokio::sync::RwLock;
use tokio::task;
use tokio_tungstenite::WebSocketStream;

use crate::connection;
use crate::event_collector;
use crate::handler;
use crate::server;
use crate::server_request;
use crate::server_request::ServerRequest;
use crate::sql_world_serializer;
use mmolib::game_event::GameEvent;
use mmolib::generator;
use mmolib::raws::RawTree;
use mmolib::world;
pub struct Game {
    world: world::World,
    generators: Vec<Box<dyn generator::Generator>>,
    handlers: Vec<Box<dyn handler::HandlerInterface>>,
    event_collector: event_collector::EventCollector,
    conn: Pool<MySql>,
    active_connections: Vec<connection::Connection>,
}

impl Game {
    pub fn new(path: &str, conn: Pool<MySql>, world_id: String) -> Self {
        let rt = RawTree::new(path);
        let serializer = Box::new(sql_world_serializer::SqlWorldSerializer::new(conn.clone()));
        Game {
            world: world::World::new(world_id, rt, serializer),
            generators: Vec::new(),
            handlers: Vec::new(),
            event_collector: event_collector::EventCollector::new(),
            conn: conn,
            active_connections: Vec::new(),
        }
    }
    pub async fn handle(gm: Arc<RwLock<Self>>, request: ServerRequest) {
        match &request.get_dat() {
            mmolib::server_request_type::ServerRequestType::Join { world_name } => {
                gm.write()
                    .await
                    .active_connections
                    .push(request.get_connection());
                request.handle(&ServerResponseType::Ok {}).await;
                println!("Someone else has joined game")
            }
            _ => {
                println!("Request sent to game was not handled")
            }
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
        let mut counter: u128 = 0;
        task::spawn(async move {
            let mut gmw1 = gm.write().await;
            let eb = entity::EntityBuilder::new(gmw1.world.get_registry(), &gmw1.world)
                .add(mmolib::player::Player {
                    username: "admin".to_string(),
                })
                .add(mmolib::pos::Pos {
                    pos: (100, 100),
                    load_with_chunk: false,
                })
                .build();
            gmw1.world.spawn(eb);
            drop(gmw1);
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
                //send tick message to all connections.
                let mut gmw2 = gm.write().await;
                for conn in &gmw2.active_connections {
                    conn.send(ServerResponseType::Ticked {
                        world_name: gmw2.world.get_world_name().to_owned(),
                    })
                    .await;
                }
                gmw2.world.unload_and_load_chunks().await;
                gmw2.world
                    .cleanup_deleted_and_removed_entities_and_components()
                    .await;
                gmw2.world.save().await;
                counter += 1;
                println!("Ticked {} times", counter);
            }
        });
    }
}
