use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::future::join;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::chunk::{self, Chunk};
use crate::{chunk_generator, player, position};
use crate::raws::RawTree;
use crate::registry::Registry;
use crate::{entity, uuid_system};
//use crate::game;
use crate::component;
use crate::{raws, registry};
use bevy_ecs::schedule::SystemStage;
use bevy_ecs::world::EntityMut;
use serde_json::Value;

pub struct GameWorld {
    world: bevy_ecs::world::World,
    render_distance: i64,
    world_id: String,
    between_ticks_scheduler: bevy_ecs::schedule::Schedule
}

impl GameWorld {
    pub fn new(
        world_name: String,
        raws: raws::RawTree,
    ) -> Self {
        let mut world = bevy_ecs::world::World::new();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_stage(
            "update",
            SystemStage::parallel().with_system(uuid_system::uuid_system),
        );
        world.insert_resource(raws);
        let res = Self {
            world_id: world_name,
            render_distance: 3,
            world: world,
            between_ticks_scheduler: schedule

        };
        //resources.insert(res);
        res
    }
    pub fn get_world_name(&self) -> &str {
        &self.world_id
    }
    pub async fn unload_and_load_chunks(&mut self) {
        todo!()
    }

    pub fn spawn(&mut self) -> EntityMut {
        self.world.spawn()
    }

    pub fn get_world(&self) -> &bevy_ecs::world::World {
        &self.world
    }

    pub fn run_between_ticks_scheduler(&mut self) {
        self.between_ticks_scheduler.run_once(&mut self.world);
    }

    pub fn get_world_mut(&mut self) -> &mut bevy_ecs::world::World {
        &mut self.world
    }

    pub fn get_raws(&self) -> &raws::RawTree {
        self.world.get_resource::<raws::RawTree>().unwrap()
    }

    /**
     * Saves the world to the serializer.
     * Includes all entities and chunks.
     */
    pub async fn save(&self) {
        todo!()
    }
    
    /**
     * Gets all chunks in a radius of a position.
     */
    fn get_chunks_in_radius_of_position(render_distance:  i64, position: (u32, u32)) -> Vec<chunk::ChunkId> {
        let mut chunks_that_should_be_loaded = Vec::new();
        for x in render_distance..render_distance {
            for y in render_distance..render_distance {
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
                pos.pos
            ));
        }
        chunk_ids
    }
    fn add_entity_to_position_map_if_has_position(&mut self) {}

    /**
     * Gets a component from its parent entity's id
     */
    fn remove_entity(&mut self, iid: entity::EntityId) -> bool {
        todo!()
    }
}
