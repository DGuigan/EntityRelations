use bevy::ecs::{
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::SystemParam,
};
use bevy::prelude::*;
use std::{collections::HashMap, marker::PhantomData};

trait RelationStore {}

// Private types. Cannot be queried for directly.
struct Exclusive<T>(Entity, T);
struct Multi<T>(HashMap<Entity, T>);

impl<T> RelationStore for Exclusive<T> {}
impl<T> RelationStore for Multi<T> {}

impl<T: Component> Component for Exclusive<T> {
    type Storage = T::Storage;
}

impl<T: Component> Component for Multi<T> {
    type Storage = T::Storage;
}

trait Relation: Component {
    type Store: RelationStore;
}

#[derive(WorldQuery)]
struct RelationQuery<T, Components>
where
    T: Relation + Component,
    T::Store: Component + 'static,
    Components: WorldQuery + ReadOnlyWorldQuery,
{
    relations: &'static T::Store,
    components: Components,
    #[world_query(ignore)]
    _phantom: PhantomData<T>,
}

#[derive(SystemParam)]
struct All<'w, 's, RelatedBy, FosterQuery, TargetQuery>
where
    RelatedBy: Relation,
    RelatedBy::Store: Component,
    FosterQuery: 'static + WorldQuery + ReadOnlyWorldQuery,
    TargetQuery: 'static + WorldQuery + ReadOnlyWorldQuery,
{
    relation_query: Query<'w, 's, RelationQuery<RelatedBy, FosterQuery>>,
    target_components: Query<'w, 's, TargetQuery>,
}

impl<'w, 's, RelatedBy, FosterQuery, TargetQuery> All<'w, 's, RelatedBy, FosterQuery, TargetQuery>
where
    RelatedBy: Relation,
    RelatedBy::Store: Component,
    FosterQuery: 'static + WorldQuery + ReadOnlyWorldQuery,
    TargetQuery: 'static + WorldQuery + ReadOnlyWorldQuery,
{
    fn iter(&self) -> impl '_ + Iterator {
        self.relation_query.iter().map(|rel_q| {})
    }
}

#[derive(SystemParam)]
struct Only<'w, 's, RelatedBy, FosterQuery, TargetQuery>
where
    RelatedBy: Relation,
    RelatedBy::Store: Component,
    FosterQuery: 'static + WorldQuery + ReadOnlyWorldQuery,
    TargetQuery: 'static + WorldQuery + ReadOnlyWorldQuery,
{
    relation_query: Query<'w, 's, RelationQuery<RelatedBy, FosterQuery>>,
    target_components: Query<'w, 's, TargetQuery>,
}

fn main() {
    println!("Hello, world!");
}
