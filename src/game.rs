use avian2d::{
    math::TAU,
    prelude::{AngularDamping, AngularVelocity, Collider, Collisions, LinearVelocity, RigidBody},
};
use bevy::prelude::*;
use bevy_enhanced_input::{
    events::{Completed, Fired},
    input_action,
    prelude::{Actions, InputAction, InputContext, InputContextAppExt},
    preset::Bidirectional,
};
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

pub fn game_plugin(app: &mut App) {
    app.add_input_context::<ShipController>()
        .add_systems(OnEnter(GameState::Game), display_level)
        .add_systems(Update, (tick_explotion).run_if(in_state(GameState::Game)));
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

fn control_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<
        (
            &Transform,
            &mut AngularVelocity,
            &mut LinearVelocity,
            &Children,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    mut visibility: Query<&mut Visibility>,
) -> Result {
    let Ok((player_transform, mut angular_velocity, mut linear_velocity, children)) =
        player.single_mut()
    else {
        // No player at the moment, skip control logic
        return Ok(());
    };

    let fixed_rotation_rate = 0.2;
    let rotation_rate = fixed_rotation_rate / (1.0 / (60.0 * time.delta().as_secs_f32()));

    if keyboard_input.pressed(KeyCode::KeyA) {
        angular_velocity.0 += rotation_rate;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        angular_velocity.0 -= rotation_rate;
    }
    if keyboard_input.pressed(KeyCode::KeyW) {
        let forward = player_transform.local_y();
        linear_velocity.0 += forward.xy() * 2.0;
        linear_velocity.0 = linear_velocity.0.clamp_length_max(300.0);
        visibility
            .get_mut(children[0])?
            .set_if_neq(Visibility::Visible);
    } else {
        visibility
            .get_mut(children[0])?
            .set_if_neq(Visibility::Hidden);
    }
    Ok(())
}

fn collision(
    collisions: Collisions,
    player: Query<(&Transform, Entity), With<Player>>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) -> Result {
    let Ok((transform, entity)) = player.single() else {
        return Ok(());
    };

    if collisions.collisions_with(entity).next().is_some() {
        commands.spawn((
            Sprite::from_image(game_assets.explotion.clone()),
            (*transform).with_scale(Vec3::splat(0.2)),
            Explotion(Timer::from_seconds(1.0, TimerMode::Once)),
            StateScoped(GameState::Game),
        ));
        commands.entity(entity).despawn();
    }

    Ok(())
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
        .observe(thrust_stop);
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
