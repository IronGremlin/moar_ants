use bevy::{prelude::*, utils::HashMap};

use std::marker::PhantomData;

use crate::{
    ant::AntSettings,
    colony::{AntCapacity, Colony, MaxFood},
    food::FoodQuant,
};

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (AntMaxPop::init, ColonyMaxFood::init, AntCarryCapacity::init)
                .run_if(when_colony_exists.and_then(run_once())),
        )
        .add_systems(
            Update,
            (
                AntMaxPop::upgrade_colony,
                AntMaxPop::progress_bar_update,
                AntMaxPop::progress_bar_display_effect,
                AntMaxPop::set_upgrade_button_able,
                ColonyMaxFood::upgrade_colony,
                ColonyMaxFood::progress_bar_update,
                ColonyMaxFood::progress_bar_display_effect,
                ColonyMaxFood::set_upgrade_button_able,
                AntCarryCapacity::upgrade_ants,
                AntCarryCapacity::progress_bar_update,
                AntCarryCapacity::progress_bar_display_effect,
                AntCarryCapacity::set_upgrade_button_able,
            ),
        );
    }
}

pub fn spawn_upgrade_buttons(commands: &mut Commands) -> [Entity; 3] {
    [
        AntMaxPop::spawn_button(commands),
        ColonyMaxFood::spawn_button(commands),
        AntCarryCapacity::spawn_button(commands),
    ]
}

fn when_colony_exists(q: Query<&Colony>) -> bool {
    q.get_single().is_ok()
}

#[derive(Component, Default)]
pub struct ColonyUpgradeButton<T: ColonyUpgrade + Default + Component> {
    marker: PhantomData<T>,
}
#[derive(Component, Default)]
pub struct ColonyUpgradeProgress<T: ColonyUpgrade + Default + Component> {
    marker: PhantomData<T>,
}

pub trait ColonyUpgrade
where
    Self: Component + Default,
{
    fn cost(cost_index: &i32) -> i32;
    fn name() -> String;
    fn init(mut q: Query<&mut UpgradeStringIndex, With<Colony>>) {
        let mut index = q.get_single_mut().unwrap();
        index.costs.insert(Self::name(), 1);
    }
    fn button_tag() -> ColonyUpgradeButton<Self> {
        ColonyUpgradeButton::<Self>::default()
    }
    fn progress_bar_tag() -> ColonyUpgradeProgress<Self> {
        ColonyUpgradeProgress::<Self>::default()
    }
    fn spawn_button(commands: &mut Commands) -> Entity {
        let upgrade_button_node = (
            ButtonBundle {
                background_color: Color::BLUE.into(),
                style: Style {
                    width: Val::Percent(80.),
                    height: Val::Vh(5.),
                    margin: UiRect::all(Val::Px(10.)),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Self::button_tag(),
        );
        let upgrade_progress_bar_layout_node: NodeBundle = NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(50.),
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            background_color: Color::WHITE.into(),
            border_color: Color::BLACK.into(),
            ..default()
        };
        let upgrade_progress_bar_progress_mask_node = (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Relative,

                    ..default()
                },
                background_color: Color::GREEN.into(),
                z_index: ZIndex::Local(10),
                ..default()
            },
            Self::progress_bar_tag(),
        );
        let upgrade_progress_bar_label_node = (
            TextBundle {
                text: Text::from_sections([
                    TextSection::new("0", TextStyle::default()),
                    TextSection::new("\n", TextStyle::default()),
                    TextSection::new("", TextStyle::default()),
                ]),
                ..default()
            },
            Self::progress_bar_tag(),
        );

        let upgrade_button_label_node = TextBundle {
            text: Text::from_section(Self::name(), TextStyle::default()),
            ..default()
        };
        let upgrade_button_label = commands.spawn(upgrade_button_label_node).id();
        let upgrade_progress_bar_layout = commands.spawn(upgrade_progress_bar_layout_node).id();
        let upgrade_progress_bar_progress_mask =
            commands.spawn(upgrade_progress_bar_progress_mask_node).id();
        let upgrade_progress_bar_label = commands.spawn(upgrade_progress_bar_label_node).id();
        let upgrade_button = commands.spawn(upgrade_button_node).id();
        commands.entity(upgrade_button).push_children(&[
            upgrade_button_label,
            upgrade_progress_bar_label,
            upgrade_progress_bar_layout,
        ]);
        commands
            .entity(upgrade_progress_bar_layout)
            .push_children(&[upgrade_progress_bar_progress_mask]);

        return upgrade_button;
    }
    fn progress_bar_update(
        col_q: Query<(&FoodQuant, &UpgradeStringIndex), With<Colony>>,
        mut text_q: Query<&mut Text, With<ColonyUpgradeProgress<Self>>>,
        mut style_q: Query<&mut Style, (With<ColonyUpgradeProgress<Self>>, Without<Text>)>,
    ) {
        let (food, upgrades) = col_q.get_single().unwrap();
        if let Some(cost) = upgrades.costs.get(&Self::name()).map(Self::cost) {
            let bar_display_food = food.0.clamp(0, cost);
            for mut text in text_q.iter_mut() {
                text.sections[0].value = format!("Cost: {:?}", cost);
            }
            for mut style in style_q.iter_mut() {
                style.width = Val::Percent(100. * (bar_display_food as f32 / cost as f32));
            }
        }
    }
    fn set_upgrade_button_able(
        mut commands: Commands,
        q: Query<(&UpgradeStringIndex, &FoodQuant), With<Colony>>,
        button_q: Query<Entity, With<ColonyUpgradeButton<Self>>>,
    ) {
        for id in button_q.iter() {
            //TODO - figure this out when we have player colony id logic.
            let (upgrades, food) = q.single();
            if let Some(cost) = upgrades.costs.get(&Self::name()).map(Self::cost) {
                if cost <= food.0 {
                    commands.entity(id).insert(Interaction::default());
                } else {
                    commands.entity(id).remove::<Interaction>();
                }
            } else {
                info!("could not locate entry for upgrade: {:?}", Self::name());
            }
        }
    }
}

