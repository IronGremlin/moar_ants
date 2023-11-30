use bevy::{input::InputSystem, prelude::*, window::PrimaryWindow};
use bevy_spatial::SpatialAccess;
use leafwing_input_manager::{
    axislike::DualAxisData, plugin::InputManagerSystem, prelude::*, systems::run_if_enabled,
    user_input::InputKind,
};

use crate::{
    buildables::{BuildableCommandsExt, BuildableObject, BuildableStore, PlacementOperation},
    colony::Colony,
    food::FoodQuant,
    gametimer::TickRate,
    spawner::HasFootPrint,
    MainCamera, SimState, SpatialIndex, SpatialMarker, UIFocus, upgrades::{UpgradeStringIndex, self, BuySpawner, ColonyUpgrade},
};

const CAMERA_PAN_SPEED_FACTOR: f32 = 10.0;
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraControl>::default())
            .add_plugins(InputManagerPlugin::<GamefieldActions>::default())
            .add_systems(OnEnter(UIFocus::Gamefield), setup.run_if(run_once()))
            .add_systems(
                Update,
                (
                    pan_camera,
                    zoom_camera,
                    user_adjust_sim_speed,
                    user_toggle_pause,
                    player_open_menu,
                )
                    .run_if(in_state(UIFocus::Gamefield)),
            )
            .add_systems(
                Update,
                track_cursor_for_building_placement
                    .pipe(user_place_building)
                    .run_if(
                        resource_exists::<PlacementStore>().and_then(in_state(UIFocus::Gamefield)),
                    ),
            );
    }
}


#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect)]
pub enum CameraControl {
    PanCam,
    Zoom,
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect)]
pub enum GamefieldActions {
    IncreaseTimeScale,
    DecreaseTimeScale,
    TogglePause,
    GameFieldClick,
    OpenMainMenu
}

#[derive(Resource)]
pub struct PlacementStore {
    selected_building_key: String,
    cost: i32,
}
impl PlacementStore {
    pub fn new(selected_building_key: String, cost: i32) -> PlacementStore {
        PlacementStore {
            selected_building_key,
            cost,
        }
    }
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
                .insert(KeyCode::Comma, GamefieldActions::DecreaseTimeScale)
                .insert(KeyCode::Period, GamefieldActions::IncreaseTimeScale)
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
    win_q: Query<&Window, With<PrimaryWindow>>,
) {
    let (projection, mut camera_transform, action_state) = q.single_mut();
    let win = win_q.single();
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

fn user_adjust_sim_speed(
    q: Query<&ActionState<GamefieldActions>>,
    mut rate: ResMut<NextState<TickRate>>,
    current_rate: Res<State<TickRate>>,
) {
    for action in q.iter() {
        if action.just_pressed(GamefieldActions::IncreaseTimeScale) {
            info!("speed up");
            rate.set(current_rate.faster());
        }
        if action.just_pressed(GamefieldActions::DecreaseTimeScale) {
            info!("slow down");
            rate.set(current_rate.slower())
        }
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
fn track_cursor_for_building_placement(
    opstore: Res<PlacementStore>,
    bstore: Res<BuildableStore>,
    space: Res<SpatialIndex>,
    space_q: Query<Entity, (With<HasFootPrint>, With<SpatialMarker>)>,
    win_q: Query<&Window, With<PrimaryWindow>>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut gizmos: Gizmos,
) -> Result<Vec2, String> {
    let mut result = Err("unspecified problem".into());
    let mut exc_color = Color::GREEN;
    let mut inc_color = Color::GREEN;
    if let Some(cursor) = win_q.get_single().ok().and_then(|n| n.cursor_position()) {
        let (cam, cam_xform) = cam_q.single();
        let pos = cam.viewport_to_world_2d(cam_xform, cursor).unwrap();
        result = Ok(pos);
        if let Some(exclusion_params) = bstore
            .get(&opstore.selected_building_key)
            .and_then(|x| x.exclusion_params())
        {
            let in_exclusion = space
                .within_distance(pos, exclusion_params.exclusion_radius)
                .iter()
                .filter_map(|(_, n)| *n)
                .filter_map(|ent| space_q.get(ent).ok())
                .count();

            if let Some(inclusion_radius) = exclusion_params.maximum_distance {
                let in_inclusion = space
                    .within_distance(pos, inclusion_radius)
                    .iter()
                    .filter_map(|(_, n)| *n)
                    .filter_map(|ent| space_q.get(ent).ok())
                    .count();
                if in_inclusion < 1 {
                    inc_color = Color::RED;
                    result = Err("Building too far from colony".into());
                }
                gizmos.circle_2d(pos, inclusion_radius, inc_color);
            }

            if in_exclusion != 0 {
                exc_color = Color::RED;
                result = Err("Building intersects footprint".into());
            }
            gizmos.circle_2d(pos, exclusion_params.exclusion_radius, exc_color);
        }
    }
    result
}


fn user_place_building(
    In(param): In<Result<Vec2, String>>,
    mut commands: Commands,
    q: Query<&ActionState<GamefieldActions>>,

    mut col_q: Query<(Entity,&mut UpgradeStringIndex, &mut FoodQuant), With<Colony>>,

    opstore: Res<PlacementStore>,
) {
    match param {
        Ok(pos) => {
            let (colony,mut upgrades, mut food) = col_q.single_mut();
            for action in q.iter() {
                if action.just_pressed(GamefieldActions::GameFieldClick) {
                    //guard placement by position

                    if opstore.cost > food.0 {
                        commands.remove_resource::<PlacementStore>();
                        return;
                    }
                    food.0 -= opstore.cost;
                    //TODO - fix this to be parametric over building type when we have more buildings.
                    upgrades.increment_index(BuySpawner::name());
                    commands.place_buildable(PlacementOperation {
                        position: pos,
                        owning_colony: colony,
                        key: opstore.selected_building_key.clone(),
                    });
                    commands.remove_resource::<PlacementStore>();
                    return;
                }
            }
        }
        Err(msg) => {
           // info!(msg)
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