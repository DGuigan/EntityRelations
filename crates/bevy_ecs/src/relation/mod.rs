use bevy_utils::{HashMap, HashSet};
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

mod type_magic;
//mod joins;
//mod traversals;

pub use type_magic::*;
//pub use joins::*;
//pub use traversals::*;

mod sealed {
    use super::*;
    pub trait Sealed {}
    impl<'a, T: Relation> Sealed for &'a T {}
    impl<'a, T: Relation> Sealed for &'a mut T {}
    impl<T: RelationSet> Sealed for Option<T> {}
    // TODO: All tuple
    impl<P0> Sealed for (P0,) {}
    impl<P0, P1> Sealed for (P0, P1) {}
}

pub struct Foster<T: Relation>(PhantomData<T>);

pub struct Storage<R: Relation>(pub(crate) SmallVec<[R; 1]>);

impl<R: Relation> Component for Storage<R> {
    type Storage = R::Storage;
}

#[derive(Component)]
pub(crate) struct Index {
    pub(crate) targets: [HashMap<ComponentId, HashMap<Entity, usize>>; 4],
    pub(crate) fosters: HashMap<ComponentId, Entity>,
}

use sealed::*;

// Precedence: Most data latering operation is preferred.
// Smaller number -> Higher precedence
pub enum DespawnPolicy {
    RecursiveDespawn = 0,
    RecursiveDelink = 1,
    Reparent = 2,
    Orphan = 3,
}

pub trait Relation: 'static + Sized + Send + Sync {
    const DESPAWN_POLICY: DespawnPolicy = DespawnPolicy::Orphan;
    type Storage: ComponentStorage;
}

#[derive(Default, Clone, Copy)]
pub struct Drop;

#[derive(Default, Clone, Copy)]
pub struct Get;

pub trait RelationSet: Sealed {
    type Types;
    type WorldQuery: WorldQuery;
    type EmptyJoinSet: Default;
}

impl<T: Relation> RelationSet for &'_ T {
    type Types = T;
    type WorldQuery = &'static Storage<T>;
    type EmptyJoinSet = Drop;
}

impl<T: Relation> RelationSet for &'_ mut T {
    type Types = T;
    type WorldQuery = &'static mut Storage<T>;
    type EmptyJoinSet = Drop;
}

impl<T: RelationSet> RelationSet for Option<T> {
    type Types = T::Types;
    type WorldQuery = Option<T::WorldQuery>;
    type EmptyJoinSet = Drop;
}

// TODO: All tuple
impl<P0: RelationSet> RelationSet for (P0,) {
    type Types = (P0::Types,);
    type WorldQuery = (P0::WorldQuery,);
    type EmptyJoinSet = (P0::EmptyJoinSet,);
}

impl<P0, P1> RelationSet for (P0, P1)
where
    P0: RelationSet,
    P1: RelationSet,
{
    type Types = (P0::Types, P1::Types);
    type WorldQuery = (P0::WorldQuery, P1::WorldQuery);
    type EmptyJoinSet = (P0::EmptyJoinSet, P1::EmptyJoinSet);
}

type FetchItem<'a, R> =
    <<<R as RelationSet>::WorldQuery as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>;
type FetchItemMut<'a, R> = <<R as RelationSet>::WorldQuery as WorldQuery>::Item<'a>;

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct Relations<T: RelationSet + Send + Sync> {
    storages: T::WorldQuery,
    index: &'static Index,
    #[world_query(ignore)]
    _phantom: PhantomData<T>,
}

pub struct Ops<Query, Joins, Traversal = (), TraversalQueue = ()> {
    query: Query,
    joins: Joins,
    traversal: Traversal,
    queue: TraversalQueue,
}

impl<'w, 's, Q, F, R> Query<'w, 's, (Q, Relations<R>), F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
{
    fn ops(&self) -> Ops<&Self, R::EmptyJoinSet> {
        Ops {
            query: self,
            joins: R::EmptyJoinSet::default(),
            traversal: (),
            queue: (),
        }
    }

    fn ops_mut(&mut self) -> Ops<&mut Self, R::EmptyJoinSet> {
        Ops {
            query: self,
            joins: R::EmptyJoinSet::default(),
            traversal: (),
            queue: (),
        }
    }
}

// Lending iterator workaround
// - https://blog.rust-lang.org/2022/10/28/gats-stabilization.html#implied-static-requirement-from-higher-ranked-trait-bounds
pub trait LendingForEach {
    type In<'e, 'j>;
    fn for_each(self, func: impl FnMut(Self::In<'_, '_>));
}
