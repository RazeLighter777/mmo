use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::chunk::{self, Chunk};
use crate::chunk_map::ChunkMap;
use crate::entity_id::EntityId;
use crate::raws::RawTree;
use crate::registry::Registry;
use crate::uuid_map::{self, UuidMap};
use crate::{chunk_generator, chunk_map, entity_deletion_list, player, position, position_map};
use crate::{entity_id, uuid_system};
//use crate::game;
use crate::component;
use crate::{raws, registry};
use bevy_ecs::event::Events;
use bevy_ecs::prelude::{Entity, IntoSystem, World};
use bevy_ecs::schedule::{IntoSystemDescriptor, SystemStage};
use bevy_ecs::world::EntityMut;
use serde_json::Value;

type EventUpdateClosure = Box<dyn Fn(&mut World) -> () + Send + Sync>;
pub struct GameWorld {
    world: bevy_ecs::world::World,
    render_distance: i64,
    world_id: String,
    between_ticks_scheduler: bevy_ecs::schedule::Schedule,
    pre_update_schedule: bevy_ecs::schedule::Schedule,
    post_update_schedule: bevy_ecs::schedule::Schedule,
    event_update_closures: Vec<EventUpdateClosure>,
}

pub struct GameWorldBuilder {
    world: GameWorld,
}

impl GameWorldBuilder {
    pub fn new(world_id: &str) -> Self {
        let mut world = bevy_ecs::world::World::new();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        let mut pre_schedule = bevy_ecs::schedule::Schedule::default();
        pre_schedule.add_stage("update", SystemStage::parallel());
        let mut post_schedule = bevy_ecs::schedule::Schedule::default();
        post_schedule.add_stage("update", SystemStage::single_threaded());
        schedule.add_stage(
            "update",
            SystemStage::parallel()
                .with_system(uuid_system::uuid_system)
                .with_system(position_map::update_position_map_on_position_change)
                .with_system(position_map::update_position_map_on_position_removal),
        );
        let mut world = bevy_ecs::world::World::default();
        world.insert_resource(uuid_map::UuidMap::new());
        world.insert_resource(position_map::PositionMap::new());
        world.insert_resource(chunk_map::ChunkMap::new());
        world.insert_resource(entity_deletion_list::EntityDeletionList::new());
        GameWorldBuilder {
            world: GameWorld {
                world: world,
                render_distance: 10,
                world_id: world_id.to_string(),
                between_ticks_scheduler: schedule,
                pre_update_schedule: pre_schedule,
                post_update_schedule: post_schedule,
                event_update_closures: Vec::new(),
            },
        }
    }
    pub fn add_pre_update_system<Params>(
        mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> Self {
        self.world
            .pre_update_schedule
            .add_system_to_stage("update", system);
        self
    }
    pub fn add_event<EventType: Send + Sync + 'static>(mut self) -> Self {
        self.world
            .world
            .insert_resource(Events::<EventType>::default());
        self.world
            .event_update_closures
            .push(Box::new(move |mut world| {
                let mut events = world.get_resource_mut::<Events<EventType>>().unwrap();
                events.update();
            }));
        self
    }
    pub fn add_post_update_system<Params>(
        mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> Self {
        self.world
            .post_update_schedule
            .add_system_to_stage("update", system);
        self
    }
    pub fn with_world_id(mut self, world_id: String) -> Self {
        self.world.world_id = world_id;
        self
    }
    pub fn with_raws(mut self, raws: RawTree) -> Self {
        self.world.world.insert_resource(raws);
        self
    }
    pub fn with_render_distance(mut self, render_distance: i64) -> Self {
        self.world.render_distance = render_distance;
        self
    }
    pub fn with_between_ticks_scheduler(
        mut self,
        between_ticks_scheduler: bevy_ecs::schedule::Schedule,
    ) -> Self {
        self.world.between_ticks_scheduler = between_ticks_scheduler;
        self
    }
    pub fn build(self) -> GameWorld {
        self.world
    }
}
impl GameWorld {
    pub fn get_world_name(&self) -> &str {
        &self.world_id
    }

    pub fn spawn(&mut self) -> EntityMut {
        let mut res = self.world.spawn().id().clone();
        let entity_id = entity_id::EntityId::new();
        //get uuid map and insert it
        self.world
            .get_resource_mut::<uuid_map::UuidMap>()
            .unwrap()
            .add(entity_id, res);
        let mut r = self.world.entity_mut(res);
        r.insert(entity_id);
        r
    }

    pub fn run_event_update_closures(&mut self) {
        self.event_update_closures.iter().for_each(|x| {
            x(&mut self.world);
        });
    }

