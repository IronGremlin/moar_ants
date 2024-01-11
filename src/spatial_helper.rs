

use bevy::{
    ecs::{
        query::{QueryManyIter, ReadOnlyWorldQuery, WorldQuery, Has},
        system::SystemParam,
    },
    
    prelude::*, math::Vec3Swizzles,
};
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};


pub struct SpatialHelperPlugin;

impl Plugin for SpatialHelperPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(SystemParam)]
pub struct DistanceAwareQuery<'w, 's, Comp, Q, F = ()>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
    Comp: Component,
{
    all_t: Query<'w, 's, Q, (F, With<Comp>)>,
    space: Res<'w, KDTree2<Comp>>,
}

impl<'w, 's, Comp, Q, F> DistanceAwareQuery<'w, 's, Comp, Q, F>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
    Comp: Component,
{
    pub fn within_distance(
        &self,
        loc: Vec2,
        distance: f32,
    ) -> QueryManyIter<
        '_,
        's,
        <Q as WorldQuery>::ReadOnly,
        (F, With<Comp>),
        <Vec<Entity> as IntoIterator>::IntoIter,
    > {
        let spacevec: Vec<Entity> = self
            .space
            .within_distance(loc, distance)
            .iter()
            .filter_map(|(_, x)| *x)
            .collect();

        self.all_t.iter_many(spacevec)
    }

    pub fn within_distance_mut(
        &mut self,
        loc: Vec2,
        distance: f32,
    ) -> QueryManyIter<'_, 's, Q, (F, With<Comp>), <Vec<Entity> as IntoIterator>::IntoIter> {
        let spacevec: Vec<Entity> = self
            .space
            .within_distance(loc, distance)
            .iter()
            .filter_map(|(_, x)| *x)
            .collect();

        self.all_t.iter_many_mut(spacevec)
    }
}


