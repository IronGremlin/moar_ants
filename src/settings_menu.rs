use bevy::{
    ecs::system::SystemParam, prelude::*, ui::RelativeCursorPosition, window::PrimaryWindow,
};

use bevy_nine_slice_ui::{NineSliceUiMaterialBundle, NineSliceUiTexture};
use leafwing_input_manager::{
    action_state::{ActionState, ActionStateDriver},
    plugin::ToggleActions,
    InputManagerBundle,
};

use crate::{
    menu_ui::UIAnchorNode,
    playerinput::{AudioMenuUIActions, DisplaySettingsMenuUIActions, SettingsMenuUIActions},
    ui_helpers::{into_pct, px, ProjectLocalStyle, UICommandsExt, ALL, MEDIUM},
    DisplaySettings, UIFocus, VolumeSettings,
};

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UIFocus::SettingsMenu), instantiate_settings_menu)
            .add_systems(
                Update,
                (
                    exit_settings_menu,
                    activate_audio_settings_card,
                    activate_display_settings_card,
                    (interactive_range_set, watch_audio_bars).chain(),
                    (
                        show_active_window_settings,
                        (set_fullscreen_mode, set_window_resolution),
                    )
                        .chain(),
                ),
            )
            .add_systems(OnExit(UIFocus::SettingsMenu), settings_menu_teardown);
    }
}

#[derive(Component)]
struct SettingsMenuRoot;

#[derive(Component)]
struct SFXVolumeLevel;
#[derive(Component)]
struct MusicVolumeLevel;
#[derive(Component)]
struct GlobalVolumeLevel;

#[derive(Component)]
struct DisplaySettingsNode(Entity);
#[derive(Component)]
struct AudioSettingsNode(Entity);
#[derive(Component)]
struct Target(Entity);
#[derive(Component)]
struct UiToggle(bool);

#[derive(Component)]
struct Fullscreen;
#[derive(Component)]
struct UiResolutionInput((f32, f32));

#[derive(Component)]
pub struct UiRange(pub f32);

#[derive(Component)]
pub struct FillBar;

