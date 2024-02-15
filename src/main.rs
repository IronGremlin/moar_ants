#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod ant;
mod colony;
mod food;
mod gamefield_ui;
mod gametimer;
mod gizmodable;
mod larva;
mod menu_ui;
mod misc_utility;
mod playerinput;
mod scentmap;
mod settings_menu;
mod spatial_helper;
mod ui_helpers;
mod upgrades;
mod credits_ui;


use std::time::Duration;

use ant::AntPlugin;
use bevy::audio::VolumeLevel;
use bevy::window::WindowMode;
use bevy::winit::WinitWindows;
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
use credits_ui::CreditsPlugin;
use food::FoodPlugin;
use gamefield_ui::GamefieldUI;
use gametimer::GameTimerPlugin;

use gizmodable::Gizmotastic;
use larva::LarvaPlugin;
use menu_ui::MainMenuUI;
use playerinput::PlayerInputPlugin;
use scentmap::ScentMapPlugin;
use settings_menu::SettingsMenuPlugin;
use upgrades::UpgradePlugin;

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
        .add_systems(
            Startup,
            (setup, populate_volume_settings_changes, play_music).chain(),
        )
        .add_systems(
            First,
            set_default_font.run_if(resource_exists::<DefaultFontHandle>()),
        )
        
        .add_systems(OnEnter(UIFocus::Gamefield), (start_game, flag_game_as_started.run_if(run_once())).chain())
        .add_systems(OnExit(UIFocus::Gamefield), pause_game)
        .add_systems(
            Update,
            (
                soundscape_processor,
                populate_volume_settings_changes.run_if(volume_changed),
                populate_display_settings_changes.run_if(display_changed),
            )
                .chain(),
        )
        .run();
}

#[derive(Resource, Default)]
pub struct GameStarted;


#[derive(Component, Default)]
pub struct SpatialMarker;

#[derive(Component, Default)]
pub struct AntSpatialMarker;

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
    Credits
}

#[derive(Event)]
pub enum SoundScape {
    AntDeath,
    FoodSpawn,
    FoodEmpty,
    AntBorn,
}
#[derive(Component)]
pub enum SoundType {
    Music,
    SFX,
}
#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct VolumeSettings {
    sfx_user_setting: f32,
    music_user_setting: f32,
    global_user_setting: f32,
    sfx: f32,
    music: f32,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct DisplaySettings {
    resolution: (f32, f32),
    fullscreen: bool,
}
impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            resolution: (1280., 720.),
            fullscreen: true,
        }
    }
}

fn rescale_volume_setting(setting: f32) -> f32 {
    // For some math reason I do not understand, the propotionate volume factor
    // sounds way better if you scale the range as a cubic fraction that gets doubled.
    let x = setting.clamp(0.0, 1.0);
    2.0 * x * x * x
}
impl VolumeSettings {
    fn refresh_volume_levels(&mut self) {
        self.sfx = rescale_volume_setting(self.sfx_user_setting);
        self.music = rescale_volume_setting(self.music_user_setting);
    }
    fn sfx_level(&self) -> f32 {
        self.sfx
    }
    fn music_level(&self) -> f32 {
        self.music
    }
}
impl Default for VolumeSettings {
    fn default() -> Self {
        let sfx_user_setting: f32 = 0.5;
        let music_user_setting: f32 = 0.5;
        let global_user_setting: f32 = 0.5;

        VolumeSettings {
            sfx_user_setting,
            music_user_setting,
            global_user_setting,
            // these values are derived on the first frame anyway so it doesn't matter.
            sfx: 0.0,
            music: 0.0,
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

// This basically is acting as a marker resource to let us know at startup that we've found our replacement default font asset.
#[derive(Resource)]
struct DefaultFontHandle(Handle<Font>);

#[derive(Component)]
pub struct MainMusicTrack;

fn setup(
    mut commands: Commands,
    mut q: Query<&mut Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    display_settings: Res<DisplaySettings>,
) {
    let new_default_font = asset_server.load("monogram.ttf");
    commands.insert_resource(DefaultFontHandle(new_default_font));

    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 800.0,
        min_height: 450.0,
    };
    camera.camera_2d.clear_color = ClearColorConfig::Custom(Color::BLACK);
    commands.spawn((camera, MainCamera));
    let mut win = q.single_mut();
    win.set_maximized(true);
    win.resolution = display_settings.resolution.into();
    win.mode = if display_settings.fullscreen {
        WindowMode::SizedFullscreen
    } else {
        WindowMode::Windowed
    };
    win.resizable = !display_settings.fullscreen
}

fn set_default_font(
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
                        2.0 * settings.sfx * rescale_volume_setting(settings.global_user_setting),
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
fn volume_changed(settings: Res<VolumeSettings>) -> bool {
    settings.is_changed()
}
fn display_changed(settings: Res<DisplaySettings>) -> bool {
    settings.is_changed()
}

fn populate_volume_settings_changes(
    mut settings: ResMut<VolumeSettings>,
    mut audio_sinks: Query<(&mut AudioSink, &SoundType)>,
) {
    settings.refresh_volume_levels();
    let chosen_global = rescale_volume_setting(settings.global_user_setting);
    audio_sinks.iter_mut().for_each(|(sink, kind)| {
        let vol = match kind {
            SoundType::Music => settings.music_level(),
            SoundType::SFX => 2.0 * settings.sfx_level(),
        };
        sink.set_volume(vol * chosen_global);
    });
}
fn populate_display_settings_changes(
    display_settings: Res<DisplaySettings>,
    winit_windows: NonSend<WinitWindows>,
    mut ui_scale: ResMut<UiScale>,
    mut q: Query<(Entity, &mut Window), With<PrimaryWindow>>,
) {
    let (entity, mut window) = q.single_mut();
    let screen_size = winit_windows
        .get_window(entity)
        .and_then(|our_window| our_window.current_monitor())
        .map(|monitor| monitor.size())
        .map(|size| (size.width, size.height));

    let targetx = display_settings.resolution.0;
    let targety = display_settings.resolution.1;

    if let Some((rx, _ry)) = screen_size {
        let nativex = rx as f32;
        let necessary_scale_factor = nativex as f64 / targetx as f64;
        let ui_scale_factor = targety as f64 / 720.;
        ui_scale.0 = ui_scale_factor;
        if necessary_scale_factor >= 1.0  && display_settings.fullscreen {
            window
                .resolution
                .set_scale_factor_override(Some(necessary_scale_factor));
        }
        if !display_settings.fullscreen {
            window
                .resolution
                .set_scale_factor_override(Some(1.0));
        }
    }

    window.resolution.set(targetx, targety);
    if display_settings.fullscreen {
        window.set_maximized(true);
    };

    window.mode = if display_settings.fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };
}
fn flag_game_as_started(world: &mut World) {
    world.init_resource::<GameStarted>();
}