#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod ant;
mod app_settings;
mod colony;
mod food;
mod gametimer;
mod gizmodable;
mod larva;
mod misc_utility;
mod nav;
mod playerinput;
mod ui;

use std::time::Duration;

use ant::AntPlugin;
use app_settings::{AppSettingsPlugin, DisplaySettings, SoundType, VolumeSettings};
use bevy::audio::VolumeLevel;
use bevy::window::WindowMode;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, input::common_conditions::input_toggle_active,
    prelude::*, render::camera::ScalingMode, window::PrimaryWindow,
};
use bevy_inspector_egui::quick::*;
use bevy_nine_slice_ui::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::*;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use colony::ColonyPlugin;
use food::FoodPlugin;
use gametimer::GameTimerPlugin;
use gizmodable::Gizmotastic;
use larva::LarvaPlugin;
use nav::ScentMapPlugin;
use playerinput::PlayerInputPlugin;
use ui::{CreditsPlugin, GamefieldUI, MainMenuUI, SettingsMenuPlugin, UpgradePlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Moar Ants!".into(),
                        mode: WindowMode::BorderlessFullscreen,
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .build(),
        )
        .add_plugins(NineSliceUiPlugin::default())
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Grave)),
        )
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .register_type::<VolumeSettings>()
        .register_type::<DisplaySettings>()
        .add_state::<UIFocus>()
        .add_state::<SimState>()
        .add_event::<SoundScape>()
        .init_resource::<VolumeSettings>()
        .init_resource::<DisplaySettings>()
        .add_plugins(
            AutomaticUpdate::<SpatialMarker>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(Duration::from_secs_f32(0.5)),
        )
        .add_plugins(
            AutomaticUpdate::<AntSpatialMarker>::new().with_spatial_ds(SpatialStructure::KDTree2),
        )
        .add_plugins((
            AppSettingsPlugin,
            MainMenuUI,
            SettingsMenuPlugin,
            CreditsPlugin,
            GameTimerPlugin,
            PlayerInputPlugin,
        ))
        .add_plugins((
            Gizmotastic,
            ColonyPlugin,
            LarvaPlugin,
            AntPlugin,
            UpgradePlugin,
            ScentMapPlugin,
            FoodPlugin,
            GamefieldUI,
        ))
        .configure_sets(Startup, 
            (
                (InitializationPhase::LoadFont, InitializationPhase::LoadConfigurations),
                (InitializationPhase::InitializeDisplay,InitializationPhase::InitializeAudio)
        ).chain()
        )
        .add_systems(
            Startup,
            (
                load_custom_font.in_set(InitializationPhase::LoadFont),
                boot_camera.in_set(InitializationPhase::InitializeDisplay),
                play_music.in_set(InitializationPhase::InitializeAudio),

           )
                .chain(),
        )
        .add_systems(
            First,
            override_default_font.run_if(resource_exists::<DefaultFontHandle>()),
        )
        .add_systems(
            OnEnter(UIFocus::Gamefield),
            (start_game, flag_game_as_started.run_if(run_once())).chain(),
        )
        .add_systems(OnExit(UIFocus::Gamefield), pause_game)
        .add_systems(Update, (soundscape_processor,))
        .run();
}

#[derive(Resource, Default)]
pub struct GameStarted;

#[derive(Component, Default)]
pub struct SpatialMarker;

#[derive(Component, Default)]
pub struct AntSpatialMarker;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum InitializationPhase {
    LoadFont,
    LoadConfigurations,
    InitializeDisplay,
    InitializeAudio
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SimState {
    #[default]
    Paused,
    MenuOpenedWhilePaused,
    Playing,
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum UIFocus {
    #[default]
    NullFocus,
    MainMenu,
    Gamefield,
    SettingsMenu,
    Credits,
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

// This basically is acting as a marker resource to let us know at startup that we've found our replacement default font asset.
#[derive(Resource)]
struct DefaultFontHandle(Handle<Font>);

#[derive(Component)]
pub struct MainMusicTrack;

fn load_custom_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let new_default_font = asset_server.load("monogram.ttf");
    commands.insert_resource(DefaultFontHandle(new_default_font));
}
fn boot_camera(
    mut commands: Commands,
    mut q: Query<&mut Window, With<PrimaryWindow>>,
    display_settings: Res<DisplaySettings>,
) {
    

    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 800.0,
        min_height: 450.0,
    };
    camera.camera_2d.clear_color = ClearColorConfig::Custom(Color::BLACK);
    commands.spawn((camera, MainCamera));
    let mut win = q.single_mut();
    if display_settings.fullscreen {
        win.set_maximized(true);
    }
    win.resolution = display_settings.resolution.into();
    win.mode = if display_settings.fullscreen {
        WindowMode::SizedFullscreen
    } else {
        WindowMode::Windowed
    };
    win.resizable = !display_settings.fullscreen
}

fn override_default_font(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    font_handle: Res<DefaultFontHandle>,
) {
    if let Some(font) = fonts.remove(&font_handle.0) {
        fonts.insert(TextStyle::default().font, font);
        commands.remove_resource::<DefaultFontHandle>();
    }
}

fn start_game(mut sim_state: ResMut<NextState<SimState>>, current_sim_state: Res<State<SimState>>) {
    if matches!(current_sim_state.get(), SimState::MenuOpenedWhilePaused) {
        sim_state.set(SimState::Paused);
        return;
    }
    sim_state.set(SimState::Playing);
}
fn pause_game(mut sim_state: ResMut<NextState<SimState>>, current_sim_state: Res<State<SimState>>) {
    if matches!(current_sim_state.get(), SimState::Paused) {
        sim_state.set(SimState::MenuOpenedWhilePaused);
        return;
    }
    sim_state.set(SimState::Paused);
}
fn play_music(mut commands: Commands, assets: Res<AssetServer>, settings: Res<VolumeSettings>) {
    commands.spawn((
        AudioBundle {
            source: assets.load("Kevin MacLeod Limit 70.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: bevy::audio::Volume::Relative(VolumeLevel::new(settings.music)),
                ..default()
            },
            ..default()
        },
        SoundType::Music,
        MainMusicTrack,
    ));
}

fn soundscape_processor(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut sound_events: EventReader<SoundScape>,
    settings: Res<VolumeSettings>,
) {
    for event in sound_events.read().take(5) {
        let asset_path = match *event {
            SoundScape::AntBorn => "B_vib.wav",
            SoundScape::AntDeath => "Click.wav",
            SoundScape::FoodEmpty => "D_vib.wav",
            SoundScape::FoodSpawn => "G_vib.wav",
        };
        commands.spawn((
            AudioBundle {
                source: assets.load(asset_path),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::Relative(VolumeLevel::new(
                        2.0 * settings.sfx
                            * crate::app_settings::rescale_volume_setting(
                                settings.global_user_setting,
                            ),
                    )),
                    ..default()
                },
                ..default()
            },
            SoundType::SFX,
        ));
    }
    // 5 sounds in a 60th of a second is pretty intense TBH
    sound_events.clear();
}

fn flag_game_as_started(world: &mut World) {
    world.init_resource::<GameStarted>();
}
