use bevy::ecs::system::{Resource, SystemParam};
use bevy::window::WindowMode;
use bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows};
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::InitializationPhase;

pub struct AppSettingsPlugin;
impl Plugin for AppSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                initialize_persistent_app_settings,
                apply_deferred,
                load_app_settings,
                apply_deferred,
            )
                .chain()
                .in_set(InitializationPhase::LoadConfigurations),
        )
        .add_systems(
            Update,
            (
                populate_volume_settings_changes.run_if(volume_changed),
                populate_display_settings_changes.run_if(display_changed),
            ),
        );
    }
}

#[derive(Resource, Serialize, Deserialize)]
#[serde(tag = "version")]
enum UserSettings {
    V1 {
        global_volume: f32,
        sfx_volume: f32,
        music_volume: f32,

        resolution: (f32, f32),
        fullscreen: bool,

        display_first_time_help: bool,
    },
}
impl UserSettings {
    fn volume_settings(&self) -> VolumeSettings {
        match self {
            Self::V1 {
                global_volume,
                sfx_volume,
                music_volume,
                ..
            } => {
                let mut settings = VolumeSettings {
                    global_user_setting: *global_volume,
                    sfx_user_setting: *sfx_volume,
                    music_user_setting: *music_volume,
                    ..default()
                };
                settings.refresh_volume_levels();
                settings
            }
        }
    }
    fn display_settings(&self) -> DisplaySettings {
        match self {
            UserSettings::V1 {
                resolution,
                fullscreen,
                ..
            } => DisplaySettings {
                resolution: *resolution,
                fullscreen: *fullscreen,
            },
        }
    }
    //placeholder for future versions.
    fn migrate(&mut self) {
        match self {
            Self::V1 { .. } => {}
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for UserSettings {
    fn default() -> Self {
        Self::V1 {
            global_volume: 0.5,
            sfx_volume: 0.5,
            music_volume: 0.5,
            resolution: (1280., 720.),
            fullscreen: false,
            display_first_time_help: true,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for UserSettings {
    fn default() -> Self {
        Self::V1 {
            global_volume: 0.5,
            sfx_volume: 0.5,
            music_volume: 0.5,
            resolution: (1280., 720.),
            fullscreen: true,
            display_first_time_help: true,
        }
    }
}

#[derive(Component)]
pub enum SoundType {
    Music,
    SFX,
}
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct VolumeSettings {
    pub sfx_user_setting: f32,
    pub music_user_setting: f32,
    pub global_user_setting: f32,
    pub sfx: f32,
    pub music: f32,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DisplaySettings {
    pub resolution: (f32, f32),
    pub fullscreen: bool,
}
#[cfg(not(target_arch = "wasm32"))]
impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            resolution: (1280., 720.),
            fullscreen: true,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            resolution: (1280., 720.),
            fullscreen: false,
        }
    }
}

pub fn rescale_volume_setting(setting: f32) -> f32 {
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

#[derive(SystemParam)]
pub struct ApplicationSettings<'w> {
    user_settings: ResMut<'w, Persistent<UserSettings>>,
    volume_settings: Res<'w, VolumeSettings>,
    display_settings: Res<'w, DisplaySettings>,
}
impl ApplicationSettings<'_> {
    pub fn save_app_settings(&mut self) {
        //AFAICT this is a false positive on this lint error
        #[allow(unused_mut)]
        let mut da_settings = self.user_settings.get_mut();
        let UserSettings::V1 {
            display_first_time_help,
            ..
        } = da_settings;
        *da_settings = UserSettings::V1 {
            global_volume: self.volume_settings.global_user_setting,
            sfx_volume: self.volume_settings.sfx_user_setting,
            music_volume: self.volume_settings.music_user_setting,
            resolution: self.display_settings.resolution,
            fullscreen: self.display_settings.fullscreen,
            display_first_time_help: *display_first_time_help,
        };

        self.user_settings
            .persist()
            .expect("failed to persist user supplied settings");
    }
    #[allow(dead_code)]
    pub fn register_introductory_help(&mut self) {
        //AFAICT this is a false positive on this lint error
        #[allow(unused_mut)]
        let mut da_settings = self.user_settings.get_mut();
        let UserSettings::V1 {
            global_volume,
            sfx_volume,
            music_volume,
            fullscreen,
            resolution,
            ..
        } = da_settings;
        *da_settings = UserSettings::V1 {
            global_volume: *global_volume,
            sfx_volume: *sfx_volume,
            music_volume: *music_volume,
            resolution: *resolution,
            fullscreen: *fullscreen,
            display_first_time_help: false,
        };

        self.user_settings
            .persist()
            .expect("failed to persist user supplied settings");
    }
}

fn volume_changed(settings: Res<VolumeSettings>) -> bool {
    settings.is_changed()
}
fn display_changed(settings: Res<DisplaySettings>) -> bool {
    settings.is_changed()
}

pub fn populate_volume_settings_changes(
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
        
        let nativex = if cfg!(target_arch = "wasm32") {
            // The web canvas will only take 2/3rds of what it is given, so in order to get it to render a 1280 pixel canvas, I have to lie and tell it that it has 1920 pixels to play with.
            // May god forgive me this sin, because I certainly will not.
            1920.
        } else {
            rx as f32
        };

        let necessary_scale_factor = nativex as f64 / (targetx) as f64;

        let ui_scale_factor = targety as f64 / 720.;
        ui_scale.0 = ui_scale_factor;
        if necessary_scale_factor >= 1.0 && display_settings.fullscreen
            || cfg!(target_arch = "wasm32")
        {
            window
                .resolution
                .set_scale_factor_override(Some(necessary_scale_factor));
        }
        if !display_settings.fullscreen && cfg!(not(target_arch = "wasm32")) {
            window.resolution.set_scale_factor_override(None);
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

fn initialize_persistent_app_settings(mut commands: Commands) {
    let cfg_dir = dirs::config_dir()
        .map(|dir| dir.join("moar_ants"))
        .unwrap_or(Path::new("local").join("configuration"));
    commands.insert_resource(
        Persistent::<UserSettings>::builder()
            .name("user settings")
            .format(StorageFormat::Json)
            .path(cfg_dir.join("player_settings.json"))
            .default(UserSettings::default())
            .build()
            .expect("failed to initialize player settings"),
    );
}
fn load_app_settings(mut commands: Commands, mut user_settings: ResMut<Persistent<UserSettings>>) {
    user_settings.migrate();
    user_settings.persist().expect("settings migration error");
    commands.insert_resource::<VolumeSettings>(user_settings.volume_settings());
    commands.insert_resource::<DisplaySettings>(user_settings.display_settings());
}
