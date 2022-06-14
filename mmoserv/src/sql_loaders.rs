use std::sync::Arc;

use mmolib::{
    chunk::{self, Chunk},
    component, entity,
    raws::RawTree,
    registry::Registry,
    world::World,
};
use serde_json::Value;
use sqlx::{MySql, Pool, Row, Transaction};

pub async fn load_entity(
    mut conn: Transaction<'_, MySql>,
    entity_id: entity::EntityId,
    world: &mut World,
) -> Option<(Vec<Box<dyn component::ComponentInterface>>, entity::Entity)> {
    let r = sqlx::query("SELECT dat, type_id FROM components JOIN entities ON components.entity_id = entities.entity_id WHERE components.entity_id = ?")
    .bind(entity_id)
    .fetch_all(&mut conn).await.expect("Error in database when loading entity");
    let mut eb = entity::EntityBuilder::new_with_id(entity_id, world);
    for row in r {
        let type_id: component::ComponentTypeId =
            row.try_get("type_id").expect("Could not get type_id");
        let dat: &str = row.try_get("dat").expect("Could not get data");
        let v: Value = serde_json::from_str(dat).expect("Saved component was not valid json");
        for cmp in world
            .get_registry()
            .generate_component(v, entity_id, type_id, world)
        {
            eb = eb.add_existing(cmp);
        }
    }
    Some(eb.build())
}
pub async fn load_chunk(
    conn: &Pool<MySql>,
    chunk_id: chunk::ChunkId,
    world_id: &str,
) -> Option<chunk::Chunk> {
    let r = sqlx::query("SELECT dat FROM chunks WHERE chunk_id = ? AND world_id = ?")
        .bind(chunk_id)
        .bind(world_id)
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
                Ok(chunk) => Some(chunk),
                Err(_) => None,
            };
        }
        None => {
            return None;
        }
    }
    None
}

pub async fn save_chunk<'a>(
    mut conn: Transaction<'a, MySql>,
    chunk: &'a chunk::Chunk,
    chunk_id: chunk::ChunkId,
    world: &'a World,
    loaded: bool,
) -> Transaction<'a, MySql> {
    let r =
        sqlx::query("REPLACE INTO chunks (chunk_id, world_id, chunk_dat, loaded) VALUES (?,?,?,?)")
            .bind(chunk_id)
            .bind(world.get_world_name())
            .bind(serde_cbor::to_vec(chunk).expect("Could not serialize chunk as cbor"))
            .bind(loaded)
            .execute(&mut conn)
            .await
            .expect("Could not save chunk");
    conn
}
pub async fn save_entity<'a>(
    mut tx: Transaction<'a, MySql>,
    entity_id: entity::EntityId,
    world: &'a World,
) -> Transaction<'a, MySql> {
    // Acquire a new connection and immediately begin a transaction

    let r = sqlx::query(
        "REPLACE INTO entities (entity_id, chunk_id, world_id)  
    VALUES (?,?,?)",
    )
    .bind(entity_id)
    .bind(0)
    .bind(world.get_world_name())
    .execute(&mut tx)
    .await
    .expect("Could not insert into entity table");
    let entity_ref = world
        .get_entity_by_id(entity_id)
        .expect("Tried to save entity that does not exist");
    let ids = entity_ref.get_component_ids();
    for id in ids {
        let comp = world
            .get_component_interface(entity_ref.get_assured(id))
            .expect("Could not unwrap component when saving entity");
        sqlx::query(
            "REPLACE INTO components (component_id, type_id, dat, entity_id) VALUES (?,?,?,?)",
        )
        .bind(id)
        .bind(comp.get_type_id())
        .bind(comp.get_json().to_string())
        .bind(entity_id)
        .execute(&mut tx)
        .await;
    }
    tx
}
pub async fn delete_entity<'a>(
    mut tx: Transaction<'a, MySql>,
    entity_id: entity::EntityId,
    world: &'a World,
) -> Transaction<'a, MySql> {
    let r = sqlx::query("DELETE FROM entities WHERE entities.entity_id = ?")
        .bind(entity_id)
        .execute(&mut tx)
        .await
        .expect("Could not delete entity from table");
    tx
}
pub async fn check_if_chunk_exists<'a>(
    mut tx: Transaction<'a, MySql>,
    chunk_id: chunk::ChunkId,
    world: &'a World,
) -> (bool, Transaction<'a, MySql>) {
    match sqlx::query("SELECT chunk_id FROM chunks WHERE chunks.chunk_id = ?")
        .bind(chunk_id)
        .fetch_one(&mut tx)
        .await
    {
        Ok(res) => (true, tx),
        Err(_) => (false, tx),
    }
}

