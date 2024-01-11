#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod ant;
mod colony;
mod food;
mod gamefield_ui;
mod gametimer;
mod gizmodable;
mod larva;
mod menu_ui;
mod playerinput;
mod scentmap;
mod spatial_helper;
mod spawner;
mod ui_helpers;
mod upgrades;

use std::time::Duration;

use ant::AntPlugin;
use bevy::audio::VolumeLevel;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, input::common_conditions::input_toggle_active,
    prelude::*, render::camera::ScalingMode, window::PrimaryWindow,
};
use bevy_inspector_egui::quick::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use colony::ColonyPlugin;
use food::FoodPlugin;
use gamefield_ui::GamefieldUI;
use gametimer::GameTimerPlugin;
use gizmodable::Gizmotastic;
use larva::LarvaPlugin;
use menu_ui::MainMenuUI;
use playerinput::PlayerInputPlugin;
use scentmap::ScentMapPlugin;
use spawner::SpawnerPlugin;
use upgrades::UpgradePlugin;
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "My Ant Sim".into(),
                        resizable: false,
                        resolution: (800.0, 600.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Grave)),
        )
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_state::<UIFocus>()
        .add_state::<SimState>()
        .add_plugins(
            AutomaticUpdate::<SpatialMarker>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(Duration::from_secs_f32(0.5)),
        )
        .add_plugins((MainMenuUI, GameTimerPlugin, PlayerInputPlugin))
        .add_plugins((
            Gizmotastic,
            ColonyPlugin,
            LarvaPlugin,
            AntPlugin,
            UpgradePlugin,
            ScentMapPlugin,
            FoodPlugin,
            SpawnerPlugin,
            GamefieldUI,
        ))
        .add_event::<SoundScape>()
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(UIFocus::Gamefield),
            (start_sim, play_music.run_if(run_once())),
        )
        .add_systems(OnEnter(UIFocus::MainMenu), pause_sim)
        .add_systems(
            Update,
            soundscape_processor.run_if(in_state(UIFocus::Gamefield)),
        )
        .run();
}
#[derive(Component, Default)]
pub struct SpatialMarker;
type SpatialIndex = KDTree2<SpatialMarker>;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimState {
    #[default]
    Paused,
    Playing,
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum UIFocus {
    #[default]
    MainMenu,
    Gamefield,
}

#[derive(Event)]
pub enum SoundScape {
    AntDeath,
    FoodSpawn,
    FoodEmpty,
    AntBorn,
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct MainMusicTrack;

fn setup(mut commands: Commands, mut q: Query<&mut Window, With<PrimaryWindow>>) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 800.0,
        min_height: 600.0,
    };
    camera.camera_2d.clear_color = ClearColorConfig::Custom(Color::Rgba {
        red: 0.,
        green: 0.,
        blue: 0.,
        alpha: 1.,
    });
    commands.spawn((camera, MainCamera));
    let mut win = q.single_mut();
    win.set_maximized(true);
}

fn start_sim(mut sim_state: ResMut<NextState<SimState>>) {
    sim_state.set(SimState::Playing);
}
fn pause_sim(mut sim_state: ResMut<NextState<SimState>>) {
    sim_state.set(SimState::Paused);
}
fn play_music(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: assets.load("Limit 70.mp3"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::Relative(VolumeLevel::new(0.2f32)),
                ..default()
            },
            ..default()
        },
        MainMusicTrack,
    ));
}

fn soundscape_processor(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut money_events: EventReader<SoundScape>,
) {
    for event in money_events.iter() {
        let asset_path = match *event {
            SoundScape::AntBorn => "B_vib.wav",
            SoundScape::AntDeath => "Click.wav",
            SoundScape::FoodEmpty => "D_vib.wav",
            SoundScape::FoodSpawn => "G_vib.wav",
        };
        commands.spawn((AudioBundle {
            source: assets.load(asset_path),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Relative(VolumeLevel::new(0.3f32)),
                ..default()
            },
            ..default()
        },));
    }
}

fn my_cursor_system(
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();
    camera
        .viewport
        .as_ref()
        .map(|vp| eprintln!("View size: {}", vp.physical_size));

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    if let Some(cursor) = window.cursor_position() {
        if let Some(pos) = camera.viewport_to_world_2d(camera_transform, cursor) {
            eprintln!("World coords: {}/{}", pos.x, pos.y);
            eprintln!("screen coords: {}/{}", cursor.x, cursor.y);
        }
    }
}
