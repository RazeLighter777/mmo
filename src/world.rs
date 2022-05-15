use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread::JoinHandle;

use crate::component;
use crate::entity;
use crate::game_event;
use crate::generator;
use log::{info, trace, warn};

pub struct World {
    entities: HashMap<entity::EntityId, entity::Entity>,
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
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
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
            let aworld = Arc::new(Mutex::new(self));
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
