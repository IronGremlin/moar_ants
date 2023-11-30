
use bevy::prelude::*;

use crate::{
    colony::{AntCapacity, AntPopulation, Colony},
    food::FoodQuant,
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

impl Plugin for GamefieldUI {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            init_gamefield_ui.run_if(in_state(UIFocus::Gamefield).and_then(run_once())),
        )
        .add_systems(Update, (food_meter_update, ant_pop_meter_update));
    }
}

fn init_gamefield_ui(mut commands: Commands) {
    let root_node = (
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
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
            height: Val::Vh(100.0),
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

    commands
        .entity(resource_label_layout)
        .push_children(&[food_label, growth_label]);
}

fn food_meter_update(
    mut food_text: Query<&mut Text, With<GamefieldUIFoodLabel>>,
    q_col: Query<&FoodQuant, With<Colony>>,
) {
    for mut text in food_text.iter_mut() {
        if let Ok(food) = q_col.get_single() {
            text.sections[0].value = format!("Food: {:?} ", food.0);
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
