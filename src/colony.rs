use std::{f32::consts::TAU, marker::PhantomData};

use bevy::{prelude::*, utils::HashMap};
use bevy_prng::WyRand;
use bevy_rand::resource::GlobalEntropy;
use rand::Rng;

use crate::{
    ant::{Ant, AntCommandsExt, AntSettings, ForagerAnt, IdleAnt, NursemaidAnt},
    food::FoodQuant,
    gizmodable::{GizmoDrawOp, VisualDebug},
    larva::LarvaSettings,
    UIFocus,
};

pub struct ColonyPlugin;

#[derive(Component)]
pub struct UpgradeStringIndex {
    pub costs: HashMap<String, i32>,
}
impl UpgradeStringIndex {
    pub fn new() -> Self {
        UpgradeStringIndex {
            costs: HashMap::new(),
        }
    }
    pub fn increment_index(&mut self, upgrade: String) {
        self.costs.get_mut(&upgrade).map(|x| {
            *x += 1;
        });
    }
}

const STARTING_ANT_CAP: i32 = 35;
#[derive(Component)]
pub struct Colony;
#[derive(Component)]
pub struct ColonyPos(Vec2);
#[derive(Reflect, Component)]
pub struct AntCapacity(pub i32);
#[derive(Component, Reflect)]
pub struct AntPopulation(pub i32);

#[derive(Component)]
pub struct LarvaTarget(pub i32);

#[derive(Component, Reflect)]
pub struct MaxFood(pub i32);

#[derive(Component)]
pub struct StartingAnts(i32);

#[derive(Component, Default)]
pub struct LaborData<T: Component + Default> {
    marker: PhantomData<T>,
    pub requested: i32,
    pub active: i32,
}

#[derive(Bundle, Default)]
pub struct LaborStats {
    forager_stats: LaborData<ForagerAnt>,
    nursemaid_stats: LaborData<NursemaidAnt>,
    idle_stats: LaborData<IdleAnt>,
}

#[derive(Bundle)]
pub struct ColonyData {
    col: Colony,
    ant_cap: AntCapacity,
    ant_pop: AntPopulation,
    food: FoodQuant,
    labor_stats: LaborStats,
    target_number_of_larva: LarvaTarget,
    max_food: MaxFood,
    home: ColonyPos,
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum LaborPhase {
    TakeCensus,
    AssignRoles,
    Task,
}

impl Plugin for ColonyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AntPopulation>()
            .register_type::<AntCapacity>()
            .register_type::<MaxFood>()
            .add_systems(Startup, init_default_colony)
            .add_systems(
                OnEnter(UIFocus::Gamefield),
                spawn_starting_ants.run_if(run_once()),
            )
            .configure_sets(
                Update,
                (
                    LaborPhase::TakeCensus,
                    LaborPhase::AssignRoles,
                    LaborPhase::Task,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    labor_census.in_set(LaborPhase::TakeCensus),
                    request_nursemaids.before(LaborPhase::TakeCensus),
                ),
            );
    }
}

pub fn init_default_colony(mut commands: Commands) {
    commands.spawn((
        ColonyData {
            col: Colony,
            ant_cap: AntCapacity(STARTING_ANT_CAP),
            ant_pop: AntPopulation(0),
            food: FoodQuant::empty(),
            max_food: MaxFood(200),
            target_number_of_larva: LarvaTarget(1),
            labor_stats: LaborStats::default(),
            home: ColonyPos((0., 0.).into()),
        },
        StartingAnts(25),
        UpgradeStringIndex::new(),
        VisualDebug::from_persistent(GizmoDrawOp::circle(Vec2::ZERO, 30.0, Color::YELLOW)),
        Name::new("Player_Colony"),
    ));
}

fn labor_census(
    q: Query<(Option<&ForagerAnt>, Option<&NursemaidAnt>, Option<&IdleAnt>), With<Ant>>,
    ant_settings: Res<AntSettings>,
    mut col_q: Query<
        (
            &mut LaborData<ForagerAnt>,
            &mut LaborData<NursemaidAnt>,
            &mut LaborData<IdleAnt>,
            &FoodQuant,
            &MaxFood,
        ),
        With<Colony>,
    >,
) {
    let (mut forager_stats, mut nursemaid_stats, mut idle_stats, food, max_food) =
        col_q.single_mut();

    let (mut nursemaids, mut foragers, mut idle) = (0, 0, 0);
    q.iter().for_each(|n| match n {
        (Some(_), _, _) => foragers += 1,
        (_, Some(_), _) => nursemaids += 1,
        (_, _, Some(_)) => idle += 1,
        _ => {}
    });
    idle_stats.active = idle;
    nursemaid_stats.active = nursemaids;
    forager_stats.active = foragers;
    forager_stats.requested = (max_food.0 - food.0) / ant_settings.carry_capacity;
}

fn request_nursemaids(
    mut q: Query<(&LarvaTarget, &mut LaborData<NursemaidAnt>), With<Colony>>,
    larva_settings: Res<LarvaSettings>,
) {
    let (target, mut nursemaid_stats) = q.single_mut();
    nursemaid_stats.requested =
        (larva_settings.nursemaids_per_larva * target.0 as f32).round() as i32;
}
fn spawn_starting_ants(
    mut commands: Commands,
    mut rng: ResMut<GlobalEntropy<WyRand>>,
    assets: Res<AssetServer>,
    q: Query<(Entity, &StartingAnts)>,
) {
    commands.spawn(SpriteBundle {
        texture: assets.load("ant_hill.png"),
        transform: Transform::from_xyz(0., 0., 0.1),
        ..default()
    });
    for (ent, starting_ants) in q.iter() {
        for _ in 1..starting_ants.0 {
            let offset_vec = random_offset_vec(&mut rng);
            let ant_pos = Vec2::ZERO + offset_vec;
            commands.spawn_ant(ent, ant_pos)
        }
        commands.entity(ent).remove::<StartingAnts>();
    }
}

fn random_offset_vec(rng: &mut ResMut<GlobalEntropy<WyRand>>) -> Vec2 {
    let rand_angle = rng.gen_range(0.002..TAU);
    let mut offset_vec = Vec2::from((f32::sin(rand_angle), f32::cos(rand_angle)));
    offset_vec *= rng.gen_range(2.0..5.0);
    offset_vec
}
