use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use futures::future::join_all;
use mmolib::chunk::Chunk;
use mmolib::chunk_generator;
use mmolib::entity_id;
use mmolib::game_world::GameWorld;
use mmolib::server_response_type;
use mmolib::server_response_type::ServerResponseType;
use mmolib::uuid_map;
use serde_json::json;
use sqlx::MySql;
use sqlx::Pool;
use tokio::join;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task;
use tokio_tungstenite::WebSocketStream;
use tracing::info;
use tracing::span;
use tracing::trace;
use tracing::warn;
use tracing::Level;

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
    active_connections: HashMap<String, connection::Connection>,
    registry: Arc<mmolib::registry::Registry>,
}

impl Game {
    pub fn new(path: &str, conn: Pool<MySql>, world_id: String) -> Self {
        let rt = RawTree::new(path);
        Game {
            conn: conn,
            registry: Arc::new(
                mmolib::registry::RegistryBuilder::new()
                    .load_block_raws(&["block".to_owned()], &rt)
                    .build(),
            ),
            world: Arc::new(Mutex::new(
                game_world::GameWorldBuilder::new(&world_id)
                    .with_render_distance(10)
                    .add_event::<mmolib::movement_event::MovementEvent>()
                    .with_raws(rt)
                    .build(),
            )),
            active_connections: HashMap::new(),
            chunk_generator: Box::new(flat_world_generator::FlatWorldGenerator::new()),
        }
    }
    pub async fn handle(gm: Arc<RwLock<Self>>, req: ServerRequest) {
        match &req.get_dat() {
            mmolib::server_request_type::ServerRequestType::Join { world_name } => {
                match req.get_user() {
                    Some(username) => {
                        info!("Player {} has joined the game", username.to_owned());
                        gm.write()
                            .await
                            .active_connections
                            .insert(username.to_owned(), req.get_connection());
                        req.handle(&ServerResponseType::Ok {}).await;
                    }
                    None => {}
                }
            }

            mmolib::server_request_type::ServerRequestType::SendChat {
                world_name,
                message,
            } => {
                for (username, connection) in &gm.read().await.active_connections {
                    connection
                        .send(ServerResponseType::ChatMessage {
                            message: message.clone(),
                            username: username.clone(),
                        })
                        .await;
                }
            }
            mmolib::server_request_type::ServerRequestType::Spawn {
                world_name,
                player_parameters,
            } => match req.get_user() {
                Some(user) => {
                    let mut lk = gm.write().await;
                    let mut wlk = lk.world.lock().await;
                    match sql_loaders::check_if_player_exists_in_world(
                        lk.conn.clone(),
                        &wlk.get_world_name(),
                        user,
                    )
                    .await
                    {
                        Some(entity_id) => {
                            if lk.active_connections.contains_key(user)
                                && lk
                                    .active_connections
                                    .get(user)
                                    .unwrap()
                                    .get_player()
                                    .is_none()
                            {
                                sql_loaders::load_entity(
                                    lk.conn.clone(),
                                    entity_id,
                                    &mut *wlk,
                                    &lk.registry,
                                )
                                .await;
                                drop(wlk);
                                lk.active_connections
                                    .get_mut(user)
                                    .unwrap()
                                    .set_player(entity_id);                            
                                info!("Player {} has loaded a character", user);
                            }
                            info!("Player {} tried to spawn character without Join-ing game",user);
                            req.handle(&ServerResponseType::Error {
                                message: "Tried to spawn character in a game without joining",
                            });
                        }
                        None => {
                            if lk.active_connections.contains_key(user)
                                && lk
                                    .active_connections
                                    .get(user)
                                    .unwrap()
                                    .get_player()
                                    .is_none()
                            {
                                drop(wlk);
                                drop(lk);
                                info!("Player {} has created a new character", user);
                                let eid = spawn_player(&gm.clone(), user).await;
                                let conn = gm.read().await.conn.clone();    
                                let lk = gm.read().await;
                                let mut wlk = lk.world.lock().await;
                                sql_loaders::save_entity(conn.clone(), eid, &mut *wlk, &lk.registry).await;
                                sql_loaders::spawn_player(conn, user, eid).await;
                            }
                        }
                    }
                }
                None => {
                    req.handle(&ServerResponseType::Error {
                        message: "You are not logged in",
                    })
                    .await;
                }
            },
            mmolib::server_request_type::ServerRequestType::PlayerList { world_name } => {
                let mut lk = gm.read().await;
                let mut players = Vec::new();
                for (username, connection) in &lk.active_connections {
                    players.push(username.clone());
                }
                req.handle(&ServerResponseType::PlayerList { players }).await;
            }
            _ => {
                warn!("Request sent to game was not handled")
            }
        }
    }

