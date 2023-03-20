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
mod type_magic;
//mod traversals;

pub use joins::*;
pub use type_magic::*;
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

pub struct Storage<R: Relation> {
    pub(crate) values: SmallVec<[R; 1]>,
}

impl<R: Relation> Component for Storage<R> {
    type Storage = R::Storage;
}

#[derive(Component)]
pub struct Register {
    pub(crate) targets: [HashMap<TypeId, HashMap<Entity, usize>>; 4],
    pub(crate) fosters: HashMap<TypeId, Entity>,
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
    type Registers: WorldQuery;
    type Storage: WorldQuery;
    type EmptyJoinSet: Default;
}

impl<R: Relation> RelationSet for &'_ R {
    type Types = R;
    type Registers = &'static Register;
    type Storage = &'static Storage<R>;
    type EmptyJoinSet = Drop;
}

impl<R: Relation> RelationSet for &'_ mut R {
    type Types = R;
    type Registers = &'static Register;
    type Storage = &'static mut Storage<R>;
    type EmptyJoinSet = Drop;
}

impl<R: RelationSet> RelationSet for Option<R> {
    type Types = R;
    type Registers = Option<R::Registers>;
    type Storage = Option<R::Storage>;
    type EmptyJoinSet = Drop;
}

// TODO: All tuple
impl<P0: RelationSet> RelationSet for (P0,) {
    type Types = (P0::Types,);
    type Registers = (P0::Registers,);
    type Storage = (P0::Storage,);
    type EmptyJoinSet = (P0::EmptyJoinSet,);
}

// TODO:
// - Manual `WorldQuery` impl to get `ComponentId` from `World` to remove the usage of `TypeId`
// - `TypeId` is not guarenteed to be stable which is a problem for serialization.
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct Relations<T: RelationSet + Send + Sync> {
    registers: T::Registers,
    storage: T::Storage,
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
