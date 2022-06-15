use mmolib::{world, world_serializer};
use sqlx::{MySql, Pool};

use crate::{flat_world_generator, sql_loaders};

pub struct SqlWorldSerializer {
    conn: Pool<MySql>,
    generator: Box<dyn mmolib::chunk_generator::ChunkGenerator>,
}

impl SqlWorldSerializer {
    pub fn new(conn: Pool<MySql>) -> Self {
        Self {
            conn: conn,
            generator: Box::new(flat_world_generator::FlatWorldGenerator::new()),
        }
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

    fn save_entities(&self, entities: Vec<&mmolib::entity::Entity>, world: &world::World) {
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

    fn delete_components(&mut self, components: Vec<mmolib::component::ComponentId>) {
        futures::executor::block_on(async move {
            let mut tx = self
                .conn
                .begin()
                .await
                .expect("Could not create transaction");
            for component in components {
                tx = sql_loaders::delete_component(tx, component).await;
            }
            tx.commit().await.expect("Could not delete components");
        });
    }

    fn delete_entities(&mut self, entities: Vec<mmolib::entity::EntityId>) {
        futures::executor::block_on(async move {
            let mut tx = self
                .conn
                .begin()
                .await
                .expect("Could not create transaction");
            for entity in entities {
                tx = sql_loaders::delete_entity(tx, entity).await;
            }
            tx.commit().await.expect("Could not delete entities");
        });
    }

    fn set_generator(&mut self, gen: Box<dyn mmolib::chunk_generator::ChunkGenerator>) {
        self.generator = gen;
    }
}
