use std::{any::Any, sync::Arc};

use bevy_ecs::{prelude::ReflectComponent, world::EntityMut};
use mmolib::{
    chunk::{self, Chunk, ChunkId},
    component, entity_id,
    game_world::{self, GameWorld},
    hashing,
    raws::RawTree,
    registry::Registry,
    uuid_map,
};
use serde_json::Value;
use sqlx::{MySql, Pool, Row, Transaction};

pub async fn create_world(conn: Pool<MySql>, world_id: &str) -> bool {
    let r = sqlx::query("INSERT INTO worlds (world_id) VALUES (?)")
        .bind(world_id)
        .execute(&conn)
        .await
        .is_ok();
    r
}
pub async fn check_if_world_exists(conn: Pool<MySql>, world_id: &str) -> bool {
    let r = sqlx::query("SELECT * FROM worlds WHERE world_id = ?")
        .bind(world_id)
        .fetch_one(&conn)
        .await
        .is_ok();
    r
}
pub async fn initialize_database(conn: Pool<MySql>) {
    sqlx::query(
        r"CREATE TABLE IF NOT EXISTS worlds (
            world_id VARCHAR(50) PRIMARY KEY NOT NULL)",
    )
    .execute(&conn)
    .await
    .unwrap();
    sqlx::query(
        r"CREATE TABLE IF NOT EXISTS users (
            user_id INT PRIMARY KEY NOT NULL AUTO_INCREMENT,
            user_name TEXT,
            password_hash TEXT,
            admin BOOLEAN)",
    )
    .execute(&conn)
    .await
    .unwrap();
    sqlx::query(
        r"CREATE TABLE IF NOT EXISTS chunks (
            chunk_id BIGINT UNSIGNED,
            world_id VARCHAR(50)  NOT NULL,
            chunk_dat BLOB,
            loaded BOOLEAN,
            FOREIGN KEY (world_id)
                REFERENCES worlds(world_id)
                ON DELETE CASCADE,
            PRIMARY KEY (chunk_id,world_id))",
    )
    .execute(&conn)
    .await
    .unwrap();
    sqlx::query(
        r"CREATE TABLE IF NOT EXISTS entities (
            entity_id BIGINT UNSIGNED PRIMARY KEY,
            chunk_id BIGINT UNSIGNED,
            world_id VARCHAR(50) NOT NULL,
            FOREIGN KEY(chunk_id) 
                REFERENCES chunks(chunk_id),
            FOREIGN KEY(world_id)
                REFERENCES worlds(world_id)
                ON DELETE CASCADE
            )",
    )
    .execute(&conn)
    .await
    .unwrap();
    sqlx::query(
        r"CREATE TABLE IF NOT EXISTS components (
            type_id VARCHAR(50),
            entity_id BIGINT UNSIGNED, 
            dat TEXT,
            FOREIGN KEY(entity_id) 
                REFERENCES entities(entity_id)
                ON DELETE CASCADE,
            PRIMARY KEY (entity_id,type_id))",
    )
    .execute(&conn)
    .await
    .unwrap();
}
pub async fn load_entity(
    conn: Pool<MySql>,
    entity_id: entity_id::EntityId,
    world: &mut GameWorld,
    registry: &Registry,
) {
    let rows = sqlx::query("SELECT components.type_id, components.dat FROM components JOIN entities ON entities.entity_id = components.entity_id")
        .bind(entity_id.id())
        .fetch_all(&conn)
        .await.unwrap();
    for row in rows {
        let dat: String = row.try_get("dat").unwrap();
        let dat = serde_json::from_str(&dat).unwrap();
        let type_string = row.try_get("type_id").expect("Could not query type_id");
        registry.add_component_to_entity(&mut world.spawn(), type_string, dat);
    }
}
pub async fn retreive_all_loaded_chunks_and_entities(
    conn: &Pool<MySql>,
    world: &mut GameWorld,
    registry: &Registry,
) -> Vec<(chunk::ChunkId, chunk::Chunk)> {
    let rows = sqlx::query("SELECT chunk_id FROM chunks WHERE loaded = true")
        .fetch_all(conn)
        .await
        .expect("Error in database when loading chunks previously set as loaded");
    //load every chunk in using load_chunk_and_entities
    let mut chunks = Vec::new();
    for row in rows {
        let chunk_id: u64 = row.try_get("chunk_id").expect("Could not get chunk_id");
        match load_chunk_and_entities(conn, ChunkId::new_raw(chunk_id), world, registry).await {
            Some(chunk) => {
                chunks.push((ChunkId::new_raw(chunk_id), chunk));
            }
            None => {
                println!("Error loading chunk");
            }
        }
    }
    chunks
}

