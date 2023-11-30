use std::{f32::consts::TAU, time::Duration};

use bevy::{math::Vec3Swizzles, prelude::*, utils::HashMap, time::common_conditions::on_timer};
use bevy_prng::WyRand;
use bevy_rand::resource::GlobalEntropy;
use rand::prelude::*;

use crate::{SimState, gametimer::SimTimer, colony::Colony, SpatialMarker, SoundScape};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FoodDeltaEvent>()
            .add_systems(
                OnEnter(SimState::Playing),
                spawn_first_chunk.run_if(run_once()),
            )
            .add_systems(Update, freebie_food_spawn.run_if(on_food_timer()))
            
            .add_systems(PreUpdate, process_food_delta);
    }
}

#[derive(Component)]
pub struct FoodQuant(pub i32);
impl FoodQuant {
    fn take_food(&mut self, to_quant: &mut FoodQuant, requested: i32) {
        let actual = requested.clamp(0, self.0);

        self.0 -= actual;
        to_quant.0 += actual;
    }
    pub fn empty() -> FoodQuant {
        FoodQuant(0)
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
const FOOD_CHUNK_STARTING_AMOUNT: i32 = 1800;
const FREEBIE_FOOD_CAP :usize = 30;
const FREEBIE_FOOD_INTERVAL: u64 = 15; 

fn on_food_timer() -> impl Condition<()> {
    IntoSystem::into_system(|q: Query<&SimTimer, With<FoodSpawnTimer>>| {
        if let Ok(food_timer) = q.get_single() {
            food_timer.time.finished()
        } else {
            false
        }
    })
}

fn freebie_food_spawn(
    mut commands: Commands,
    mut sounds: EventWriter<SoundScape>,
    assets: Res<AssetServer>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    q: Query<&Transform, With<FoodQuant>>
) {

    if q.iter().len() >= FREEBIE_FOOD_CAP {
        return;
    }
    let mut pos: Option<Vec2> = None;
    let mut iters: u8 = 0;
    while pos.is_none() && iters < 50 {
        iters += 1;
        let random_angle = rng.gen_range(0.0..TAU);
        let try_pos = Vec2::from((random_angle.cos(), random_angle.sin()))
            * rng.gen_range(FOOD_SPAWN_MIN_DIST..FOOD_SPAWN_MAX_DIST);
        pos = if q
            .iter()
            .all(|xform| xform.translation.xy().distance(try_pos) >= 60.0)
        {
            Some(try_pos)
        } else {
            None
        }
    }
    if let Some(food_pos) = pos {
        let texture = assets.load("food_chunk.png");
        sounds.send(SoundScape::FoodSpawn);
        commands.spawn((
            FoodQuant(FOOD_CHUNK_STARTING_AMOUNT),
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(food_pos.x, food_pos.y, 1.),
                ..default()
            },
            SpatialMarker
        ));
    }
}

fn spawn_first_chunk(mut commands: Commands, assets: Res<AssetServer>, mut rng: ResMut<GlobalEntropy<WyRand>>) {
    let random_angle = rng.gen_range(0.0..TAU);
    let pos = Vec2::from((random_angle.cos(),random_angle.sin())) * FOOD_SPAWN_MIN_DIST;
    let texture = assets.load("food_chunk.png");
    commands.spawn((
        FoodQuant(FOOD_CHUNK_STARTING_AMOUNT),
        SpriteBundle {
            texture,
            transform: Transform::from_xyz(pos.x, pos.y, 1.),
            ..default()
        },
        SpatialMarker
    ));
    commands.spawn((SimTimer{ time: Timer::new(Duration::new(FREEBIE_FOOD_INTERVAL,0), TimerMode::Repeating)}, FoodSpawnTimer));
}


fn process_food_delta(
    mut commands: Commands,
    mut q: Query<(&mut FoodQuant, Option<&Parent>, Option<&Sprite>, Option<&mut Transform>, Option<&Colony>)>,
    mut sounds: EventWriter<SoundScape>,
    mut food_events: EventReader<FoodDeltaEvent>,
) {
    for event in food_events.iter() {
        
        if let Ok([(mut source_food, s_par,s_spr,s_xform,s_iscol), (mut dest_food, d_par,_,_, d_iscol)]) =
            q.get_many_mut([event.food_from, event.food_to])
        {
            let source_is_colony = s_iscol.is_some();
            let dest_is_colony = d_iscol.is_some();
            source_food.take_food(&mut dest_food, event.requested);
            //TODO - we should be handling cases other than transfer in other systems.
            if let Some(mut xform) = s_xform {
                if s_spr.is_some() {
                    let scale = source_food.0 as f32 / FOOD_CHUNK_STARTING_AMOUNT as f32;
                    xform.scale = Vec3::from((scale,scale, 1.0));
                }
            }

            //Let us try not to despawn the colony entity when we run out of food.
            if source_food.0 <= 0 && !source_is_colony {
                if let Some(parent) = s_par {
                    commands
                        .entity(parent.get())
                        .remove_children(&[event.food_from]);
                }
                commands.entity(event.food_from).despawn();
                //We're assuming if you have a sprite you're a food chunk in the world
                if s_spr.is_some() {
                    sounds.send(SoundScape::FoodEmpty);
                }
            }
            if dest_food.0 <= 0 && !dest_is_colony {
                if let Some(parent) = d_par {
                    commands
                        .entity(parent.get())
                        .remove_children(&[event.food_to]);
                }
                commands.entity(event.food_to).despawn();
            }
        }
    }
}
