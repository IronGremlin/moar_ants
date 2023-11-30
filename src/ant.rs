use std::{
    f32::consts::{PI, TAU},
    time::Duration, iter::{FlatMap, FilterMap},
};

use bevy::{math::Vec3Swizzles, ecs::query::{WorldQuery, ReadOnlyWorldQuery, ROQueryItem}};
use bevy::{gizmos, prelude::*, render::camera, time::common_conditions::on_timer};
use bevy_inspector_egui::egui::util::undoer::Settings;
use bevy_rand::prelude::*;
use bevy_spatial::SpatialAccess;
use rand::prelude::*;

use bevy_prng::WyRand;

use crate::{
    colony::{AntPopulation, Colony},
    food::{FoodDeltaEvent, FoodQuant},
    gametimer::{scaled_time, GameClock, SimTimer, TickRate},
    gizmodable::VisualDebug,
    main,
    scentmap::{self, ScentMap, ScentSettings, ScentType, WeightType},
    MainCamera, SimState, SpatialMarker, spawner::Spawner, SpatialIndex, SoundScape,
};

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(AntSettings::default())
        .add_systems(Update, (ant_get_pos, navigate_move).chain())
        .add_systems(Update, ant_vagrancy_check.run_if(on_timer(Duration::from_secs_f32(0.25))))
            //.add_systems(PreUpdate, cull_mortal_ants)
            .add_systems(
                Update,
                ant_stink.run_if(on_timer(Duration::from_secs_f32(0.25))),
            );
    }
}

#[derive(Component)]
pub struct Ant {
    colony: Entity,
    home: Vec2,
}

#[derive(Component, Default)]
pub enum AntBehavior {
    #[default]
    Wander,
    Gather,
    BringHomeFood,
}
#[derive(Component)]
pub struct Navigate {
    max_speed: f32,
    max_radians_per_sec: f32,
    move_to: Option<Vec2>,
}

// #[derive(Component)]
// pub struct MoveTo(Vec2);

#[derive(Component)]
pub struct Carried;

const ANT_STARTING_MAX_AGE: u64 = 240;
const ANT_STARTING_CARRY_CAPACITY : i32 = 5;
const ANT_MOVE_SPEED: f32 = 5.0;
const ANT_SEC_PER_ROTATION: f32 = 5.0;

#[derive(Component)]
pub struct Lifespan;

//TODO - probably make this keyed to colony entity at some point
#[derive(Resource)]
pub struct AntSettings {
    pub carry_capacity: i32,
    pub life_span: u64,
}
impl Default for AntSettings {
    fn default() -> Self {
        AntSettings { carry_capacity: ANT_STARTING_CARRY_CAPACITY, life_span: ANT_STARTING_MAX_AGE }
    }
}

#[derive(Bundle)]
pub struct AntBundle {
    ant: Ant,
    nav: Navigate,
    dbg: VisualDebug,
    behavior: AntBehavior,
    sprite: SpriteBundle,
    marker: SpatialMarker,
}

fn cull_mortal_ants(
    mut commands: Commands,
    mut sounds: EventWriter<SoundScape>,
    mortals: Query<(&Parent, &SimTimer), With<Lifespan>>,
    ant_q: Query<&Ant>,
    mut col_q: Query<&mut AntPopulation>,
) {
    for (parent, age) in mortals.iter() {
        if age.time.finished() {
            if let Ok(mut ant_pop) = ant_q
                .get(parent.get())
                .and_then(|ant| col_q.get_mut(ant.colony))
            {
                ant_pop.0 = (ant_pop.0 - 1).max(0);
            }
            sounds.send(SoundScape::AntDeath);
            commands.entity(parent.get()).despawn_recursive();
        }
    }
}

