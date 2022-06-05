#![feature(const_type_name)]
#![feature(arbitrary_enum_discriminant)]
#![feature(scoped_threads)]
#![feature(nll)]
#![allow(unused)]
#![deny(warnings)]
use bevy::{prelude::*, window::WindowMode};
use clap::Parser;
use connection::Connection;
use fltk::dialog;
mod args;
mod connection;
mod help_menu;
mod game_state;
mod main_menu;
#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    let mut wind = dialog::input(0, 0, "Enter IP Address", "mo");
    let gs = game_state::GameState::MainMenu;
    bevy::app::App::new()
        .insert_resource(WindowDescriptor {
            title: "Mordax".to_owned(),
            transparent: true,
            ..default()
        })
        .add_state(gs)
        //.insert_resource(connection)
        .add_plugins(DefaultPlugins)
        .add_plugin(main_menu::MainMenuPlugin {})
        .add_plugin(help_menu::HelpMenuPlugin {})
        .run();
}
