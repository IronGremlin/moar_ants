use bevy::app::AppExit;
use bevy::prelude::*;

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
        app.add_systems(OnEnter(UIFocus::MainMenu), display_main_menu)
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

fn display_main_menu(mut root_commands: Commands) {
    let root_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    };
    let menu_layout_node = NodeBundle {
        style: Style {
            width: Val::Percent(40.0),
            height: Val::Percent(40.0),
            align_self: AlignSelf::Center,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    };

    root_commands
        .spawn((root_node, MainMenu, Name::new("Main Menu")))
        .with_children(|commands0| {
            commands0
                .spawn((menu_layout_node, Name::new("Main Menu Layout Container")))
                .with_children(|commands1| {
                    commands1
                        .spawn((
                            ButtonBundle {
                                style: main_menu_button_style(),
                                background_color: BUTTON_COLOR.into(),
                                ..default()
                            },
                            StartButton,
                            Name::new("Start Button"),
                        ))
                        .with_children(|commands| {
                            commands.spawn(TextBundle {
                                text: Text::from_section(
                                    "Start Game",
                                    text_style(26.0, BUTTON_TEXT_COLOR),
                                ),
                                ..default()
                            });
                        });
                    commands1
                        .spawn((
                            ButtonBundle {
                                style: main_menu_button_style(),
                                background_color: BUTTON_COLOR.into(),
                                ..default()
                            },
                            ToggleMusicButton,
                            Name::new("Toggle Music Button"),
                        ))
                        .with_children(|commands| {
                            commands.spawn(TextBundle {
                                text: Text::from_section(
                                    "Toggle Music",
                                    text_style(26.0, BUTTON_TEXT_COLOR),
                                ),
                                ..default()
                            });
                        });
                    commands1
                        .spawn((
                            ButtonBundle {
                                style: main_menu_button_style(),
                                background_color: BUTTON_COLOR.into(),
                                ..default()
                            },
                            QuitButton,
                            Name::new("Quit Button"),
                        ))
                        .with_children(|commands| {
                            commands.spawn(TextBundle {
                                text: Text::from_section(
                                    "Quit",
                                    text_style(26.0, BUTTON_TEXT_COLOR),
                                ),
                                ..default()
                            });
                        });
                });
        });
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
        align_self: AlignSelf::Center,
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
