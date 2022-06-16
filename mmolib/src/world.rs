use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::future::join;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::chunk::{self, Chunk};
use crate::entity::{self, Entity};
use crate::pos::Pos;
use crate::raws::RawTree;
use crate::registry::Registry;
use crate::{game_event, world_serializer};
use crate::{generator, player};
//use crate::game;
use crate::component::{self, ComponentInterface};
use crate::world_serializer::WorldSerializer;
use crate::{pos, raws, registry};
use serde_json::Value;

pub struct World {
    copmonents_by_type_id: HashMap<component::ComponentTypeId, HashSet<component::ComponentId>>,
    components: HashMap<component::ComponentId, Box<dyn component::ComponentInterface>>,
    entities: HashMap<entity::EntityId, entity::Entity>,
    render_distance: i64,
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

    pub fn only_list(&mut self, tid: &[component::ComponentTypeId]) -> &EntityFilterTree {
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
                        .filter(|x| self.filtered.contains(&self.world.get_component_interface(*x).unwrap().get_parent())  )
                        .map(|x| self.world.get_component_parent_id(x))
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
    pub fn search(&self, tid: &[component::ComponentTypeId]) -> &HashSet<entity::EntityId> {
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
                .with_component::<player::Player>()
                .build(),
            raws: raws,
            world_serializer: worldserializer,
            positions_of_entities: HashMap::new(),
            entities_queued_for_removal: Vec::new(),
            components_queued_for_removal: Vec::new(),
            entities_queued_for_deletion: Vec::new(),
            components_queued_for_deletion: Vec::new(),
            render_distance: 3,
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
    pub fn get_component_parent_id(&self, id: component::ComponentId) -> entity::EntityId {
        self.get_component_interface(id).unwrap().get_parent()
    }
    pub fn update_position_map(&mut self) {
        // self.positions_of_entities.clear();
        // for component_id in self
        //     .copmonents_by_type_id
        //     .get(&component::get_type_id::<pos::Pos>())
        //     .unwrap()
        // {
        //     let pos = self.get::<pos::Pos>(*component_id).unwrap();
        //     self.positions_of_entities
        //         .insert(*component_id, pos.dat().pos);
        // }
    }
    pub fn get_position_of_entity(&self, entity_id: entity::EntityId) -> Option<chunk::Position> {
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
    pub fn spawn_multiple(
        &mut self,
        e: (
            Vec<Box<dyn component::ComponentInterface>>,
            Vec<entity::Entity>,
        ),
    ) {
        for comp in e.0 {
            match self.copmonents_by_type_id.entry(comp.get_type_id()) {
                Occupied(mut set) => {
                    set.get_mut().insert(comp.get_id());
                }
                Vacant(ent) => {
                    let mut hs = HashSet::new();
                    hs.insert(comp.get_id());
                    ent.insert(hs);
                }
            }
            self.components.insert(comp.get_id(), comp);
        }
        for entity in e.1 {
            self.entities.insert(entity.get_id(), entity);
        }
    }
    pub fn spawn(&mut self, e: (Vec<Box<dyn component::ComponentInterface>>, entity::Entity)) {
        for comp in e.0 {
            match self.copmonents_by_type_id.entry(comp.get_type_id()) {
                Occupied(mut set) => {
                    set.get_mut().insert(comp.get_id());
                }
                Vacant(ent) => {
                    let mut hs = HashSet::new();
                    hs.insert(comp.get_id());
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
    pub fn get_all_components_of_type(
        &self,
        type_id: component::ComponentTypeId,
    ) -> Vec<component::ComponentId> {
        self.copmonents_by_type_id
            .get(&type_id)
            .unwrap_or(&HashSet::new())
            .clone()
            .into_iter()
            .collect()
    }
    pub async fn unload_and_load_chunks(&mut self) {
        let chunks_that_should_be_loaded = self.get_list_of_chunk_ids_close_to_players();
        for (chunk_id, chunk) in &self.chunks {
            self.world_serializer
                .save_entities(
                    chunk
                        .get_entities()
                        .iter()
                        .map(|id| self.get_entity_by_id(*id).unwrap())
                        .collect(),
                    self,
                )
                .await;
        }
        for chunk in self
            .chunks
            .keys()
            .map(|x| *x)
            .collect::<HashSet<_>>()
            .union(
                &chunks_that_should_be_loaded
                    .iter()
                    .map(|x| *x)
                    .collect::<HashSet<_>>(),
            )
        {
            match (
                self.chunks.contains_key(chunk),
                chunks_that_should_be_loaded.contains(chunk),
            ) {
                (true, true) => {
                    //do nothing
                }
                (true, false) => {
                    //save and unload the chunk
                    self.world_serializer
                        .save_entities(
                            self.chunks
                                .get(chunk)
                                .unwrap()
                                .get_entities()
                                .iter()
                                .map(|id| self.get_entity_by_id(*id).unwrap())
                                .collect(),
                            self,
                        )
                        .await;
                    self.world_serializer
                        .save_chunks(
                            vec![(*chunk, self.chunks.get(&chunk).unwrap())],
                            self,
                            false,
                        )
                        .await;
                }
                (false, true) => {
                    //load the chunk with its entities and components
                    let (new_chunk, entities, components) = self
                        .world_serializer
                        .retrieve_chunk_and_entities(*chunk, self)
                        .await;

                    self.chunks.insert(*chunk, new_chunk);
                    self.spawn_multiple((components, entities));
                }
                (_, false) => {
                    //unload every entity in the chunk
                    for entity in self.chunks.get(chunk).unwrap().get_entities() {
                        self.add_entity_to_removal_queue(entity);
                    }
                    self.chunks.remove(chunk);
                }
            }
        }
        //let mut chunks_to_unload = Vec::new();
        self.chunks.retain(|id, chunk| {
            if chunks_that_should_be_loaded.contains(id) {
                true
            } else {
                false
            }
        });
    }

    /**
     * Saves the world to the serializer.
     * Includes all entities and chunks.
     */
    pub async fn save(&self) {
        self.world_serializer
            .save_entities(self.entities.values().collect(), self).await;
        self.world_serializer.save_chunks(self.chunks.iter().map(|(k,v)| (*k, v)  ).collect::<Vec<_>>(), self, true).await;
    }
    /**
     * gets the list of chunk ids that are close to the players
     */
    fn get_list_of_chunk_ids_close_to_players(&self) -> Vec<u64> {
        //create an empty list of chunk positions
        let mut player_positions: HashSet<chunk::Position> = HashSet::new();
        //get all player components
        let players = self
            .get_all_components_of_type(component::get_type_id::<player::Player>())
            .iter()
            .map(|id| self.get::<player::Player>(*id).unwrap())
            .collect::<Vec<_>>();
        for player in players {
            let parent = player.get_parent();
            let position_component = self.get_by_entity_id::<Pos>(parent).expect(
                "Player did not have a position, which is a necessary component of players",
            );
            player_positions.insert(position_component.dat().pos);
        }
        //find all chunks that are within render distance of the player
        let mut chunks_that_should_be_loaded: Vec<chunk::ChunkId> = Vec::new();
        for position in player_positions {
            chunks_that_should_be_loaded.extend(self.get_chunks_in_radius_of_position(position));
        }
        chunks_that_should_be_loaded
    }

    /**
     * Gets all chunks in a radius of a position.
     */
    fn get_chunks_in_radius_of_position(&self, position: (u32, u32)) -> Vec<chunk::ChunkId> {
        let mut chunks_that_should_be_loaded = Vec::new();
        for x in -self.render_distance..self.render_distance {
            for y in -self.render_distance..self.render_distance {
                let mut posx = position.0;
                let mut posy = position.1;
                if x > 0 {
                    posx.wrapping_add((x.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                } else {
                    posx.wrapping_sub((x.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                }
                if y > 0 {
                    posy.wrapping_add((y.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                } else {
                    posy.wrapping_sub((y.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                }
                chunks_that_should_be_loaded
                    .push(chunk::chunk_id_from_position((posx as u32, posy as u32)));
            }
        }
        chunks_that_should_be_loaded
    }

    fn add_entity_to_position_map_if_has_position(&mut self) {}

    /**
     * Gets a component from its id.
     */
    pub fn get<T: component::ComponentDataType + 'static>(
        &self,
        cid: component::ComponentId,
    ) -> Option<&component::Component<T>> {
        match self.components.get(&cid) {
            Some(component) => component.as_any().downcast_ref::<component::Component<T>>(),
            None => None,
        }
    }
    /**
     * Gets a component from its parent entity's id
     */
    pub fn get_by_entity_id<T: component::ComponentDataType + 'static>(
        &self,
        eid: entity::EntityId,
    ) -> Option<&component::Component<T>> {
        match self.get_entity_by_id(eid) {
            Some(e) => match e.get(component::get_type_id::<T>()) {
                Some(c) => match self.get::<T>(c) {
                    Some(c) => Some(c),
                    None => None,
                },
                None => None,
            },
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
