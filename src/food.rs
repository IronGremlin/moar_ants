use std::{f32::consts::TAU, time::Duration};

use bevy::{ecs::query::Has, math::Vec3Swizzles, prelude::*};
use bevy_prng::WyRand;
use bevy_rand::resource::GlobalEntropy;
use rand::prelude::*;

use crate::{
    ant::Carried,
    colony::{Colony, MaxFood},
    gametimer::SimTimer,
    misc_utility::remap,
    SimState, SoundScape, SpatialMarker,
};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FoodQuant>()
            .add_event::<FoodDeltaEvent>()
            .add_systems(
                OnEnter(SimState::Playing),
                spawn_first_chunk.run_if(run_once()),
            )
            .add_systems(
                Update,
                (
                    freebie_food_spawn.run_if(on_food_timer),
                    scale_food,
                    apply_sprite_to_carried,
                ),
            )
            .add_systems(PreUpdate, (cull_empty, process_food_delta).chain());
    }
}

#[derive(Component, Reflect)]
pub struct FoodQuant(pub i32);
impl FoodQuant {
    fn take_food(&mut self, to_quant: &mut FoodQuant, requested: i32, max: Option<i32>) {
        let max = if let Some(maximum) = max {
            maximum - to_quant.0
        } else {
            self.0
        };
        let actual = requested.clamp(0, max);

        self.0 -= actual;
        to_quant.0 += actual;
    }
    pub fn empty() -> FoodQuant {
        FoodQuant(0)
    }
    fn exclusion_distance(&self) -> f32 {
        BASELINE_EXCLUSION_DISTANCE * (self.0 / FOOD_CHUNK_MAX_STARTING_AMOUNT) as f32
    }
    pub fn interaction_distance(&self) -> f32 {
        (self.exclusion_distance() * 0.5).max(5.0)
    }
}

#[derive(Event)]
pub struct FoodDeltaEvent {
    pub requested: i32,
    pub food_from: Entity,
    pub food_to: Entity,
}
#[derive(Component)]
pub struct FoodSpawnTimer;

const FOOD_SPAWN_MIN_DIST: f32 = 80.0;
const FOOD_SPAWN_MAX_DIST: f32 = 600.0;
const FOOD_CHUNK_MAX_STARTING_AMOUNT: i32 = 1800;
const FOOD_CHUNK_MIN_STARTING_AMOUNT: i32 = 180;
const FREEBIE_FOOD_CAP: usize = 200;
const FREEBIE_FOOD_INTERVAL: u64 = 15;
const BASELINE_EXCLUSION_DISTANCE: f32 = 65.0;
const FOOD_MIN_SCALE: f32 = 0.125;

fn on_food_timer(q: Query<&SimTimer, With<FoodSpawnTimer>>) -> bool {
    if let Ok(food_timer) = q.get_single() {
        food_timer.time.finished()
    } else {
        false
    }
}

fn freebie_food_spawn(
    mut commands: Commands,
    mut sounds: EventWriter<SoundScape>,
    assets: Res<AssetServer>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    q: Query<(&GlobalTransform, &FoodQuant), Without<Carried>>,
) {
    if q.iter().len() >= FREEBIE_FOOD_CAP {
        return;
    }
    let mut pos: Option<Vec2> = None;
    let mut iters: u8 = 0;
    let food_q = FoodQuant(
        rng.gen_range(FOOD_CHUNK_MIN_STARTING_AMOUNT / 20..FOOD_CHUNK_MAX_STARTING_AMOUNT / 20)
            * 20,
    );

    let food_scale = food_q.0 as f32 / FOOD_CHUNK_MAX_STARTING_AMOUNT as f32;

    while pos.is_none() && iters < 50 {
        iters += 1;
        let random_angle = rng.gen_range(0.0..TAU);
        let try_pos = Vec2::from((random_angle.cos(), random_angle.sin()))
            * rng.gen_range(FOOD_SPAWN_MIN_DIST..FOOD_SPAWN_MAX_DIST);
        pos = if q.iter().all(|(xform, quant)| {
            xform.translation().xy().distance(try_pos)
                >= (food_q.exclusion_distance() + quant.exclusion_distance())
        }) {
            Some(try_pos)
        } else {
            None
        }
    }

    if let Some(food_pos) = pos {
        let texture = assets.load("food_chunk.png");
        sounds.send(SoundScape::FoodSpawn);
        let mut transform = Transform::from_xyz(food_pos.x, food_pos.y, 0.1);
        transform.scale = Vec3::from((food_scale, food_scale, 1.0));
        commands.spawn((
            food_q,
            SpriteBundle {
                texture,
                transform: transform,
                ..default()
            },
            SpatialMarker,
        ));
    }
}

