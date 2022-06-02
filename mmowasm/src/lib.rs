#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
mod utils;
use std::sync::Arc;
use mmolib;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    let rg = mmolib::registry::RegistryBuilder::new().build();
    let rw = mmolib::raws::RawTree::new_empty();
    let ctxt = mmolib::context::Context {
        registry: &rg,
        raws: &rw,
    };
    let eb = mmolib::entity::EntityBuilder::new(&rg, Arc::new(ctxt));
    alert("Hello, {{project-name}}!");
}
