use std::sync::Arc;

use crate::game_state::{self, GameState};
use bevy::{prelude::*, ui};
use fltk::{app, prelude::*, window::Window, dialog};

struct HelpMenuRoot {
    camera: Entity,
    ui_root: Entity,
}
enum HelpMenuOptions { 
    Back,
    Connect,
}
pub struct HelpMenuPlugin {}
impl Plugin for HelpMenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(
            SystemSet::on_enter(game_state::GameState::HelpMenu).with_system(setup_main_menu),
        )
        .add_system_set(
            SystemSet::on_exit(game_state::GameState::HelpMenu).with_system(cleanup),
        );
    }
}

pub fn setup_main_menu(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let back_button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: Rect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let ip_button_style = Style {
        size: Size::new(Val::Px(400.), Val::Px(65.0)),
        margin: Rect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let main_bundle: NodeBundle = NodeBundle {
        color: UiColor(Color::BLACK),
        style: Style {
            // We want the menu to take up 100% of the available width and
            // height. This means that on our 800x600 window, the menu will be
            // 800x600.
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            // Align the items in the main menu to the center both horizontally
            // and vertically.
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            // Use the default styles for everything else.
            ..Style::default()
        },
        ..NodeBundle::default()
    };
    let ui_root = commands.spawn_bundle(main_bundle)
    .with_children(|mut parent| {
        parent.spawn_bundle(ButtonBundle {
            image : UiImage(asset_server.load("images/connect.png")),
            style : ip_button_style.clone(),
            ..Default::default()
        });
    }).id();
    let camera = commands.spawn_bundle(UiCameraBundle::default()).id();
    commands.insert_resource(
        HelpMenuRoot {
            camera : camera,
            ui_root : ui_root
        }
    );
}

fn cleanup(mut commands: Commands, menu_data: Res<HelpMenuRoot>) {
    commands.entity(menu_data.ui_root).despawn_recursive();
    commands.entity(menu_data.camera).despawn_recursive();
}
