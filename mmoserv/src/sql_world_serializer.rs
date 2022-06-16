use async_trait::async_trait;
use mmolib::{component::ComponentInterface, world, world_serializer};
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
#[async_trait]
impl world_serializer::WorldSerializer for SqlWorldSerializer {
    async fn retrieve_chunk_and_entities(
        &self,
        chunk_id: mmolib::chunk::ChunkId,
        world: &world::World,
    ) -> (
        mmolib::chunk::Chunk,
        Vec<mmolib::entity::Entity>,
        Vec<Box<dyn ComponentInterface>>,
    ) {
        //check if the chunk is already in the database
        if sql_loaders::check_if_chunk_exists(&self.conn, chunk_id, world).await {
            //call load_chunk_and_entities
            let (entities, comps, chunk) =
                sql_loaders::load_chunk_and_entities(&self.conn, chunk_id, world)
                    .await
                    .expect("Chunk does not exist in database");
            //return the result
            (chunk, entities, comps)
        } else {
            //generate the chunk
            let chunk = self.generator.generate_chunk(chunk_id, world);
            //in future, run chunk pregeneration
            (chunk, vec![], vec![])
        }
    }

    async fn save_chunks(
        &self,
        chunks: Vec<(mmolib::chunk::ChunkId, &mmolib::chunk::Chunk)>,
        world: &world::World,
        loaded: bool,
    ) {
        let mut tx = self
            .conn
            .begin()
            .await
            .expect("Could not create transaction");
        for (chunk_id, chunk) in chunks {
            tx = sql_loaders::save_chunk(tx, chunk, chunk_id, world, loaded).await;
        }
        tx.commit().await.expect("Could not save chunks");
    }

    async fn save_entities(&self, entities: Vec<&mmolib::entity::Entity>, world: &world::World) {
        let mut tx = self
            .conn
            .begin()
            .await
            .expect("Could not create transaction");
        for entity in entities {
            tx = sql_loaders::save_entity(tx, entity.get_id(), world).await;
        }
        tx.commit().await.expect("Could not save entities");
    }

    async fn delete_components(&mut self, components: Vec<mmolib::component::ComponentId>) {
        //create a transaction
        let mut tx = self
            .conn
            .begin()
            .await
            .expect("Could not create transaction");
        //delete the components
        for component in components {
            tx = sql_loaders::delete_component(tx, component).await;
        }
        //commit the transaction
        tx.commit().await.expect("Could not delete components");
    }
    async fn delete_entities(&mut self, entities: Vec<mmolib::entity::EntityId>) {
        let mut tx = self
            .conn
            .begin()
            .await
            .expect("Could not create transaction");
        for entity in entities {
            tx = sql_loaders::delete_entity(tx, entity).await;
        }
        tx.commit().await.expect("Could not delete entities");
    }

    async fn set_generator(&mut self, gen: Box<dyn mmolib::chunk_generator::ChunkGenerator>) {
        self.generator = gen;
    }
}
