use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_nine_slice_ui::{NineSliceUiMaterialBundle, NineSliceUiTexture};
use leafwing_input_manager::{
    action_state::{ActionState, ActionStateDriver},
    plugin::ToggleActions,
    InputManagerBundle,
};

use crate::{playerinput::MainMenuUIActions, MainMusicTrack, UIFocus};

pub struct MainMenuUI;

#[derive(Component)]
pub struct MainMenu;

#[derive(Resource, Default)]
pub struct GameStarted;

impl Plugin for MainMenuUI {
    fn build(&self, app: &mut App) {
        app.register_type::<UIAnchorNode>()
            .add_systems(Update, open_menu_on_start.run_if(run_once()))
            .add_systems(
                OnEnter(UIFocus::MainMenu),
                (
                    generate_ui_anchor_node.run_if(not(resource_exists::<UIAnchorNode>())),
                    apply_deferred,
                    display_main_menu,
                )
                    .chain(),
            )
            .add_systems(OnExit(UIFocus::MainMenu), main_menu_teardown)
            .add_systems(
                OnEnter(UIFocus::Gamefield),
                (|mut world: &mut World| {
                    world.init_resource::<GameStarted>();
                })
                .run_if(run_once()),
            )
            .add_systems(
                Update,
                (
                    quit_button_onclick.run_if(in_state(UIFocus::MainMenu)),
                    toggle_settings.run_if(in_state(UIFocus::MainMenu)),
                    start_button_onclick.run_if(in_state(UIFocus::MainMenu)),
                ),
            );
    }
}
#[derive(Resource, Reflect)]
pub struct UIAnchorNode(pub Entity);

fn open_menu_on_start(mut ui_focus: ResMut<NextState<UIFocus>>) {
    ui_focus.set(UIFocus::MainMenu);
}

fn generate_ui_anchor_node(mut commands: Commands) {
    let whole_screen = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .id();
    commands.insert_resource(UIAnchorNode(whole_screen));
}

fn display_main_menu(
    mut commands: Commands,
    anchor: Res<UIAnchorNode>,
    asset_server: Res<AssetServer>,
    mut main_menu_actions: ResMut<ToggleActions<MainMenuUIActions>>,
    game_start: Option<Res<GameStarted>>,
) {
    let start_text = if game_start.is_some() {
        "Resume Game"
    } else {
        "Start Game"
    };
    main_menu_actions.enabled = true;
    let root_node = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                z_index: ZIndex::Local(30),
                ..default()
            },
            MainMenu,
            Name::new("Main Menu"),
        ))
        .insert(InputManagerBundle::<MainMenuUIActions>::default())
        .id();

    let menu_layout_node = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(157.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .id();
    let start_button = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: main_menu_button_style(),
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/main_menu_buttons.png"),
                ),
                ..default()
            },
            Interaction::None,
            Name::new("Start Button"),
        ))
        .insert(ActionStateDriver {
            action: MainMenuUIActions::ExitMainMenu,
            targets: root_node.into(),
        })
        .id();
    let start_button_label = commands
        .spawn(TextBundle {
            text: Text::from_section(start_text, text_style(24.0, BUTTON_TEXT_COLOR)),
            ..default()
        })
        .id();
    let settings_button = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: main_menu_button_style(),
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/main_menu_buttons.png"),
                ),
                ..default()
            },
            Interaction::None,
            Name::new("Settings Menu Button"),
        ))
        .insert(ActionStateDriver {
            action: MainMenuUIActions::OpenSettings,
            targets: root_node.into(),
        })
        .id();
    let settings_button_label = commands
        .spawn(TextBundle {
            text: Text::from_section("Settings", text_style(24.0, BUTTON_TEXT_COLOR)),
            ..default()
        })
        .id();

    let quit_button = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: main_menu_button_style(),
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/main_menu_buttons.png"),
                ),
                ..default()
            },
            Interaction::None,
            Name::new("Quit Button"),
        ))
        .insert(ActionStateDriver {
            action: MainMenuUIActions::ExitGame,
            targets: root_node.into(),
        })
        .id();
    let quit_button_label = commands
        .spawn(TextBundle {
            text: Text::from_section("Quit", text_style(24.0, BUTTON_TEXT_COLOR)),
            ..default()
        })
        .id();
    commands.entity(anchor.0).add_child(root_node);
    commands.entity(root_node).add_child(menu_layout_node);
    commands
        .entity(menu_layout_node)
        .push_children(&[start_button, settings_button, quit_button]);
    commands.entity(start_button).add_child(start_button_label);
    commands
        .entity(settings_button)
        .add_child(settings_button_label);
    commands.entity(quit_button).add_child(quit_button_label);
    commands.entity(anchor.0).add_child(root_node);
}

fn start_button_onclick(
    q: Query<&ActionState<MainMenuUIActions>>,
    mut next_state: ResMut<NextState<UIFocus>>,
) {
    for n in q.iter() {
        if n.just_pressed(MainMenuUIActions::ExitMainMenu) {
            next_state.set(UIFocus::Gamefield);
        }
    }
}

fn quit_button_onclick(
    q: Query<&ActionState<MainMenuUIActions>>,
    mut exit_event: EventWriter<AppExit>,
) {
    for n in q.iter() {
        if n.just_pressed(MainMenuUIActions::ExitGame) {
            info!("send exit");
            exit_event.send(AppExit);
        }
    }
}

fn toggle_settings(
    q: Query<&ActionState<MainMenuUIActions>>,
    mut next_state: ResMut<NextState<UIFocus>>,
    //mut music: Query<&mut AudioSink, With<MainMusicTrack>>,
) {
    for n in q.iter() {
        if n.just_pressed(MainMenuUIActions::OpenSettings) {
            next_state.set(UIFocus::SettingsMenu);
            //let _ = music.get_single_mut().map(|x| x.toggle());
        }
    }
}

fn main_menu_teardown(
    mut commands: Commands,
    mut main_menu_actions: ResMut<ToggleActions<MainMenuUIActions>>,
    main_menu: Query<Entity, With<MainMenu>>,
) {
    main_menu_actions.enabled = false;
    for entity in &main_menu {
        commands.entity(entity).despawn_recursive();
    }
}

const BUTTON_COLOR: Color = Color::BLUE;
const BUTTON_TEXT_COLOR: Color = Color::BLACK;

fn main_menu_button_style() -> Style {
    Style {
        width: Val::Px(155.0),
        height: Val::Px(57.0),
        align_items: AlignItems::Center,
        justify_self: JustifySelf::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect {
            left: Val::Px(10.),
            right: Val::Px(10.),
            top: Val::Px(10.),
            bottom: Val::Px(10.),
        },
        ..default()
    }
}
fn text_style(size: f32, col: Color) -> TextStyle {
    TextStyle {
        font_size: size,
        color: col,
        ..default()
    }
}
