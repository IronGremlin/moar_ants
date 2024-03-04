use std::marker::PhantomData;

use bevy::prelude::*;
use leafwing_input_manager::plugin::ToggleActions;

use super::{menu_ui::UIAnchorNode, ui_util::*, upgrades::spawn_upgrade_buttons};
use crate::{
    ant::{ForagerAnt, IdleAnt, NursemaidAnt},
    colony::{AntCapacity, AntPopulation, Colony, LaborData, LaborPhase, LarvaTarget, MaxFood},
    food::FoodQuant,
    playerinput::{CameraControl, GamefieldActions},
    UIFocus,
};
use bevy_nine_slice_ui::*;

pub struct GamefieldUI;

impl Plugin for GamefieldUI {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(UIFocus::Gamefield), activate_gamefield_actions)
            .add_systems(OnExit(UIFocus::Gamefield), deactivate_gamefield_actions)
            .add_systems(
                Update,
                init_gamefield_ui.run_if(in_state(UIFocus::Gamefield).and_then(run_once())),
            )
            .add_systems(
                Update,
                (
                    (food_text_update, ant_text_update),
                    (
                        ant_bar_update::<ForagerAnt>,
                        ant_bar_update::<NursemaidAnt>,
                        ant_bar_update::<IdleAnt>,
                    )
                        .after(LaborPhase::Task),
                    (
                        increment_target_larva,
                        decrement_target_larva,
                        larva_target_display,
                    ),
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct GamefieldUIRoot;
#[derive(Component, Default)]
pub struct GamefieldUIFoodLabel;
#[derive(Component, Default)]
pub struct GamefieldUIAntPopLabel;

#[derive(Component, Default)]
struct GamefieldUIFoodBar;
#[derive(Component, Default)]
struct GamefieldUIAntBar;

#[derive(Component, Default)]
struct AntCount<T: Component + Default> {
    marker: PhantomData<T>,
}
trait LaborTableDisplay {
    fn icon() -> String;
    fn left_edge_image() -> String;
    fn color() -> Color;
    fn name() -> String;
}

impl LaborTableDisplay for ForagerAnt {
    fn icon() -> String {
        "food_icon.png".into()
    }
    fn left_edge_image() -> String {
        "nine_slice/bgW_bB_soft_corner_upper_left_trimmed_inner.png".into()
    }
    fn color() -> Color {
        GREEN()
    }

    fn name() -> String {
        "Foragers:".into()
    }
}
impl LaborTableDisplay for NursemaidAnt {
    fn icon() -> String {
        "egg_icon.png".into()
    }
    fn left_edge_image() -> String {
        "nine_slice/bgW_bB_soft_corner_middle_left_trimmed_inner.png".into()
    }
    fn color() -> Color {
        PURPLE()
    }

    fn name() -> String {
        "NurseMaids:".into()
    }
}
impl LaborTableDisplay for IdleAnt {
    fn icon() -> String {
        "zs_icon.png".into()
    }
    fn left_edge_image() -> String {
        "nine_slice/bgW_bB_soft_corner_lower_left_trimmed_inner.png".into()
    }
    fn color() -> Color {
        RED()
    }

    fn name() -> String {
        "Idlers:".into()
    }
}

#[derive(Component)]
struct LarvaPlus;
#[derive(Component)]
struct LarvaMinus;

#[derive(Component)]
struct TargetLarvaDisplay;

fn activate_gamefield_actions(
    mut gamefield_actions: ResMut<ToggleActions<GamefieldActions>>,
    mut camera_actions: ResMut<ToggleActions<CameraControl>>,
) {
    gamefield_actions.enabled = true;
    camera_actions.enabled = true;
}
fn deactivate_gamefield_actions(
    mut gamefield_actions: ResMut<ToggleActions<GamefieldActions>>,
    mut camera_actions: ResMut<ToggleActions<CameraControl>>,
) {
    gamefield_actions.enabled = false;
    camera_actions.enabled = false;
}
fn init_gamefield_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    anchor: Res<UIAnchorNode>,
) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    ..default()
                },
                ..default()
            },
            Name::new("Gamefield UI Root"),
            GamefieldUIRoot,
        ))
        .id();

    let big_bar_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: px(682.0),
                height: px(109.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::left(px(6.)),
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgW_bG_soft_corner_2.png"),
            ),

            ..default()
        })
        .id();
    let food_bar_layout = make_big_bar::<GamefieldUIFoodBar>(
        &mut commands,
        &asset_server,
        GREEN(),
        "food_icon.png".into(),
    );
    let ant_bar_layout = make_big_bar::<GamefieldUIAntBar>(
        &mut commands,
        &asset_server,
        Color::rgb_u8(99, 155, 255),
        "ant_icon.png".into(),
    );

    let egg_button_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: px(582.),
                height: px(120.),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .id();
    let egg_button_grid_container = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: px(243.),
                height: px(71.),
                padding: UiRect {
                    top: px(1.),
                    bottom: px(1.),
                    left: px(5.),
                    right: px(5.),
                },
                display: Display::Grid,
                grid_template_rows: vec![GridTrack::percent(50.), GridTrack::percent(50.)],
                grid_template_columns: vec![GridTrack::px(151.), GridTrack::px(91.)],
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgW_bG_soft_corner_2.png"),
            ),
            ..default()
        })
        .insert(Name::new("Egg Button Grid Container"))
        .id();
    //This re-paints an equivalent border on top of our child node borders.
    //we end up painting this twice, but the first time gives us clipped corners - without that we'd get little white spots addorning the outer edge.
    let egg_button_grid_container_mask = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            z_index: ZIndex::Local(10),

            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgT_bG_soft_corner_2.png"),
            ),
            ..default()
        })
        .id();
    commands
        .entity(egg_button_grid_container)
        .add_child(egg_button_grid_container_mask);

    //tears
    let cost_legend_ant = commands
        .spawn(NodeBundle {
            style: Style {
                grid_row: GridPlacement::start(1),
                grid_column: GridPlacement::start(1),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::End,
                column_gap: Val::Percent(1.5),
                border: UiRect::right(Val::Px(2.)),

                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let bs_1_1 = commands
        .make_text("1", TextStyle::local(LARGE, Color::BLACK))
        .id();
    let bs_1_2 = commands.make_icon("egg_icon.png".to_owned());
    commands.entity(bs_1_2).insert(Style {
        aspect_ratio: Some(1.0),
        width: Val::Percent(12.),
        ..default()
    });
    let bs_1_3 = commands
        .make_text("+20", TextStyle::local(LARGE, Color::BLACK))
        .id();
    let bs_1_4 = commands.make_icon("food_icon.png".to_owned());
    commands.entity(bs_1_4).insert(Style {
        aspect_ratio: Some(1.0),
        width: Val::Percent(12.),
        ..default()
    });
    let bs_1_5 = commands.make_icon("arrow_icon.png".to_owned());
    commands.entity(bs_1_5).insert(Style {
        aspect_ratio: Some(1.0),
        margin: UiRect::horizontal(Val::Percent(0.5)),
        width: Val::Percent(12.),
        ..default()
    });
    let bs_1_6 = commands
        .make_text("1", TextStyle::local(LARGE, Color::BLACK))
        .id();
    let bs_1_7 = commands.make_icon("ant_icon.png".to_owned());
    commands.entity(bs_1_7).insert(Style {
        aspect_ratio: Some(1.0),
        width: Val::Percent(12.),
        margin: UiRect::right(Val::Percent(2.0)),
        ..default()
    });

    commands
        .entity(cost_legend_ant)
        .push_children(&[bs_1_1, bs_1_2, bs_1_3, bs_1_4, bs_1_5, bs_1_6, bs_1_7]);

    let cost_legend_egg = commands
        .spawn(NodeBundle {
            style: Style {
                grid_row: GridPlacement::start(2),
                grid_column: GridPlacement::start(1),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::End,
                column_gap: Val::Percent(1.5),
                border: UiRect {
                    top: Val::Px(2.),
                    bottom: Val::Px(0.),
                    left: Val::Px(0.),
                    right: Val::Px(2.),
                },

                ..default()
            },
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let bs_2_1 = commands
        .make_text("5", TextStyle::local(LARGE, Color::BLACK))
        .id();
    let bs_2_2 = commands.make_icon("ant_icon.png".to_owned());
    commands.entity(bs_2_2).insert(Style {
        aspect_ratio: Some(1.0),
        width: Val::Percent(12.),
        ..default()
    });
    let bs_2_3 = commands.make_icon("arrow_icon.png".to_owned());
    commands.entity(bs_2_3).insert(Style {
        aspect_ratio: Some(1.0),
        width: Val::Percent(12.),
        margin: UiRect::horizontal(Val::Percent(0.5)),
        ..default()
    });
    let bs_2_4 = commands
        .make_text("1", TextStyle::local(LARGE, Color::BLACK))
        .id();
    let bs_2_5 = commands.make_icon("egg_icon.png".to_owned());
    commands.entity(bs_2_5).insert(Style {
        aspect_ratio: Some(1.0),
        width: Val::Percent(12.),
        margin: UiRect::right(Val::Percent(2.)),
        ..default()
    });
    commands
        .entity(cost_legend_egg)
        .push_children(&[bs_2_1, bs_2_2, bs_2_3, bs_2_4, bs_2_5]);
    let big_ole_egg_button_layout = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Grid,
                margin: UiRect::right(px(10.)),
                grid_column: GridPlacement::start(2),
                grid_row: GridPlacement::start(1).set_span(2),
                grid_template_columns: vec![
                    GridTrack::percent(25.),
                    GridTrack::percent(25.),
                    GridTrack::percent(50.),
                ],
                grid_template_rows: vec![GridTrack::percent(50.)],
                ..default()
            },
            ..default()
        })
        .id();
    let egg_plus_button = commands
        .spawn((
            NodeBundle {
                style: Style {
                    grid_column: GridPlacement::start(1),
                    grid_row: GridPlacement::start(1),
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::End,
                    height: Val::Percent(45.),
                    bottom: Val::Percent(5.56),
                    border: UiRect::all(px(1.)),
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                ..default()
            },
            Interaction::None,
            LarvaPlus,
        ))
        .id();
    let egg_plus_button_icon = commands.make_icon("green_plus.png".into());
    commands
        .entity(egg_plus_button)
        .add_child(egg_plus_button_icon);

    let egg_minus_button = commands
        .spawn((
            NodeBundle {
                style: Style {
                    grid_column: GridPlacement::start(1),
                    grid_row: GridPlacement::start(2),
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Start,
                    height: Val::Percent(45.),
                    top: Val::Percent(5.56),
                    border: UiRect::all(px(1.)),
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                ..default()
            },
            Interaction::None,
            LarvaMinus,
        ))
        .id();
    let egg_minus_button_icon = commands.make_icon("red_minus.png".into());
    commands
        .entity(egg_minus_button)
        .add_child(egg_minus_button_icon);

    let big_egg = commands.make_icon("egg_icon.png".into());
    commands.entity(big_egg).insert(Style {
        grid_column: GridPlacement::start(2),
        grid_row: GridPlacement::start(1).set_span(2),
        height: Val::Percent(40.),
        align_self: AlignSelf::Center,
        justify_self: JustifySelf::Center,
        ..default()
    });
    let egg_count = commands
        .make_text("1", TextStyle::local(LARGE, Color::BLACK))
        .insert(TargetLarvaDisplay)
        .insert(Style {
            grid_column: GridPlacement::start(3),
            grid_row: GridPlacement::start(1).set_span(2),
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        })
        .id();

    let ant_labor_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: Val::Percent(66.5),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgW_bG_soft_corner_2.png"),
            ),
            ..default()
        })
        .id();

    let ant_labor_table = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(96.),
                height: Val::Percent(85.),
                margin: UiRect {
                    left: Val::Percent(2.),
                    right: Val::Auto,
                    top: Val::Percent(1.9),
                    bottom: Val::Auto,
                },
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .id();
    let forager_row = make_ant_labor_row::<ForagerAnt>(&mut commands, &asset_server);
    let nursemaid_row = make_ant_labor_row::<NursemaidAnt>(&mut commands, &asset_server);
    let idler_row = make_ant_labor_row::<IdleAnt>(&mut commands, &asset_server);

    let upgrade_menu_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: px(162.),
                max_height: Val::Vh(70.0),
                padding: UiRect::vertical(px(4.)),
                grid_template_columns: vec![GridTrack::percent(100.)],
                align_self: AlignSelf::End,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                row_gap: px(4.),
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgW_bG_soft_corner_2.png"),
            ),
            ..default()
        })
        .id();

    let upgrade_buttons = spawn_upgrade_buttons(&mut commands, &asset_server);
    let menu_children = [upgrade_buttons.as_slice()].concat();
    commands.entity(anchor.0).add_child(root);
    commands.entity(root).add_child(big_bar_layout);
    commands
        .entity(big_bar_layout)
        .push_children(&[food_bar_layout, ant_bar_layout]);

    commands.entity(root).add_child(egg_button_layout);
    commands
        .entity(egg_button_layout)
        .push_children(&[egg_button_grid_container, ant_labor_layout]);
    commands.entity(egg_button_grid_container).push_children(&[
        cost_legend_ant,
        cost_legend_egg,
        big_ole_egg_button_layout,
    ]);
    commands.entity(big_ole_egg_button_layout).push_children(&[
        egg_plus_button,
        egg_minus_button,
        big_egg,
        egg_count,
    ]);
    commands.entity(ant_labor_layout).add_child(ant_labor_table);
    commands
        .entity(ant_labor_table)
        .push_children(&[forager_row, nursemaid_row, idler_row]);

    commands.entity(root).add_child(upgrade_menu_layout);
    commands
        .entity(upgrade_menu_layout)
        .push_children(menu_children.as_slice());
}

