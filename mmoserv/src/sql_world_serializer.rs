use mmolib::{world, world_serializer};
use sqlx::{MySql, Pool};

use crate::sql_loaders;

pub struct SqlWorldSerializer {
    conn: Pool<MySql>,
}

impl SqlWorldSerializer {
    fn new(conn: Pool<MySql>) -> Self {
        Self { conn: conn }
    }
}

impl world_serializer::WorldSerializer for SqlWorldSerializer {
    fn retrieve_chunk_and_entities(
        &mut self,
        chunk_id: mmolib::chunk::ChunkId,
        world: &world::World,
    ) -> (mmolib::chunk::Chunk, Vec<mmolib::entity::Entity>) {
        todo!()
    }

    fn save_chunks(
        &mut self,
        chunks: Vec<(mmolib::chunk::ChunkId, &mmolib::chunk::Chunk)>,
        world: &world::World,
        loaded: bool,
    ) {
        futures::executor::block_on(async move {
            let mut tx = self
                .conn
                .begin()
                .await
                .expect("Could not create transaction");
            for (chunk_id, chunk) in chunks {
                tx = sql_loaders::save_chunk(tx, chunk, chunk_id, world, loaded).await;
            }
            tx.commit().await.expect("Could not save chunks");
        });
    }

    fn save_entities(&mut self, entities: Vec<&mmolib::entity::Entity>, world: &world::World) {
        futures::executor::block_on(async move {
            let mut tx = self
                .conn
                .begin()
                .await
                .expect("Could not create transaction");
            for entity in entities {
                tx = sql_loaders::save_entity(tx, entity.get_id(), world).await;
            }
            tx.commit().await.expect("Could not save entities");
        });
    }

    fn delete_entities(&mut self, entities: Vec<&mmolib::entity::Entity>, world: &world::World) {
        todo!()
    }
}
