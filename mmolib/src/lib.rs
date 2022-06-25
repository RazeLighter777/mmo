#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![allow(unused)]
#![deny(warnings)]
#![feature(future_join, future_poll_fn)]
pub mod block_type;
pub mod chunk;
pub mod chunk_generator;
pub mod chunk_map;
pub mod component;
pub mod effect;
pub mod entity;
pub mod game_world;
pub mod hashing;
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
