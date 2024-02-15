use std::time::Duration;

use bevy::prelude::*;

use crate::SimState;

#[derive(Resource)]
pub struct GameClock {
    pub delta: Duration,
}
impl Default for GameClock {
    fn default() -> Self {
        GameClock {
            delta: Duration::new(0, 0),
        }
    }
}

#[derive(Component, Clone)]
pub struct SimTimer {
    pub time: Timer,
}
impl SimTimer {
    pub fn once_from(duration: Duration) -> Self {
        SimTimer {
            time: Timer::new(duration, TimerMode::Once),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TickRate {
    Paused,
    #[default]
    X4,
}

pub struct GameTimerPlugin;

impl Plugin for GameTimerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameClock::default())
            .add_state::<TickRate>()
            .add_systems(OnEnter(SimState::Playing), start_sim)
            .add_systems(OnEnter(SimState::Paused), pause_sim)
            .add_systems(PreUpdate, tick_sim_timers);
    }
}

pub fn scaled_time(rate: &TickRate, duration: Duration) -> Duration {
    let scalar: u64 = match rate {
        TickRate::Paused => 0,
        _ => 4,
    };
    Duration::from_nanos((duration.as_nanos() as u64 * scalar) as u64)
}

pub fn pause_sim(mut rate: ResMut<NextState<TickRate>>) {
    info!("pausing simulation");
    rate.set(TickRate::Paused);
}

pub fn start_sim(mut rate: ResMut<NextState<TickRate>>) {
    info!("starting simulation");
    rate.set(TickRate::X4);
}

fn tick_sim_timers(
    time: Res<Time>,
    rate: Res<State<TickRate>>,
    mut simtimers: Query<&mut SimTimer>,
    mut game_time: ResMut<GameClock>,
) {
    let delta = scaled_time(rate.get(), time.delta());
    game_time.delta = delta;
    for mut simtimer in simtimers.iter_mut() {
        simtimer.time.tick(delta);
    }
}