fn spawn_first_chunk(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    let random_angle = rng.gen_range(0.0..TAU);
    let pos = Vec2::from((random_angle.cos(), random_angle.sin())) * FOOD_SPAWN_MIN_DIST;
    let texture = assets.load("food_chunk.png");
    commands.spawn((
        FoodQuant(FOOD_CHUNK_MAX_STARTING_AMOUNT),
        SpriteBundle {
            texture,
            transform: Transform::from_xyz(pos.x, pos.y, 0.1),
            ..default()
        },
        SpatialMarker,
    ));
    commands.spawn((
        SimTimer {
            time: Timer::new(
                Duration::new(FREEBIE_FOOD_INTERVAL, 0),
                TimerMode::Repeating,
            ),
        },
        FoodSpawnTimer,
    ));
}

fn scale_food(mut q: Query<(&mut Transform, &FoodQuant), (With<Sprite>, Without<Carried>)>) {
    q.iter_mut().for_each(|(mut transform, quant)| {
        let mut scale = quant.0 as f32 / FOOD_CHUNK_MAX_STARTING_AMOUNT as f32;
        scale = remap(0.0, 1.0, FOOD_MIN_SCALE, 1.0, scale);
        transform.scale = Vec3::from((scale, scale, 1.0));
    });
}
fn apply_sprite_to_carried(
    mut commands: Commands,
    q: Query<(Entity, &FoodQuant, Has<Sprite>), (With<Carried>, With<Parent>)>,
    assets: Res<AssetServer>,
) {
    q.iter().for_each(|(entity, food, sprite)| {
        if food.0 != 0 && !sprite {
            let scale = FOOD_MIN_SCALE * 1.5;
            let mut transform = Transform::from_xyz(0., 12.5, 0.1);
            transform.scale = Vec3::from((scale, scale, 1.0));
            let texture = assets.load("food_chunk.png");
            commands.entity(entity).insert((SpriteBundle {
                texture,
                transform,
                ..default()
            },));
            return;
        }
        if food.0 == 0 && sprite {
            commands.entity(entity).remove::<Sprite>();
            return;
        }
    })
}

fn cull_empty(
    mut commands: Commands,
    q: Query<(Entity, &FoodQuant), (Without<Colony>, Without<Carried>)>,
) {
    q.iter().for_each(|(entity, quant)| {
        if quant.0 <= 0 {
            commands.entity(entity).despawn();
        }
    });
}

fn process_food_delta(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut q: Query<(&mut FoodQuant, Option<&MaxFood>, Has<Carried>)>,
    mut food_events: EventReader<FoodDeltaEvent>,
) {
    for event in food_events.read() {
        if let Ok([(mut source_food, _, was_carried), (mut dest_food, maxfood, _)]) =
            q.get_many_mut([event.food_from, event.food_to])
        {
            source_food.take_food(&mut dest_food, event.requested, maxfood.map(|x| x.0));
            // If we are an ant carrying food, and we tried to drop it off but the destination was full
            if was_carried && source_food.0 > 0 {
                let scale = (source_food.0 as f32 / FOOD_CHUNK_MAX_STARTING_AMOUNT as f32)
                    .clamp(FOOD_MIN_SCALE, 1.0);
                let mut transform = Transform::from_xyz(0., 0., 0.1);
                transform.scale = Vec3::from((scale, scale, 1.0));
                let texture = assets.load("food_chunk.png");
                commands.spawn((
                    FoodQuant(source_food.0),
                    SpriteBundle {
                        texture,
                        transform,
                        ..default()
                    },
                ));
                commands.entity(event.food_from).despawn_recursive();
            }
        }
    }
}