pub async fn load_chunk_and_entities(
    conn: &Pool<MySql>,
    chunk_id: chunk::ChunkId,
    world: &mut game_world::GameWorld,
    registry: &Registry,
) -> Option<chunk::Chunk> {
    let r = sqlx::query("SELECT chunk_dat FROM chunks WHERE chunk_id = ? AND world_id = ?")
        .bind(chunk_id.id())
        .bind(world.get_world_name())
        .fetch_optional(conn)
        .await
        .expect("error querying database for chunk");
    match r {
        Some(row) => {
            let c = Chunk::new(
                row.try_get("chunk_dat")
                    .expect("chunk format in database invalid"),
            );
            return match c {
                Ok(chunk) => {
                    //change column loaded in chunks to true
                    sqlx::query("UPDATE chunks SET loaded = true WHERE chunk_id = ?")
                        .bind(chunk_id.id())
                        .execute(conn)
                        .await
                        .expect("error updating chunk");
                    //select entities in chunk
                    let r = sqlx::query(
                        "SELECT entity_id FROM entities WHERE chunk_id = ? AND world_id = ?",
                    )
                    .bind(chunk_id.id())
                    .bind(world.get_world_name())
                    .fetch_all(conn)
                    .await
                    .expect("error querying database for entities");
                    for row in r {
                        let entity_id: entity_id::EntityId =
                            entity_id::EntityId::new_with_number(row.try_get("entity_id").unwrap());
                        let ent = world.spawn();
                        load_entity(conn.clone(), entity_id, world, registry).await;
                    }
                    Some(chunk)
                }
                Err(_) => None,
            };
        }
        None => {}
    }
    None
}

pub async fn save_chunk<'a>(
    mut conn: Pool<MySql>,
    chunk: &'a chunk::Chunk,
    chunk_id: chunk::ChunkId,
    world: &'a GameWorld,
    loaded: bool,
) {
    let r =
        sqlx::query("INSERT INTO chunks (chunk_id, world_id, chunk_dat, loaded) VALUES (?,?,?,?) ON DUPLICATE KEY UPDATE chunk_id=chunk_id")
            .bind(chunk_id.id())
            .bind(world.get_world_name())
            .bind(serde_cbor::to_vec(chunk).expect("Could not serialize chunk as cbor"))
            .bind(loaded)
            .execute(& conn)
            .await
            .expect("Could not save chunk");
    //update loaded column in chunks
    sqlx::query("UPDATE chunks SET loaded = ? WHERE chunk_id = ? AND world_id = ?")
        .bind(loaded)
        .bind(chunk_id.id())
        .bind(world.get_world_name())
        .execute(&conn)
        .await
        .expect("Could not update chunk loaded status");
}
pub async fn save_entity<'a>(
    conn: Pool<MySql>,
    entity_id: entity_id::EntityId,
    world: &'a mut GameWorld,
    registry: &'a Registry,
) {
    // Acquire a new connection and immediately begin a transaction
    let ent = world
        .get_world()
        .get_resource::<uuid_map::UuidMap>()
        .unwrap()
        .get(entity_id)
        .unwrap()
        .clone();
    let r = sqlx::query(
        "INSERT INTO entities (entity_id, chunk_id, world_id)  
    VALUES (?,?,?) ON DUPLICATE KEY UPDATE entity_id=entity_id",
    )
    .bind(entity_id.id())
    .bind({
        match world.get_world().get::<mmolib::position::Position>(ent) {
            Some(pos) => {
                if pos.load_with_chunk {
                    Some(chunk::chunk_id_from_position(pos.pos).id())
                } else {
                    None
                }
            }
            None => None,
        }
    })
    .bind(world.get_world_name())
    .execute(&conn)
    .await
    .expect("Could not insert into entity table");
    let mut results: Vec<(String, String, u64)> = Vec::new();
    //moved out to seperate block because can't call await while !Send values are in scope, which are specifically bevy internal items.
    {
        let entity_ref = world.get_world().get_entity(ent).unwrap();
        let archtype = entity_ref.archetype();
        for id in archtype.components() {
            let mut component_long_name_string: String = "".to_owned();
            match world
                .get_world()
                .components()
                .get_info(id)
                .and_then(|info| registry.type_registry().get(info.type_id().unwrap()))
                .and_then(|registration| {
                    component_long_name_string = registration.name().to_owned();
                    println!("type: {}", &component_long_name_string);
                    registration.data::<ReflectComponent>()
                }) {
                Some(reflect_component) => {
                    let reflect = reflect_component
                        .reflect_component(world.get_world(), ent)
                        .and_then(|refl| refl.serializable())
                        .unwrap();
                    let ser = reflect.borrow();
                    let ser = serde_json::to_string(&ser).unwrap();
                    println!("Serialization : {}", &ser);
                    //let ser = ser.get("value").unwrap();
                    results.push((component_long_name_string.to_owned(), ser, entity_id.id()));
                }
                None => {
                    println!("Could not find component info for {:?}", id);
                }
            }
        }
    }

    for (component_type, serialization, entity_id) in results {
        sqlx::query(
            "INSERT INTO components (type_id, dat, entity_id) VALUES (?,?,?) ON DUPLICATE KEY UPDATE type_id=type_id",
        )
        .bind(&component_type)
        .bind(&serialization)
        .bind(entity_id)
        .execute(&conn)
        .await.expect("Could not insert into component table");
    }
}
pub async fn delete_entity<'a>(
    mut tx: Transaction<'a, MySql>,
    entity_id: entity_id::EntityId,
) -> Transaction<'a, MySql> {
    let r = sqlx::query("DELETE FROM entities WHERE entities.entity_id = ?")
        .bind(entity_id.id())
        .execute(&mut tx)
        .await
        .expect("Could not delete entity from table");
    tx
}
pub async fn check_if_chunk_exists<'a>(
    tx: &Pool<MySql>,
    chunk_id: chunk::ChunkId,
    world: &'a GameWorld,
) -> bool {
    match sqlx::query(
        "SELECT chunk_id FROM chunks WHERE chunks.chunk_id = ? AND chunks.world_id = ?",
    )
    .bind(chunk_id.id())
    .bind(world.get_world_name())
    .fetch_one(tx)
    .await
    {
        Ok(res) => true,
        Err(_) => false,
    }
}