fn instantiate_settings_menu(
    mut commands: Commands,
    mut settings_menu_actions: ResMut<ToggleActions<SettingsMenuUIActions>>,
    asset_server: Res<AssetServer>,
    volume_settings: Res<VolumeSettings>,
    active_window_settings: ActiveWindowSettings,
    anchor: Res<UIAnchorNode>,
) {
    settings_menu_actions.enabled = true;

    let music_level = volume_settings.music_user_setting;
    let sfx_level = volume_settings.sfx_user_setting;
    let global_level = volume_settings.global_user_setting;

    let settings_root_layout = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    align_self: AlignSelf::Center,
                    width: px(321.),
                    height: px(181.),
                    padding: UiRect {
                        left: px(16.),
                        right: px(24.),
                        top: px(34.),
                        bottom: px(21.),
                    },
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/settings_tile.png"),
                ),

                // they all look like ants from here
                z_index: ZIndex::Local(100),
                ..default()
            },
            SettingsMenuRoot,
        ))
        .insert(InputManagerBundle::<SettingsMenuUIActions>::default())
        .id();
    let card_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: ALL,
                height: ALL,
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::px(233.), GridTrack::px(44.)],
                border: UiRect::all(px(2.)),
                //padding: UiRect::all(into_pct(4. / big_width)),
                ..default()
            },
            border_color: Color::rgb_u8(203, 219, 252).into(),
            ..default()
        })
        .id();
    let tags_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: ALL,
                height: px(20.),
                // border.
                left: px(-2.),
                top: px(-22.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .id();
    let display_settings_tag = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: Style {
                    height: ALL,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect {
                        left: px(6.),
                        right: px(6.),
                        top: px(6.),
                        bottom: px(5.),
                    },
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/bgG_bMCG_soft_corner_flat_bottom.png"),
                ),
                ..default()
            },
            Interaction::None,
        ))
        .insert(ActionStateDriver {
            action: SettingsMenuUIActions::ToggleDisplaySettings,
            targets: settings_root_layout.into(),
        })
        .id();
    let display_settings_label = commands
        .spawn(TextBundle {
            text: Text::from_section("Display", TextStyle::local(MEDIUM, Color::WHITE)),
            style: Style {
                margin: UiRect::bottom(px(5.)),
                ..default()
            },
            ..default()
        })
        .id();
    let audio_settings_tag = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: Style {
                    height: ALL,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect {
                        left: px(6.),
                        right: px(6.),
                        top: px(6.),
                        bottom: px(5.),
                    },
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/bgG_bMCG_soft_corner_flat_bottom.png"),
                ),
                ..default()
            },
            Interaction::None,
        ))
        .insert(ActionStateDriver {
            action: SettingsMenuUIActions::ToggleAudioSettings,
            targets: settings_root_layout.into(),
        })
        .id();
    let audio_settings_label = commands
        .spawn(TextBundle {
            text: Text::from_section("Audio", TextStyle::local(MEDIUM, Color::WHITE)),
            style: Style {
                margin: UiRect::bottom(px(5.)),
                ..default()
            },
            ..default()
        })
        .id();
    let display_settings_grid = commands
        .spawn(NodeBundle {
            style: Style {
                width: ALL,
                height: px(90.),
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(1),
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::px(88.), GridTrack::fr(1.)],
                grid_template_rows: vec![GridTrack::px(26.), GridTrack::fr(1.)],
                justify_self: JustifySelf::Start,
                align_self: AlignSelf::Start,
                ..default()
            },
            background_color: Color::rgb_u8(89, 86, 82).into(),
            ..default()
        })
        .id();
    commands
        .entity(display_settings_tag)
        .insert(DisplaySettingsNode(display_settings_grid));

    let display_settings_fullscreen_section_header = commands
        .spawn(NodeBundle {
            style: Style {
                // width: into_pct(1.),
                // height: into_pct(1.),
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(1),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                border: UiRect {
                    top: px(0.),
                    bottom: px(1.),
                    right: px(1.),
                    left: px(0.),
                },
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let display_settings_fullscreen_label = commands
        .make_text("FullScreen:", TextStyle::local(MEDIUM, Color::BLACK))
        .id();
    commands
        .entity(display_settings_fullscreen_section_header)
        .add_child(display_settings_fullscreen_label);
    let even_dumber_checkbox = commands.make_icon("green_check_icon.png".into());
    commands.entity(even_dumber_checkbox).insert(Style {
        width: into_pct(0.5),
        height: into_pct(0.5),
        align_self: AlignSelf::Center,
        justify_self: JustifySelf::Center,
        ..default()
    });
    let stupid_fucking_fullscreen_checkbox = commands
        .spawn(NodeBundle {
            style: Style {
                grid_column: GridPlacement::start(2),
                grid_row: GridPlacement::start(1),
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                border: UiRect {
                    bottom: px(1.),
                    right: px(1.),
                    left: px(0.),
                    top: px(0.),
                },
                padding: UiRect::bottom(px(2.)),
                ..default()
            },
            border_color: Color::BLACK.into(),
            background_color: Color::rgb_u8(105, 106, 106).into(),
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn((
                    NineSliceUiMaterialBundle {
                        style: Style {
                            width: px(16.),
                            height: px(16.),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..default()
                        },
                        nine_slice_texture: NineSliceUiTexture::from_image(
                            asset_server.load("nine_slice/fullscreen_checkbox.png"),
                        ),
                        z_index: ZIndex::Local(20),
                        ..default()
                    },
                    UiToggle(true),
                    Fullscreen,
                    Interaction::None,
                    Name::new("Fucking checkbox"),
                ))
                .add_child(even_dumber_checkbox);
        })
        .id();
    let display_settings_resolution_section_header = commands
        .spawn(NodeBundle {
            style: Style {
                // width: into_pct(1.),
                // height: into_pct(1.),
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(2),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                border: UiRect {
                    top: px(0.),
                    bottom: px(1.),
                    right: px(1.),
                    left: px(0.),
                },
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let display_settings_resolution_label = commands
        .make_text("Resolution:", TextStyle::local(MEDIUM, Color::BLACK))
        .id();
    commands
        .entity(display_settings_resolution_section_header)
        .add_child(display_settings_resolution_label);
    let display_settings_resolution_selection = make_resolution_selection_component(
        &mut commands,
        vec![(3840., 2160.), (1920., 1080.), (1600., 900.), (1280., 720.)],
        active_window_settings.current_resolution(),
    );

    let audio_settings_grid = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::None,
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(1),
                grid_template_columns: vec![GridTrack::px(49.), GridTrack::px(102.)],
                grid_template_rows: vec![
                    GridTrack::percent(33.),
                    GridTrack::percent(33.),
                    GridTrack::percent(34.),
                ],
                border: UiRect::all(px(2.)),
                justify_self: JustifySelf::Start,
                align_self: AlignSelf::Start,
                width: ALL,
                height: px(70.),
                ..default()
            },
            background_color: Color::rgb_u8(89, 86, 82).into(),
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    commands
        .entity(audio_settings_tag)
        .insert(AudioSettingsNode(audio_settings_grid));
    let audio_settings_global_section_header = commands
        .spawn(NodeBundle {
            style: Style {
                // width: into_pct(1.),
                // height: into_pct(1.),
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(1),
                border: UiRect {
                    top: px(0.),
                    bottom: px(1.),
                    right: px(1.),
                    left: px(0.),
                },
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let audio_settings_global_label = commands
        .make_text(" Global:", TextStyle::local(MEDIUM, Color::BLACK))
        .id();
    commands
        .entity(audio_settings_global_section_header)
        .add_child(audio_settings_global_label);
    let audio_settings_music_section_header = commands
        .spawn(NodeBundle {
            style: Style {
                // width: into_pct(1.),
                // height: into_pct(1.),
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(2),

                border: UiRect {
                    top: px(0.),
                    bottom: px(1.),
                    right: px(1.),
                    left: px(0.),
                },
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let audio_settings_music_label = commands
        .make_text(" Music:", TextStyle::local(MEDIUM, Color::BLACK))
        .id();
    commands
        .entity(audio_settings_music_section_header)
        .add_child(audio_settings_music_label);
    let audio_settings_sfx_section_header = commands
        .spawn(NodeBundle {
            style: Style {
                // width: into_pct(1.),
                // height: into_pct(1.),
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(3),
                border: UiRect::right(px(1.)),
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let audio_settings_sfx_label = commands
        .make_text(" SFX:", TextStyle::local(MEDIUM, Color::BLACK))
        .id();
    commands
        .entity(audio_settings_sfx_section_header)
        .add_child(audio_settings_sfx_label);
    let global_bar = make_audio_level_bar(&mut commands, 1, global_level, GlobalVolumeLevel);

    let music_bar = make_audio_level_bar(&mut commands, 2, music_level, MusicVolumeLevel);
    let sfx_bar = make_audio_level_bar(&mut commands, 3, sfx_level, SFXVolumeLevel);

    let exit_button = commands
        .spawn((
            NineSliceUiMaterialBundle {
                style: Style {
                    width: px(40.),
                    height: px(20.),
                    grid_column: GridPlacement::start(2),
                    grid_row: GridPlacement::start(1),
                    align_self: AlignSelf::End,
                    align_items: AlignItems::Center,
                    justify_self: JustifySelf::End,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::bottom(px(5.)),
                    ..default()
                },
                nine_slice_texture: NineSliceUiTexture::from_image(
                    asset_server.load("nine_slice/settings_menu_exit_button.png"),
                ),
                ..default()
            },
            Interaction::None,
        ))
        .insert(ActionStateDriver {
            action: SettingsMenuUIActions::ExitSettings,
            targets: settings_root_layout.into(),
        })
        .id();

    let exit_button_label = commands
        .make_text("Exit", TextStyle::local(MEDIUM, Color::WHITE))
        .id();
    commands.entity(anchor.0).add_child(settings_root_layout);
    commands.entity(settings_root_layout).add_child(card_layout);
    commands.entity(card_layout).push_children(&[
        tags_layout,
        display_settings_grid,
        audio_settings_grid,
        exit_button,
    ]);
    commands.entity(display_settings_grid).push_children(&[
        display_settings_fullscreen_section_header,
        stupid_fucking_fullscreen_checkbox,
        display_settings_resolution_section_header,
        display_settings_resolution_selection,
    ]);
    commands.entity(audio_settings_grid).push_children(&[
        audio_settings_global_section_header,
        global_bar,
        audio_settings_music_section_header,
        music_bar,
        audio_settings_sfx_section_header,
        sfx_bar,
    ]);
    commands
        .entity(tags_layout)
        .push_children(&[display_settings_tag, audio_settings_tag]);
    commands
        .entity(display_settings_tag)
        .add_child(display_settings_label);
    commands
        .entity(audio_settings_tag)
        .add_child(audio_settings_label);
    commands.entity(exit_button).add_child(exit_button_label);
}

fn settings_menu_teardown(
    mut commands: Commands,
    q: Query<Entity, With<SettingsMenuRoot>>,
    mut settings_menu_actions: ResMut<ToggleActions<SettingsMenuUIActions>>,
    mut audio_settings_actions: ResMut<ToggleActions<AudioMenuUIActions>>,
    mut display_settings_actions: ResMut<ToggleActions<DisplaySettingsMenuUIActions>>,
) {
    settings_menu_actions.enabled = false;
    audio_settings_actions.enabled = false;
    display_settings_actions.enabled = false;

    q.iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}
fn activate_display_settings_card(
    q: Query<&ActionState<SettingsMenuUIActions>>,
    to_show: Query<&DisplaySettingsNode, With<ActionStateDriver<SettingsMenuUIActions>>>,
    to_hide: Query<
        &AudioSettingsNode,
        (
            With<ActionStateDriver<SettingsMenuUIActions>>,
            Without<DisplaySettingsNode>,
        ),
    >,
    mut toggle_display: Query<&mut Style>,
    mut audio_settings_actions: ResMut<ToggleActions<AudioMenuUIActions>>,
    mut display_settings_actions: ResMut<ToggleActions<DisplaySettingsMenuUIActions>>,
) {
    for n in q.iter() {
        if n.just_pressed(SettingsMenuUIActions::ToggleDisplaySettings) {
            audio_settings_actions.enabled = false;
            display_settings_actions.enabled = true;
            let [mut show_style, mut hide_style] =
                toggle_display.many_mut([to_show.single().0, to_hide.single().0]);
            show_style.display = Display::Grid;
            hide_style.display = Display::None;
        }
    }
}
fn activate_audio_settings_card(
    q: Query<&ActionState<SettingsMenuUIActions>>,
    to_hide: Query<&DisplaySettingsNode, With<ActionStateDriver<SettingsMenuUIActions>>>,
    to_show: Query<
        &AudioSettingsNode,
        (
            With<ActionStateDriver<SettingsMenuUIActions>>,
            Without<DisplaySettingsNode>,
        ),
    >,
    mut toggle_display: Query<&mut Style>,
    mut audio_settings_actions: ResMut<ToggleActions<AudioMenuUIActions>>,
    mut display_settings_actions: ResMut<ToggleActions<DisplaySettingsMenuUIActions>>,
) {
    for n in q.iter() {
        if n.just_pressed(SettingsMenuUIActions::ToggleAudioSettings) {
            audio_settings_actions.enabled = true;
            display_settings_actions.enabled = false;
            let [mut show_style, mut hide_style] =
                toggle_display.many_mut([to_show.single().0, to_hide.single().0]);
            show_style.display = Display::Grid;
            hide_style.display = Display::None;
        }
    }
}

fn exit_settings_menu(
    q: Query<&ActionState<SettingsMenuUIActions>>,
    mut next_state: ResMut<NextState<UIFocus>>,
) {
    for n in q.iter() {
        if n.just_pressed(SettingsMenuUIActions::ExitSettings) {
            next_state.set(UIFocus::MainMenu);
        }
    }
}

fn make_audio_level_bar(
    commands: &mut Commands,
    row: i16,
    level: f32,
    comp: impl Component,
) -> Entity {
    let audio_level_root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: ALL,
                    height: ALL,
                    grid_column: GridPlacement::start(2),
                    grid_row: GridPlacement::start(row),

                    padding: UiRect::horizontal(into_pct(2. / 103.)),
                    ..default()
                },
                z_index: ZIndex::Local(20),
                ..default()
            },
            comp,
            Interaction::None,
            RelativeCursorPosition::default(),
            UiRange(level),
        ))
        .id();

    let icon_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: ALL,
                height: ALL,
                ..default()
            },
            z_index: ZIndex::Local(15),
            ..default()
        })
        .id();
    let cool_bar = commands.make_icon("temp_bar.png".into());
    commands
        .entity(cool_bar)
        .insert(Style {
            width: ALL,
            height: ALL,
            ..default()
        })
        .insert(ZIndex::Local(15));

    let white_fill = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: ALL,
                    height: ALL,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                z_index: ZIndex::Local(10),
                background_color: Color::WHITE.into(),
                ..default()
            },
            FillBar,
        ))
        .id();
    commands
        .entity(audio_level_root)
        .insert(Target(white_fill))
        .add_child(icon_layout);
    commands
        .entity(icon_layout)
        .push_children(&[white_fill, cool_bar]);

    audio_level_root
}

fn watch_audio_bars(
    mut fill_targets: Query<&mut Style, With<FillBar>>,
    ranges: Query<
        (
            &UiRange,
            &Target,
            Has<SFXVolumeLevel>,
            Has<GlobalVolumeLevel>,
            Has<MusicVolumeLevel>,
        ),
        (Changed<UiRange>, Without<FillBar>),
    >,
    mut volume_settings: ResMut<VolumeSettings>,
) {
    ranges
        .iter()
        .for_each(|(range, target, sfx, global, music)| {
            if sfx {
                volume_settings.sfx_user_setting = range.0;
            }
            if music {
                volume_settings.music_user_setting = range.0;
            }
            if global {
                volume_settings.global_user_setting = range.0;
            }
            if let Ok(mut style) = fill_targets.get_mut(target.0) {
                style.width = into_pct(range.0);
            }
        });
}

fn make_resolution_selection_component(
    commands: &mut Commands,
    resolution_options: Vec<(f32, f32)>,
    current: (f32, f32),
) -> Entity {
    let mut grid_row_tracks: Vec<_> = vec![];
    for _ in 1..resolution_options.len() {
        grid_row_tracks.push(GridTrack::fr(1.));
    }
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Grid,
                grid_column: GridPlacement::start(2),
                grid_row: GridPlacement::start(2),
                grid_template_rows: grid_row_tracks,
                border: UiRect {
                    bottom: px(1.),
                    right: px(1.),
                    left: px(0.),
                    top: px(0.),
                },
                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();

    let mut i: usize = 1;
    for entry in resolution_options.iter() {
        let (bgcolor, textcolor) = if *entry == current {
            (Color::BLACK, Color::WHITE)
        } else {
            (Color::WHITE, Color::BLACK)
        };
        let bordersize = if i < resolution_options.len() { 1. } else { 0. };
        let text = commands
            .spawn(TextBundle::from_section(
                format!("{:?}x{:?}", entry.0 as i32, entry.1 as i32),
                TextStyle::local(12., textcolor),
            ))
            .id();
        let section = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        grid_row: GridPlacement::start(i as i16),
                        border: UiRect::bottom(px(bordersize)),
                        ..default()
                    },
                    background_color: bgcolor.into(),
                    border_color: Color::BLACK.into(),
                    ..default()
                },
                UiResolutionInput(*entry),
                Interaction::None,
            ))
            .id();
        commands.entity(root).add_child(section);
        commands.entity(section).add_child(text);
        i += 1;
    }
    root
}

