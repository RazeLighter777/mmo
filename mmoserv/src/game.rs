use std::collections::HashMap;
use std::sync::Arc;

use mmolib::chunk_generator;
use mmolib::entity;
use mmolib::server_response_type::ServerResponseType;
use serde_json::json;
use sqlx::MySql;
use sqlx::Pool;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task;
use tokio_tungstenite::WebSocketStream;

use crate::connection;
use crate::flat_world_generator;
use crate::server;
use crate::server_request;
use crate::server_request::ServerRequest;
use crate::sql_loaders;
use mmolib::game_world;
use mmolib::raws::RawTree;
pub struct Game {
    world: Arc<Mutex<game_world::GameWorld>>,
    chunk_generator : Box<dyn chunk_generator::ChunkGenerator>,
    conn: Pool<MySql>,
    active_connections: Vec<connection::Connection>,
    registry : Arc<mmolib::registry::Registry>
}

impl Game {
    pub fn new(path: &str, conn: Pool<MySql>, world_id: String) -> Self {
        let rt = RawTree::new(path);
        Game {
            world: Arc::new(Mutex::new(game_world::GameWorld::new(world_id, rt))),
            conn: conn,
            registry : Arc::new(mmolib::registry::RegistryBuilder::new()
            .with_component::<mmolib::position::Position>()
            .build()),
            active_connections: Vec::new(),
            chunk_generator : Box::new(flat_world_generator::FlatWorldGenerator::new()),
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
    pub async fn start_game(gm: Arc<RwLock<Self>>) {
        let mut counter: u128 = 0;
        task::spawn(async move {
            let mut lk = gm.read().await;
            let chunks = sql_loaders::retreive_all_loaded_chunks_and_entities(&lk.conn.clone(), &mut *lk.world.lock().await, &*(lk.registry)).await;
            drop(lk);
            loop {
                //retrieve list of chunks close to player and load them
                let mut lk = gm.write().await;
                let mut wlk = lk.world.lock().await;
                let mut chunks = wlk.get_list_of_chunk_ids_close_to_players();
                for chunk_id in chunks {
                    let chunk_in_db = sql_loaders::check_if_chunk_exists(&lk.conn.clone(), chunk_id, &*lk.world.lock().await);
                    let chunk_loaded = 
                    match (,) {
                        _ => {}
                    }
                }
                print!("Tick number {}", counter);
                counter += 1;
            }
        });
    }
}