fn make_big_bar<C: Component + Default>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    color: Color,
    icon_path: String,
) -> Entity {
    let big_bar_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: px(593.),
                height: px(32.),
                margin: UiRect {
                    left: px(5.),
                    right: px(13.),
                    top: px(13.),
                    bottom: px(5.),
                },

                flex_direction: FlexDirection::Row,
                ..default()
            },
            z_index: ZIndex::Local(5),

            ..default()
        })
        .id();
    let icon = commands
        .spawn(ImageBundle {
            image: UiImage {
                texture: asset_server.load(icon_path),

                ..default()
            },
            style: Style {
                width: px(32.),
                aspect_ratio: Some(1.0),
                margin: UiRect {
                    top: px(1.),
                    bottom: px(1.),
                    left: px(1.),
                    right: px(6.),
                },
                ..default()
            },
            ..default()
        })
        .id();
    let bar_mask = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                left: px(39.), // image size + image margin + 1px leeway
                width: ALL,
                height: ALL,
                align_self: AlignSelf::Start,
                position_type: PositionType::Absolute,
                ..default()
            },
            z_index: ZIndex::Local(15),
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgT_bB_rounded_hard.png"),
            ),

            ..default()
        })
        .id();
    let bar_fill = commands
        .spawn((
            NodeBundle {
                style: Style {
                    left: px(39.),
                    width: ALL,
                    height: ALL,
                    align_self: AlignSelf::Start,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: color.into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            C::default(),
        ))
        .id();
    let bar_label = commands
        .make_text_sections(vec![
            ("0", TextStyle::local(LARGE, Color::BLACK)),
            ("\n", TextStyle::local(LARGE, Color::BLACK)),
            ("", TextStyle::local(LARGE, Color::BLACK)),
        ])
        .insert(C::default())
        .id();

    let bar_label_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: ALL,
                height: ALL,
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            z_index: ZIndex::Local(20),
            ..default()
        })
        .id();
    commands
        .entity(big_bar_layout)
        .push_children(&[icon, bar_label_layout, bar_mask, bar_fill]);
    commands.entity(bar_label_layout).add_child(bar_label);
    big_bar_layout
}

