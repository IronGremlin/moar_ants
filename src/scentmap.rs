use bevy::{math::Vec3Swizzles, prelude::*, time::common_conditions::on_timer, utils::HashMap};
use kd_tree::KdTree;
use std::time::Duration;

use crate::SimState;

pub struct ScentMapPlugin;

impl Plugin for ScentMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScentSettings::default())
            .insert_resource(ScentMap::new())
            .add_systems(
                Update,
                update_index.run_if(
                    in_state(SimState::Playing).and_then(on_timer(Duration::from_secs_f32(0.5))),
                ),
            );
    }
}

fn update_index(mut map: ResMut<ScentMap>, settings: Res<ScentSettings>) {
    map.decay_smells(settings.decay_rate);
    map.cull_zeros_and_nans();
    map.update_trees();
}

#[derive(Resource)]
pub struct ScentMap {
    found_food_smell_data: HashMap<(i32, i32), f32>,
    ant_smell_data: HashMap<(i32, i32), f32>,
    found_food_smell_index: KdTree<[f32; 2]>,
    ant_smell_index: KdTree<[f32; 2]>,
}
pub enum WeightType {
    CloserTo(Vec2),
    FurtherFrom(Vec2),
    Unweighted,
}
impl ScentMap {
    fn new() -> ScentMap {
        let found_food_smell_data = HashMap::new();
        let ant_smell_data = HashMap::new();
        let found_food_smell_index = KdTree::build_by_ordered_float(Vec::new());
        let ant_smell_index = KdTree::build_by_ordered_float(Vec::new());

        ScentMap {
            found_food_smell_data,
            ant_smell_data,
            found_food_smell_index,
            ant_smell_index,
        }
    }
    fn update_trees(&mut self) {
        let span = info_span!("scentmap: update");
        let _ = span.enter();
        let mut foodsmells = Vec::new();
        for ((x, y), _) in self.found_food_smell_data.iter() {
            foodsmells.push([*x as f32, *y as f32]);
        }
        self.found_food_smell_index = KdTree::build_by_ordered_float(foodsmells);
        let mut antsmells = Vec::new();
        for ((x, y), _) in self.ant_smell_data.iter() {
            antsmells.push([*x as f32, *y as f32]);
        }
        self.ant_smell_index = KdTree::build_by_ordered_float(antsmells);
    }
    pub fn log_scent(
        &mut self,
        max_strength: f32,
        transform: &Transform,
        scent: ScentType,
        strength: f32,
    ) {
        let span = info_span!("scentmap: log scent");
        let _ = span.enter();
        let hashmap: &mut HashMap<(i32, i32), f32> = match scent {
            ScentType::FoundFoodSmell => &mut self.found_food_smell_data,
            ScentType::AntSmell => &mut self.ant_smell_data,
            _ => &mut self.ant_smell_data,
        };
        let vec = transform.translation.xy();
        let k = (vec.x as i32, vec.y as i32);

        match hashmap.get_mut(&k) {
            Some(val) => {
                *val = (strength + *val).min(max_strength);
                //info!("Ant overwriting stink - {:?}", val);
            }
            None => {
                hashmap.insert(k, strength);
                //info!("Ant attempting new stink - {:?}", strength);
            }
        }
    }
    fn cull_zeros_and_nans(&mut self) {
        let span = info_span!("scentmap: cull");
        let _ = span.enter();
        //  info!("count prior to cull - {:?}", self.ant_smell_data.len());
        self.ant_smell_data.retain(|_g, n| *n != 0.0 && !n.is_nan());
        //info!("count after cull - {:?}", self.ant_smell_data.len());

        self.found_food_smell_data
            .retain(|_, n| *n != 0.0 && !n.is_nan());
    }
    fn decay_smells(&mut self, decay_by: f32) {
        let span = info_span!("scentmap: decay");
        let _ = span.enter();
        for (_, v) in self.ant_smell_data.iter_mut() {
            *v = (*v - decay_by).max(0.0);
        }
        for (_, v) in self.found_food_smell_data.iter_mut() {
            *v = (*v - decay_by).max(0.0);
        }
    }


    pub fn strongest_smell_weighted(
        &mut self,
        radius: f32,
        scent: ScentType,
        weighting: WeightType,
        transform: &GlobalTransform,
    ) -> Option<Vec2> {
        let span = info_span!("scentmap: get smell");
        let _ = span.enter();
        let tree = match scent {
            ScentType::FoundFoodSmell => &self.found_food_smell_index,
            ScentType::AntSmell => &self.ant_smell_index,
        };
        let hashmap = match scent {
            ScentType::FoundFoodSmell => &self.found_food_smell_data,
            ScentType::AntSmell => &self.ant_smell_data,
            _ => &self.ant_smell_data,
        };
        let coords = scent_grid_coords(transform);
        let key = (scent, coords.0, coords.1);
        
        let current_pos = transform.translation().xy();
        let my_distance = match weighting {
            WeightType::CloserTo(home) |  WeightType::FurtherFrom(home) => home.distance(current_pos),
            _ => 0.0
        };
        

        let dump: Vec<(f32, f32, f32)> = tree
            .within_radius(&[coords.0 as f32, coords.1 as f32], radius)
            .iter_mut()
            .filter(|[x, y]| (*x as i32, *y as i32) != coords)
            .filter_map(|[x, y]| match hashmap.get(&(*x as i32, *y as i32)) {
                Some(value) => Some((*x, *y, *value)),
                _ => None,
            })
            .filter(|(p0, p1, _)| match weighting {
                WeightType::CloserTo(home) => Vec2::from((*p0, *p1)).distance(home) < my_distance,
                WeightType::FurtherFrom(home) => Vec2::from((*p0, *p1)).distance(home) > my_distance,
                _ => true,
            })
            .collect();

        if dump.len() == 0 {
            return None;
        }


        let mut total_weight = 0.0;
        let mut weighted_sum_x = 0.0;
        let mut weighted_sum_y = 0.0;

        dump.iter().for_each(|(p0, p1, p2)| {

            let point_weight = p2;
            total_weight += point_weight;
            weighted_sum_x += p0 * point_weight;
            weighted_sum_y += p1 * point_weight;
        });

        let total_weight_recip = total_weight.recip();
        let weighted_midpoint_x = weighted_sum_x * total_weight_recip;
        let weighted_midpoint_y = weighted_sum_y * total_weight_recip;
        if (weighted_midpoint_x.is_nan() || weighted_midpoint_y.is_nan()) {
            info!("FUCKING NAN")
        }
        let vec = Vec2 {
            x: weighted_midpoint_x,
            y: weighted_midpoint_y,
        };
        Some(vec)
    }
}

#[derive(Resource)]
pub struct ScentSettings {
    pub decay_rate: f32,
    pub smell_radius: f32,
    pub starting_strength: f32,
    pub max_smell: f32,
}
impl Default for ScentSettings {
    fn default() -> Self {
        ScentSettings {
            decay_rate: 2.5,
            smell_radius: 10.0,
            starting_strength: 50.0,
            max_smell: 150.0,
        }
    }
}
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum ScentType {
    FoundFoodSmell,
    AntSmell,
}

fn scent_grid_coords(t: &GlobalTransform) -> (i32, i32) {
    let vec = t.translation().xy();
    //note to self : f -> i is a truncation
    (vec.x as i32, vec.y as i32)
}
