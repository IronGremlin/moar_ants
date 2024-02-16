use bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows};
use bevy::window::WindowMode;

pub struct AppSettingsPlugin;
impl Plugin for AppSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                populate_volume_settings_changes.run_if(volume_changed),
                populate_display_settings_changes.run_if(display_changed),
            ),
        );
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
  pub  sfx_user_setting: f32,
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
impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            resolution: (1280., 720.),
            fullscreen: true,
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
        let nativex = rx as f32;
        let necessary_scale_factor = nativex as f64 / targetx as f64;
        let ui_scale_factor = targety as f64 / 720.;
        ui_scale.0 = ui_scale_factor;
        if necessary_scale_factor >= 1.0 && display_settings.fullscreen {
            window
                .resolution
                .set_scale_factor_override(Some(necessary_scale_factor));
        }
        if !display_settings.fullscreen {
            window.resolution.set_scale_factor_override(Some(1.0));
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