fn make_ant_labor_row<C: Component + Default + LaborTableDisplay>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) -> Entity {
    let labor_row = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(33.3),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .id();
    let labor_label_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: Val::Percent(30.),
                height: Val::Percent(100.),
                margin: UiRect::right(Val::Px(-1.0)),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load(C::left_edge_image()),
            ),
            ..default()
        })
        .id();
    let labor_label = commands
        .make_text(&C::name().clone(), TextStyle::local(MEDIUM, Color::BLACK))
        .insert(Style {
            margin: UiRect::right(Val::Px(5.0)),
            ..default()
        })
        .id();
    let labor_icon_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: Val::Percent(16.),
                height: Val::Percent(100.),
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgW_bB_square_trimmed_inner.png"),
            ),
            ..default()
        })
        .id();
    let labor_icon = commands
        .spawn(ImageBundle {
            image: UiImage {
                texture: asset_server.load(C::icon()),

                ..default()
            },
            style: Style {
                margin: UiRect {
                    left: Val::Percent(10.),
                    right: Val::Percent(10.),
                    top: Val::Percent(2.),
                    bottom: Val::Percent(2.),
                },
                ..default()
            },
            ..default()
        })
        .id();
    let labor_bar_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(54.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();
    let labor_bar_mask = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_self: AlignSelf::Start,
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            z_index: ZIndex::Local(15),
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgT_bB_right_rounded_soft.png"),
            ),

            ..default()
        })
        .id();
    let labor_bar_fill = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_self: AlignSelf::Start,
                    position_type: PositionType::Relative,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: C::color().into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            AntCount::<C>::default(),
        ))
        .id();
    let labor_bar_label = commands
        .make_text_sections(vec![
            ("0", TextStyle::local(MEDIUM, Color::BLACK)),
            ("\n", TextStyle::local(MEDIUM, Color::BLACK)),
            ("", TextStyle::local(MEDIUM, Color::BLACK)),
        ])
        .insert((
            Style {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            ZIndex::Local(15),
            AntCount::<C>::default(),
        ))
        .id();
    commands.entity(labor_row).push_children(&[
        labor_label_layout,
        labor_icon_layout,
        labor_bar_layout,
    ]);
    commands.entity(labor_label_layout).add_child(labor_label);
    commands.entity(labor_icon_layout).add_child(labor_icon);
    commands
        .entity(labor_bar_layout)
        .push_children(&[labor_bar_mask, labor_bar_fill]);
    commands.entity(labor_bar_mask).add_child(labor_bar_label);
    labor_row
}

