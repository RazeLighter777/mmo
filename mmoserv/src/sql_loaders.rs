use std::{sync::Arc, any::Any};

use bevy_ecs::{world::EntityMut, prelude::ReflectComponent};
use mmolib::{
    chunk::{self, Chunk, ChunkId},
    component, entity,
    game_world::{self, GameWorld},
    raws::RawTree,
    registry::Registry,
    uuid_map, hashing,
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

pub async fn load_entity(
    conn: Pool<MySql>,
    entity_id: entity::EntityId,
    world: &mut  GameWorld,
    registry : &Registry,
) {
    let rows = sqlx::query("SELECT type_id, dat FROM components JOIN entities ON entities.entity_id = components.entity_id?")
        .bind(entity_id.id())
        .fetch_all(&conn)
        .await.unwrap();
    for row in rows {
        let dat : String = row.try_get("dat").unwrap();
        let dat = serde_json::from_str(&dat).unwrap();
        let type_string = row.try_get("type_id").expect("Could not query type_id");
        registry
            .add_component_to_entity(&mut world.spawn(), type_string, dat);
    }
}
pub async fn retreive_all_loaded_chunks_and_entities(conn: &Pool<MySql>, world: &mut GameWorld, registry: &Registry) {
    let rows = sqlx::query("SELECT chunk_id FROM chunks WHERE loaded = true")
        .fetch_all(conn)
        .await
        .expect("Error in database when loading chunks previously set as loaded");
    //load every chunk in using load_chunk_and_entities
    for row in rows {
        let chunk_id: u64 = row.try_get("chunk_id").expect("Could not get chunk_id");
        load_chunk_and_entities(conn, ChunkId::new_raw(chunk_id), world, registry).await;
    }
}

pub async fn load_chunk_and_entities(
    conn: &Pool<MySql>,
    chunk_id: chunk::ChunkId,
    world: &mut game_world::GameWorld,
    registry : &Registry,
) {
    let r = sqlx::query("SELECT dat FROM chunks WHERE chunk_id = ? AND world_id = ?")
        .bind(chunk_id.id())
        .bind(world.get_world_name())
        .fetch_optional(conn)
        .await
        .expect("error querying database for chunk");
    match r {
        Some(row) => {
            let c = Chunk::new(
                row.try_get("dat")
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
                        let entity_id: entity::EntityId =
                            entity::EntityId::new_with_number(row.try_get("entity_id").unwrap());
                        let ent = world.spawn();
                        load_entity(conn.clone(), entity_id, world, registry).await;
                    }
                }
                Err(_) => {}
            };
        }
        None => {}
    }
}

pub async fn save_chunk<'a>(
    mut conn: Transaction<'a, MySql>,
    chunk: &'a chunk::Chunk,
    chunk_id: chunk::ChunkId,
    world: &'a GameWorld,
    loaded: bool,
) -> Transaction<'a, MySql> {
    let r =
        sqlx::query("INSERT INTO chunks (chunk_id, world_id, chunk_dat, loaded) VALUES (?,?,?,?) ON DUPLICATE KEY UPDATE chunk_id=chunk_id")
            .bind(chunk_id.id())
            .bind(world.get_world_name())
            .bind(serde_cbor::to_vec(chunk).expect("Could not serialize chunk as cbor"))
            .bind(loaded)
            .execute(&mut conn)
            .await
            .expect("Could not save chunk");
    //update loaded column in chunks
    sqlx::query("UPDATE chunks SET loaded = ? WHERE chunk_id = ? AND world_id = ?")
        .bind(loaded)
        .bind(chunk_id.id())
        .bind(world.get_world_name())
        .execute(&mut conn)
        .await
        .expect("Could not update chunk loaded status");
    conn
}
pub async fn save_entity<'a>(
    conn: Pool<MySql>,
    entity_id: entity::EntityId,
    ent_mut: EntityMut<'a>,
    world: &'a mut GameWorld,
    registry: &'a Registry,
) {
    // Acquire a new connection and immediately begin a transaction
let ent = world
            .get_world()
            .get_resource::<uuid_map::UuidMap>()
            .unwrap()
            .get(entity_id).unwrap().clone();
    let r = sqlx::query(
        "INSERT INTO entities (entity_id, chunk_id, world_id)  
    VALUES (?,?,?) ON DUPLICATE KEY UPDATE entity_id=entity_id",
    )
    .bind(entity_id.id())
    .bind(
        { 
        match ent_mut.get::<mmolib::position::Position>() {
            Some(pos) => {
                if pos.load_with_chunk {
                    Some(chunk::chunk_id_from_position(pos.pos).id())
                } else {
                    None
                }
            }
            None => None,

        }
    }
       
    )
    .bind(world.get_world_name())
    .execute(&conn)
    .await
    .expect("Could not insert into entity table");
    for id in ent_mut.archetype().components() {
        let mut component_long_name_string : String = "".to_owned();
        let reflect_component = world.get_world()
        .components()
        .get_info(id)
        .and_then(|info| registry.type_registry().get(info.type_id().unwrap()))
        .and_then(|registration| {component_long_name_string = registration.name().to_owned() ;registration.data::<ReflectComponent>()}).unwrap();
        let reflect = reflect_component.reflect_component(world.get_world(),ent).and_then(|refl| refl.serializable()).unwrap();
        let ser = reflect.borrow();
        let ser = serde_json::to_value(&ser).unwrap();
        let ser = ser.get("value").unwrap();
        sqlx::query(
            "INSERT INTO components (type_id, dat, entity_id) VALUES (?,?,?) ON DUPLICATE KEY UPDATE type_id=type_id",
        )
        .bind(&component_long_name_string)
        .bind(serde_json::to_string(&ser).expect("Could not serialize component"))
        .bind(entity_id.id())
        .execute(&conn)
        .await;
    }
}
pub async fn delete_entity<'a>(
    mut tx: Transaction<'a, MySql>,
    entity_id: entity::EntityId,
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
    match sqlx::query("SELECT chunk_id FROM chunks WHERE chunks.chunk_id = ?")
        .bind(chunk_id.id())
        .fetch_one(tx)
        .await
    {
        Ok(res) => true,
        Err(_) => false,
    }
}