fn interactive_range_set(
    mut q: Query<(&Interaction, &RelativeCursorPosition, &mut UiRange), With<Node>>,
) {
    q.iter_mut()
        .for_each(|(interaction, cursor, mut range)| match interaction {
            Interaction::Pressed => {
                if let Some(position) = cursor.normalized {
                    range.0 = position.x.clamp(0., 1.);
                }
            }
            _ => {}
        })
}

#[derive(SystemParam)]
struct ActiveWindowSettings<'w, 's> {
    q: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}
impl<'w, 's> ActiveWindowSettings<'w, 's> {
    fn current_resolution(&self) -> (f32, f32) {
        let res = &self.q.single().resolution;
        (res.width(), res.height())
    }
}

fn show_active_window_settings(
    mut commands: Commands,
    active_window_settings: Res<DisplaySettings>,
    mut resolution_selection: Query<(&mut BackgroundColor, &UiResolutionInput, &Children)>,
    mut child_text: Query<&mut Text, Without<UiResolutionInput>>,
    fullscreen_toggle: Query<&Children, (With<UiToggle>, With<Fullscreen>)>,
    asset_server: Res<AssetServer>,
) {
    let current = active_window_settings.resolution;

    for (mut color, UiResolutionInput(entry), children) in resolution_selection.iter_mut() {
        let (bgcolor, textcolor) = if *entry == current {
            (Color::BLACK, Color::WHITE)
        } else {
            (Color::WHITE, Color::BLACK)
        };

        *color = bgcolor.into();
        for child in children.iter() {
            if let Ok(mut text) = child_text.get_mut(*child) {
                text.sections[0].style.color = textcolor;
            }
        }
    }
    fullscreen_toggle.iter().for_each(|children| {
        for child in children.iter() {
            let checkbox_name = if active_window_settings.fullscreen {
                "green_check_icon.png"
            } else {
                "empty_5x5.png"
            };
            let image_bundle = ImageBundle {
                image: UiImage {
                    texture: asset_server.load(checkbox_name),
                    ..default()
                },
                ..default()
            };
            commands.entity(*child).insert(image_bundle);
        }
    })
}
fn set_window_resolution(
    q: Query<(&UiResolutionInput, &Interaction)>,
    mut display_settings: ResMut<DisplaySettings>,
) {
    q.iter().for_each(
        |(UiResolutionInput(target), interaction)| match interaction {
            Interaction::Pressed => display_settings.resolution = *target,
            _ => {}
        },
    );
}
fn set_fullscreen_mode(
    q: Query<&Interaction, With<Fullscreen>>,
    mut display_settings: ResMut<DisplaySettings>,
) {
    q.iter().for_each(|interaction| match interaction {
        Interaction::Pressed => display_settings.fullscreen = !display_settings.fullscreen,
        _ => {}
    });
}
