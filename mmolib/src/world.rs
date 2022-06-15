use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::chunk::{self, Chunk};
use crate::entity::{self, Entity};
use crate::generator;
use crate::raws::RawTree;
use crate::registry::Registry;
use crate::{game_event, world_serializer};
//use crate::game;
use crate::component;
use crate::world_serializer::WorldSerializer;
use crate::{pos, raws, registry};
use serde_json::Value;

pub struct World {
    copmonents_by_type_id: HashMap<component::ComponentTypeId, HashSet<component::ComponentId>>,
    components: HashMap<component::ComponentId, Box<dyn component::ComponentInterface>>,
    entities: HashMap<entity::EntityId, entity::Entity>,
    chunks: HashMap<chunk::ChunkId, chunk::Chunk>,
    world_id: String,
    registry: registry::Registry,
    world_serializer: Box<dyn WorldSerializer>,
    raws: raws::RawTree,
    positions_of_entities: HashMap<entity::EntityId, chunk::Position>,
    entities_queued_for_removal: Vec<entity::EntityId>,
    components_queued_for_removal: Vec<component::ComponentId>,
    entities_queued_for_deletion: Vec<entity::EntityId>,
    components_queued_for_deletion: Vec<component::ComponentId>,
}

struct EntityFilterTree<'a> {
    filtered: HashSet<entity::EntityId>,
    subtrees: HashMap<component::ComponentTypeId, EntityFilterTree<'a>>,
    world: &'a World,
}

