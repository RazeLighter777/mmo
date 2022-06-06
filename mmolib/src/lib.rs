#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
pub mod block_type;
pub mod chunk;
pub mod component;
pub mod effect;
pub mod entity;
pub mod game_event;
pub mod hashing;
pub mod pos;
pub mod mass;
pub mod world;
pub mod chunk_loader;
pub mod generator;
pub mod raws;
pub mod registry;
pub mod resource;
pub mod server_request_type;
pub mod server_response_type;
