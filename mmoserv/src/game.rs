use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use mmolib::chunk_generator;
use mmolib::entity;
use mmolib::server_response_type::ServerResponseType;
use mmolib::uuid_map;
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
    chunk_generator: Box<dyn chunk_generator::ChunkGenerator>,
    conn: Pool<MySql>,
    active_connections: Vec<connection::Connection>,
    registry: Arc<mmolib::registry::Registry>,
}

impl Game {
    pub fn new(path: &str, conn: Pool<MySql>, world_id: String) -> Self {
        let rt = RawTree::new(path);
        Game {
            conn: conn,
            registry: Arc::new(
                mmolib::registry::RegistryBuilder::new()
                    .with_component::<mmolib::position::Position>()
                    .with_component::<mmolib::entity::EntityId>()
                    .with_component::<mmolib::player::Player>()
                    .load_block_raws(&["block".to_owned()], &rt)
                    .build(),
            ),
            world: Arc::new(Mutex::new(game_world::GameWorld::new(world_id, rt))),
            active_connections: Vec::new(),
            chunk_generator: Box::new(flat_world_generator::FlatWorldGenerator::new()),
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

    pub async fn start_game(gm: Arc<RwLock<Self>>) {
        let mut counter = 0;
        task::spawn(async move {
            load_world_state(&gm).await;
            spawn_player(&gm).await;
            loop {
                //retrieve list of chunks close to player and load them
                load_and_unload_chunks(&gm).await;
                run_between_ticks_scheduler(&gm).await;
                save_world_state(&gm).await;
                clear_trackers(&gm).await;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                println!("Tick number {}", counter);
                std::io::stdout().flush();
                counter += 1;
            }
        });
    }
}

async fn spawn_player(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    wlk.spawn()
        .insert(mmolib::player::Player {
            username: "admin".to_string(),
            last_ping_timestamp: 0,
        })
        .insert(mmolib::position::Position {
            pos: (128, 128),
            load_with_chunk: false,
        });
}

async fn load_world_state(gm: &Arc<RwLock<Game>>) {
    let mut lk = gm.read().await;
    let chunks = sql_loaders::retreive_all_loaded_chunks_and_entities(
        &lk.conn.clone(),
        &mut *lk.world.lock().await,
        &*(lk.registry),
    )
    .await;
    drop(lk);
    let mut lk = gm.read().await;
    for chunk in chunks {
        lk.world.lock().await.insert_chunk(chunk);
    }
}

async fn save_world_state(gm: &Arc<RwLock<Game>>) {
    let mut lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    let chunks: Vec<mmolib::chunk::ChunkId> = wlk
        .get_chunk_map()
        .get_loaded_chunks()
        .iter()
        .map(|x| **x)
        .collect();
    for c in chunks {
        let chunk = wlk.get_chunk_map().get(c).unwrap();
        sql_loaders::save_chunk(lk.conn.clone(), chunk, c, &*wlk, true).await;
        for entity in wlk.get_entities_in_chunk(c) {
            sql_loaders::save_entity(lk.conn.clone(), entity, &mut *wlk, &lk.registry).await;
        }
    }
}
async fn clear_trackers(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    wlk.clear_trackers();
}

async fn run_between_ticks_scheduler(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    wlk.run_between_ticks_scheduler();
}
async fn load_and_unload_chunks(gm: &Arc<RwLock<Game>>) {
    let mut lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    let mut chunks = wlk.get_list_of_chunk_ids_close_to_players();
    let chunks_that_should_be_loaded = chunks.clone();
    chunks.extend(wlk.get_loaded_chunks());
    drop(wlk);
    drop(lk);
    for chunk_id in chunks {
        let mut lk = gm.read().await;
        let chunk_in_db =
            sql_loaders::check_if_chunk_exists(&lk.conn.clone(), chunk_id, &*lk.world.lock().await)
                .await;
        let chunk_in_game = lk.world.lock().await.is_chunk_loaded(chunk_id);
        let chunk_should_be_in_game = chunks_that_should_be_loaded.contains(&chunk_id);
        drop(lk);
        match (chunk_in_db, chunk_in_game, chunk_should_be_in_game) {
            (true, true, true) => {
                //do nothing, chunk is already loaded
            }
            (true, false, true) => {
                //load the chunk
                let mut lk = gm.read().await;
                let mut wlk = lk.world.lock().await;
                let conn = &lk.conn.clone();
                let chunk =
                    sql_loaders::load_chunk_and_entities(conn, chunk_id, &mut *wlk, &*lk.registry)
                        .await;
                match chunk {
                    Some(chunk) => {
                        wlk.insert_chunk((chunk_id, chunk));
                    }
                    None => {
                        println!("Could not load chunk");
                    }
                }
            }
            (false, true, true) => {
                //do nothing, chunk will be saved eventually
            }
            (false, false, true) => {
                //
                //generate the chunk
                let mut lk = gm.read().await;
                let chunk = lk.chunk_generator.generate_chunk(chunk_id, &*lk.registry);
                let mut wlk = lk.world.lock().await;
                wlk.insert_chunk((chunk_id, chunk));
            }
            (_, true, false) => {
                let mut lk = gm.read().await;
                let mut wlk = lk.world.lock().await;
                let ents = wlk.get_entities_in_chunk(chunk_id);
                for ent in ents {
                    sql_loaders::save_entity(lk.conn.clone(), ent, &mut *wlk, &*lk.registry).await;
                    wlk.remove_entity(ent);
                }
                let chk = wlk.unload_chunk(chunk_id).unwrap();
                sql_loaders::save_chunk(lk.conn.clone(), &chk, chunk_id, &mut *wlk, false).await;
            }
            (_, false, false) => {
                //do nothing
            }
        }
    }
}
