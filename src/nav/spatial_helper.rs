use bevy::{
    ecs::{
        query::{QueryManyIter, ReadOnlyWorldQuery, WorldQuery},
        system::SystemParam,
    },
    prelude::*,
};
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};

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
#[allow(dead_code)]
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
