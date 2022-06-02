use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread::JoinHandle;

use mmolib::chunk::{self, Chunk};
use mmolib::entity;
use mmolib::game_event;
use crate::generator;
use crate::game;
use mmolib::{component};
use mmolib::{context, pos, raws, registry};
use log::{info, trace, warn};
use serde_json::Value;
use sqlx::{MySql, Pool, Row};

pub struct World {
    entities: HashMap<entity::EntityId, entity::Entity>,
    chunks: HashMap<chunk::ChunkId, chunk::Chunk>,
    conn: Pool<MySql>,
    world_id: String,
    registry: registry::Registry,
    raws: raws::RawTree,
}

struct EntityFilterTree<'a> {
    filtered: Vec<entity::EntityId>,
    subtrees: HashMap<component::ComponentTypeId, EntityFilterTree<'a>>,
    world: &'a World,
}

impl<'a> EntityFilterTree<'a> {
    pub fn new(w: &'a World) -> Self {
        let mut filtered = Vec::new();
        for ent in w.entities.keys() {
            filtered.push(*ent);
        }
        Self {
            filtered: filtered,
            world: w,
            subtrees: HashMap::new(),
        }
    }
    pub fn only_list(&mut self, tid: &[component::ComponentId]) -> &EntityFilterTree {
        if tid.is_empty() {
            return self;
        }
        let val = match self.subtrees.entry(tid[0]) {
            Vacant(mut entry) => {
                let mut t = EntityFilterTree {
                    filtered: self
                        .filtered
                        .clone()
                        .into_iter()
                        .filter(|x| self.world.get_entity_by_id(*x).unwrap().has(tid[0]))
                        .collect(),
                    subtrees: HashMap::new(),
                    world: self.world,
                };
                entry.insert(t).only_list(&tid[1..]);
            }
            Occupied(mut entry) => {}
        };
        self.subtrees.get_mut(&tid[0]).unwrap().only_list(&tid[1..])
    }
    pub fn search(&self, tid: &[component::ComponentId]) -> &Vec<entity::EntityId> {
        if tid.is_empty() {
            &self.filtered
        } else {
            return self.subtrees.get(&tid[0]).unwrap().search(&tid[1..]);
        }
    }
}

impl World {
    pub fn new(conn: Pool<MySql>, world_name: String, raws: raws::RawTree) -> Self {
        Self {
            entities: HashMap::new(),
            chunks: HashMap::new(),
            conn: conn,
            world_id: world_name,
            registry: registry::RegistryBuilder::new()
                .load_block_raws(&["block".to_owned()], &raws)
                .with_component::<pos::Pos>()
                .build(),
            raws: raws,
        }
    }
    async fn load_entity(&self, entity_id: entity::EntityId) -> Option<entity::Entity> {
        let r = sqlx::query("SELECT dat, type_id FROM components JOIN entities ON components.entity_id = entities.entity_id WHERE components.entity_id = ?")
        .bind(entity_id)
        .fetch_all(&self.conn).await.expect("Error in database when loading entity");
        let mut eb = entity::EntityBuilder::new_with_id(
            entity_id,
            Arc::new(context::Context {
                registry: &self.registry,
                raws: &self.raws,
            }),
        );
        for row in r {
            let type_id: component::ComponentTypeId =
                row.try_get("type_id").expect("Could not get type_id");
            let dat: &str = row.try_get("dat").expect("Could not get data");
            let v: Value = serde_json::from_str(dat).expect("Saved component was not valid json");
            for cmp in self.registry.generate_component(
                v,
                entity_id,
                type_id,
                Arc::new(context::Context {
                    raws: &self.raws,
                    registry: &self.registry,
                }),
            ) {
                eb = eb.add_existing(cmp);
            }
        }
        Some(eb.build())
    }
    async fn save_entity(&self, entity_id: entity::EntityId) {
        //sqlx::query("INSERT (")
    }
    async fn load_chunk(&self, chunk_id: chunk::ChunkId) -> Option<chunk::Chunk> {
        let r = sqlx::query("SELECT dat FROM chunks WHERE chunk_id = ? AND world_id = ?")
            .bind(chunk_id)
            .bind(&self.world_id)
            .fetch_optional(&self.conn)
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

    pub fn process(
        &self,
        gens: &Vec<Box<dyn generator::Generator>>,
    ) -> Vec<Box<dyn game_event::GameEventInterface>> {
        let mut res: Vec<Box<dyn game_event::GameEventInterface>> = Vec::new();
        let aresult: Arc<RwLock<&mut Vec<Box<dyn game_event::GameEventInterface>>>> =
            Arc::new(RwLock::new(&mut res));
        std::thread::scope(move |s| {
            let mut tree = EntityFilterTree::new(self);
            for g in gens {
                let mut requests = g.request();
                requests.sort();
                tree.only_list(&requests);
            }
            let aworld = Arc::new(self);
            for g in gens {
                let mut q = g.request();
                q.sort();
                let stuff = tree.search(&q).clone();
                let aworldc = aworld.clone();
                let aeventc = aresult.clone();
                s.spawn(move || {
                    let res = g.generate(aworldc, &stuff);
                    aeventc.write().unwrap().extend(res);
                });
            }
        });
        res
    }
    pub fn spawn(&mut self, e: entity::Entity) {
        info!("Entity spawned into world");
        self.entities.insert(e.get_id(), e);
    }
    pub fn get_entity_by_id(&self, iid: entity::EntityId) -> Option<&entity::Entity> {
        self.entities.get(&iid)
    }
}
