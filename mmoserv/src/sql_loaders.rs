use std::sync::Arc;

use mmolib::{
    chunk::{self, Chunk},
    component, entity,
    raws::RawTree,
    registry::Registry,
    world::{self, World},
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

pub async fn retreive_all_loaded_chunks_and_entities(
    conn : & Pool<MySql>,
    world : &World,
) ->(
    Vec<mmolib::chunk::Chunk>,
    Vec<mmolib::entity::Entity>,
    Vec<Box<dyn component::ComponentInterface>>,
) {
    let mut result: (
        Vec<mmolib::chunk::Chunk>,
        Vec<mmolib::entity::Entity>,
        Vec<Box<dyn component::ComponentInterface>>,
    ) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    //retrieve all chunks where loaded is true
    let rows = sqlx::query("SELECT chunk_id, dat FROM chunks WHERE loaded = true")
        .fetch_all(conn)
        .await
        .expect("Error in database when loading chunks previously set as loaded");
    //load every chunk in using load_chunk_and_entities
    for row in rows {
        let chunk_id: chunk::ChunkId = row.try_get("chunk_id").expect("Could not get chunk_id");
        let chunk = load_chunk_and_entities(conn, chunk_id, world).await.expect("Could not load chunk");
        let (entities, comps, chunk) = chunk;
        result.0.push(chunk);
        result.1.extend(entities);
        result.2.extend(comps);
    }
    result
}

pub async fn load_chunk_and_entities(
    conn: &Pool<MySql>,
    chunk_id: chunk::ChunkId,
    world: &world::World,
) -> Option<(
    Vec<entity::Entity>,
    Vec<Box<dyn component::ComponentInterface>>,
    chunk::Chunk,
)> {
    let r = sqlx::query("SELECT dat FROM chunks WHERE chunk_id = ? AND world_id = ?")
        .bind(chunk_id)
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
                        .bind(chunk_id)
                        .execute(conn)
                        .await
                        .expect("error updating chunk");
                    //select entities in chunk
                    let r = sqlx::query(
                        "SELECT entity_id, dat, type_id FROM entities WHERE chunk_id = ? AND world_id = ?",
                    )
                    .bind(chunk_id)
                    .bind(world.get_world_name())
                    .fetch_all(conn)
                    .await
                    .expect("error querying database for entities");
                    //create an array to store boxed components
                    let mut comps = Vec::new();
                    let mut entities = Vec::new();
                    for row in r {
                        let entity_id = row.try_get("entity_id").expect("Could not get entity_id");
                        let type_id: component::ComponentTypeId =
                            row.try_get("type_id").expect("Could not get type_id");
                        let dat: &str = row.try_get("dat").expect("Could not get data");
                        let v: Value =
                            serde_json::from_str(dat).expect("Saved component was not valid json");
                        let mut eb = entity::EntityBuilder::new_with_id(entity_id, world);

                        for cmp in world
                            .get_registry()
                            .generate_component(v, entity_id, type_id, world)
                        {
                            eb = eb.add_existing(cmp);
                        }
                        let (cps, ent) = eb.build();
                        comps.extend(cps);
                        entities.push(ent);
                    }
                    Some((entities, comps, chunk))
                }
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
        //update loaded column in chunks
        sqlx::query("UPDATE chunks SET loaded = ? WHERE chunk_id = ? AND world_id = ?")
            .bind(loaded)
            .bind(chunk_id)
            .bind(world.get_world_name())
            .execute(&mut conn).await.expect("Could not update chunk loaded status");
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
    .bind(
        match world.get_position_of_entity(entity_id) {
            Some(pos) => {
                let chunk_id = mmolib::chunk::chunk_id_from_position(pos);
                Some(chunk_id)
            },
            None => None,
        }
    )
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
) -> Transaction<'a, MySql> {
    let r = sqlx::query("DELETE FROM entities WHERE entities.entity_id = ?")
        .bind(entity_id)
        .execute(&mut tx)
        .await
        .expect("Could not delete entity from table");
    tx
}
pub async fn check_if_chunk_exists<'a>(
    tx : &Pool<MySql>,
    chunk_id: chunk::ChunkId,
    world: &'a World,
) -> bool {
    match sqlx::query("SELECT chunk_id FROM chunks WHERE chunks.chunk_id = ?")
        .bind(chunk_id)
        .fetch_one(tx)
        .await
    {
        Ok(res) => true,
        Err(_) => false,
    }
}

pub async fn delete_component<'a>(
    mut tx: Transaction<'a, MySql>,
    component_id: component::ComponentId,
) -> Transaction<'a, MySql> {
    let r = sqlx::query("DELETE FROM components WHERE components.component_id = ?")
        .bind(component_id)
        .execute(&mut tx)
        .await;
    tx
}
