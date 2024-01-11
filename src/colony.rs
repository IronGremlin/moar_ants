use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    ant::{Ant, AntSettings, ForagerAnt, IdleAnt, NursemaidAnt},
    food::FoodQuant,
    larva::LarvaSettings,
    upgrades::UpgradeStringIndex,
    UIFocus, gizmodable::{VisualDebug, GizmoDrawOp},
};

pub struct ColonyPlugin;

const STARTING_ANT_CAP: i32 = 35;
#[derive(Component)]
pub struct Colony;
#[derive(Component)]
pub struct ColonyPos(Vec2);
#[derive(Reflect,Component)]
pub struct AntCapacity(pub i32);
#[derive(Component, Reflect)]
pub struct AntPopulation(pub i32);

#[derive(Component)]
pub struct LarvaTarget(pub i32);

#[derive(Component,Reflect)]
pub struct MaxFood(pub i32);

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
        .add_systems(Startup, init_default_colony.run_if(run_once()))
            .configure_sets(
                Update,
                (LaborPhase::TakeCensus, LaborPhase::AssignRoles, LaborPhase::Task).chain(),
            )
            .add_systems(
                Update,
                (
                    labor_census.in_set(LaborPhase::TakeCensus),
                    request_nursemaids.in_set(LaborPhase::AssignRoles),
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
        UpgradeStringIndex::new(),
        VisualDebug::from_persistent(GizmoDrawOp::circle(Vec2::ZERO, 30.0, Color::YELLOW)),        
        Name::new("Player_Colony")
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
