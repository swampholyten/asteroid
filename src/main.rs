use avian2d::{PhysicsPlugins, prelude::Gravity};
use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use game::game_plugin;
use hud::hud_plugin;
use level::level_loader_plugin;
use splash::splash_plugin;
use start_menu::menu_plugin;

use level::Level;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States, Default)]
enum GameState {
    #[default]
    Splash,
    Game,
    StartMenu,
    Won,
}

#[derive(Resource)]
struct GameAssets {
    player_ship: Handle<Image>,
    asteroid: Handle<Image>,
    jets: Handle<Image>,
    explotion: Handle<Image>,
}

#[derive(Resource)]
pub struct LoadedLevel {
    pub level: Handle<Level>,
}

mod game;
mod hud;
mod level;
mod splash;
mod start_menu;
mod won;

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
        .add_plugins((PhysicsPlugins::default(), EnhancedInputPlugin))
        .insert_resource(Gravity::ZERO)
        .add_plugins((
            splash_plugin,
            menu_plugin,
            game_plugin,
            level_loader_plugin,
            hud_plugin,
        ))
        .run();
}
