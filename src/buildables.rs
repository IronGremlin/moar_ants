use bevy::ecs::system::{Command, CommandQueue};
use bevy::prelude::{Plugin, *};
use bevy::utils::HashMap;

use crate::colony::Colony;
use crate::food::FoodQuant;
use crate::gametimer::SimTimer;
use crate::spawner::{HasFootPrint, SpawnPulse, Spawner};
use crate::SpatialMarker;

pub struct BuildablePlugin;

impl Plugin for BuildablePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildableStore::new());
    }
}

#[derive(Clone)]
pub struct BuildableObject {
    //TO-DO - make this a real ID type at some point
    kind_id: String,
    sprite_path: String,
    exclusion_params: Option<PlacementExlusionParams>,
    capabilites: Vec<Capability>,
}
//TODO - this is a stand-in for an actual "from asset defs" loading system
impl BuildableObject {
    fn spawner() -> BuildableObject {
        BuildableObject {
            kind_id: "Spawner".into(),
            sprite_path: "spawner.png".into(),
            exclusion_params: Some(PlacementExlusionParams {
                exclusion_radius: 40.0,
                maximum_distance: Some(160.0),
            }),
            capabilites: vec![Capability::SpawnPoint],
        }
    }
    pub fn exclusion_params(&self) -> Option<PlacementExlusionParams> {
        self.exclusion_params.clone()
    }
}

#[derive(Resource)]
pub struct BuildableStore {
    store: HashMap<String, BuildableObject>,
}
impl BuildableStore {
    pub fn get(&self, kind_id: &String) -> Option<BuildableObject> {
        self.store.get(kind_id).map(|x| x.clone())
    }
    //TODO - see above commentary regarding stand-in
    pub fn new() -> BuildableStore {
        let mut map = HashMap::new();
        map.insert("Spawner".into(), BuildableObject::spawner());
        BuildableStore { store: map }
    }
}

#[derive(Clone)]
pub struct PlacementExlusionParams {
    //Anything that collides with buidling cannot exist within the exclusion radius
   pub exclusion_radius: f32,
    //If specified, this entity must be within X distance of some building
    // TODO - Do we care -which- buildings?
    pub maximum_distance: Option<f32>,
}
#[derive(Clone, Copy)]
pub enum Capability {
    SpawnPoint,
}

pub struct PlacementOperation {
    pub position: Vec2,
    pub owning_colony: Entity,
    pub key: String,
}
pub trait BuildableCommandsExt {
    fn place_buildable(&mut self, op: PlacementOperation);
}
impl<'a, 'b> BuildableCommandsExt for Commands<'a, 'b> {
    fn place_buildable(&mut self, op: PlacementOperation) {
        self.add(op)
    }
}

impl Command for PlacementOperation {
    fn apply(self, world: &mut World) {
        // These are minimum requirements for successfully building anything, so we panic if none of these are met.
        // TODO - wrap these assumptions in a run condition so we can export this set of assumptions.

        let buildable_store = world.get_resource::<BuildableStore>().unwrap();
        let buildable = buildable_store.get(&self.key).unwrap();
        let assets = world.get_resource::<AssetServer>().unwrap();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let sprite = SpriteBundle {
            transform: Transform::from_xyz(self.position.x, self.position.y, 1.),
            texture: assets.load(buildable.sprite_path),
            ..default()
        };
        let root_entity = if buildable.exclusion_params.is_some() {
            commands.spawn((sprite, HasFootPrint)).id()
        } else {
            commands.spawn(sprite).id()
        };
        for capability in buildable.capabilites {
            match capability {
                Capability::SpawnPoint => {
                    commands
                        .entity(root_entity)
                        .insert((
                            Spawner {
                                colony: self.owning_colony,
                            },
                            SpatialMarker,
                        ))
                        .with_children(|child_commands| {
                            child_commands.spawn((
                                SpawnPulse,
                                SimTimer {
                                    time: Timer::from_seconds(10.0, TimerMode::Repeating),
                                },
                            ));
                        });
                }
            }
        }
        queue.apply(world);
    }
}
