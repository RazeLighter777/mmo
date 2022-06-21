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
use crate::server;
use crate::server_request;
use crate::server_request::ServerRequest;
use mmolib::game_event::GameEvent;
use mmolib::game_world;
use mmolib::raws::RawTree;
pub struct Game {
    world: game_world::GameWorld,
    event_collector: event_collector::EventCollector,
    conn: Pool<MySql>,
    active_connections: Vec<connection::Connection>,
    registry : mmolib::registry::Registry
}

impl Game {
    pub fn new(path: &str, conn: Pool<MySql>, world_id: String) -> Self {
        let rt = RawTree::new(path);
        Game {
            world: game_world::GameWorld::new(world_id, rt),
            event_collector: event_collector::EventCollector::new(),
            conn: conn,
            registry : mmolib::registry::RegistryBuilder::new()
            .with_component::<mmolib::position::Position>()
            .build(),
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

    pub fn tick(&mut self) {}
    pub fn get_world(&mut self) -> &mut game_world::GameWorld {
        &mut self.world
    }
    pub async fn start_game(gm: Arc<RwLock<Self>>) {
        let mut counter: u128 = 0;
        task::spawn(async move {
            loop {
            }
        });
    }
}