pub trait AntUpgrade: ColonyUpgrade {
    fn display_effect(ant_settings: &AntSettings, idx: i32) -> String;

    fn progress_bar_display_effect(
        q: Query<&UpgradeStringIndex, With<Colony>>,
        ant_settings: Res<AntSettings>,
        mut text_q: Query<&mut Text, With<ColonyUpgradeProgress<Self>>>,
    ) {
        let upgrades = q.single();
        for mut text in text_q.iter_mut() {
            text.sections[2].value =
                Self::display_effect(&ant_settings, *upgrades.costs.get(&Self::name()).unwrap());
        }
    }

    fn upgrade_ants(
        mut q: Query<(&mut UpgradeStringIndex, &mut FoodQuant), With<Colony>>,
        mut ant_settings: ResMut<AntSettings>,
        button_q: Query<&Interaction, With<ColonyUpgradeButton<Self>>>,
    ) {
        //TODO - figure this out when we have player colony id logic.
        if button_q
            .get_single()
            .is_ok_and(|f| matches!(f, Interaction::Pressed))
        {
            let (mut upgrades, mut food) = q.single_mut();
            let feature_index = *upgrades.costs.get(&Self::name()).unwrap();
            let cost = Self::cost(&feature_index);
            if cost <= food.0 {
                Self::do_upgrade(
                    &mut food,
                    &mut ant_settings,
                    &mut upgrades,
                    cost,
                    feature_index,
                )
            }
        }
    }
    fn do_upgrade(
        food: &mut FoodQuant,
        ant_settings: &mut AntSettings,
        upgrades: &mut UpgradeStringIndex,
        cost: i32,
        feature_index: i32,
    );
}
#[derive(Component, Default)]
pub struct AntCarryCapacity;
impl AntCarryCapacity {
    fn val(idx: i32) -> i32 {
        match idx {
            1 => 10,
            2 => 15,
            _ => 20,
        }
    }
}
impl ColonyUpgrade for AntCarryCapacity {
    fn name() -> String {
        "Ant Carry Capacity".into()
    }
    fn cost(cost_index: &i32) -> i32 {
        squarish(*cost_index, 5.0, 100.0) as i32
    }
}
impl AntUpgrade for AntCarryCapacity {
    fn display_effect(ant_settings: &AntSettings, idx: i32) -> String {
        if idx < 4 {
            format!(
                "|Current: {:?} Next: {:?}|",
                ant_settings.carry_capacity,
                Self::val(idx)
            )
        } else {
            "MAX: 20".into()
        }
    }
    fn do_upgrade(
        food: &mut FoodQuant,
        ant_settings: &mut AntSettings,
        upgrades: &mut UpgradeStringIndex,
        cost: i32,
        feature_index: i32,
    ) {
        if feature_index < 4 {
            food.0 -= cost;
            ant_settings.carry_capacity = Self::val(feature_index);
            upgrades.increment_index(Self::name());
        }
    }
}
#[derive(Component, Default)]
pub struct ColonyMaxFood;
impl ColonyMaxFood {
    fn val() -> i32 {
        200
    }
    fn progress_bar_display_effect(
        q: Query<(&UpgradeStringIndex, &MaxFood), With<Colony>>,
        mut text_q: Query<&mut Text, With<ColonyUpgradeProgress<Self>>>,
    ) {
        let (upgrades, maxfood) = q.single();
        if let Some(_) = upgrades.costs.get(&Self::name()) {
            for mut text in text_q.iter_mut() {
                text.sections[2].value =
                    format!("|Current: {} Next: {}|", maxfood.0, maxfood.0 + Self::val());
            }
        }
    }