impl<'a> EntityFilterTree<'a> {
    pub fn new(w: &'a World) -> Self {
        let mut filtered = HashSet::new();
        for ent in w.entities.keys() {
            filtered.insert(*ent);
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
                        .world
                        .copmonents_by_type_id
                        .get(&tid[0])
                        .unwrap_or(&HashSet::new())
                        .clone()
                        .into_iter()
                        .filter(|x| self.filtered.contains(x))
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
    pub fn search(&self, tid: &[component::ComponentId]) -> &HashSet<entity::EntityId> {
        if tid.is_empty() {
            &self.filtered
        } else {
            return self.subtrees.get(&tid[0]).unwrap().search(&tid[1..]);
        }
    }
}

impl World {
    pub fn new(
        world_name: String,
        raws: raws::RawTree,
        worldserializer: Box<dyn world_serializer::WorldSerializer>,
    ) -> Self {
        Self {
            copmonents_by_type_id: HashMap::new(),
            components: HashMap::new(),
            entities: HashMap::new(),
            chunks: HashMap::new(),
            world_id: world_name,
            registry: registry::RegistryBuilder::new()
                .load_block_raws(&["block".to_owned()], &raws)
                .with_component::<pos::Pos>()
                .build(),
            raws: raws,
            world_serializer: worldserializer,
            positions_of_entities: HashMap::new(),
            entities_queued_for_removal: Vec::new(),
            components_queued_for_removal: Vec::new(),
            entities_queued_for_deletion: Vec::new(),
            components_queued_for_deletion: Vec::new(),
        }
    }
    pub fn get_registry(&self) -> &registry::Registry {
        &self.registry
    }
    pub fn get_raws(&self) -> &raws::RawTree {
        &self.raws
    }
    pub fn get_world_name(&self) -> &str {
        &self.world_id
    }
    pub fn update_position_map(&mut self) {
        self.positions_of_entities.clear();
        for component_id in self.copmonents_by_type_id.get(&component::get_type_id::<pos::Pos>()).unwrap()
        {
            let pos = self.get::<pos::Pos>(*component_id).unwrap();
            self.positions_of_entities.insert(*component_id, pos.dat().pos);
        }
    }
    pub fn get_position_of_entity(&self, entity_id : entity::EntityId) -> Option<chunk::Position> {
        self.positions_of_entities.get(&entity_id).cloned()
    }
    pub fn run_deletions_and_removals(&mut self) {}
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
                let mut entities_for_processing = Vec::new();
                for id in stuff {
                    match self.get_entity_by_id(id) {
                        Some(ent) => {
                            entities_for_processing.push(ent);
                        }
                        None => {}
                    }
                }
                let aworldc = aworld.clone();
                let aeventc = aresult.clone();
                s.spawn(move || {
                    let res = g.generate(aworldc, &entities_for_processing);
                    aeventc.write().unwrap().extend(res);
                });
            }
        });
        res
    }
    pub fn spawn(&mut self, e: (Vec<Box<dyn component::ComponentInterface>>, entity::Entity)) {
        for comp in e.0 {
            match self.copmonents_by_type_id.entry(comp.get_type_id()) {
                Occupied(mut set) => {
                    set.get_mut().insert(e.1.get_id());
                }
                Vacant(ent) => {
                    let mut hs = HashSet::new();
                    hs.insert(e.1.get_id());
                    ent.insert(hs);
                }
            }
            self.components.insert(comp.get_id(), comp);
        }
        self.entities.insert(e.1.get_id(), e.1);
    }
    pub fn get_entity_by_id(&self, iid: entity::EntityId) -> Option<&entity::Entity> {
        self.entities.get(&iid)
    }
    pub fn get_chunk(&self, chunk_id: chunk::ChunkId) -> Option<&chunk::Chunk> {
        self.chunks.get(&chunk_id)
    }

    fn add_entity_to_position_map_if_has_position(&mut self) {}

    pub fn get<T: component::ComponentDataType + 'static>(
        &self,
        cid: component::ComponentId,
    ) -> Option<&component::Component<T>> {
        match self.components.get(&cid) {
            Some(component) => component.as_any().downcast_ref::<component::Component<T>>(),
            None => None,
        }
    }
    pub fn get_mut<Q: component::ComponentDataType + 'static>(
        &mut self,
        cid: component::ComponentId,
    ) -> Option<&mut component::Component<Q>> {
        match self.components.get_mut(&cid) {
            Some(component) => component
                .as_mutable()
                .downcast_mut::<component::Component<Q>>(),
            None => None,
        }
    }
    pub fn get_component_interface(
        &self,
        cid: component::ComponentId,
    ) -> Option<&Box<dyn component::ComponentInterface>> {
        self.components.get(&cid)
    }
    pub fn add_entity_to_deletion_queue(&mut self, iid: entity::EntityId) {
        self.entities_queued_for_deletion.push(iid);
    }
    pub fn add_component_to_deletion_queue(&mut self, iid: component::ComponentId) {
        self.components_queued_for_deletion.push(iid);
    }
    pub fn add_component_to_removal_queue(&mut self, iid: component::ComponentId) {
        self.components_queued_for_removal.push(iid);
    }
    pub fn add_entity_to_removal_queue(&mut self, iid: entity::EntityId) {
        self.entities_queued_for_removal.push(iid);
    }
    pub async fn cleanup_deleted_and_removed_entities_and_components(&mut self) {
        self.world_serializer
            .delete_components(self.components_queued_for_deletion.clone())
            .await;
        self.world_serializer
            .delete_entities(self.entities_queued_for_deletion.clone())
            .await;
        for component in [
            self.components_queued_for_deletion.as_slice(),
            self.components_queued_for_removal.as_slice(),
        ]
        .concat()
        {
            self.remove_component(component);
        }
        for entity in [
            self.entities_queued_for_deletion.as_slice(),
            self.entities_queued_for_removal.as_slice(),
        ]
        .concat()
        {
            self.remove_entity(entity);
        }
    }
    fn remove_component(&mut self, iid: component::ComponentId) -> bool {
        match self.get_component_interface(iid) {
            Some(x) => {
                let tid = x.get_type_id();
                let iid = x.get_id();
                let pid = x.get_parent();
                self.copmonents_by_type_id
                    .get_mut(&tid)
                    .unwrap()
                    .remove(&iid)
                    && self.entities.get_mut(&pid).unwrap().remove(tid)
                    && self.components.remove(&iid).is_some()
            }
            None => false,
        }
    }
    fn remove_entity(&mut self, iid: entity::EntityId) -> bool {
        match self.entities.get(&iid) {
            Some(entity) => {
                let iid = entity.get_id();
                for component in entity.get_component_ids() {
                    self.remove_component(component);
                }
                let pos = self.positions_of_entities.remove(&iid).unwrap();
                match self.chunks.get_mut(&chunk::chunk_id_from_position(pos)) {
                    Some(chunk) => {
                        chunk.remove(iid);
                    }
                    None => {}
                }
                self.entities.remove(&iid).is_some()
            }
            None => false,
        }
    }
}
