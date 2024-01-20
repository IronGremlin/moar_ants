use std::{
    f32::consts::{PI, TAU},
    marker::PhantomData,
    time::Duration,
};

use bevy::{
    ecs::{
        query::Has,
        system::{Command, SystemState},
    },
    math::Vec3Swizzles,
    reflect,
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rand::prelude::*;
use rand::prelude::*;

use bevy_prng::WyRand;

use crate::{
    colony::{AntCapacity, AntPopulation, Colony, LaborData, LaborPhase},
    food::{FoodDeltaEvent, FoodQuant},
    gametimer::{scaled_time, SimTimer, TickRate},
    gizmodable::{GizmoDrawOp, GizmoSystemSet, VisualDebug},
    misc_utility::NaNGuard,
    scentmap::{self, ScentMap, ScentSettings, ScentType, WeightType},
    spatial_helper::DistanceAwareQuery,
    spawner::Spawner,
    AntSpatialMarker, SoundScape, SpatialMarker,
};

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AntSettings>()
            .insert_resource(AntSettings::default())
            .add_systems(
                Update,
                (
                    task_idlers
                        .pipe(task_foragers)
                        .pipe(task_nursemaids)
                        .in_set(LaborPhase::Task),
                    (
                        idle_ant_behavior,
                        nursmaid_ant_behavior,
                        forager_ant_behavior,
                    )
                        .after(LaborPhase::Task),
                    (
                        forager_timer_reset,
                        forager_behavior_debug,
                        debug_ant_assignment,
                        nav_debug,
                    )
                        .before(GizmoSystemSet::GizmoQueueDraw),
                    (ant_i_gravity, navigate_move).chain(),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                ant_vagrancy_check.run_if(on_timer(Duration::from_secs_f32(0.25))),
            )
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

trait AssignmentCommand<X>
where
    X: Command,
{
    fn get_command(n: Entity) -> X;
}

struct Assign<T: Default> {
    entity: Entity,
    marker: PhantomData<T>,
}

#[derive(Component, Clone, Copy)]
pub enum ForagerAnt {
    Seeking,
    FollowingTrail,
    BringingHomeFood,
    GoingHomeEmpty,
}
impl Default for ForagerAnt {
    fn default() -> Self {
        Self::Seeking
    }
}

impl Command for Assign<ForagerAnt> {
    fn apply(self, world: &mut World) {
        let mut ent = world.entity_mut(self.entity);
        ent.remove::<NursemaidAnt>();
        ent.remove::<IdleAnt>();
        ent.insert(ForagerAnt::default());
    }
}

impl AssignmentCommand<Assign<ForagerAnt>> for ForagerAnt {
    fn get_command(n: Entity) -> Assign<ForagerAnt> {
        Assign {
            entity: n,
            marker: PhantomData,
        }
    }
}

#[derive(Component, Default)]
pub struct NursemaidAnt;
impl Command for Assign<NursemaidAnt> {
    fn apply(self, world: &mut World) {
        let mut ent = world.entity_mut(self.entity);
        ent.remove::<ForagerAnt>();
        ent.remove::<IdleAnt>();
        ent.insert(NursemaidAnt);
    }
}

impl AssignmentCommand<Assign<NursemaidAnt>> for NursemaidAnt {
    fn get_command(n: Entity) -> Assign<NursemaidAnt> {
        Assign {
            entity: n,
            marker: PhantomData,
        }
    }
}
#[derive(Component, Default)]
pub struct IdleAnt;
impl Command for Assign<IdleAnt> {
    fn apply(self, world: &mut World) {
        let mut ent = world.entity_mut(self.entity);
        ent.remove::<ForagerAnt>();
        ent.remove::<NursemaidAnt>();
        ent.insert(IdleAnt);
    }
}

impl AssignmentCommand<Assign<IdleAnt>> for IdleAnt {
    fn get_command(n: Entity) -> Assign<IdleAnt> {
        Assign {
            entity: n,
            marker: PhantomData,
        }
    }
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
const ANT_STARTING_CARRY_CAPACITY: i32 = 5;
const ANT_MOVE_SPEED: f32 = 5.0;
const ANT_SEC_PER_ROTATION: f32 = 5.0;
const ANT_I_GRAVITY_FACTOR: f32 = 25.0;
const ANT_I_GRAVITY_MAXIMUM: f32 = 50.0;

#[derive(Component)]
pub struct Lifespan;

//TODO - probably make this keyed to colony entity at some point
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct AntSettings {
    pub carry_capacity: i32,
    pub life_span: u64,
    pub ant_i_gravity: f32,
    pub ant_i_gravity_max: f32,
}
impl Default for AntSettings {
    fn default() -> Self {
        AntSettings {
            carry_capacity: ANT_STARTING_CARRY_CAPACITY,
            life_span: ANT_STARTING_MAX_AGE,
            ant_i_gravity: ANT_I_GRAVITY_FACTOR,
            ant_i_gravity_max: ANT_I_GRAVITY_MAXIMUM,
        }
    }
}

#[derive(Bundle)]
pub struct AntBundle {
    ant: Ant,
    nav: Navigate,
    dbg: VisualDebug,
    sprite: SpriteBundle,
    marker: AntSpatialMarker,
}
struct AntSpawn {
    colony_entity: Entity,
    home: Vec2,
}
impl Command for AntSpawn {
    fn apply(self, world: &mut World) {
        //mut q_colony: Query<(&AntCapacity, &mut AntPopulation), With<Colony>>
        let mut state: SystemState<(
            Commands,
            Res<AntSettings>,
            EventWriter<SoundScape>,
            Res<AssetServer>,
            Query<(&AntCapacity, &mut AntPopulation), With<Colony>>,
        )> = SystemState::from_world(world);
        let (mut commands, _ant_settings, mut soundscape, assets, mut q_colony) =
            state.get_mut(world);
        let (ant_cap, mut ant_pop) = q_colony.single_mut();
        if ant_pop.0 < ant_cap.0 {
            let mut pos = Transform::from_xyz(self.home.x, self.home.y, 2.);
            pos.scale = Vec3::from((0.4, 0.4, 1.0));
            let texture = assets.load("ant.png");
            let sprite = SpriteBundle {
                texture,
                transform: pos,
                ..default()
            };

            commands
                .spawn((
                    AntBundle {
                        ant: Ant {
                            colony: self.colony_entity,
                            home: self.home,
                        },
                        nav: Navigate {
                            max_speed: ANT_MOVE_SPEED,
                            max_radians_per_sec: TAU / ANT_SEC_PER_ROTATION,
                            move_to: None,
                        },
                        dbg: VisualDebug::default(),
                        sprite,
                        marker: AntSpatialMarker,
                    },
                    IdleAnt,
                    SimTimer::once_from(Duration::from_secs(120)),
                ))
                .with_children(|child_c| {
                    child_c.spawn((Carried, FoodQuant::empty()));
                });
            soundscape.send(SoundScape::AntBorn);
            ant_pop.0 += 1;
        }

        state.apply(world);
    }
}
pub trait AntCommandsExt {
    fn spawn_ant(&mut self, owning_colony: Entity, home: Vec2);
}
impl<'a, 'b> AntCommandsExt for Commands<'a, 'b> {
    fn spawn_ant(&mut self, owning_colony: Entity, home: Vec2) {
        self.add(AntSpawn {
            colony_entity: owning_colony,
            home,
        })
    }
}

fn navigate_move(
    mut q: Query<(&GlobalTransform, &mut Transform, &mut Navigate)>,
    game_time: Res<Time>,
    game_clock: Res<State<TickRate>>,
) {
    let frame_delta = scaled_time(game_clock.get(), game_time.delta())
        .as_secs_f32()
        .clamp(f32::EPSILON, 1.0);

    for (global_transform, mut transform, mut nav) in q.iter_mut() {
        if let Some(destination) = nav.move_to {
            let mut pos_2d = global_transform.translation().xy();
            let max_speed = destination.distance(pos_2d);
            let mut scaled_speed = (nav.max_speed * frame_delta).clamp(0.0, max_speed);
            let scaled_rot_speed = nav.max_radians_per_sec * frame_delta;

            let mut vec = (destination - pos_2d).normalize();
            let facing = (transform.rotation * Vec3::Y).xy();
            let angle_delta = vec.angle_between(facing);

            //If we're ~ one frame away just teleport there - this fixes a host of xeno's paradox type edge-cases.
            if destination.distance(pos_2d) <= (scaled_speed * 2.0) {
                transform.translation = Vec3::from((pos_2d, 1.));
                transform.rotate_local_axis(Vec3::Z, -angle_delta);
                nav.move_to = None;
                continue;
            }

            // Figure out if our destination is inside our turn radius
            let turn_radius = nav.max_speed / nav.max_radians_per_sec;
            let face_angle = Vec2::Y.angle_between(facing);
            //These should represent the respective centers of our left + right "deadzones"
            let left_void_center = (Vec2::from_angle(face_angle + PI) * turn_radius) + pos_2d;
            let right_void_center = (Vec2::from_angle(face_angle) * turn_radius) + pos_2d;

            //If our destination is within our deadzones, scale down our speed based on the arc we'd need to make to get there

            if destination.distance(left_void_center) < turn_radius
                || destination.distance(right_void_center) < turn_radius
            {
                scaled_speed = nav.max_radians_per_sec * (destination.distance(pos_2d) / 2.0)
                    / angle_delta.cos();
                scaled_speed = (scaled_speed * frame_delta).clamp(0.0, max_speed);
            }

            if f32::abs(angle_delta) > scaled_rot_speed {
                let adjusted_angle = -f32::signum(angle_delta) * scaled_rot_speed;
                transform.rotate_local_axis(Vec3::Z, adjusted_angle);
                vec = (transform.rotation * Vec3::Y).xy();
            } else {
                transform.rotate_local_axis(Vec3::Z, -angle_delta);
            }

            vec *= scaled_speed;
            pos_2d += vec;

            transform.translation = Vec3::from((pos_2d, 1.));
        }
    }
}
fn nav_debug(mut q: Query<(&Transform, &Navigate, &mut VisualDebug)>) {
    q.iter_mut().for_each(|(transform, nav, mut dbg)| {
        if let Some(destination) = nav.move_to {
            dbg.add(GizmoDrawOp::line(
                transform.translation.truncate(),
                destination,
                Color::GREEN,
            ));
        }
    });
}

fn ant_i_gravity(
    ant_locations: DistanceAwareQuery<AntSpatialMarker, &GlobalTransform, With<Ant>>,
    ant_settings: Res<AntSettings>,
    mut q: Query<(&mut Transform, &mut VisualDebug), With<Ant>>,
    time: Res<Time>,
) {
    q.iter_mut().for_each(|(mut transform, mut dbg)| {
        let mypos = transform.translation.xy();
        let nearby_ants = ant_locations.within_distance(mypos, 25.0);

        let mut count: f32 = 0.0;

        let delta = nearby_ants
            .map(|ant_transform| {
                count += 1.0;
                let ant_pos = ant_transform.translation().xy();
                mypos - ant_pos
            })
            .fold(Vec2::ZERO, |acc, n| acc + n);
        if count == 0.0 {
            return;
        }
        let magnitude = delta.distance(Vec2::ZERO);
        // magnitude = (magnitude * magnitude);
        // let mut scaled_magnitude = (magnitude * ant_settings.ant_i_gravity)
        //     .clamp(0.0, ant_settings.ant_i_gravity_max)
        //     * time.delta_seconds();
        let mut scaled_magnitude = (magnitude * ant_settings.ant_i_gravity)
            .clamp(0.0, ant_settings.ant_i_gravity_max)
            * time.delta_seconds();
        scaled_magnitude.if_nan(0.0);
        let mut vector = delta.normalize_or_zero() * scaled_magnitude;
        vector.if_nan(Vec2::ZERO);

        vector.if_nan(Vec2::ZERO);
        dbg.add(GizmoDrawOp::line(mypos, mypos + vector, Color::PURPLE));

        transform.translation = (mypos + vector).extend(transform.translation.z);
    })
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

fn select_random_pos_along_bearing(
    transform: &Transform,
    dest: Vec2,
    rng: &mut ResMut<GlobalEntropy<WyRand>>,
) -> Vec2 {
    let mypos = transform.translation.xy();
    let to_dest = (dest - mypos).normalize_or_zero();
    let mut base_point_towards_dest = mypos + (to_dest * 20.0);
    if mypos.distance(dest) < 20.0 || base_point_towards_dest.distance(dest) > mypos.distance(dest)
    {
        base_point_towards_dest = dest;
    }
    let upper_jitter_threshold = mypos.distance(dest).clamp(5.1, 10.0).nan_guard(5.1);
    let distance_jitter = rng.gen_range(5.0..upper_jitter_threshold);
    let directional_offset = rng.gen_range(0.0..TAU);

    let offset_vec = Vec2::from_angle(directional_offset) * distance_jitter;
    let mut target_pos = base_point_towards_dest + offset_vec;

    if (target_pos).distance(dest) > transform.translation.xy().distance(dest) {
        let all_ahead_full = transform.up().truncate().normalize_or_zero() * 7.0 + mypos;
        target_pos = all_ahead_full;
    }

    target_pos
}

fn ant_vagrancy_check(
    mut q: Query<(&mut Ant, &Transform)>,
    space: DistanceAwareQuery<SpatialMarker, &GlobalTransform, With<Spawner>>,
) {
    for (mut ant, ant_transform) in q.iter_mut() {
        let my_loc = ant_transform.translation.xy();
        //If we are already close enough, don't re-home.
        // This should prevent ants from pin-balling between spawners.
        if my_loc.distance(ant.home) <= 120.0 {
            continue;
        }
        let mut distance: f32 = -1.0;
        let mut nearest = Vec2::ZERO;
        for spawner_transform in space.within_distance(ant_transform.translation.xy(), 60.0) {
            let this_loc = spawner_transform.translation().xy();
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

struct AntVaccancies {
    nursemaids: i32,
    foragers: i32,
}
fn task_idlers(
    mut commands: Commands,
    labor_q: Query<(
        &LaborData<ForagerAnt>,
        &LaborData<NursemaidAnt>,
        &LaborData<IdleAnt>,
    )>,
    q: Query<Entity, (With<IdleAnt>, With<Ant>)>,
) -> Result<AntVaccancies, ()> {
    let (forager_stats, nursemaid_stats, _idle_stats) = labor_q.single();
    let mut vaccancies = AntVaccancies {
        nursemaids: nursemaid_stats.requested - nursemaid_stats.active,
        foragers: forager_stats.requested - forager_stats.active,
    };

    if vaccancies.nursemaids <= 0 && vaccancies.foragers <= 0 {
        return Ok(vaccancies);
    }
    q.iter().for_each(|n| {
        if vaccancies.nursemaids > 0 {
            commands.add(NursemaidAnt::get_command(n));
            vaccancies.nursemaids -= 1;
            return;
        }
        if vaccancies.foragers > 0 {
            commands.add(ForagerAnt::get_command(n));
            vaccancies.foragers -= 1;
            return;
        }
    });
    Ok(vaccancies)
}
fn task_foragers(
    In(res): In<Result<AntVaccancies, ()>>,
    mut commands: Commands,
    q: Query<(Entity, &ForagerAnt), With<Ant>>,
) -> Result<AntVaccancies, ()> {
    if let Ok(mut vaccancies) = res {
        if vaccancies.nursemaids <= 0 && vaccancies.foragers > 0 {
            return Ok(vaccancies);
        }
        q.iter().for_each(|(n, state)| match *state {
            ForagerAnt::Seeking | ForagerAnt::GoingHomeEmpty => {
                if vaccancies.nursemaids > 0 {
                    commands.add(NursemaidAnt::get_command(n));
                    vaccancies.nursemaids -= 1;
                }
                if vaccancies.foragers <= 0 {
                    commands.add(IdleAnt::get_command(n));
                    vaccancies.foragers += 1
                }
            }
            _ => {}
        });
        Ok(vaccancies)
    } else {
        Err(())
    }
}
fn task_nursemaids(
    In(res): In<Result<AntVaccancies, ()>>,
    mut commands: Commands,
    q: Query<Entity, (With<Ant>, With<NursemaidAnt>)>,
) {
    if let Ok(mut vaccancies) = res {
        if vaccancies.nursemaids >= 0 {
            return;
        }
        q.iter().for_each(|n| {
            if vaccancies.nursemaids < 0 {
                commands.add(IdleAnt::get_command(n));
                vaccancies.nursemaids += 1;
            }
        });
    }
}

fn idle_ant_behavior(
    mut q: Query<
        (&Ant, &GlobalTransform, &Transform, &mut Navigate),
        (With<IdleAnt>, Without<NursemaidAnt>, Without<ForagerAnt>),
    >,
    mut scentmap: ResMut<ScentMap>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    q.iter_mut()
        .for_each(|(ant, transform, local_transform, mut nav)| {
            if let Some(_) = nav.move_to {
                return;
            }
            let distance_home = transform.translation().xy().distance(ant.home);
            if distance_home >= 30.0 {
                if let Some(homebound_pos) = scentmap.strongest_smell_weighted(
                    10.0,
                    ScentType::AntSmell,
                    WeightType::CloserTo(ant.home),
                    transform,
                ) {
                    let home = ant.home;
                    let self_pos = transform.translation().xy();
                    let mut scent_vec = (home - homebound_pos).normalize_or_zero();
                    scent_vec *= 10.0;

                    let scent_dest = homebound_pos + scent_vec;
                    if ant.home.distance(scent_dest) < distance_home {
                        let mut dest_vec = (scent_dest - self_pos).normalize_or_zero();
                        dest_vec *= self_pos.distance(scent_dest) + 5.0;
                        let dest = self_pos + dest_vec;

                        //commands.entity(entity).insert(MoveTo(dest));
                        nav.move_to = Some(dest);
                    }
                } else {
                    let dest = select_random_pos_along_bearing(local_transform, ant.home, &mut rng);
                    nav.move_to = Some(dest);
                }
                return;
            }
            let new_pos = select_random_wander_pos(local_transform, &mut rng);
            nav.move_to = Some(new_pos);
        })
}
fn debug_ant_assignment(
    mut q: Query<
        (
            &mut VisualDebug,
            &GlobalTransform,
            Has<ForagerAnt>,
            Has<NursemaidAnt>,
            Has<IdleAnt>,
        ),
        With<Ant>,
    >,
) {
    q.iter_mut()
        .for_each(|(mut dbg, transform, is_forager, is_nursemaid, is_idle)| {
            if is_forager {
                dbg.add(GizmoDrawOp::circle(
                    transform.translation().xy(),
                    10.0,
                    Color::WHITE,
                ));
            }
            if is_nursemaid {
                dbg.add(GizmoDrawOp::circle(
                    transform.translation().xy(),
                    10.0,
                    Color::PINK,
                ));
            }
            if is_idle {
                dbg.add(GizmoDrawOp::circle(
                    transform.translation().xy(),
                    10.0,
                    Color::ORANGE,
                ));
            }
        });
}

fn nursmaid_ant_behavior(
    mut q: Query<
        (&Ant, &GlobalTransform, &Transform, &mut Navigate),
        (With<NursemaidAnt>, Without<IdleAnt>, Without<ForagerAnt>),
    >,
    mut scentmap: ResMut<ScentMap>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
) {
    //TODO - make this "real"
    q.iter_mut()
        .for_each(|(ant, transform, local_transform, mut nav)| {
            if let Some(_) = nav.move_to {
                return;
            }
            let distance_home = transform.translation().xy().distance(ant.home);
            if distance_home >= 45.0 {
                if let Some(homebound_pos) = scentmap.strongest_smell_weighted(
                    10.0,
                    ScentType::AntSmell,
                    WeightType::CloserTo(ant.home),
                    transform,
                ) {
                    let home = ant.home;
                    let self_pos = transform.translation().xy();
                    let mut scent_vec = (home - homebound_pos).normalize_or_zero();
                    scent_vec *= 10.0;

                    let scent_dest = homebound_pos + scent_vec;
                    if ant.home.distance(scent_dest) < distance_home {
                        let mut dest_vec = (scent_dest - self_pos).normalize_or_zero();
                        dest_vec *= self_pos.distance(scent_dest) + 5.0;
                        let dest = self_pos + dest_vec;

                        nav.move_to = Some(dest);
                    }
                } else {
                    let dest = select_random_pos_along_bearing(local_transform, ant.home, &mut rng);
                    nav.move_to = Some(dest);
                }
                return;
            }
            let new_pos = select_random_wander_pos(local_transform, &mut rng);
            nav.move_to = Some(new_pos);
        })
}

fn forager_ant_behavior(
    mut q: Query<
        (
            &Ant,
            &mut ForagerAnt,
            &GlobalTransform,
            &Transform,
            &mut Navigate,
            &mut SimTimer,
            &Children,
        ),
        (Without<IdleAnt>, Without<NursemaidAnt>),
    >,
    mut scentmap: ResMut<ScentMap>,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    ant_settings: Res<AntSettings>,
    mut foodevents: EventWriter<FoodDeltaEvent>,
    carried_q: Query<(Entity, &FoodQuant), (With<Parent>, With<Carried>)>,
    space: DistanceAwareQuery<
        SpatialMarker,
        (Entity, &GlobalTransform, &FoodQuant),
        (Without<Carried>, Without<Colony>),
    >,
) {
    q.iter_mut().for_each(
        |(ant, mut behavior, transform, local_transform, mut nav, seek_timer, children)| {
            let mypos = transform.translation().xy();
            if let Some(_) = nav.move_to {
                return;
            }

            let food_in_sight: Vec<(Entity, &GlobalTransform, &FoodQuant)> =
                space.within_distance(mypos, 60.0).collect();
            let food_nearby = food_in_sight.len() > 0;

            let mut move_to_nearest_chunk = || {
                let mut pos_of_nearest_chunk: Vec2 = mypos + Vec2::from((120.0, 120.0));
                let mut res_behavior: Option<ForagerAnt> = None;

                for (food_ent, food_xform, food_q) in food_in_sight.iter() {
                    let foodpos = food_xform.translation().xy();
                    if foodpos.distance(mypos) <= food_q.interaction_distance() {
                        for child in children.iter() {
                            if let Ok(_) = carried_q.get(*child) {
                                foodevents.send(FoodDeltaEvent {
                                    food_from: *food_ent,
                                    food_to: *child,
                                    requested: ant_settings.carry_capacity,
                                });

                                res_behavior = Some(ForagerAnt::BringingHomeFood);
                                nav.move_to = None;
                                return res_behavior;
                            }
                        }
                    } else {
                        let this_dist = mypos.distance(foodpos);
                        if this_dist < mypos.distance(pos_of_nearest_chunk) {
                            pos_of_nearest_chunk = foodpos;
                            nav.move_to = Some(pos_of_nearest_chunk);
                            res_behavior = Some(ForagerAnt::FollowingTrail);
                        }
                    }
                }
                res_behavior
            };

            match (*behavior, food_nearby) {
                (ForagerAnt::BringingHomeFood, _) | (ForagerAnt::GoingHomeEmpty, false) => {
                    if transform.translation().xy().distance(ant.home) <= 3.0 {
                        for child in children.iter() {
                            if let Ok((entity, carried_food)) = carried_q.get(*child) {
                                foodevents.send(FoodDeltaEvent {
                                    requested: carried_food.0,
                                    food_from: entity,
                                    food_to: ant.colony,
                                })
                            }
                        }

                        *behavior = ForagerAnt::default();

                        return;
                    }
                    if ant.home.distance(transform.translation().xy()) <= 60.0 {
                        nav.move_to = Some(ant.home);

                        return;
                    }
                    if let Some(homebound_pos) = scentmap.strongest_smell_weighted(
                        10.0,
                        ScentType::AntSmell,
                        WeightType::CloserTo(ant.home),
                        transform,
                    ) {
                        let self_pos = transform.translation().xy();
                        let mut scent_vec = (ant.home - homebound_pos).normalize_or_zero();
                        scent_vec *= 10.0;

                        let scent_dest = homebound_pos + scent_vec;

                        let mut dest_vec = (scent_dest - self_pos).normalize_or_zero();
                        dest_vec *= self_pos.distance(scent_dest) + 5.0;
                        let dest = self_pos + dest_vec;
                        if dest.distance(ant.home) < mypos.distance(ant.home) {
                            nav.move_to = Some(dest);
                            return;
                        }
                    }
                    // pretend we're facing home so that we get a random destination in a vaguely homeward direction.

                    let dest =
                        select_random_pos_along_bearing(&local_transform, ant.home, &mut rng);

                    nav.move_to = Some(dest);
                }
                (
                    ForagerAnt::Seeking | ForagerAnt::FollowingTrail | ForagerAnt::GoingHomeEmpty,
                    true,
                ) => {
                    if let Some(new_behavior) = move_to_nearest_chunk() {
                        *behavior = new_behavior;
                    }
                }
                (ForagerAnt::Seeking | ForagerAnt::FollowingTrail, false) => {
                    if let Some(outbound_pos) = scentmap.strongest_smell_weighted(
                        10.0,
                        ScentType::FoundFoodSmell,
                        WeightType::FurtherFrom(ant.home),
                        transform,
                    ) {
                        let home = ant.home;
                        let mut scent_vec = (outbound_pos - home).normalize_or_zero();
                        scent_vec *= 20.0;

                        let scent_dest = outbound_pos + scent_vec;
                        let mut dest_vec = (scent_dest - mypos).normalize_or_zero();
                        dest_vec *= mypos.distance(scent_dest) + 5.0;
                        let dest = mypos + dest_vec;
                        nav.move_to = Some(dest);

                        *behavior = ForagerAnt::FollowingTrail;

                        return;
                    }
                    //No food is in sight, and we don't smell anything.
                    *behavior = ForagerAnt::Seeking;

                    if seek_timer.time.finished() {
                        *behavior = ForagerAnt::GoingHomeEmpty;
                        return;
                    }

                    if let Some(pos) = scentmap.strongest_smell_weighted(
                        10.0,
                        ScentType::AntSmell,
                        WeightType::Unweighted,
                        transform,
                    ) {
                        let mut new_vec = pos - transform.translation().xy();
                        new_vec = new_vec.normalize();
                        new_vec *= -30.0;

                        let dest = new_vec + transform.translation().xy();
                        nav.move_to = Some(dest);

                        return;
                    }
                    let dest =
                        select_random_pos_along_bearing(&local_transform, ant.home, &mut rng);
                    nav.move_to = Some(dest);
                }
            }
        },
    )
}
fn forager_timer_reset(mut q: Query<(&ForagerAnt, &mut SimTimer)>) {
    q.iter_mut()
        .for_each(|(behavior, mut seek_timer)| match *behavior {
            ForagerAnt::Seeking => {}
            _ => {
                seek_timer.time.reset();
            }
        });
}
fn forager_behavior_debug(
    mut q: Query<(&ForagerAnt, &SimTimer, &GlobalTransform, &mut VisualDebug), With<Ant>>,
) {
    q.iter_mut()
        .for_each(|(behavior, seek_timer, transform, mut dbg)| {
            let mypos = transform.translation().xy();
            match *behavior {
                ForagerAnt::Seeking => {
                    dbg.add(GizmoDrawOp::circle(mypos, 5.0, Color::GREEN));
                    dbg.add(GizmoDrawOp::circle(
                        mypos,
                        ((120.0 - seek_timer.time.remaining_secs()) / 120.0) * 10.0,
                        Color::CYAN,
                    ));
                }
                ForagerAnt::FollowingTrail => dbg.add(GizmoDrawOp::circle(mypos, 5.0, Color::BLUE)),
                ForagerAnt::GoingHomeEmpty => dbg.add(GizmoDrawOp::circle(mypos, 5.0, Color::RED)),
                ForagerAnt::BringingHomeFood => {
                    dbg.add(GizmoDrawOp::circle(mypos, 5.0, Color::YELLOW))
                }
            };
        });
}

fn ant_stink(
    mut scentmap: ResMut<ScentMap>,
    settings: Res<ScentSettings>,
    q: Query<(Option<&ForagerAnt>, &Transform), With<Ant>>,
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
            Some(ForagerAnt::BringingHomeFood) => {
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