    fn upgrade_colony(
        mut q: Query<(&mut UpgradeStringIndex, &mut FoodQuant, &mut MaxFood), With<Colony>>,
        button_q: Query<&Interaction, With<ColonyUpgradeButton<Self>>>,
    ) {
        //TODO - figure this out when we have player colony id logic.
        if button_q
            .get_single()
            .is_ok_and(|f| matches!(f, Interaction::Pressed))
        {
            let (mut upgrades, mut food, mut ant_cap) = q.single_mut();
            let cost = Self::cost(&1);
            if cost <= food.0 {
                food.0 -= cost;
                ant_cap.0 += Self::val();
                upgrades.increment_index(Self::name());
            }
        }
    }
}
impl ColonyUpgrade for ColonyMaxFood {
    fn name() -> String {
        "Colony Max Food".into()
    }
    fn cost(_: &i32) -> i32 {
        50
    }
}

#[derive(Component, Default)]
pub struct AntMaxPop;
impl AntMaxPop {
    fn val(idx: i32) -> i32 {
        match idx {
            1 => 5,
            2 => 10,
            3 => 20,
            4..=10 => 50,
            _ => 100,
        }
    }
    fn progress_bar_display_effect(
        q: Query<(&UpgradeStringIndex, &AntCapacity), With<Colony>>,
        mut text_q: Query<&mut Text, With<ColonyUpgradeProgress<Self>>>,
    ) {
        let (upgrades, ant_cap) = q.single();
        if let Some(feature_index) = upgrades.costs.get(&Self::name()) {
            for mut text in text_q.iter_mut() {
                text.sections[2].value = format!(
                    "|Current: {} Next: {}|",
                    ant_cap.0,
                    ant_cap.0 + Self::val(*feature_index)
                );
            }
        }
    }

    fn upgrade_colony(
        mut q: Query<(&mut UpgradeStringIndex, &mut FoodQuant, &mut AntCapacity), With<Colony>>,
        button_q: Query<&Interaction, With<ColonyUpgradeButton<Self>>>,
    ) {
        //TODO - figure this out when we have player colony id logic.
        if button_q
            .get_single()
            .is_ok_and(|f| matches!(f, Interaction::Pressed))
        {
            let (mut upgrades, mut food, mut ant_cap) = q.single_mut();
            let feature_index = upgrades.costs.get(&Self::name()).unwrap();
            let cost = Self::cost(feature_index);
            if cost <= food.0 {
                food.0 -= cost;
                ant_cap.0 += Self::val(*feature_index);
                upgrades.increment_index(Self::name());
            }
        }
    }
}
impl ColonyUpgrade for AntMaxPop {
    fn name() -> String {
        "Ant Capacity".into()
    }
    fn cost(cost_index: &i32) -> i32 {
        squarish(*cost_index, 8.0, 100.0) as i32
    }
}

#[derive(Component)]
pub struct UpgradeStringIndex {
    pub costs: HashMap<String, i32>,
}
impl UpgradeStringIndex {
    pub fn new() -> Self {
        UpgradeStringIndex {
            costs: HashMap::new(),
        }
    }
    pub fn increment_index(&mut self, upgrade: String) {
        self.costs.get_mut(&upgrade).map(|x| {
            *x += 1;
        });
    }
}

fn squarish(i: i32, flattener: f32, scalar: f32) -> f32 {
    let f = i as f32;
    (f * (f / flattener)) * scalar
}
#[allow(dead_code)]
fn cubeish(i: i32, flattener: f32, scalar: f32) -> f32 {
    let f = i as f32;
    (f * f * (f / flattener)) * scalar
}
