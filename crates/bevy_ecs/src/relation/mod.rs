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

use sealed::*;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(crate) struct RelationIds<R: Relation> {
    _phantom: PhantomData<R>,
}

#[derive(Component)]
pub(crate) struct StorageId(pub(crate) ComponentId);

#[derive(Component)]
pub(crate) struct FosterId(pub(crate) ComponentId);

pub struct Storage<R: Relation> {
    pub(crate) values: SmallVec<[R; 1]>,
    pub(crate) relation_id_entity: Entity,
}

impl<R: Relation> Component for Storage<R> {
    type Storage = R::Storage;
}

#[derive(Component)]
pub struct Foster<T: Relation>(PhantomData<T>);

#[derive(Component)]
pub struct Index {
    pub(crate) targets: [HashMap<Entity, HashMap<Entity, usize>>; 4],
    pub(crate) fosters: HashMap<Entity, Entity>,
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
    const DESPAWN_POLICY: DespawnPolicy = DespawnPolicy::Orphan;
    type Storage: ComponentStorage;
    const EXCLUSIVE: bool = false;
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

impl<'a, R: Relation> RelationSet for &'a R {
    type Types = R;
    type WorldQuery = (&'a Index, &'a Storage<R>);
    type EmptyJoinSet = Drop;
}

impl<'a, R: Relation> RelationSet for &'a mut R {
    type Types = R;
    type WorldQuery = (&'a Index, &'a mut Storage<R>);
    type EmptyJoinSet = Drop;
}

impl<T: RelationSet> RelationSet for Option<T> {
    type Types = T::Types;
    type WorldQuery = Option<(&'static Index, T::WorldQuery)>;
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
