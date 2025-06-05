use std::time::Duration;

use avian2d::{
    math::TAU,
    prelude::{
        AngularDamping, AngularVelocity, Collider, CollisionEventsEnabled, LinearVelocity,
        OnCollisionStart, RigidBody,
    },
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use rand::Rng;

use crate::{GameAssets, GameState, LoadedLevel, level::Level};

#[derive(Component)]
struct Player;

#[derive(Component)]
pub struct Asteroid;

#[derive(Component)]
struct Explotion(Timer);

#[derive(Resource)]
pub struct LivesRemanining(pub u32);

#[derive(InputContext)]
struct ShipController;

#[derive(Debug, InputAction)]
#[input_action(output = f32)]
struct Rotate;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Thrust;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct FireLaser;

pub fn game_plugin(app: &mut App) {
    app.add_input_context::<ShipController>()
        .add_systems(OnEnter(GameState::Game), display_level)
        .add_systems(
            Update,
            (tick_explotion, has_won, laser_range, closest, follow_player)
                .run_if(in_state(GameState::Game)),
        );
}

fn display_level(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    loaded_level: Res<LoadedLevel>,
    levels: Res<Assets<Level>>,
) {
    let level = levels.get(&loaded_level.level).unwrap();

    commands.insert_resource(LivesRemanining(level.lives - 1));

    spawn_player(&mut commands, &game_assets, Vec2::ZERO);

    let mut rng = rand::rng();

    for (x, y) in std::iter::repeat(())
        .filter_map(|_| {
            let x = rng.random_range(-(level.width as f32) / 2.0..(level.width as f32) / 2.0);
            let y = rng.random_range(-(level.height as f32) / 2.0..(level.height as f32) / 2.0);

            if Vec2::new(x, y).distance(Vec2::ZERO) < 200.0 {
                return None;
            }

            Some((x, y))
        })
        .take(level.asteroids as usize)
        .collect::<Vec<_>>()
    {
        commands.spawn((
            Sprite::from_image(game_assets.asteroid.clone()),
            Transform::from_xyz(x, y, 0.0),
            RigidBody::Dynamic,
            Collider::circle(50.0),
            LinearVelocity(Vec2::from_angle(
                rng.random_range(0.0..TAU) * rng.random_range(10.0..100.0),
            )),
            AngularVelocity(rng.random_range(-1.5..1.5)),
            Asteroid,
            StateScoped(GameState::Game),
        ));
    }
}

fn tick_explotion(
    mut commands: Commands,

    mut explotions: Query<(Entity, &mut Explotion, &Transform)>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut lives_remaining: ResMut<LivesRemanining>,
    game_assets: Res<GameAssets>,
) {
    for (entity, mut timer, transform) in explotions.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            if lives_remaining.0 == 0 {
                next_state.set(GameState::StartMenu);
            } else {
                commands.entity(entity).despawn();
                lives_remaining.0 -= 1;
                spawn_player(&mut commands, &game_assets, transform.translation.xy());
            }
        }
    }
}

fn spawn_player(commands: &mut Commands, game_assets: &GameAssets, position: Vec2) {
    let mut actions = Actions::<ShipController>::default();

    actions.bind::<Rotate>().to(Bidirectional {
        positive: KeyCode::KeyA,
        negative: KeyCode::KeyD,
    });

    actions.bind::<Thrust>().to(KeyCode::KeyW);
    actions.bind::<FireLaser>().to(KeyCode::Space);

    commands
        .spawn((
            Sprite::from_image(game_assets.player_ship.clone()),
            RigidBody::Dynamic,
            Collider::circle(40.0),
            AngularDamping(5.0),
            Player,
            Transform::from_translation(position.extend(0.0)),
            StateScoped(GameState::Game),
            children![(
                Sprite::from_image(game_assets.jets.clone()),
                Transform::from_xyz(0.0, -40.0, -1.0),
                Visibility::Hidden
            )],
            actions,
        ))
        .observe(rotate)
        .observe(thrust)
        .observe(thrust_stop)
        .observe(fire_laser)
        .observe(asteroid_collision);
}

fn rotate(
    trigger: Trigger<Fired<Rotate>>,
    mut player: Query<&mut AngularVelocity>,
    time: Res<Time>,
) -> Result {
    let fixed_rotation_rate = 0.2;
    let delta = time.delta().as_secs_f32();
    let rate = fixed_rotation_rate / (1.0 / (60.0 * delta));
    let mut angular_velocity = player.get_mut(trigger.target())?;

    angular_velocity.0 += trigger.value.signum() * rate;

    Ok(())
}

