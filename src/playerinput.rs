use bevy::{input::mouse, prelude::*, window::PrimaryWindow};
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use crate::{
    ui::{
        credits_ui::CreditsUIActions,
        menu_ui::MainMenuUIActions,
        settings_menu::{AudioMenuUIActions, DisplaySettingsMenuUIActions, SettingsMenuUIActions},
    },
    MainCamera, SimState, UIFocus,
};

const CAMERA_PAN_SPEED_FACTOR: f32 = 10.0;
#[cfg(not(target_arch = "wasm32"))]
const CAMERA_ZOOM_RATE: f32 = 0.05;
#[cfg(target_arch = "wasm32")]
const CAMERA_ZOOM_RATE: f32 = 0.005;
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraControl>::default())
            .add_plugins(InputManagerPlugin::<GamefieldActions>::default())
            .add_systems(Startup, setup)
            .add_systems(
                OnEnter(UIFocus::Gamefield),
                game_field_setup.run_if(run_once()),
            )
            .add_systems(
                Update,
                (pan_camera, zoom_camera, user_toggle_pause, player_open_menu)
                    .run_if(in_state(UIFocus::Gamefield)),
            );
    }
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CameraControl {
    // We absolutely should not have to do this but LWIM has mouse input on a scale from 0 - 180 and virtual dpad at 0-1, so our choices are to do this or have raw-input drive an ActionStateDriver
    // so this is a question of "Which method of violating the core concept of an input management system feels less arduous"
    PanCamMouse,
    PanCamDPad,
    Zoom,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum GamefieldActions {
    TogglePause,
    GameFieldClick,
    OpenMainMenu,
}

// TODO - We should really figure out a way to clean this up so that the input module doesn't have to import every UI module forever.
fn setup(
    mut gamefield_actions: ResMut<ToggleActions<GamefieldActions>>,
    mut camera_actions: ResMut<ToggleActions<CameraControl>>,
    mut main_menu_actions: ResMut<ToggleActions<MainMenuUIActions>>,
    mut credits_actions: ResMut<ToggleActions<CreditsUIActions>>,
    mut settings_menu_actions: ResMut<ToggleActions<SettingsMenuUIActions>>,
    mut display_settings_actions: ResMut<ToggleActions<DisplaySettingsMenuUIActions>>,
    mut audio_settings_actions: ResMut<ToggleActions<AudioMenuUIActions>>,
) {
    gamefield_actions.enabled = false;
    camera_actions.enabled = false;
    main_menu_actions.enabled = true;
    credits_actions.enabled = false;
    settings_menu_actions.enabled = false;
    display_settings_actions.enabled = false;
    audio_settings_actions.enabled = false;
}

fn game_field_setup(
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
                .insert_multiple([
                    (
                        UserInput::chord([
                            InputKind::Mouse(MouseButton::Middle),
                            InputKind::DualAxis(DualAxis::mouse_motion()),
                        ]),
                        CameraControl::PanCamMouse,
                    ),
                    (
                        VirtualDPad::wasd().inverted_x().into(),
                        CameraControl::PanCamDPad,
                    ),
                ])
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
    let (mut camera_projection, action_state) = query.single_mut();

    let zoom_delta = action_state.value(CameraControl::Zoom);

    camera_projection.scale =
        (camera_projection.scale * 1. - zoom_delta * CAMERA_ZOOM_RATE).clamp(0.1, 20.);
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
    let (mousepan, dpadpan) = (
        action_state.pressed(CameraControl::PanCamMouse),
        action_state.pressed(CameraControl::PanCamDPad),
    );
    if !mousepan && !dpadpan {
        return;
    }

    let (zoom_scale, action_data) = if dpadpan {
        (projection.scale * 0.5, CameraControl::PanCamDPad)
    } else {
        (projection.scale / 20., CameraControl::PanCamMouse)
    };

    action_state
        .action_data(action_data)
        .axis_pair
        .map(|axis_data| {
            camera_transform.translation.x +=
                axis_data.x() * -(CAMERA_PAN_SPEED_FACTOR * zoom_scale);
            camera_transform.translation.y +=
                axis_data.y() * (CAMERA_PAN_SPEED_FACTOR * zoom_scale);
        });

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
                SimState::Paused | SimState::MenuOpenedWhilePaused => SimState::Playing,
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