fn food_text_update(
    mut food_text: Query<&mut Text, With<GamefieldUIFoodBar>>,
    mut style_q: Query<&mut Style, (With<GamefieldUIFoodBar>, Without<Text>)>,
    q_col: Query<(&FoodQuant, &MaxFood), With<Colony>>,
) {
    if let Ok((food, maxfood)) = q_col.get_single() {
        for mut text in food_text.iter_mut() {
            text.sections[0].value = format!("Food: {:?} ", food.0);
            text.sections[1].value = "/".into();
            text.sections[2].value = format!(" {:?}", maxfood.0);
        }
        for mut style in style_q.iter_mut() {
            style.width = Val::Percent(100. * (food.0 as f32 / maxfood.0 as f32));
        }
    }
}
fn ant_bar_update<T: Component + Default>(
    mut text_query: Query<&mut Text, With<AntCount<T>>>,
    mut bar_query: Query<&mut Style, (With<AntCount<T>>, Without<Text>)>,
    q_col: Query<(&AntPopulation, &LaborData<T>), With<Colony>>,
) {
    if let Ok((ant_pop, labor_stats)) = q_col.get_single() {
        for mut text in text_query.iter_mut() {
            text.sections[0].value = format!("{:?} ", labor_stats.active);
            text.sections[1].value = "/".into();
            text.sections[2].value = format!(" {:?}", ant_pop.0);
        }
        for mut style in bar_query.iter_mut() {
            style.width = Val::Percent(100. * (labor_stats.active as f32 / ant_pop.0 as f32));
        }
    }
}

