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
pub(crate) struct Storage<R: Relation> {
    pub(crate) values: SmallVec<[R; 1]>,
}

impl<R: Relation> Default for Storage<R> {
    fn default() -> Self {
        Self {
            values: SmallVec::default(),
        }
    }
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

pub trait ForEachPermutations {
    type Components<'c>;
    type Joins<'i, 'a, 'j>;
    fn for_each<Func, Ret>(self, func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'r, 'c, 'i, 'a, 'j> FnMut(
            &'r mut Self::Components<'c>,
            Self::Joins<'i, 'a, 'j>,
        ) -> Ret;
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
        let Some((mut foster_edges, mut foster_storage)) = world
            .get_entity_mut(self.foster)
            .map(|mut foster| (
                foster.take::<Edges>().unwrap_or_default(),
                foster.take::<Storage<R>>().unwrap_or_default()
            ))
        else {
            // TODO: Logging
            return
        };

        let foster_indices = foster_edges.targets[R::DESPAWN_POLICY as usize]
            .entry(TypeId::of::<Storage<R>>())
            .or_default();

        let mut exclusive_overwrite = None;

        // TODO: Logging
        if let Some(index) = foster_indices.get(&self.target) {
            foster_storage.values[*index] = self.relation;
        } else if let Some((old_target, index)) = foster_indices
            .iter()
            .next()
            .map(|(target, index)| (*target, *index))
            .filter(|_| R::EXCLUSIVE)
        {
            world
                .get_entity_mut(old_target)
                .expect("Foster should not have dangling entries")
                .get_mut::<Edges>()
                .expect("Edge component should exist")
                .fosters
                .get_mut(&TypeId::of::<Storage<R>>())
                .expect("Target should have relation entry")
                .remove(&self.foster);

            foster_indices.clear();
            foster_indices.insert(self.target, index);
            foster_storage.values[index] = self.relation;
            exclusive_overwrite = Some(old_target);
        } else if let Some(mut target_edges) = world
            .get_entity_mut(self.target)
            .map(|mut target| target.take::<Edges>().unwrap_or_default())
        {
            foster_indices.insert(self.target, foster_storage.values.len());
            foster_storage.values.push(self.relation);

            target_edges
                .fosters
                .entry(TypeId::of::<Storage<R>>())
                .or_default()
                .insert(self.foster);

            world
                .get_entity_mut(self.target)
                .unwrap()
                .insert(target_edges);
        }

        world
            .get_entity_mut(self.foster)
            .unwrap()
            .insert((foster_edges, foster_storage));

        // TODO: Cleanup
    }
}

pub struct UnSet<R>
where
    R: Relation,
{
    pub foster: Entity,
    pub target: Entity,
    pub _phantom: PhantomData<R>,
}

impl<R> Command for UnSet<R>
where
    R: Relation,
{
    fn write(self, world: &mut World) {
        if world
            .get_mut::<Edges>(self.foster)
            .map_or(false, |mut edges| {
                edges.targets[R::DESPAWN_POLICY as usize]
                    .get_mut(&TypeId::of::<Storage<R>>())
                    .and_then(|indices| indices.remove(&self.target))
                    .is_some()
            })
        {
            world
                .get_mut::<Edges>(self.target)
                .expect("Edge component should exist")
                .fosters
                .get_mut(&TypeId::of::<Storage<R>>())
                .expect("Target should have relation entry")
                .remove(&self.foster);

            // TODO: Cleanup
        }
    }
}

pub struct CheckeDespawn {
    pub entity: Entity,
}

impl Command for CheckeDespawn {
    fn write(self, world: &mut World) {
        if let Some(indices) = world.get_mut::<Edges>(self.entity) {
            // TODO: Cleanup
        }
        world.despawn(self.entity);
    }
}
