use super::Relation;
use crate::{component::Component, entity::Entity, system::Command, world::World};
use bevy_utils::HashMap;
use std::collections::VecDeque;

mod sealed {
    use super::*;
    pub trait Sealed {}
    impl<R: Relation> Sealed for Multi<R> {}
    impl<R: Relation> Sealed for Exclusive<R> {}
}

use sealed::*;

// [IMPORTANT] Privacy:
// Multi + Exclusive should NEVER be Clone/Copy and their inner field should NEVER
// leak outside of the crate. Otherwise users can perform changes that disobey policies.
// Additionally traits that expose public ways to create these types like `Default` or 'From'
// Should not be implemented. The user should never be able to manually create these types!!!
pub struct Multi<R>(pub(crate) HashMap<Entity, R>);
pub struct Exclusive<R>(pub(crate) Entity, pub(crate) R);

impl<R> Multi<R> {
    pub fn iter(&self) -> impl '_ + Iterator<Item = &R> {
        self.0.iter().map(|(_, r)| r)
    }

    pub fn iter_mut(&mut self) -> impl '_ + Iterator<Item = &mut R> {
        self.0.iter_mut().map(|(_, r)| r)
    }
}

impl<R: Relation + 'static> Component for Multi<R> {
    type Storage = R::Storage;
}

impl<R: Relation + 'static> Component for Exclusive<R> {
    type Storage = R::Storage;
}

pub trait RelationArity<R: Relation>: Component + Sealed {
    type OptimalQueue;

    fn set(world: &mut World, foster: Entity, target: Entity, relation: R);
}

impl<R: Relation + 'static> RelationArity<R> for Multi<R> {
    type OptimalQueue = VecDeque<Entity>;

    fn set(world: &mut World, foster: Entity, target: Entity, relation: R) {
        let Some(mut foster) = world.get_entity_mut(foster) else { return };
        if let Some(mut multi) = foster.get_mut::<Self>() {
            multi.0.insert(target, relation);
        } else {
            foster.insert(Self(HashMap::from([(target, relation)])));
        }
    }
}

impl<R: Relation + 'static> RelationArity<R> for Exclusive<R> {
    type OptimalQueue = Option<Entity>;

    fn set(world: &mut World, foster: Entity, target: Entity, relation: R) {
        let Some(mut foster) = world.get_entity_mut(foster) else { return };
        if let Some(mut exclusive) = foster.get_mut::<Self>() {
            let Exclusive(t, r) = &mut *exclusive;
            let old = *t;
            *t = target;
            *r = relation;
            if old != target {
                R::DESPAWN_POLICY.apply(world, old);
            }
        } else {
            foster.insert(Self(target, relation));
        }
    }
}

pub(crate) struct Set<R>
where
    R: Relation,
{
    pub(crate) foster: Entity,
    pub(crate) target: Entity,
    pub(crate) relation: R,
}

impl<R> Command for Set<R>
where
    R: 'static + Relation,
{
    fn write(self, world: &mut World) {
        R::Arity::set(world, self.foster, self.target, self.relation)
    }
}
