use bevy::ecs::{
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::SystemParam,
};
use bevy::prelude::*;
use std::{
    collections::{hash_map, HashMap},
    marker::PhantomData,
};

// Private types. Cannot be queried for directly.
struct Multi<T>(HashMap<Entity, T>);
struct Exclucive<T>(Entity, T);

trait RelationStore {
    type Relation;
    // Need this atm because return position impl trait in traits are unstable
    type Iter<'a>: Iterator<Item = (&'a Entity, &'a Self::Relation)>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
}

impl<T> RelationStore for Exclucive<T> {
    type Relation = T;
    type Iter<'a> = std::iter::Once<(&'a Entity, &'a Self::Relation)>
    where
        T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        std::iter::once((&self.0, &self.1))
    }
}

impl<T> RelationStore for Multi<T> {
    type Relation = T;
    type Iter<'a> = hash_map::Iter<'a, Entity, Self::Relation>
    where
        T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter()
    }
}

impl<T: Component> Component for Exclucive<T> {
    type Storage = T::Storage;
}

impl<T: Component> Component for Multi<T> {
    type Storage = T::Storage;
}

trait Relation: Component {
    type Store: RelationStore<Relation = Self>;
}

#[derive(WorldQuery)]
struct RelationQuery<T, Components>
where
    T: Relation + Component,
    T::Store: Component,
    Components: WorldQuery + ReadOnlyWorldQuery,
{
    relations: &'static T::Store,
    components: Components,
    #[world_query(ignore)]
    _phantom: PhantomData<T>,
}

fn main() {
    println!("Hello, world!");
}
