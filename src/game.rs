use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Asteroid;

pub fn game_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), display_level);
}

fn display_level(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::linear_rgb(1.0, 0.0, 0.0), Vec2::new(50.0, 80.0)),
        Player,
        StateScoped(GameState::Game),
    ));

    commands.spawn((
        Sprite::from_color(Color::linear_rgb(0.0, 0.0, 1.0), Vec2::new(100.0, 100.0)),
        Transform::from_xyz(300.0, -200.0, 0.0),
        Asteroid,
        StateScoped(GameState::Game),
    ));
}

fn control_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) -> Result {
    let mut player_transform = player.single_mut()?;

    let fixed_rotation_rate = 0.2;
    let rotation_rate = fixed_rotation_rate / (1.0 / 60.0 * time.delta().as_secs_f32());

    if keyboard_input.pressed(KeyCode::KeyA) {
        player_transform.rotate_z(rotation_rate);
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        player_transform.rotate_z(-rotation_rate);
    }
    Ok(())
}
