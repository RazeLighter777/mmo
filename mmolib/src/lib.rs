#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![allow(incomplete_features)]
#![feature(unsize)]
#![feature(specialization)]
#![allow(unused)]
#![deny(warnings)]
pub mod block_type;
pub mod chunk;
pub mod util;
pub mod chunk_generator;
pub mod chunk_map;
pub mod component;
pub mod effect;
pub mod entity_deletion_list;
pub mod entity_id;
pub mod game_world;
pub mod hashing;
pub mod movement_event;
pub mod player;
pub mod position;
pub mod position_map;
pub mod raws;
pub mod registry;
pub mod resource;
pub mod server_request_type;
pub mod server_response_type;
pub mod uuid_map;
mod uuid_system;
