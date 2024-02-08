use std::{borrow::BorrowMut, default, marker::PhantomData, time::Duration};

use bevy::{prelude::*, text::TextLayoutInfo};

use crate::{
    ant::{ForagerAnt, IdleAnt, NursemaidAnt},
    colony::{AntCapacity, AntPopulation, Colony, LaborData, LaborPhase, LarvaTarget, MaxFood},
    food::FoodQuant,
    menu_ui::UIAnchorNode,
    ui_helpers::*,
    upgrades::spawn_upgrade_buttons,
    UIFocus,
};
use bevy_nine_slice_ui::*;

pub struct GamefieldUI;

#[derive(Component)]
pub struct GamefieldUIRoot;
#[derive(Component)]
pub struct GamefieldUIFoodLabel;
#[derive(Component)]
pub struct GamefieldUIAntPopLabel;

#[derive(Component)]
struct GamefieldUIFoodBar;
#[derive(Component)]
struct GamefieldUIAntBar;

#[derive(Component, Default)]
struct AntCount<T: Component + Default> {
    marker: PhantomData<T>,
}

#[derive(Component)]
struct LarvaPlus;
#[derive(Component)]
struct LarvaMinus;

#[derive(Component)]
struct TargetLarvaDisplay;

impl Plugin for GamefieldUI {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
//We have to make this public so that our settings menu can re-draw the UI when the resolution changes. Very stupid, very necessary.
pub fn init_gamefield_ui(
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
    let food_bar_layout = commands
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
    let food_icon = commands
        .spawn(ImageBundle {
            image: UiImage {
                texture: asset_server.load("food_icon.png"),

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
    let food_bar_mask = commands
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
    let food_bar_fill = commands
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
                background_color: Color::rgb_u8(106, 190, 48).into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            GamefieldUIFoodBar,
        ))
        .id();
    let food_bar_label = commands
        .make_text_sections(vec![
            ("0", TextStyle::local(LARGE, Color::BLACK)),
            ("\n", TextStyle::local(LARGE, Color::BLACK)),
            ("", TextStyle::local(LARGE, Color::BLACK)),
        ])
        .insert(GamefieldUIFoodBar)
        .id();

    let food_bar_label_layout = commands
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

    let ant_bar_layout = commands
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
    let ant_icon = commands
        .spawn(ImageBundle {
            image: UiImage {
                texture: asset_server.load("ant_icon.png"),
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
    let ant_bar_mask = commands
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
    let ant_bar_fill = commands
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
                background_color: Color::rgb_u8(99, 155, 255).into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            GamefieldUIAntBar,
        ))
        .id();
    let ant_bar_label = commands
        .make_text_sections(vec![
            ("0", TextStyle::local(LARGE, Color::BLACK)),
            ("\n", TextStyle::local(LARGE, Color::BLACK)),
            ("", TextStyle::local(LARGE, Color::BLACK)),
        ])
        .insert(GamefieldUIAntBar)
        .id();

    let ant_bar_label_layout = commands
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

    let egg_button_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: px(572.),
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
                width: px(233.),
                height: px(71.),
                padding: UiRect {
                    top: px(1.),
                    bottom: px(1.),
                    left: px(5.),
                    right: px(5.),
                },
                display: Display::Grid,
                grid_template_rows: vec![GridTrack::percent(50.), GridTrack::percent(50.)],
                grid_template_columns: vec![GridTrack::px(151.), GridTrack::px(81.)],
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
    let forager_row = commands
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
    let forager_label_layout = commands
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
                asset_server.load("nine_slice/bgW_bB_soft_corner_upper_left_trimmed_inner.png"),
            ),
            ..default()
        })
        .id();
    let forager_label = commands
        .make_text("Foragers:", TextStyle::local(MEDIUM, Color::BLACK))
        .insert(Style {
            margin: UiRect::right(Val::Px(5.0)),
            ..default()
        })
        .id();
    let forager_icon_layout = commands
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
    let forager_icon = commands
        .spawn(ImageBundle {
            image: UiImage {
                texture: asset_server.load("food_icon.png"),

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
    let forager_bar_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(54.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();
    let forager_bar_mask = commands
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
    let forager_bar_fill = commands
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
                background_color: Color::rgb_u8(106, 190, 48).into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            AntCount::<ForagerAnt>::default(),
        ))
        .id();
    let forager_bar_label = commands
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
            AntCount::<ForagerAnt>::default(),
        ))
        .id();

