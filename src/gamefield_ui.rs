use std::time::Duration;

use bevy::prelude::*;

use crate::{
    colony::{AntCapacity, AntPopulation, Colony, LarvaTarget, MaxFood},
    food::FoodQuant,
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
                    ant_pop_meter_update,
                    food_meter_update,
                    larva_target_display,
                ),
                (increment_target_larva, decrement_target_larva),
            )
                .chain(),
        );
    }
}

fn init_gamefield_ui(mut commands: Commands) {
    let root_node = (
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::End,
                ..default()
            },
            ..default()
        },
        Name::new("Gamefield UI Root"),
        GamefieldUIRoot,
    );
    let menu_layout_node = NodeBundle {
        style: Style {
            width: Val::Vw(20.0),
            height: Val::Vh(90.0),
            align_self: AlignSelf::End,
            align_items: AlignItems::End,
            justify_content: JustifyContent::Start,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::BLACK.into(),
        ..default()
    };
    let resource_label_layout_node = NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(10.),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        background_color: Color::BLUE.into(),
        ..default()
    };
    let food_bar_layout_node = NodeBundle {
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
    };
    let food_bar_mask_node = (
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
    );
    let food_bar_label_node = (
        TextBundle {
            text: Text::from_sections([
                TextSection::new("0", TextStyle::default()),
                TextSection::new("\n", TextStyle::default()),
                TextSection::new("", TextStyle::default()),
            ]),
            background_color: Color::BLACK.into(),
            ..default()
        },
        GamefieldUIFoodBar,
    );

    let food_label_node = (
        TextBundle {
            text: Text::from_section(
                "0",
                TextStyle {
                    font_size: 12.,
                    color: Color::WHITE.into(),
                    ..default()
                },
            ),
            style: Style {
                width: Val::Percent(50.),
                ..default()
            },
            ..default()
        },
        Name::new("GamefieldUI Food Label"),
        GamefieldUIFoodLabel,
    );
    let growth_label_node = (
        TextBundle {
            text: Text::from_section(
                "0/100",
                TextStyle {
                    font_size: 12.,
                    color: Color::WHITE.into(),
                    ..default()
                },
            ),
            style: Style {
                width: Val::Percent(50.),
                ..default()
            },
            ..default()
        },
        Name::new("GamefieldUI Ant Pop Label"),
        GamefieldUIAntPopLabel,
    );

    let root = commands.spawn(root_node).id();
    let menu_layout = commands.spawn(menu_layout_node).id();
    let resource_label_layout = commands.spawn(resource_label_layout_node).id();

    let food_label = commands.spawn(food_label_node).id();
    let growth_label = commands.spawn(growth_label_node).id();

    let larva_button_plus = commands
        .make_button(
            "+",
            TextStyleBuilder::new().set_size(12.0).build(),
            StyleBuilder::new()
                .set_size(Val::Px(14.0), Val::Px(14.0))
                .build(),
            Color::RED,
            LarvaPlus,
        )
        .id();
    let larva_target_display = commands
        .make_text(
            "1",
            TextStyleBuilder::new().set_size(12.0).build(),
            Some(TargetLarvaDisplay),
        )
        .id();
    let larva_button_minus = commands
        .make_button(
            "-",
            TextStyleBuilder::new().set_size(12.0).build(),
            StyleBuilder::new()
                .set_size(Val::Px(14.0), Val::Px(14.0))
                .build(),
            Color::RED,
            LarvaMinus,
        )
        .id();

    let food_bar_layout = commands.spawn(food_bar_layout_node).id();
    let food_bar_label = commands.spawn(food_bar_label_node).id();
    let food_bar_mask = commands.spawn(food_bar_mask_node).id();

    let upgrade_buttons = spawn_upgrade_buttons(&mut commands);
    //resource_label_layout,
    let menu_children = [
        [resource_label_layout].as_slice(),
        upgrade_buttons.as_slice(),
    ]
    .concat();

    commands.entity(root).add_child(menu_layout);
    commands
        .entity(menu_layout)
        .push_children(menu_children.as_slice());

    commands.entity(resource_label_layout).push_children(&[
        food_label,
        growth_label,
        larva_button_minus,
        larva_target_display,
        larva_button_plus,
    ]);
    commands.entity(root).add_child(food_bar_layout);
    commands
        .entity(food_bar_layout)
        .push_children(&[food_bar_label, food_bar_mask]);
}

fn food_text_update(
    mut food_text: Query<&mut Text, With<GamefieldUIFoodLabel>>,
    q_col: Query<&FoodQuant, With<Colony>>,
) {
    for mut text in food_text.iter_mut() {
        if let Ok(food) = q_col.get_single() {
            text.sections[0].value = format!("Food: {:?} ", food.0);
        }
    }
}
fn food_meter_update(
    mut text_q: Query<&mut Text, With<GamefieldUIFoodBar>>,
    mut style_q: Query<&mut Style, (With<GamefieldUIFoodBar>, Without<Text>)>,

    q_col: Query<(&FoodQuant, &MaxFood), With<Colony>>,
) {
    if let Ok((food, maxfood)) = q_col.get_single() {
        if let Ok(mut text) = text_q.get_single_mut() {
            text.sections[0].value = format!("Food: {:?}", food.0);
            text.sections[1].value = " / ".into();
            text.sections[2].value = format!("{:?}", maxfood.0);
        }
        if let Ok(mut style) = style_q.get_single_mut() {
            style.width = Val::Percent(100.0 * (food.0 as f32 / maxfood.0 as f32));
        }
    }
}

fn ant_pop_meter_update(
    mut food_text: Query<&mut Text, With<GamefieldUIAntPopLabel>>,
    q_col: Query<(&AntPopulation, &AntCapacity), With<Colony>>,
) {
    for mut text in food_text.iter_mut() {
        if let Ok((ant_pop, ant_cap)) = q_col.get_single() {
            text.sections[0].value = format!("Ants: {:?}/{:?}", ant_pop.0, ant_cap.0);
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
