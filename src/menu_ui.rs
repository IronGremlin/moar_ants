use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_inspector_egui::egui::menu;

use crate::{MainMusicTrack, UIFocus};

pub struct MainMenuUI;

#[derive(Component)]
struct StartButton;
#[derive(Component)]
struct ToggleMusicButton;
#[derive(Component)]
struct QuitButton;

#[derive(Component)]
pub struct MainMenu;

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
                Update,
                (
                    quit_button_onclick.run_if(in_state(UIFocus::MainMenu)),
                    toggle_music_button.run_if(in_state(UIFocus::MainMenu)),
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
                ..default()
            },
            ..default()
        })
        .id();
    commands.insert_resource(UIAnchorNode(whole_screen));
}

fn display_main_menu(mut commands: Commands, anchor: Res<UIAnchorNode>) {
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
        .id();

    let menu_layout_node = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(40.0),
                height: Val::Percent(40.0),
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
            ButtonBundle {
                style: main_menu_button_style(),
                background_color: BUTTON_COLOR.into(),
                ..default()
            },
            StartButton,
            Name::new("Start Button"),
        ))
        .id();
    let start_button_label = commands
        .spawn(TextBundle {
            text: Text::from_section("Start Game", text_style(26.0, BUTTON_TEXT_COLOR)),
            ..default()
        })
        .id();
    let toggle_music_button = commands
        .spawn((
            ButtonBundle {
                style: main_menu_button_style(),
                background_color: BUTTON_COLOR.into(),
                ..default()
            },
            ToggleMusicButton,
            Name::new("Toggle Music Button"),
        ))
        .id();
    let toggle_music_button_label = commands
        .spawn(TextBundle {
            text: Text::from_section("Toggle Music", text_style(26.0, BUTTON_TEXT_COLOR)),
            ..default()
        })
        .id();

    let quit_button = commands
        .spawn((
            ButtonBundle {
                style: main_menu_button_style(),
                background_color: BUTTON_COLOR.into(),
                ..default()
            },
            QuitButton,
            Name::new("Quit Button"),
        ))
        .id();
    let quit_button_label = commands
        .spawn(TextBundle {
            text: Text::from_section("Quit", text_style(26.0, BUTTON_TEXT_COLOR)),
            ..default()
        })
        .id();
    commands.entity(anchor.0).add_child(root_node);
    commands.entity(root_node).add_child(menu_layout_node);
    commands.entity(menu_layout_node).push_children(&[
        start_button,
        toggle_music_button,
        quit_button,
    ]);
    commands.entity(start_button).add_child(start_button_label);
    commands
        .entity(toggle_music_button)
        .add_child(toggle_music_button_label);
    commands.entity(quit_button).add_child(quit_button_label);
    commands.entity(anchor.0).add_child(root_node);
}
fn start_button_onclick(
    start_button: Query<&Interaction, With<StartButton>>,
    mut next_state: ResMut<NextState<UIFocus>>,
) {
    let button = start_button.single();
    match button {
        Interaction::Pressed => {
            next_state.set(UIFocus::Gamefield);
        }
        Interaction::Hovered | Interaction::None => {}
    }
}

fn quit_button_onclick(
    quit_button: Query<&Interaction, With<QuitButton>>,
    mut exit_event: EventWriter<AppExit>,
) {
    let button = quit_button.single();
    match button {
        Interaction::Pressed => exit_event.send(AppExit),
        Interaction::Hovered | Interaction::None => {}
    }
}

fn toggle_music_button(
    quit_button: Query<&Interaction, With<ToggleMusicButton>>,
    mut music: Query<&mut AudioSink, With<MainMusicTrack>>,
) {
    let button = quit_button.single();
    match button {
        Interaction::Pressed => {
            let _ = music.get_single_mut().map(|x| x.toggle());
        }
        Interaction::Hovered | Interaction::None => {}
    }
}

fn main_menu_teardown(mut commands: Commands, main_menu: Query<Entity, With<MainMenu>>) {
    for entity in &main_menu {
        commands.entity(entity).despawn_recursive();
    }
}
const BUTTON_COLOR: Color = Color::BLUE;
const BUTTON_TEXT_COLOR: Color = Color::BLACK;

fn main_menu_button_style() -> Style {
    Style {
        width: Val::Percent(80.0),
        height: Val::Percent(20.0),
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
