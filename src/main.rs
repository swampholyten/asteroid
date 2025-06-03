use bevy::prelude::*;
use game::game_plugin;
use splash::splash_plugin;
use start_menu::menu_plugin;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
enum GameState {
    #[default]
    Splash,
    Game,
    StartMenu,
}

mod game;
mod splash;
mod start_menu;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Workshop".into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .enable_state_scoped_entities::<GameState>()
        .add_plugins((splash_plugin, menu_plugin, game_plugin))
        .run();
}