    pub async fn start_game(gm: Arc<RwLock<Self>>) {
        let mut counter = 0;
        task::spawn(async move {
            load_world_state(&gm).await;
            loop {
                //retrieve list of chunks close to player and load them
                load_and_unload_chunks(&gm).await;
                run_pre_update_scheduler(&gm).await;
                run_between_ticks_scheduler(&gm).await;
                run_event_updater(&gm).await;
                run_post_update_scheduler(&gm).await;
                if counter % 5 == 0 {
                    save_world_state(&gm).await;
                } //save every 25 ticks
                send_ticked_messages(&gm).await;
                join!(clear_trackers(&gm), delete_scheduled_entities(&gm));
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                trace!("Tick number {}", counter);
                std::io::stdout().flush();
                counter += 1;
            }
        });
    }
}

async fn delete_scheduled_entities(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    for ent in wlk.get_entities_scheduled_for_deletion() {
        sql_loaders::delete_entity(lk.conn.clone(), ent);
        wlk.despawn_entity_by_entity_id(ent);
    }
}
async fn send_ticked_messages(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    for (player, position) in wlk
        .get_world_mut()
        .query::<(&mmolib::player::Player, &mmolib::position::Position)>()
        .iter(wlk.get_world())
    {
        let mut chunks = Vec::new();
        match lk.active_connections.get(&player.username) {
            Some(connection) => {
                let ids = GameWorld::get_chunks_in_radius_of_position(3, position.pos);
                for id in ids {
                    let chunk_map = wlk.get_chunk_map();
                    match chunk_map.get(id) {
                        Some(chunk) => {
                            chunks.push((id, chunk.clone()));
                        }
                        None => {
                            tracing::error!("Chunk not found within the radius of the player when attempting to send ticked message");
                        }
                    }
                }
                let response = server_response_type::ServerResponseType::Ticked {
                    world_name: wlk.get_world_name().to_owned(),
                    chunks: chunks,
                    entities: Vec::new(),
                };
                let conn = connection.clone();
                let username = player.username.clone();
                let gmcl = gm.clone();
                tokio::task::spawn(async move {
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(3),
                        conn.send(response),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {
                            tracing::trace!("Sent update to player {}", username);
                        }
                        Err(_) | Ok(Err(_)) => {
                            tracing::info!("Player {} disconnected", username);
                            disconnect_username(gmcl, username).await;
                        }
                    }
                });
            }
            None => {
                warn!("Player {} is not connected", player.username);
            }
        }
    }
    //remove timed out connections
}

async fn disconnect_username(gmcl: Arc<RwLock<Game>>, username: String) {
    let mut lk = gmcl.write().await;
    match lk.active_connections.get(&username) {
        Some(conn) => match conn.get_player() {
            Some(id) => {
                let mut wlk = lk.world.lock().await;
                wlk.despawn_entity_by_entity_id(id);
            }
            None => {}
        },
        None => {
            warn!(
                "Tried to disconnect player {} who is not connected",
                &username
            );
        }
    }
    lk.active_connections.remove(&username);
}
async fn spawn_player(gm: &Arc<RwLock<Game>>, username: &str) -> entity_id::EntityId {
    let mut lk = gm.write().await;
    let mut wlk = lk.world.lock().await;
    let mut e = wlk.spawn();
    e.insert(mmolib::player::Player {
        username: username.to_owned(),
        last_ping_timestamp: 0,
    })
    .insert(mmolib::position::Position {
        pos: (128, 128),
        load_with_chunk: false,
    });
    let id = *e.get::<entity_id::EntityId>().unwrap();
    drop(wlk);
    let mut conn = lk.active_connections.get_mut(username).unwrap();
    conn.set_player(id);
    id
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
    info!("Saving world state");
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
async fn run_pre_update_scheduler(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    wlk.run_pre_update_scheduler();
}
async fn run_post_update_scheduler(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    wlk.run_post_update_scheduler();
}
async fn run_event_updater(gm: &Arc<RwLock<Game>>) {
    let lk = gm.read().await;
    let mut wlk = lk.world.lock().await;
    wlk.run_event_update_closures();
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
                        tracing::error!("Could not load chunk");
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
