use std::time::Duration;

use bevy::prelude::*;

use crate::{
    ant::{ForagerAnt, IdleAnt, NursemaidAnt},
    colony::{AntCapacity, AntPopulation, Colony, LaborData, LaborPhase, LarvaTarget, MaxFood},
    food::FoodQuant,
    menu_ui::UIAnchorNode,
    ui_helpers::*,
    upgrades::spawn_upgrade_buttons,
    UIFocus,
};

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
                (
                    food_text_update,
                    ant_pop_meter_update.after(LaborPhase::Task),
                    larva_target_display,
                ),
                (increment_target_larva, decrement_target_larva),
            )
                .chain(),
        );
    }
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
                    ..default()
                },
                ..default()
            },
            Name::new("Gamefield UI Root"),
            GamefieldUIRoot,
        ))
        .id();
    let menu_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Vw(20.0),
                height: Val::Vh(90.0),
                align_self: AlignSelf::End,
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .id();
    let resource_label_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(10.),
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            background_color: Color::rgb_u8(99, 155, 255).into(),
            ..default()
        })
        .id();
    let food_bar_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Vw(95.0),
                height: Val::Vh(5.),
                border: UiRect::all(Val::Px(2.)),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Center,
                ..default()
            },

            background_color: Color::WHITE.into(),
            border_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    let food_bar_mask = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_self: AlignSelf::Start,
                    position_type: PositionType::Relative,
                    ..default()
                },
                background_color: Color::GREEN.into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            GamefieldUIFoodBar,
        ))
        .id();
    let food_bar_label = commands
        .spawn((
            TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "0",
                        TextStyle {
                            font_size: 32.0,
                            color: Color::BLACK.into(),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "\n",
                        TextStyle {
                            font_size: 32.0,
                            color: Color::BLACK.into(),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 32.0,
                            color: Color::BLACK.into(),
                            ..default()
                        },
                    ),
                ]),
                ..default()
            },
            GamefieldUIFoodBar,
        ))
        .id();
    let food_bar_label_layout = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            z_index: ZIndex::Local(20),
            ..default()
        })
        .id();
    let growth_label = commands
        .spawn((
            TextBundle {
                text: Text::from_sections([
                    TextSection::new(
                        "0/100",
                        TextStyle {
                            font_size: 32.,
                            color: Color::WHITE.into(),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "\n0",
                        TextStyle {
                            font_size: 16.,
                            color: Color::WHITE.into(),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "\n0",
                        TextStyle {
                            font_size: 16.,
                            color: Color::WHITE.into(),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "\n0",
                        TextStyle {
                            font_size: 16.,
                            color: Color::WHITE.into(),
                            ..default()
                        },
                    ),
                ]),
                style: Style {
                    width: Val::Percent(50.),
                    ..default()
                },
                ..default()
            },
            Name::new("GamefieldUI Ant Pop Label"),
            GamefieldUIAntPopLabel,
        ))
        .id();

    let larva_button_plus = commands
        .make_button(
            "+",
            TextStyleBuilder::new().set_size(16.0).build(),
            StyleBuilder::new()
                .set_size(Val::Px(18.0), Val::Px(18.0))
                .build(),
            Color::RED,
            LarvaPlus,
        )
        .id();
    let larva_target_display = commands
        .make_text(
            "1",
            TextStyleBuilder::new().set_size(16.0).build(),
            Some(TargetLarvaDisplay),
        )
        .id();
    let larva_button_minus = commands
        .make_button(
            "-",
            TextStyleBuilder::new().set_size(16.0).build(),
            StyleBuilder::new()
                .set_size(Val::Px(18.0), Val::Px(18.0))
                .build(),
            Color::RED,
            LarvaMinus,
        )
        .id();

    let upgrade_buttons = spawn_upgrade_buttons(&mut commands, &asset_server);
    //resource_label_layout,
    let menu_children = [
        [resource_label_layout].as_slice(),
        upgrade_buttons.as_slice(),
    ]
    .concat();
    commands.entity(anchor.0).add_child(root);
    commands.entity(root).add_child(food_bar_layout);
    commands.entity(root).add_child(menu_layout);
    commands
        .entity(menu_layout)
        .push_children(menu_children.as_slice());

    commands.entity(resource_label_layout).push_children(&[
        growth_label,
        larva_button_minus,
        larva_target_display,
        larva_button_plus,
    ]);

    commands
        .entity(food_bar_layout)
        .push_children(&[food_bar_label_layout, food_bar_mask]);
    commands
        .entity(food_bar_label_layout)
        .add_child(food_bar_label);
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
        text.sections[0].value = format!("  {:?}  ", target_larva)
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
