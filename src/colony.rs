use bevy::prelude::*;

use crate::{UIFocus, food::FoodQuant, ant::Ant, upgrades::UpgradeStringIndex};



pub struct ColonyPlugin;

const STARTING_ANT_CAP: i32 =  20;
#[derive(Component)]
pub struct Colony;
#[derive(Component)]
pub struct ColonyPos(Vec2);
#[derive(Component)]
pub struct AntCapacity(pub i32);
#[derive(Component)]
pub struct AntPopulation(pub i32);

#[derive(Bundle)]
pub struct ColonyData {
    col: Colony,
    ant_cap: AntCapacity,
    ant_pop: AntPopulation,
    food: FoodQuant,
    home: ColonyPos
}



impl Plugin for ColonyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup,init_default_colony.run_if(run_once()));
    }
}

pub fn init_default_colony(mut commands : Commands) {
    commands.spawn((ColonyData {
        col: Colony,
        ant_cap: AntCapacity(STARTING_ANT_CAP),
        ant_pop: AntPopulation(0),
        food: FoodQuant::empty(),
        home: ColonyPos((0.,0.).into())
    }, UpgradeStringIndex::new()));
}

