use bevy::prelude::*;

use crate::{GameAssets, GameState, LoadedLevel};

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        player_ship: asset_server.load("playerShip1_green.png"),
        asteroid: asset_server.load("meteorBrown_big1.png"),
        jets: asset_server.load("fire07.png"),
        explotion: asset_server.load("explosion00.png"),
        laser: asset_server.load("laserRed07.png"),
    });

    commands.insert_resource(LoadedLevel {
        level: asset_server.load("level.bw"),
    });
}

pub fn splash_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Splash), (display_title, load_assets))
        .add_systems(Update, switch_to_menu.run_if(in_state(GameState::Splash)));
}

fn display_title(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                Text::new("Beavy Workshop"),
                TextFont {
                    font_size: 130.0,
                    ..default()
                },
            ),
            (
                Text::new("Beavy Workshop"),
                TextFont {
                    font_size: 130.0,
                    ..default()
                },
            )
        ],
        StateScoped(GameState::Splash),
    ));

    commands.insert_resource(SplashScreenTimer(Timer::from_seconds(2.0, TimerMode::Once)));
}

#[derive(Resource)]
struct SplashScreenTimer(Timer);

fn switch_to_menu(
    mut next: ResMut<NextState<GameState>>,
    mut timer: ResMut<SplashScreenTimer>,
    time: Res<Time>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        next.set(GameState::StartMenu);
    }
}