    //
    let nursemaid_row = commands
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
    let nursemaid_label_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: Val::Percent(30.),
                height: Val::Percent(100.),
                margin: UiRect::right(Val::Px(-1.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgW_bB_soft_corner_middle_left_trimmed_inner.png"),
            ),
            ..default()
        })
        .id();
    let nursemaid_label = commands
        .make_text("NurseMaids:", TextStyle::local(MEDIUM, Color::BLACK))
        .insert(Style {
            margin: UiRect::right(Val::Px(5.)),
            ..default()
        })
        .id();
    let nursemaid_icon_layout = commands
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
    let nursemaid_icon = commands
        .spawn(ImageBundle {
            image: UiImage {
                texture: asset_server.load("egg_icon.png"),

                ..default()
            },
            style: Style {
                margin: UiRect {
                    left: Val::Percent(10.),
                    right: Val::Percent(5.),
                    top: Val::Percent(6.),
                    bottom: Val::Percent(6.),
                },
                ..default()
            },
            ..default()
        })
        .id();
    let nursemaid_bar_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(54.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();
    let nursemaid_bar_mask = commands
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
    let nursemaid_bar_fill = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_self: AlignSelf::Start,
                    position_type: PositionType::Relative,
                    ..default()
                },
                background_color: Color::rgb_u8(69, 40, 60).into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            AntCount::<NursemaidAnt>::default(),
        ))
        .id();
    let nursemaid_bar_label = commands
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
            AntCount::<NursemaidAnt>::default(),
        ))
        .id();
    //
    let idler_row = commands
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
    let idler_label_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: Val::Percent(30.),
                height: Val::Percent(100.),
                margin: UiRect::right(Val::Px(-1.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            nine_slice_texture: NineSliceUiTexture::from_image(
                asset_server.load("nine_slice/bgW_bB_soft_corner_lower_left_trimmed_inner.png"),
            ),
            ..default()
        })
        .id();
    let idler_label = commands
        .make_text("Idlers:", TextStyle::local(MEDIUM, Color::BLACK))
        .insert(Style {
            margin: UiRect::right(Val::Px(5.0)),
            ..default()
        })
        .id();

    let idler_icon_layout = commands
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
    let idler_icon = commands
        .spawn(ImageBundle {
            image: UiImage {
                texture: asset_server.load("zs_icon.png"),

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
    let idler_bar_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(54.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();
    let idler_bar_mask = commands
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
    let idler_bar_fill = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_self: AlignSelf::Start,
                    position_type: PositionType::Relative,
                    ..default()
                },
                background_color: Color::rgb_u8(172, 50, 50).into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            AntCount::<IdleAnt>::default(),
        ))
        .id();
    let idler_bar_label = commands
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
            AntCount::<IdleAnt>::default(),
        ))
        .id();

    //

    let upgrade_menu_layout = commands
        .spawn(NineSliceUiMaterialBundle {
            style: Style {
                width: px(119.),
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
    //resource_label_layout,
    let menu_children = [
        //[resource_label_layout].as_slice(),
        upgrade_buttons.as_slice(),
    ]
    .concat();
    commands.entity(anchor.0).add_child(root);
    commands.entity(root).add_child(big_bar_layout);
    commands
        .entity(big_bar_layout)
        .push_children(&[food_bar_layout, ant_bar_layout]);
    commands.entity(food_bar_layout).push_children(&[
        food_icon,
        food_bar_label_layout,
        food_bar_mask,
        food_bar_fill,
    ]);
    commands
        .entity(food_bar_label_layout)
        .add_child(food_bar_label);
    commands.entity(ant_bar_layout).push_children(&[
        ant_icon,
        ant_bar_label_layout,
        ant_bar_mask,
        ant_bar_fill,
    ]);
    commands
        .entity(ant_bar_label_layout)
        .add_child(ant_bar_label);

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
    commands.entity(forager_row).push_children(&[
        forager_label_layout,
        forager_icon_layout,
        forager_bar_layout,
    ]);
    commands
        .entity(forager_label_layout)
        .add_child(forager_label);
    commands.entity(forager_icon_layout).add_child(forager_icon);
    commands
        .entity(forager_bar_layout)
        .push_children(&[forager_bar_mask, forager_bar_fill]);
    commands
        .entity(forager_bar_mask)
        .add_child(forager_bar_label);
    //
    commands.entity(nursemaid_row).push_children(&[
        nursemaid_label_layout,
        nursemaid_icon_layout,
        nursemaid_bar_layout,
    ]);
    commands
        .entity(nursemaid_label_layout)
        .add_child(nursemaid_label);
    commands
        .entity(nursemaid_icon_layout)
        .push_children(&[nursemaid_icon]);
    commands
        .entity(nursemaid_bar_layout)
        .push_children(&[nursemaid_bar_mask, nursemaid_bar_fill]);
    commands
        .entity(nursemaid_bar_mask)
        .add_child(nursemaid_bar_label);
    //
    commands.entity(idler_row).push_children(&[
        idler_label_layout,
        idler_icon_layout,
        idler_bar_layout,
    ]);
    commands.entity(idler_label_layout).add_child(idler_label);
    commands
        .entity(idler_icon_layout)
        .push_children(&[idler_icon]);
    commands
        .entity(idler_bar_layout)
        .push_children(&[idler_bar_mask, idler_bar_fill]);
    commands.entity(idler_bar_mask).add_child(idler_bar_label);

    //

    commands.entity(root).add_child(upgrade_menu_layout);
    commands
        .entity(upgrade_menu_layout)
        .push_children(menu_children.as_slice());
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

fn ant_pop_meter_update(
    mut food_text: Query<&mut Text, With<GamefieldUIAntPopLabel>>,
    q_col: Query<
        (
            &AntPopulation,
            &AntCapacity,
            &LaborData<ForagerAnt>,
            &LaborData<NursemaidAnt>,
            &LaborData<IdleAnt>,
        ),
        With<Colony>,
    >,
) {
    for mut text in food_text.iter_mut() {
        if let Ok((ant_pop, ant_cap, forager_stats, nursemaid_stats, idle_stats)) =
            q_col.get_single()
        {
            text.sections[0].value = format!("Ants: {:?}/{:?}", ant_pop.0, ant_cap.0);
            text.sections[1].value = format!("\nForagers: {:?}", forager_stats.active);
            text.sections[2].value = format!("\nNursemaids: {:?}", nursemaid_stats.active);
            text.sections[3].value = format!("\nIdlers: {:?}", idle_stats.active);
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
    mut chill: Local<ChillOut>,
    time: Res<Time>,
    mut q_col: Query<&mut LarvaTarget, With<Colony>>,
    interaction: Query<&Interaction, (With<LarvaPlus>, Without<LarvaMinus>)>,
) {
    chill.be_chill(time.delta());
    if !chill.we_chillin() {
        interaction.iter().take(1).for_each(|n| match *n {
            Interaction::Pressed => {
                q_col.single_mut().0 += 1;
                chill.be_chillin();
            }
            _ => {}
        });
    }
}

fn decrement_target_larva(
    mut chill: Local<ChillOut>,
    time: Res<Time>,
    mut q_col: Query<&mut LarvaTarget, With<Colony>>,
    interaction: Query<&Interaction, (With<LarvaMinus>, Without<LarvaPlus>)>,
) {
    chill.be_chill(time.delta());
    if !chill.we_chillin() {
        interaction.iter().take(1).for_each(|n| match *n {
            Interaction::Pressed => {
                let mut larva_target = q_col.single_mut();
                larva_target.0 = 1.max(larva_target.0 - 1);
                chill.be_chillin();
            }
            _ => {}
        });
    }
}

struct ChillOut {
    chillin: bool,
    elapsed: Timer,
}
impl Default for ChillOut {
    fn default() -> Self {
        Self {
            chillin: false,
            elapsed: Timer::from_seconds(0.25, TimerMode::Once),
        }
    }
}
impl ChillOut {
    fn we_chillin(&self) -> bool {
        self.chillin
    }

    fn be_chillin(&mut self) {
        self.chillin = true;
    }

    fn be_chill(&mut self, delta: Duration) {
        match (self.chillin, self.done()) {
            (false, _) => {
                return;
            }
            (true, false) => {
                self.tick(delta);
            }
            (true, true) => {
                self.clear();
            }
        }
    }
    fn done(&self) -> bool {
        self.elapsed.finished()
    }
    fn tick(&mut self, delta: Duration) {
        self.elapsed.tick(delta);
    }
    fn clear(&mut self) {
        self.chillin = false;
        self.elapsed.reset();
    }
}