    pub fn get_world(&self) -> &bevy_ecs::world::World {
        &self.world
    }
    pub fn get_entities_scheduled_for_deletion(&mut self) -> Vec<EntityId> {
        self.world
            .get_resource_mut::<entity_deletion_list::EntityDeletionList>()
            .unwrap()
            .get_entities()
    }
    pub fn insert_chunk(&mut self, pair: (chunk::ChunkId, Chunk)) {
        self.world
            .get_resource_mut::<ChunkMap>()
            .unwrap()
            .add(pair.0, pair.1);
    }
    pub fn run_between_ticks_scheduler(&mut self) {
        self.between_ticks_scheduler.run_once(&mut self.world);
    }
    pub fn run_pre_update_scheduler(&mut self) {
        self.pre_update_schedule.run_once(&mut self.world);
    }
    pub fn run_post_update_scheduler(&mut self) {
        self.post_update_schedule.run_once(&mut self.world);
    }
    pub fn despawn_entity_by_entity_id(&mut self, entity_id: EntityId) {
        let e = self.get_uuid_map().get(entity_id).map(|x| *x);
        match e {
            Some(ent) => {
                self.world
                    .get_resource_mut::<position_map::PositionMap>()
                    .unwrap()
                    .remove(ent);
                self.world
                    .get_resource_mut::<uuid_map::UuidMap>()
                    .unwrap()
                    .remove(ent);
                self.world.despawn(ent);
            }
            None => {
                println!("Tried to despawn an entity that was not in the world");
            }
        }
    }
    pub fn get_world_mut(&mut self) -> &mut bevy_ecs::world::World {
        &mut self.world
    }

    pub fn is_chunk_loaded(&self, chunk_id: chunk::ChunkId) -> bool {
        self.world
            .get_resource::<ChunkMap>()
            .unwrap()
            .contains(chunk_id)
    }

    pub fn get_loaded_chunks(&self) -> Vec<&chunk::ChunkId> {
        self.world
            .get_resource::<ChunkMap>()
            .unwrap()
            .get_loaded_chunks()
    }

    pub fn unload_chunk(&mut self, chunk_id: chunk::ChunkId) -> Option<Chunk> {
        self.world
            .get_resource_mut::<ChunkMap>()
            .unwrap()
            .remove(chunk_id)
    }

    pub fn get_uuid_map(&self) -> &uuid_map::UuidMap {
        self.world.get_resource::<uuid_map::UuidMap>().unwrap()
    }

    pub fn get_raws(&self) -> &raws::RawTree {
        self.world.get_resource::<raws::RawTree>().unwrap()
    }

    pub fn get_chunk_map(&self) -> &chunk_map::ChunkMap {
        self.world.get_resource::<chunk_map::ChunkMap>().unwrap()
    }

    pub fn clear_trackers(&mut self) -> () {
        self.world.clear_trackers();
        self.world
            .get_resource_mut::<chunk_map::ChunkMap>()
            .unwrap()
            .clear_trackers();
    }

    /**
     * Gets all chunks in a radius of a position.
     */
    pub fn get_chunks_in_radius_of_position(
        render_distance: i64,
        position: (u32, u32),
    ) -> Vec<chunk::ChunkId> {
        let mut chunks_that_should_be_loaded = Vec::new();
        for x in (-render_distance)..render_distance {
            for y in (-render_distance)..render_distance {
                let mut posx = position.0;
                let mut posy = position.1;
                if x > 0 {
                    posx = posx.wrapping_add((x.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                } else {
                    posx = posx.wrapping_sub((x.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                }
                if y > 0 {
                    posy = posy.wrapping_add((y.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                } else {
                    posy = posy.wrapping_sub((y.abs() as u32) * (chunk::CHUNK_SIZE as u32));
                }
                chunks_that_should_be_loaded
                    .push(chunk::chunk_id_from_position((posx as u32, posy as u32)));
            }
        }
        chunks_that_should_be_loaded
    }
    /**
     * gets the list of chunk ids that are close to the players
     */
    pub fn get_list_of_chunk_ids_close_to_players(&mut self) -> Vec<chunk::ChunkId> {
        let mut chunk_ids = Vec::new();
        let mut q = self.world.query::<(&position::Position, &player::Player)>();
        for (pos, player) in q.iter(&self.world) {
            chunk_ids.extend(GameWorld::get_chunks_in_radius_of_position(
                self.render_distance,
                pos.pos,
            ));
        }
        chunk_ids
    }
    fn add_entity_to_position_map_if_has_position(&mut self) {}

    /**
     */
    pub fn remove_entity(&mut self, iid: entity_id::EntityId) {
        let uuid_map = self.world.get_resource::<UuidMap>().unwrap();
        match uuid_map.get(iid) {
            Some(entity) => {
                self.world.despawn(*entity);
            }
            None => {
                println!(
                    "Tried to remove entity with id {} but it was not found",
                    iid
                );
            }
        }
    }

    pub fn get_entities_in_chunk(&self, chunk_id: chunk::ChunkId) -> Vec<entity_id::EntityId> {
        let mut entities = Vec::new();
        let mut position_map = self
            .world
            .get_resource::<position_map::PositionMap>()
            .unwrap();
        let mut uuid_map = self.world.get_resource::<uuid_map::UuidMap>().unwrap();
        match position_map.get_entities_in_chunk(chunk_id) {
            Some(map) => {
                for entity in position_map.get_entities_in_chunk(chunk_id).unwrap() {
                    entities.push(uuid_map.get_by_entity(*entity).expect(
                        "Entity had id component but it wasn't registired in the uuid map",
                    ));
                }
                entities
            }
            None => Vec::new(),
        }
    }
}
