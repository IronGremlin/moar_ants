use std::f32::consts::{PI, TAU};

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::prelude::*;

use bevy_prng::WyRand;

use crate::{
    ant::{self, AntSettings},
    colony::{AntCapacity, AntPopulation, Colony},
    gametimer::SimTimer,
    SimState, SpatialMarker, SoundScape,
};

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimState::Playing),
            starting_spawner.run_if(run_once()),
        )
        .add_systems(Update, (spawn_starting_ants, spawn_ants_at_interval));
    }
}

#[derive(Component)]
pub struct Spawner {
    pub colony: Entity,
}

#[derive(Component)]
pub struct SpawnPulse;

#[derive(Component)]
pub struct StartingAnts(i32);

#[derive(Component)]
pub struct HasFootPrint;

#[derive(Bundle)]
pub struct SpawnerBundle {
    spawner: Spawner,
    sprite: SpriteBundle,
    marker: SpatialMarker,
}

fn starting_spawner(
    mut commands: Commands,
    q: Query<Entity, With<Colony>>,
    assets: Res<AssetServer>,
) {
    commands
        .spawn((
            SpawnerBundle {
                spawner: Spawner {
                    colony: q.get_single().unwrap(),
                },
                sprite: SpriteBundle {
                    texture: assets.load("spawner.png"),
                    transform: Transform::from_xyz(0., 0., 1.),
                    ..default()
                },
                marker: SpatialMarker,
            },
            HasFootPrint,

            StartingAnts(10),
        ))
        .with_children(|c_commands| {
            c_commands.spawn((
                SpawnPulse,
                SimTimer {
                    time: Timer::from_seconds(10.0, TimerMode::Repeating),
                },
            ));
        });
}
pub fn spawn_spawner_at_pos(commands: &mut Commands,
    assets: &Res<AssetServer>,
    col_ent: Entity,
    home: Vec2,) {
        commands
        .spawn((
            SpawnerBundle {
                spawner: Spawner {
                    colony: col_ent,
                },
                sprite: SpriteBundle {
                    texture: assets.load("spawner.png"),
                    transform: Transform::from_xyz(home.x, home.y, 1.),
                    ..default()
                },
                marker: SpatialMarker,
            },
        ))
        .with_children(|c_commands| {
            c_commands.spawn((
                SpawnPulse,
                SimTimer {
                    time: Timer::from_seconds(10.0, TimerMode::Repeating),
                },
            ));
        });

    }

fn spawn_starting_ants(
    mut commands: Commands,
    assets: Res<AssetServer>,
    ant_settings: Res<AntSettings>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    q: Query<(Entity, &Spawner, &Transform, &StartingAnts)>,
    mut q_colony: Query<(&AntCapacity, &mut AntPopulation), With<Colony>>,
) {
    for (ent, spawner, transform, starting_ants) in q.iter() {
        for _ in 1..starting_ants.0 {
            if let Ok((ant_cap, mut ant_pop)) = q_colony.get_mut(spawner.colony) {
                if ant_pop.0 < ant_cap.0 {
                    ant_pop.0 += 1;
                    let offset_vec = random_offset_vec(&mut rng);
                    let ant_pos = transform.translation.xy() + offset_vec;
                    ant::spawn_ant_at_pos(&mut commands, &assets,&ant_settings, spawner.colony, ant_pos);
                }
            }
        }
        commands.entity(ent).remove::<StartingAnts>();
    }
}

fn spawn_ants_at_interval(
    mut commands: Commands,
    mut sounds: EventWriter<SoundScape>,
    assets: Res<AssetServer>,
    ant_settings: Res<AntSettings>,
    q_child: Query<(&Parent, &SimTimer), With<SpawnPulse>>,
    q_parent: Query<(&Spawner, &Transform)>,
    mut q_colony: Query<(&AntCapacity, &mut AntPopulation), With<Colony>>,
) {
    for (parent, spawntimer) in q_child.iter() {
        if spawntimer.time.finished() {
            if let Ok((spawner, transform)) = q_parent.get(parent.get()) {
                if let Ok((ant_cap, mut ant_pop)) = q_colony.get_mut(spawner.colony) {
                    if ant_pop.0 < ant_cap.0 {
                        ant_pop.0 += 1;
                        let pos = transform.translation.xy();
                        sounds.send(SoundScape::AntBorn);
                        ant::spawn_ant_at_pos(&mut commands, &assets, &ant_settings,spawner.colony, pos);
                    }
                }
            }
        }
    }
}

fn random_offset_vec(rng: &mut ResMut<GlobalEntropy<WyRand>>) -> Vec2 {
    let rand_angle = rng.gen_range(0.002..TAU);
    let mut offset_vec = Vec2::from((f32::sin(rand_angle), f32::cos(rand_angle)));
    offset_vec *= rng.gen_range(2.0..40.0);
    offset_vec
}
