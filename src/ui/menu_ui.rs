use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_nine_slice_ui::{NineSliceUiMaterialBundle, NineSliceUiTexture};
use leafwing_input_manager::{
    action_state::{ActionState, ActionStateDriver},
    plugin::{InputManagerPlugin, ToggleActions},
    Actionlike, InputManagerBundle,
};

use super::ui_util::ProjectLocalStyle;
use crate::{GameStarted, UIFocus};

pub struct MainMenuUI;

#[derive(Component)]
pub struct MainMenu;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
enum InputHandlers {
    ButtonClick,
}

impl Plugin for MainMenuUI {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<MainMenuUIActions>::default())
            .register_type::<UIAnchorNode>()
            .configure_sets(
                Update,
                InputHandlers::ButtonClick.run_if(in_state(UIFocus::MainMenu)),
            )
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
                Update,
                (
                    quit_button_onclick.in_set(InputHandlers::ButtonClick),
                    toggle_settings.in_set(InputHandlers::ButtonClick),
                    credits_button_onclick.in_set(InputHandlers::ButtonClick),
                    start_button_onclick.in_set(InputHandlers::ButtonClick),
                ),
            );
    }
}
#[derive(Resource, Reflect)]
pub struct UIAnchorNode(pub Entity);

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum MainMenuUIActions {
    ExitMainMenu,
    ExitGame,
    OpenSettings,
    OpenCredits,
}

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
    let button_texture = asset_server.load("nine_slice/main_menu_buttons.png");

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

    let start_button = main_menu_button(
        &mut commands,
        button_texture.clone(),
        ActionStateDriver {
            action: MainMenuUIActions::ExitMainMenu,
            targets: root_node.into(),
        },
        start_text,
    );
    let settings_button = main_menu_button(
        &mut commands,
        button_texture.clone(),
        ActionStateDriver {
            action: MainMenuUIActions::OpenSettings,
            targets: root_node.into(),
        },
        "Settings",
    );
    let credits_button = main_menu_button(
        &mut commands,
        button_texture.clone(),
        ActionStateDriver {
            action: MainMenuUIActions::OpenCredits,
            targets: root_node.into(),
        },
        "Credits",
    );
    let quit_button = quit_button(
        &mut commands,
        button_texture.clone(),
        ActionStateDriver {
            action: MainMenuUIActions::ExitGame,
            targets: root_node.into(),
        },
    );

    commands.entity(anchor.0).add_child(root_node);
    commands.entity(root_node).add_child(menu_layout_node);
    commands.entity(menu_layout_node).push_children(&[
        start_button,
        settings_button,
        credits_button,
        quit_button,
    ]);
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

fn credits_button_onclick(
    q: Query<&ActionState<MainMenuUIActions>>,
    mut next_state: ResMut<NextState<UIFocus>>,
) {
    for n in q.iter() {
        if n.just_pressed(MainMenuUIActions::OpenCredits) {
            next_state.set(UIFocus::Credits);
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
) {
    for n in q.iter() {
        if n.just_pressed(MainMenuUIActions::OpenSettings) {
            next_state.set(UIFocus::SettingsMenu);
        }
    }
}

fn main_menu_button(
    commands: &mut Commands,
    image: Handle<Image>,
    action_driver: ActionStateDriver<MainMenuUIActions>,
    label: impl Into<String>,
) -> Entity {
    let button_text = label.into();
    let button = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: main_menu_button_style(),
                nine_slice_texture: NineSliceUiTexture::from_image(image),
                ..default()
            },
            Interaction::None,
            Name::new(format!("MainMenu: {:?} Button", button_text.clone())),
        ))
        .insert(action_driver)
        .id();
    let button_label = commands
        .spawn(TextBundle {
            text: Text::from_section(button_text, TextStyle::local(24.0, Color::BLACK)),
            ..default()
        })
        .id();
    commands.entity(button).add_child(button_label);
    button
}
#[cfg(not(target_arch = "wasm32"))]
fn quit_button(commands: &mut Commands, image: Handle<Image>, action_driver: ActionStateDriver<MainMenuUIActions>) -> Entity {
   main_menu_button(commands, image, action_driver, "Quit")
   
}
//TODO - there certainly exists a better solution to this problem
#[cfg(target_arch = "wasm32")]
fn quit_button(commands: &mut Commands, image: Handle<Image>, action_driver: ActionStateDriver<MainMenuUIActions>) -> Entity {
   commands.spawn(NodeBundle::default()).id()
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
