use bevy_utils::{HashMap, HashSet};
use core::any::TypeId;
use smallvec::SmallVec;
use std::marker::PhantomData;

use crate as bevy_ecs;

use crate::{
    component::{Component, ComponentId, ComponentStorage},
    entity::Entity,
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::Query,
    world::World,
};

mod joins;
mod tuple_traits;
//mod traversals;

pub use joins::*;
pub use tuple_traits::*;
//pub use traversals::*;

mod sealed {
    use super::*;
    pub trait Sealed {}
    impl<'a, T: Relation> Sealed for &'a T {}
    impl<'a, T: Relation> Sealed for &'a mut T {}
    impl<T: RelationQuerySet> Sealed for Option<T> {}
    // TODO: All tuple
    impl<P0> Sealed for (P0,) {}
    impl<P0, P1> Sealed for (P0, P1) {}
}

use sealed::*;

// TODO: Remove tombstones
// - Removing a target entity won't reove relation valeus any fosters have to it
// - Store relation values as `(Storage<R>)` entities and replace `usize` with `Entity`
// - Cleanup code can just remove entites without knowing relation type
// - Need manual WorldQuery impl that hides this archetype access
// - Uphold invariants to ensure soundness
pub(crate) struct Storage<R: Relation> {
    pub(crate) values: SmallVec<[R; 1]>,
}

impl<R: Relation> Component for Storage<R> {
    type Storage = R::Storage;
}

#[derive(Component)]
pub(crate) struct Register {
    pub(crate) targets: [HashMap<TypeId, HashMap<Entity, usize>>; 4],
    pub(crate) fosters: HashMap<TypeId, Entity>,
}

impl Register {
    fn target_iter<R: Relation>(&self) -> impl '_ + Clone + Iterator<Item = (Entity, usize)> {
        self.targets[R::DESPAWN_POLICY as usize]
            .get(&TypeId::of::<Storage<R>>())
            .map(|map| map.iter().map(|(e, i)| (*e, *i)))
            .into_iter()
            .flatten()
    }
}

// Precedence: Most data latering operation is preferred.
// Smaller number -> Higher precedence
pub enum DespawnPolicy {
    RecursiveDespawn = 0,
    RecursiveDelink = 1,
    Reparent = 2,
    Orphan = 3,
}

pub trait Relation: 'static + Sized + Send + Sync {
    type Storage: ComponentStorage;
    const DESPAWN_POLICY: DespawnPolicy = DespawnPolicy::Orphan;
    const EXCLUSIVE: bool = false;
}

#[derive(WorldQuery)]
pub struct StorageWorldQuery<R: Relation> {
    storage: &'static Storage<R>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct StorageWorldQueryMut<R: Relation> {
    storage: &'static mut Storage<R>,
}

#[derive(WorldQuery)]
pub struct RegisterWorldQuery {
    register: &'static Register,
}

pub trait RelationQuerySet: Sealed {
    type Types;
    type WorldQuery: WorldQuery;
}

impl<R: Relation> RelationQuerySet for &'_ R {
    type Types = R;
    type WorldQuery = StorageWorldQuery<R>;
}

impl<R: Relation> RelationQuerySet for &'_ mut R {
    type Types = R;
    type WorldQuery = StorageWorldQueryMut<R>;
}

impl<R: RelationQuerySet> RelationQuerySet for Option<R> {
    type Types = R;
    type WorldQuery = Option<R::WorldQuery>;
}

// TODO: All tuple
impl<P0: RelationQuerySet> RelationQuerySet for (P0,) {
    type Types = (P0::Types,);
    type WorldQuery = (P0::WorldQuery,);
}

// TODO:
// - Manual `WorldQuery` impl to get `ComponentId` from `World` to remove the usage of `TypeId`
// - `TypeId` is not guarenteed to be stable which is a problem for serialization.
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct Relations<T: RelationQuerySet + Send + Sync> {
    register: &'static Register,
    world_query: T::WorldQuery,
    #[world_query(ignore)]
    _phantom: PhantomData<T>,
}

pub struct Ops<Query, TargetIters = (), StorageExtractions = (), JoinQueries = (), Traversal = ()> {
    query: Query,
    target_iters: TargetIters,
    storage_extractions: PhantomData<StorageExtractions>,
    join_queries: JoinQueries,
    traversal: PhantomData<Traversal>,
}

impl<'w, 's, Q, F, R> Query<'w, 's, (Q, Relations<R>), F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet + Send + Sync,
{
    fn ops(&self) -> Ops<&Self> {
        Ops {
            query: self,
            target_iters: (),
            storage_extractions: PhantomData,
            join_queries: (),
            traversal: PhantomData,
        }
    }

    fn ops_mut(&mut self) -> Ops<&mut Self> {
        Ops {
            query: self,
            target_iters: (),
            storage_extractions: PhantomData,
            join_queries: (),
            traversal: PhantomData,
        }
    }
}

/*pub(crate) type FetchItem<'a, R> =
    <<<R as RelationSet>::WorldQuery as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>;
pub(crate) type FetchItemMut<'a, R> = <<R as RelationSet>::WorldQuery as WorldQuery>::Item<'a>;*/

pub trait ForEachPermutations {
    type In<'a>;
    fn for_each(self, func: impl FnMut(Self::In<'_>));
}