pub fn spawn_ant_at_pos(
    commands: &mut Commands,
    assets: &Res<AssetServer>,
    ant_settings: &Res<AntSettings>,
    col_ent: Entity,
    home: Vec2,
) {
    let mut pos = Transform::from_xyz(home.x, home.y, 1.);
    pos.scale = Vec3::from((0.4, 0.4, 1.0));
    let texture = assets.load("ant.png");
    let sprite = SpriteBundle {
        texture,
        transform: pos,
        ..default()
    };
    commands
        .spawn(AntBundle {
            ant: Ant {
                colony: col_ent,
                home,
            },
            nav: Navigate {
                max_speed: ANT_MOVE_SPEED,
                max_radians_per_sec: TAU / ANT_SEC_PER_ROTATION,
                move_to: None,
            },
            dbg: VisualDebug::default(),
            behavior: AntBehavior::default(),
            sprite,
            marker: SpatialMarker,
        })
        .with_children(|c_commands| {
            c_commands.spawn((
                Lifespan,
                SimTimer {
                    time: Timer::new(Duration::new(ant_settings.life_span,0), TimerMode::Once),
                },
            ));
        });
}

fn navigate_move(
    mut q: Query<(&mut Transform, &mut Navigate, &mut VisualDebug)>,
    game_time: Res<Time>,
    game_clock: Res<State<TickRate>>,
    mut gizmos: Gizmos,
) {
    let frame_delta = scaled_time(game_clock.get(), game_time.delta())
        .as_secs_f32()
        .clamp(f32::EPSILON, 1.0);

    for (mut transform, mut nav, mut dbg) in q.iter_mut() {
        if let Some(destination) = nav.move_to {
            let scaled_speed = nav.max_speed * frame_delta;
            let scaled_rot_speed = nav.max_radians_per_sec * frame_delta;

            let mut pos_2d = transform.translation.xy();
            //take pity on the turn radius
            if pos_2d.distance(destination) <= (scaled_speed * 2.0 * PI) {
                transform.translation = Vec3::from((destination, 1.));
                //gizmos.circle_2d(transform.translation.xy(), 10.0, Color::RED);
                nav.move_to = None;
                continue;
            }

            let mut vec = (destination - pos_2d).normalize();
            let facing = (transform.rotation * Vec3::Y).xy();
            let angle_delta = vec.angle_between(facing);

            if f32::abs(angle_delta) > scaled_rot_speed {
                let adjusted_angle = -f32::signum(angle_delta) * scaled_rot_speed;
                transform.rotate_local_axis(Vec3::Z, adjusted_angle);
                vec = (transform.rotation * Vec3::Y).xy();
            } else {
                transform.rotate_local_axis(Vec3::Z, -angle_delta);
            }

            vec *= scaled_speed;
            pos_2d += vec;
            //*dbg = VisualDebug::circle(transform.translation.xy(), 10., Color::GREEN);
            //gizmos.circle_2d(transform.translation.xy(), 10.0, Color::GREEN);

            transform.translation = Vec3::from((pos_2d, 1.));
        } else {
            //gizmos.circle_2d(transform.translation.xy(), 10.0, Color::RED);
        }
    }
}

fn select_random_wander_pos(
    transform: &Transform,
    rng: &mut ResMut<GlobalEntropy<WyRand>>,
) -> Vec2 {
    let angle = rng.gen_range(0.0..TAU);
    let rot = Quat::from_axis_angle(Vec3::Z, angle);
    let mut vector = rot * Vec3::Y;
    vector *= 10.;
    vector += transform.translation;
    Vec2 {
        x: vector.x,
        y: vector.y,
    }
}
fn select_random_pos_along_heading(
    transform: &Transform,
    rng: &mut ResMut<GlobalEntropy<WyRand>>,
) -> Vec2 {
    let adjustment = rng.gen_range(-PI / 2.0..PI / 2.0);
    let adj_vec = Vec2::from((adjustment.cos(), adjustment.sin()));
    let vec = (transform.rotation * Vec3::Y).xy();
    (adj_vec + vec).normalize() * 30.
}



