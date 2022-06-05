// We need our game states so we can check what state we are in and states to
// transition to.
use crate::game_state::{self, GameState};

// The Exit button is going to need to be able to close the game so we have to
// use `AppExit`.
use bevy::app::AppExit;

// A general purpose import for all of our Bevy needs.
use bevy::prelude::*;
use bevy::ui::Overflow::Visible;
use bevy::winit::WinitSettings;

struct MainMenuRoot {
    camera: Entity,
    ui_root: Entity,
}
pub struct MainMenuPlugin {}
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
            .insert_resource(WinitSettings::desktop_app())
            .add_system(button_system)
            .add_system_set(
                SystemSet::on_enter(game_state::GameState::MainMenu).with_system(setup_main_menu),
            )
            .add_system_set(
                SystemSet::on_exit(game_state::GameState::MainMenu).with_system(cleanup),
            );
    }
}
pub enum MainMenuButtons {
    Connect,
    Help,
    About,
    Quit,
}
#[derive(Component, Debug)]
pub enum MainMenuButtonActions {
    Connect,
    Help,
    About,
    Quit,
}
#[derive(Component, Debug, PartialEq)]
pub enum Highlighted {
    Highlighted,
    NotHighlighted,
}
#[derive(Component)]
pub struct MainMenu;
fn button_system(
    mut exit: EventWriter<AppExit>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<game_state::GameState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut UiImage,
            &mut Highlighted,
            &MainMenuButtonActions,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut image, mut highlighted, action) in interaction_query.iter_mut() {
        if *interaction == Interaction::Hovered {
            let image_path = match action {
                MainMenuButtonActions::Connect => "images/connect2.png",
                MainMenuButtonActions::Help => "images/help2.png",
                MainMenuButtonActions::Quit => "images/quit2.png",
                _ => unreachable!(),
            };
            *image = UiImage(asset_server.load(image_path));
        } else if *interaction == Interaction::None {
            let image_path = match action {
                MainMenuButtonActions::Connect => "images/connect.png",
                MainMenuButtonActions::Help => "images/help.png",
                MainMenuButtonActions::Quit => "images/quit.png",
                _ => unreachable!(),
            };
            *image = UiImage(asset_server.load(image_path));
        } else if *interaction == Interaction::Clicked {
            match action {
                MainMenuButtonActions::Connect => {
                }
                MainMenuButtonActions::Quit => {
                    exit.send(AppExit);
                }
                MainMenuButtonActions::Help => {
                    state
                        .set(game_state::GameState::HelpMenu)
                        .expect("Couldn't change the game state");
                }
                _ => unimplemented!(),
            }
        }
    }
}

/// Sets up the main menu by defining the layout, inserting the title text, and
/// spawning the buttons.
///
/// # Arguments
///
/// `commands` - Used to create the menu.
/// `asset_server` - Used to load our custom font.
/// `clear_color` - Used to create the solid background color for the main menu.
pub fn setup_main_menu(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut clear_color: ResMut<ClearColor>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
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
    let font: Handle<Font> = asset_server.load("fonts/LaverickRegular.ttf");
    let main_pic: Handle<Image> = asset_server.load("images/title.png");
    let uimg: UiImage = UiImage(main_pic);
    clear_color.0 = Color::BLACK;
    let ui_root = commands
        // This is where we're going to define the layout of the main menu.
        .spawn_bundle(main_bundle)
        .insert(MainMenu)
        // Next, we add in the title and buttons for the main menu.
        .with_children(|mut parent| {
            parent.spawn_bundle(ImageBundle {
                image: uimg,
                ..ImageBundle::default()
            });
        })
        .with_children(|mut parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    image: UiImage(asset_server.load("images/connect.png")),
                    style: button_style.clone(),
                    ..ButtonBundle::default()
                })
                .insert(MainMenuButtonActions::Connect)
                .insert(Highlighted::NotHighlighted);
        })
        .with_children(|mut parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    image: UiImage(asset_server.load("images/help.png")),
                    style: button_style.clone(),
                    ..ButtonBundle::default()
                })
                .insert(MainMenuButtonActions::Help)
                .insert(Highlighted::NotHighlighted);
        })
        .with_children(|mut parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    image: UiImage(asset_server.load("images/quit.png")),
                    style: button_style,
                    ..ButtonBundle::default()
                })
                .insert(MainMenuButtonActions::Quit)
                .insert(Highlighted::NotHighlighted);
        })
        .id();
    let camera = commands.spawn_bundle(UiCameraBundle::default()).id();
    commands.insert_resource(MainMenuRoot {
        camera: camera,
        ui_root: ui_root,
    })
    // Our buttons to spawn. This will show as an error until we define the
    // function but we'll do it next.
}
fn cleanup(mut commands: Commands, menu_data: Res<MainMenuRoot>) {
    commands.entity(menu_data.ui_root).despawn_recursive();
    commands.entity(menu_data.camera).despawn_recursive();
}
