use std::time::Duration;

use bevy::prelude::*;

use crate::SimState;


#[derive(Resource)]
pub struct GameClock {
    pub delta: Duration,
    // We store this here so that the user can pause/unpause separately from incremently advancing our scalar all the way back down/up from 0.
    resume_speed: TickRate

}
impl Default for GameClock {
    fn default() -> Self {
        GameClock { delta: Duration::new(0, 0), resume_speed: TickRate::Standard }
    }
}

#[derive(Component,Clone)]
pub struct SimTimer {
    pub time: Timer
}
impl SimTimer {
   pub fn once_from(duration: Duration) -> Self {
        SimTimer {time : Timer::new(duration, TimerMode::Once)}
    }
    pub fn repeating_from(duration: Duration) -> Self {
        SimTimer {time : Timer::new(duration, TimerMode::Repeating)}
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum TickRate {
    Paused,
    Standard,
    X2,
    #[default]
    X4
}
impl TickRate {
    pub fn faster(&self) -> Self {
        match self {
            TickRate::Paused =>  TickRate::Standard,
            TickRate::Standard => TickRate::X2,
            TickRate::X2 => TickRate::X4,
            TickRate::X4 => TickRate::X4,
            _ => TickRate::Paused
        }
    }
    pub fn slower(&self) -> Self {
        match self {
            TickRate::Paused => TickRate::Paused,
            TickRate::Standard => TickRate::Paused,
            TickRate::X2 => TickRate::Standard,
            TickRate::X4 => TickRate::X2,
            _ => TickRate::Paused        
        }
    }

}

pub struct GameTimerPlugin;

const BASE_TICK_RATE: f32 = 1.0;


impl Plugin for GameTimerPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(GameClock::default())
        .add_state::<TickRate>()
        .add_systems(OnEnter(SimState::Playing),resume_when_sim_resumes)
        .add_systems(OnEnter(SimState::Paused),pause_when_sim_pauses)
        .add_systems(PreUpdate, tick_sim_timers);
        
    }
}

pub fn scaled_time(rate: &TickRate, duration: Duration ) -> Duration {
    let scalar :u64 = match rate {
        TickRate::Paused => 0,
        TickRate::Standard => 1,
        TickRate::X2 => 2,
        TickRate::X4 => 4
    };
    Duration::from_nanos((duration.as_nanos() as u64 * scalar) as u64)
}



fn pause_when_sim_pauses(mut rate : ResMut<NextState<TickRate>>, current_rate: Res<State<TickRate>>, mut timer: ResMut<GameClock>) {
    // we treat "paused" as a valid rate to record - this is intentional, because if the player were to open a menu or do anything else that causes out of band
    // sim halt while the sim clock is manually paused, we should not surprise the player by returning them to an unpaused sim once that state is exited.
    timer.resume_speed = current_rate.get().clone();
    rate.set(TickRate::Paused);
}

fn resume_when_sim_resumes(mut rate : ResMut<NextState<TickRate>>, timer: Res<GameClock>) {
    rate.set(timer.resume_speed.clone());
}


fn tick_sim_timers(time: Res<Time>, rate: Res<State<TickRate>>, mut simtimers : Query<&mut SimTimer>, mut game_time: ResMut<GameClock>) {
    let delta = scaled_time(rate.get(), time.delta());
    game_time.delta = delta;
    for mut simtimer in simtimers.iter_mut() {
        simtimer.time.tick(delta);
    } 
}



