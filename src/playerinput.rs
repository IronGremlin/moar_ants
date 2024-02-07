use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use crate::{MainCamera, SimState, UIFocus};

const CAMERA_PAN_SPEED_FACTOR: f32 = 10.0;
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraControl>::default())
            .add_plugins(InputManagerPlugin::<GamefieldActions>::default())
            .add_plugins(InputManagerPlugin::<MainMenuUIActions>::default())
            .add_plugins(InputManagerPlugin::<SettingsMenuUIActions>::default())
            .add_plugins(InputManagerPlugin::<DisplaySettingsMenuUIActions>::default())
            .add_plugins(InputManagerPlugin::<AudioMenuUIActions>::default())
            .add_systems(OnEnter(UIFocus::Gamefield), setup.run_if(run_once()))
            .add_systems(
                Update,
                (pan_camera, zoom_camera, user_toggle_pause, player_open_menu)
                    .run_if(in_state(UIFocus::Gamefield)),
            );
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CameraControl {
    PanCam,
    Zoom,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum GamefieldActions {
    TogglePause,
    GameFieldClick,
    OpenMainMenu,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum MainMenuUIActions {
    ExitMainMenu,
    ExitGame,
    OpenSettings,
    //TODO - OpenCredits,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SettingsMenuUIActions {
    ToggleDisplaySettings,
    ToggleAudioSettings,
    ExitSettings,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum DisplaySettingsMenuUIActions {
    ToggleResolutionSelection,
    //TODO Figure some way to represent selection as an action
    ToggleFullscreen,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum AudioMenuUIActions {
    SetMusicVolume,
    SetSFXVolume,
}

fn setup(
    mut commands: Commands,
    window: Query<Entity, With<PrimaryWindow>>,
    camera: Query<Entity, With<MainCamera>>,
) {
    let (win, cam) = (window.single(), camera.single());
    commands
        .entity(cam)
        .insert(InputManagerBundle::<CameraControl> {
            input_map: InputMap::default()
                .insert(SingleAxis::mouse_wheel_y(), CameraControl::Zoom)
                .insert(
                    UserInput::chord([
                        InputKind::Mouse(MouseButton::Right),
                        InputKind::DualAxis(DualAxis::mouse_motion()),
                    ]),
                    CameraControl::PanCam,
                )
                .build(),
            ..default()
        });
    commands
        .entity(win)
        .insert(InputManagerBundle::<GamefieldActions> {
            input_map: InputMap::default()
                .insert(MouseButton::Left, GamefieldActions::GameFieldClick)
                .insert(KeyCode::Space, GamefieldActions::TogglePause)
                .insert(KeyCode::Escape, GamefieldActions::OpenMainMenu)
                .build(),
            ..default()
        });
}

fn zoom_camera(
    mut query: Query<(&mut OrthographicProjection, &ActionState<CameraControl>), With<MainCamera>>,
) {
    const CAMERA_ZOOM_RATE: f32 = 0.05;

    let (mut camera_projection, action_state) = query.single_mut();

    let zoom_delta = action_state.value(CameraControl::Zoom);

    camera_projection.scale =
        (camera_projection.scale * 1. - zoom_delta * CAMERA_ZOOM_RATE).clamp(0.0, 20.0);
}

fn pan_camera(
    mut q: Query<
        (
            &OrthographicProjection,
            &mut Transform,
            &ActionState<CameraControl>,
        ),
        With<Camera>,
    >,
) {
    let (projection, mut camera_transform, action_state) = q.single_mut();

    if action_state.pressed(CameraControl::PanCam) {
        action_state
            .action_data(CameraControl::PanCam)
            .axis_pair
            .map(|axis_data| {
                let zoom_scale = projection.scale / 20.0;
                camera_transform.translation.x +=
                    axis_data.x() * -(CAMERA_PAN_SPEED_FACTOR * zoom_scale);
                camera_transform.translation.y +=
                    axis_data.y() * (CAMERA_PAN_SPEED_FACTOR * zoom_scale);
            });
    }
}

fn user_toggle_pause(
    q: Query<&ActionState<GamefieldActions>>,
    mut sim_next: ResMut<NextState<SimState>>,
    sim_current: Res<State<SimState>>,
) {
    for action in q.iter() {
        if action.just_pressed(GamefieldActions::TogglePause) {
            info!("Toggle pause");
            sim_next.set(match sim_current.get() {
                SimState::Paused => SimState::Playing,
                SimState::Playing => SimState::Paused,
            });
        }
    }
}

fn player_open_menu(
    mut next_state: ResMut<NextState<UIFocus>>,
    q: Query<&ActionState<GamefieldActions>>,
) {
    for action in q.iter() {
        if action.just_pressed(GamefieldActions::OpenMainMenu) {
            next_state.set(UIFocus::MainMenu);
        }
    }
}
