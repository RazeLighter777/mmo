use std::sync::Arc;

use mmolib::{
    chunk::{self, Chunk},
    component, entity,
    raws::RawTree,
    registry::Registry,
    world::World,
};
use serde_json::Value;
use sqlx::{MySql, Pool, Row};

async fn load_entity(
    conn: Pool<MySql>,
    entity_id: entity::EntityId,
    world: &mut World,
) -> Option<entity::Entity> {
    let r = sqlx::query("SELECT dat, type_id FROM components JOIN entities ON components.entity_id = entities.entity_id WHERE components.entity_id = ?")
    .bind(entity_id)
    .fetch_all(&conn).await.expect("Error in database when loading entity");
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
async fn load_chunk(
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
    todo!()
}

async fn save_entity(entity_id: entity::EntityId) {}