fn ant_text_update(
    mut ant_text: Query<&mut Text, With<GamefieldUIAntBar>>,
    mut style_q: Query<&mut Style, (With<GamefieldUIAntBar>, Without<Text>)>,
    q_col: Query<(&AntPopulation, &AntCapacity), With<Colony>>,
) {
    if let Ok((ants, maxants)) = q_col.get_single() {
        for mut text in ant_text.iter_mut() {
            text.sections[0].value = format!("Ants: {:?} ", ants.0);
            text.sections[1].value = "/".into();
            text.sections[2].value = format!(" {:?}", maxants.0);
        }
        for mut style in style_q.iter_mut() {
            style.width = Val::Percent(100. * (ants.0 as f32 / maxants.0 as f32));
        }
    }
}

fn larva_target_display(
    mut larva_text: Query<&mut Text, With<TargetLarvaDisplay>>,
    q_col: Query<&LarvaTarget, With<Colony>>,
) {
    for mut text in larva_text.iter_mut() {
        let target_larva = q_col.single().0;
        text.sections[0].value = format!("{:?}", target_larva)
    }
}

fn increment_target_larva(
    mut chill: Local<CoolDown>,
    time: Res<Time>,
    mut q_col: Query<&mut LarvaTarget, With<Colony>>,
    interaction: Query<&Interaction, (With<LarvaPlus>, Without<LarvaMinus>)>,
) {
    chill.handle_time(time.delta());
    if !chill.cooling_down() {
        interaction.iter().take(1).for_each(|n| match *n {
            Interaction::Pressed => {
                q_col.single_mut().0 += 1;
                chill.start_cooldown();
            }
            _ => {}
        });
    }
}

fn decrement_target_larva(
    mut chill: Local<CoolDown>,
    time: Res<Time>,
    mut q_col: Query<&mut LarvaTarget, With<Colony>>,
    interaction: Query<&Interaction, (With<LarvaMinus>, Without<LarvaPlus>)>,
) {
    chill.handle_time(time.delta());
    if !chill.cooling_down() {
        interaction.iter().take(1).for_each(|n| match *n {
            Interaction::Pressed => {
                let mut larva_target = q_col.single_mut();
                larva_target.0 = 1.max(larva_target.0 - 1);
                chill.start_cooldown();
            }
            _ => {}
        });
    }
}