fn thrust(
    trigger: Trigger<Fired<Thrust>>,
    mut player: Query<(&Transform, &mut LinearVelocity, &Children)>,
    mut visibility: Query<&mut Visibility>,
) -> Result {
    let (transform, mut linear_velocity, children) = player.get_mut(trigger.target())?;

    linear_velocity.0 += transform.local_y().xy() * 2.0;
    linear_velocity.0 = linear_velocity.0.clamp_length_max(200.0);
    visibility
        .get_mut(children[0])?
        .set_if_neq(Visibility::Visible);

    Ok(())
}

fn thrust_stop(
    trigger: Trigger<Completed<Thrust>>,
    player: Query<&Children>,
    mut visibility: Query<&mut Visibility>,
) -> Result {
    let children = player.get(trigger.target())?;

    visibility
        .get_mut(children[0])?
        .set_if_neq(Visibility::Hidden);

    Ok(())
}

fn asteroid_collision(
    collision: Trigger<OnCollisionStart>,
    is_asteroid: Query<(), With<Asteroid>>,
    player: Query<&Transform>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) -> Result {
    if is_asteroid.get(collision.collider).is_ok() {
        let transform = player.get(collision.target())?;
        commands.spawn((
            Sprite::from_image(game_assets.explotion.clone()),
            (*transform).with_scale(Vec3::splat(0.2)),
            Explotion(Timer::from_seconds(1.0, TimerMode::Once)),
            StateScoped(GameState::Game),
        ));
        commands.entity(collision.target()).despawn();
        commands.entity(collision.collider).despawn();
    }
    Ok(())
}

#[derive(Component)]
struct Laser(Timer);

fn fire_laser(
    trigger: Trigger<Fired<FireLaser>>,
    player: Query<&Transform>,
    time: Res<Time>,
    mut commands: Commands,
    mut last_fired: Local<Duration>,
    game_assets: Res<GameAssets>,
) -> Result {
    let mut transform = *player.get(trigger.target())?;
    transform.translation += transform.local_y() * 40.0;
    transform.scale = Vec3::ONE / 2.0;

    if time.elapsed() > *last_fired + Duration::from_secs_f32(0.5) {
        commands
            .spawn((
                Sprite::from_image(game_assets.laser.clone()),
                transform,
                RigidBody::Dynamic,
                Collider::rectangle(4.0, 15.0),
                LinearVelocity(transform.local_y().xy() * 1000.0),
                Laser(Timer::from_seconds(1.0, TimerMode::Once)),
                CollisionEventsEnabled,
                StateScoped(GameState::Game),
            ))
            .observe(laser_attack);

        *last_fired = time.elapsed();
    }

    Ok(())
}

fn laser_attack(
    collision: Trigger<OnCollisionStart>,
    is_asteroid: Query<(), With<Asteroid>>,
    mut commands: Commands,
) {
    if is_asteroid.get(collision.collider).is_ok() {
        commands.entity(collision.collider).despawn();
        commands.entity(collision.target()).despawn();
    }
}

fn has_won(asteroids: Query<(), With<Asteroid>>, mut next_state: ResMut<NextState<GameState>>) {
    if asteroids.is_empty() {
        next_state.set(GameState::Won);
    }
}

fn laser_range(mut commands: Commands, mut lasers: Query<(Entity, &mut Laser)>, time: Res<Time>) {
    for (entity, mut laser) in lasers.iter_mut() {
        if laser.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn follow_player(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };
    camera_transform.translation = player_transform.translation;
}

fn closest(
    asteroids: Query<(Entity, &Transform), With<Asteroid>>,
    player: Query<&Transform, With<Player>>,
    mut gizmos: Gizmos,
    mut commands: Commands,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let player_position = player_transform.translation.xy();
    let Some((entity, nearest)) = asteroids.iter().reduce(|a, b| {
        if a.1.translation.xy().distance_squared(player_position)
            < b.1.translation.xy().distance_squared(player_position)
        {
            a
        } else {
            b
        }
    }) else {
        return;
    };

    let nearest_position = nearest.translation.xy();
    let distance = nearest_position - player_position;

    let direction = distance.normalize();
    if distance.length() > 1000.0 {
        commands
            .entity(entity)
            .insert(LinearVelocity(direction.normalize() * -100.0));
    }

    gizmos.arrow_2d(
        player_position + direction * 45.0,
        player_position + direction * 70.0,
        Color::hsl(0.0, 1.0, 0.5),
    );
}