fn ant_vagrancy_check(mut q: Query<(&mut Ant, &Transform)>, spawner_q: Query<&Transform,(With<Spawner>, With<SpatialMarker>)>, space: Res<SpatialIndex>) {
    for (mut ant, ant_transform) in q.iter_mut() {
        let my_loc = ant_transform.translation.xy();
        //If we are already close enough, don't re-home.
        // This should prevent ants from pin-balling between spawners.
        if my_loc.distance(ant.home) <= 120.0 {
            continue;
        } 
        let mut distance:f32 = -1.0;
        let mut nearest = Vec2::ZERO;
        for spawner_transform in 
        space.within_distance(ant_transform.translation.xy(), 60.0)
        .iter()
        .filter_map(|(_,x)| x.and_then(|n|spawner_q.get(n).ok() )) {
            let this_loc = spawner_transform.translation.xy();
            let this_dist = this_loc.distance(my_loc);
            distance = this_dist.max(distance);
            if this_dist == distance {
                nearest = this_loc;
            }
        }
        if distance >= 0.0 {
            ant.home = nearest;
        }
    }

}

fn ant_get_pos(
    mut commands: Commands,
    mut q: Query<
        (
            Entity,
            &Ant,
            &mut AntBehavior,
            &Transform,
            &mut Navigate,
            Option<&Children>,
            &mut VisualDebug,
        ),
        With<Ant>,
    >,
    mut scentmap: ResMut<ScentMap>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    ant_settings: Res<AntSettings>,
    mut foodevents: EventWriter<FoodDeltaEvent>,
    carried_q: Query<(Entity, &FoodQuant), (With<Parent>, With<Carried>)>,
    food_q: Query<(Entity, &Transform), (With<FoodQuant>,With<SpatialMarker>, Without<Carried>)>,
    space: Res<SpatialIndex>
) {
    let navspan = info_span!("ant_nav: nav function call");
    let outerguard = navspan.enter();

    let wanderspan = info_span!("ant_nav: wander");
    let gatherspan = info_span!("ant_nav: gather");
    let homeboundspan = info_span!("ant_nav: homebound");
    'ants: for (entity, ant, mut behavior, transform, mut nav, maybe_kids, mut dbg) in q.iter_mut()
    {
        if let Some(_) = nav.move_to {
            //gizmos.line_2d(transform.translation.xy(), navpos, Color::YELLOW);
            continue 'ants;
        }
        //*dbg = VisualDebug::circle(transform.translation.xy(), 10., Color::YELLOW);
        match *behavior {
            AntBehavior::Wander => {
                let guard = wanderspan.enter();
                let new_pos = select_random_wander_pos(transform, &mut rng);
                //info!("Selected random pos: {:?}", new_pos);
                //commands.entity(entity).insert(MoveTo(new_pos));
                nav.move_to = Some(new_pos);
                *behavior = AntBehavior::Gather;
                drop(guard);
                continue 'ants;
            }
            AntBehavior::Gather => {
                let guard = gatherspan.enter();
                //If we are standing atop food, pick it up and go home.
                //If we are close enough to the food to see it, go to the food.
                let mypos = transform.translation.xy();
                
                //TODO - in the somewhat unlikely event that multiple food items are within 60 units, this may not select the nearest of them - unsure if we need to care.

                for (food_ent, food_xform) in space.within_distance(mypos,60.0).iter().filter_map(|(_,n)| n.map(|x|food_q.get(x).ok())).flatten() {
                    
                    let foodpos = food_xform.translation.xy();
                    if foodpos == mypos {
                        // info!("Found Food, pickup");
                        let child = commands.spawn((Carried, FoodQuant::empty())).id();
                        commands.entity(entity).add_child(child);
                        foodevents.send(FoodDeltaEvent {
                            food_from: food_ent,
                            food_to: child,
                            requested: ant_settings.carry_capacity,
                        });

                        *behavior = AntBehavior::BringHomeFood;
                        drop(guard);
                        continue 'ants;
                    } else {
                        nav.move_to = Some(foodpos);
                        drop(guard);
                        continue 'ants;

                    }
                }
                
                if let Some(outbound_pos) = scentmap.strongest_smell_weighted(
                    10.0,
                    ScentType::FoundFoodSmell,
                    WeightType::FurtherFrom(ant.home),
                    transform,
                ) {
                    let home = ant.home;
                    let self_pos = transform.translation.xy();
                    let mut scent_vec = (outbound_pos - home).normalize_or_zero();
                    scent_vec *= 20.0;

                    let scent_dest = outbound_pos + scent_vec;
                    let mut dest_vec = (scent_dest - self_pos).normalize_or_zero();
                    dest_vec *= self_pos.distance(scent_dest) + 5.0;
                    let dest = self_pos + dest_vec;

                    //info!("smelled food, goto: {:?}", dest);
                    //commands.entity(entity).insert(MoveTo(dest));
                    nav.move_to = Some(dest);
                    drop(guard);
                    continue 'ants;
                }

                if let Some(pos) = scentmap.strongest_smell_weighted(
                    10.0,
                    ScentType::AntSmell,
                    WeightType::Unweighted,
                    transform,
                ) {
                    let mut new_vec = pos - transform.translation.xy();
                    new_vec = new_vec.normalize();
                    new_vec *= -30.0;

                    let dest = new_vec + transform.translation.xy();
                    //info!("Selected gather pos - stank {:?}, new pos {:?}", pos, dest);
                    //commands.entity(entity).insert(MoveTo(dest));
                    nav.move_to = Some(dest);
                    drop(guard);

                    continue 'ants;
                }
                let dest = select_random_pos_along_heading(transform, &mut rng);
                //info!("Selected random gather pos - new pos {:?}", dest);
                //commands.entity(entity).insert(MoveTo(dest));
                nav.move_to = Some(dest);
                continue 'ants;
            }
            AntBehavior::BringHomeFood => {
                let guard = homeboundspan.enter();
                if transform.translation.xy() == ant.home {
                    if let Some((c_food_ent, carried_food)) = maybe_kids
                        .map(|n| n.iter().find_map(|x| carried_q.get(*x).ok()))
                        .flatten()
                    {
                        foodevents.send(FoodDeltaEvent {
                            requested: carried_food.0,
                            food_from: c_food_ent,
                            food_to: ant.colony,
                        })
                    }

                    *behavior = AntBehavior::Wander;
                    drop(guard);
                    continue 'ants;
                }
                if ant.home.distance(transform.translation.xy()) <= 60.0 {
                    //commands.entity(entity).insert(MoveTo(Vec2::from((0., 0.))));
                    nav.move_to = Some(ant.home);
                    drop(guard);
                    continue 'ants;
                }
                if let Some(homebound_pos) = scentmap.strongest_smell_weighted(
                    10.0,
                    ScentType::AntSmell,
                    WeightType::CloserTo(ant.home),
                    transform,
                ) {
                    let home = ant.home;
                    let self_pos = transform.translation.xy();
                    let mut scent_vec = (home - homebound_pos).normalize_or_zero();
                    scent_vec *= 10.0;

                    let scent_dest = homebound_pos + scent_vec;
                    let mut dest_vec = (scent_dest - self_pos).normalize_or_zero();
                    dest_vec *= self_pos.distance(scent_dest) + 5.0;
                    let dest = self_pos + dest_vec;

                    //commands.entity(entity).insert(MoveTo(dest));
                    nav.move_to = Some(dest);
                    drop(guard);
                    continue 'ants;
                }
                //TODO - probably figure out some way to point vaguely towards home
                // commands
                //     .entity(entity)
                //     .insert(MoveTo(select_random_pos_along_heading(transform, &mut rng)));
                nav.move_to = Some(select_random_pos_along_heading(transform, &mut rng));
                drop(guard);
                continue 'ants;
            }
        }
    }
    drop(outerguard);
}

fn ant_stink(
    mut scentmap: ResMut<ScentMap>,
    settings: Res<ScentSettings>,
    q: Query<(&AntBehavior, &Transform), With<Ant>>,
) {
    let span = info_span!("ant stink system call");
    let _ = span.enter();
    let max_smell = settings.max_smell;
    let strength = settings.starting_strength;
    for (behavior, transform) in q.iter() {
        let innerspan = info_span!("ant stink loop iter");
        let _ = innerspan.enter();
        scentmap.log_scent(
            max_smell,
            transform,
            scentmap::ScentType::AntSmell,
            strength,
        );

        match behavior {
            AntBehavior::BringHomeFood => {
                scentmap.log_scent(
                    max_smell,
                    transform,
                    scentmap::ScentType::FoundFoodSmell,
                    strength,
                );
            }
            _ => {}
        }
    }
}
