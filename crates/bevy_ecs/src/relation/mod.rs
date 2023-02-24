use std::marker::PhantomData;

use crate as bevy_ecs;

use crate::{
    component::ComponentStorage,
    entity::Entity,
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::Query,
    world::World,
};

mod bookkeeping;
mod joins;
mod lenses;
mod traversals;

pub use bookkeeping::*;
pub use joins::*;
pub use traversals::*;

pub enum DespawnPolicy {
    Orphan,
    Reparent,
    RecursiveDespawn,
    RecursiveDisconnect,
}

impl DespawnPolicy {
    pub(crate) fn apply(&self, _world: &mut World, _start: Entity) {
        todo!("Despawn Policies not implemented yet")
    }
}

pub trait Relation: Sized + Send + Sync {
    type Arity: RelationArity<Self>;
    type Storage: ComponentStorage;
    const DESPAWN_POLICY: DespawnPolicy = DespawnPolicy::Orphan;
}

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

#[derive(Default, Clone, Copy)]
pub struct Identity;

pub trait RelationSet: Sealed {
    type Types;
    type WorldQuery: WorldQuery;
    type EmptyJoinSet: Default;
}

impl<'a, T: Relation> RelationSet for &'a T {
    type Types = T;
    type WorldQuery = &'a <T as Relation>::Arity;
    type EmptyJoinSet = Identity;
}

impl<'a, T: Relation> RelationSet for &'a mut T {
    type Types = T;
    type WorldQuery = &'a mut <T as Relation>::Arity;
    type EmptyJoinSet = Identity;
}

impl<T: RelationSet> RelationSet for Option<T> {
    type Types = T::Types;
    type WorldQuery = Option<T::WorldQuery>;
    type EmptyJoinSet = T::EmptyJoinSet;
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
    world_query: T::WorldQuery,
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
