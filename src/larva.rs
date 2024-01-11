use std::{cmp::Ordering, time::Duration};

use bevy::{
    ecs::system::{Command, SystemParam, SystemState},
    prelude::*,
    render::render_phase::PhaseItem,
};

use crate::{
    ant::{AntCommandsExt, NursemaidAnt},
    colony::{Colony, LaborData, LaborPhase, AntCapacity, AntPopulation},
    food::FoodQuant,
    gametimer::SimTimer,
};

pub struct LarvaPlugin;

impl Plugin for LarvaPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(LarvaSettings::default())
        .add_systems(Update, (set_larva_pop.after(LaborPhase::Task), larva_eat).chain());
    }
}

#[derive(Component)]
pub struct Larva {
    colony: Entity,
    growth: f32,
}

#[derive(Component)]
pub struct GrowthTimer;

struct NewLarva;
impl Command for NewLarva {
    fn apply(self, world: &mut World) {
        let mut state: SystemState<(Commands, Query<Entity, With<Colony>>, Res<LarvaSettings>)> = SystemState::from_world(world);
        let (mut commands,col_q, l_settings) = state.get_mut(world);
        commands
            .spawn(Larva {
                growth: 0.0,
                colony : col_q.single(),
            })
            .with_children(|c_commands| {
                c_commands.spawn((
                    GrowthTimer,
                    SimTimer {
                        time: Timer::new(
                            Duration::from_secs_f32(l_settings.secs_per_tick()),
                            TimerMode::Repeating,
                        ),
                    },
                ));
            });
            state.apply(world);
    }
}

#[derive(Resource)]
pub struct LarvaSettings {
    pub nursemaids_per_larva: f32,
    food_per_tick: i32,
    ticks_til_grown: i32,
    ticks_per_sec: f32,
}
impl Default for LarvaSettings {
    fn default() -> Self {
        LarvaSettings {
            nursemaids_per_larva: 5.0,
            food_per_tick: 1,
            ticks_til_grown: 20,
            ticks_per_sec: 0.25,
        }
    }
}
impl LarvaSettings {
    fn growth_per_tick(&self) -> f32 {
        (self.ticks_til_grown as f32).recip()
    }
    fn secs_per_tick(&self) -> f32 {
        self.ticks_per_sec.recip()
    }
    fn food_per_cycle(&self) -> i32 {
        self.food_per_tick * self.ticks_til_grown
    }
}

#[derive(SystemParam)]
struct NurseableLarva<'w, 's> {
    larva_settings: Res<'w, LarvaSettings>,
    labor_query: Query<'w, 's, &'static LaborData<NursemaidAnt>>,
}
impl<'w, 's> NurseableLarva<'w, 's> {
    fn get(&self) -> i32 {
        let mut res = 0;
        self.labor_query.iter().for_each(|n| {
            res += (n.active as f32 / self.larva_settings.nursemaids_per_larva).trunc() as i32;
        });
        res
    }
}

fn larva_eat(
    mut commands: Commands,
    l_settings: Res<LarvaSettings>,
    mut p_q: Query<&mut Larva, With<Children>>,
    t_q: Query<(&SimTimer, &Parent), With<GrowthTimer>>,
    mut col_q: Query<(&mut FoodQuant, &AntCapacity, &AntPopulation), With<Colony>>,
) {
    let (mut food, ant_cap, ant_pop) = col_q.single_mut();
    t_q.iter().for_each(|(timer, parent_entity)| {
        if timer.time.finished() {
            if let Ok(mut larva) = p_q.get_mut(parent_entity.get()) {
                if food.0 > l_settings.food_per_tick &&  ant_pop.0  < ant_cap.0 {
                    food.0 -= l_settings.food_per_tick;
                    larva.growth += l_settings.growth_per_tick();
                    if larva.growth >= 1.0 {
                        larva.growth = (larva.growth - 1.0).trunc();
                        commands.spawn_ant(larva.colony, Vec2::ZERO)
                    }
                }
            }
        }
    })
}

fn set_larva_pop(
    mut commands: Commands,
    current_larva_cap: NurseableLarva,
    larva_q: Query<(Entity, &Larva)>,
) {
    let larva_cap = current_larva_cap.get();
    let larva_pop = larva_q.iter().len() as i32;
    let mut delta = larva_cap - larva_pop;
    match delta.signum() {
        1 => {
            while delta > 0 {
                commands.add(NewLarva);
                delta = -1;
            }
        }
        -1 => {
            let mut l_vec = larva_q.iter().collect::<Vec<(Entity, &Larva)>>();
            l_vec.sort_by(|(_, a), (_, b)| {
                a.growth.partial_cmp(&b.growth).unwrap_or(Ordering::Equal)
            });

            l_vec
                .iter()
                .take(delta.abs() as usize)
                .for_each(|(entity, _)| {
                    commands.entity(*entity).despawn_recursive();
                });
        }
        _ => {}
    }
}
