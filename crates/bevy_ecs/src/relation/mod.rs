use crate::system::Command;
use bevy_utils::{HashMap, HashSet};
use core::any::TypeId;
use smallvec::SmallVec;
use std::marker::PhantomData;

use crate as bevy_ecs;

use crate::{
    component::{Component, ComponentStorage},
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
pub struct Storage<R: Relation> {
    pub(crate) values: SmallVec<[R; 1]>,
}

impl<R: Relation> Component for Storage<R> {
    type Storage = R::Storage;
}

#[derive(Component, Default)]
pub struct Edges {
    pub(crate) targets: [HashMap<TypeId, HashMap<Entity, usize>>; 4],
    pub(crate) fosters: HashMap<TypeId, HashSet<Entity>>,
}

impl Edges {
    fn iter<R: Relation>(&self) -> impl '_ + Iterator<Item = (Entity, usize)> {
        self.targets[R::DESPAWN_POLICY as usize]
            .get(&TypeId::of::<Storage<R>>())
            .map(|map| map.iter().map(|(entity, index)| (*entity, *index)))
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

pub trait RelationQuerySet: Sealed + Send + Sync {
    type Types;
    type WorldQuery: WorldQuery;
    type ColsWith<T: Default>: Default;
}

impl<R: Relation> RelationQuerySet for &'_ R {
    type Types = R;
    type WorldQuery = StorageWorldQuery<R>;
    type ColsWith<T: Default> = T;
}

impl<R: Relation> RelationQuerySet for &'_ mut R {
    type Types = R;
    type WorldQuery = StorageWorldQueryMut<R>;
    type ColsWith<T: Default> = T;
}

impl<R: RelationQuerySet> RelationQuerySet for Option<R> {
    type Types = R::Types;
    type WorldQuery = Option<R::WorldQuery>;
    type ColsWith<T: Default> = R::ColsWith<T>;
}

// TODO: All tuple
impl<P0: RelationQuerySet> RelationQuerySet for (P0,) {
    type Types = (P0::Types,);
    type WorldQuery = (P0::WorldQuery,);
    type ColsWith<T: Default> = (P0::ColsWith<T>,);
}

impl<P0: RelationQuerySet, P1: RelationQuerySet> RelationQuerySet for (P0, P1) {
    type Types = (P0::Types, P1::Types);
    type WorldQuery = (P0::WorldQuery, P1::WorldQuery);
    type ColsWith<T: Default> = (P0::ColsWith<T>, P1::ColsWith<T>);
}

// TODO:
// - Manual `WorldQuery` impl to get `ComponentId` from `World` to remove the usage of `TypeId`
// - `TypeId` is not guarenteed to be stable which is a problem for serialization.
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct Relations<T: RelationQuerySet> {
    edges: &'static Edges,
    world_query: T::WorldQuery,
    _phantom: PhantomData<T>,
}

pub struct Ops<Query, Joins, EdgeComb, StorageComb, Traversal = ()> {
    query: Query,
    joins: Joins,
    edge_comb: PhantomData<EdgeComb>,
    storage_comb: PhantomData<StorageComb>,
    traversal: PhantomData<Traversal>,
}

impl<'w, 's, Q, F, R> Query<'w, 's, (Q, Relations<R>), F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet + Send + Sync,
{
    fn ops(&self) -> Ops<&Self, R::ColsWith<()>, R::ColsWith<Drop>, R::ColsWith<Drop>> {
        Ops {
            query: self,
            joins: R::ColsWith::<()>::default(),
            edge_comb: PhantomData,
            storage_comb: PhantomData,
            traversal: PhantomData,
        }
    }

    fn ops_mut(&mut self) -> Ops<&mut Self, R::ColsWith<()>, R::ColsWith<Drop>, R::ColsWith<Drop>> {
        Ops {
            query: self,
            joins: R::ColsWith::<()>::default(),
            edge_comb: PhantomData,
            storage_comb: PhantomData,
            traversal: PhantomData,
        }
    }
}

pub enum ControlFlow {
    Continue,
    Exit,
}

impl From<()> for ControlFlow {
    fn from(_: ()) -> Self {
        ControlFlow::Continue
    }
}

// It is nessecary to pass the tuples back into the trait itself
// otherwise we get mysterious HRTB errors about impls on FnOnce not being general enough
pub trait ForEachPermutations<EdgeTup, StorageTup, JoinTup> {
    type Components<'a>;
    //type Joins<'a>;
    fn for_each<Func, Ret>(self, func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'a> FnMut(&mut Self::Components<'a> /*, Self::Joins<'a>*/) -> Ret;
}

pub struct Set<R>
where
    R: Relation,
{
    pub foster: Entity,
    pub target: Entity,
    pub relation: R,
}

impl<R> Command for Set<R>
where
    R: Relation,
{
    fn write(self, world: &mut World) {
        let mut foster = world
            .get_entity_mut(self.foster)
            .expect("Foster dose not exist");

        let mut edges = foster.take::<Edges>().unwrap_or_else(|| Edges::default());

        let mut storage = foster.take::<Storage<R>>().unwrap_or_else(|| Storage::<R> {
            values: SmallVec::new(),
        });

        if let Some(index) = edges.targets[R::DESPAWN_POLICY as usize]
            .entry(TypeId::of::<Storage<R>>())
            .or_default()
            .get(&self.target)
        {
        } else {
        }
    }
}
