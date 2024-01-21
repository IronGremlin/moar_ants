use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
use bevy::{math::Vec3Swizzles, render::texture::ImageLoaderSettings};
use bevy_rand::prelude::*;
use rand::prelude::*;

use bevy_prng::WyRand;

use crate::{ant::AntCommandsExt, colony::Colony, SimState, SpatialMarker};

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SimState::Playing),
            starting_spawner.run_if(run_once()),
        )
        .add_systems(Update, spawn_starting_ants);
    }
}

#[derive(Component)]
pub struct Spawner {
    pub colony: Entity,
}

#[derive(Component)]
pub struct Larva;
#[derive(Component)]
pub struct FreebieLarva;

#[derive(Component)]
pub struct StartingAnts(i32);

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
    commands.spawn((
        SpawnerBundle {
            spawner: Spawner {
                colony: q.get_single().unwrap(),
            },
            sprite: SpriteBundle {
                texture: assets.load_with_settings("spawner.png", |s: &mut ImageLoaderSettings| {
                    s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear())
                }),
                transform: Transform::from_xyz(0., 0., 0.1),
                ..default()
            },
            marker: SpatialMarker,
        },
        StartingAnts(25),
    ));
}

fn spawn_starting_ants(
    mut commands: Commands,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    q: Query<(Entity, &Spawner, &Transform, &StartingAnts)>,
) {
    for (ent, spawner, transform, starting_ants) in q.iter() {
        for _ in 1..starting_ants.0 {
            let offset_vec = random_offset_vec(&mut rng);
            let ant_pos = transform.translation.xy() + offset_vec;
            commands.spawn_ant(spawner.colony, ant_pos)
        }
        commands.entity(ent).remove::<StartingAnts>();
    }
}

fn random_offset_vec(rng: &mut ResMut<GlobalEntropy<WyRand>>) -> Vec2 {
    let rand_angle = rng.gen_range(0.002..TAU);
    let mut offset_vec = Vec2::from((f32::sin(rand_angle), f32::cos(rand_angle)));
    offset_vec *= rng.gen_range(2.0..40.0);
    offset_vec
}
