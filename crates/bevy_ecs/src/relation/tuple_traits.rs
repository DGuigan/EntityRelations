use super::{Edges, Relation, Storage};
use crate::entity::Entity;
use bevy_utils::hashbrown::hash_map::Iter as HashMapIter;
use bevy_utils::HashMap;
use std::any::TypeId;
use std::iter::Flatten as FlatIter;
use std::iter::Map;
use std::option::IntoIter;

// TODO: All tuple
pub trait TypedSet<Types, Target, const POS: usize> {
    type Out<Input>;
    fn set<Input>(self, value: Input) -> Self::Out<Input>;
}

impl<K0, P0> TypedSet<K0, K0, 0> for P0 {
    type Out<Input> = Input;
    fn set<Input>(self, value: Input) -> Self::Out<Input> {
        value
    }
}

impl<K0, P0> TypedSet<(K0,), K0, 0> for (P0,) {
    type Out<Input> = (Input,);
    fn set<Input>(self, value: Input) -> Self::Out<Input> {
        (value,)
    }
}

impl<K0, K1, P0, P1> TypedSet<(K0, K1), K0, 0> for (P0, P1) {
    type Out<Input> = (Input, P1);
    fn set<Input>(self, value: Input) -> Self::Out<Input> {
        (value, self.1)
    }
}

impl<K0, K1, P0, P1> TypedSet<(K0, K1), K1, 1> for (P0, P1) {
    type Out<Input> = (P0, Input);
    fn set<Input>(self, value: Input) -> Self::Out<Input> {
        (self.0, value)
    }
}

impl<K0, K1, K2, P0, P1, P2> TypedSet<(K0, K1, K2), K0, 0> for (P0, P1, P2) {
    type Out<Input> = (Input, P1, P2);
    fn set<Input>(self, value: Input) -> Self::Out<Input> {
        (value, self.1, self.2)
    }
}

impl<K0, K1, K2, P0, P1, P2> TypedSet<(K0, K1, K2), K1, 1> for (P0, P1, P2) {
    type Out<Input> = (P0, Input, P2);
    fn set<Input>(self, value: Input) -> Self::Out<Input> {
        (self.0, value, self.2)
    }
}

impl<K0, K1, K2, P0, P1, P2> TypedSet<(K0, K1, K2), K2, 2> for (P0, P1, P2) {
    type Out<Input> = (P0, P1, Input);
    fn set<Input>(self, value: Input) -> Self::Out<Input> {
        (self.0, self.1, value)
    }
}

// TODO: (1) Use TAITs when they're stable
// TODO: (2) Use Frog types when they're stable
type PairIter<'a> = Map<HashMapIter<'a, Entity, usize>, fn((&Entity, &usize)) -> (Entity, usize)>;
type EdgeIter<'a> = FlatIter<IntoIter<PairIter<'a>>>;

fn deref_pair((entity, index): (&Entity, &usize)) -> (Entity, usize) {
    (*entity, *index)
}

fn to_pair_iter(map: &HashMap<Entity, usize>) -> PairIter<'_> {
    map.iter().map(deref_pair)
}

pub trait EdgeIters {
    type Out<'a>;
    fn edge_iters(edges: &Edges) -> Self::Out<'_>;
}

impl<R> EdgeIters for R
where
    R: Relation,
{
    type Out<'a> = EdgeIter<'a>;

    fn edge_iters(edges: &Edges) -> Self::Out<'_> {
        edges.targets[R::DESPAWN_POLICY as usize]
            .get(&TypeId::of::<Storage<R>>())
            .map(to_pair_iter)
            .into_iter()
            .flatten()
    }
}

impl<P0> EdgeIters for (P0,)
where
    P0: EdgeIters,
{
    type Out<'a> = (P0::Out<'a>,);

    fn edge_iters(edges: &Edges) -> Self::Out<'_> {
        (P0::edge_iters(edges),)
    }
}

impl<P0, P1> EdgeIters for (P0, P1)
where
    P0: EdgeIters,
    P1: EdgeIters,
{
    type Out<'a> = (P0::Out<'a>, P1::Out<'a>);

    fn edge_iters(edges: &Edges) -> Self::Out<'_> {
        (P0::edge_iters(edges), P1::edge_iters(edges))
    }
}

impl<P0, P1, P2> EdgeIters for (P0, P1, P2)
where
    P0: EdgeIters,
    P1: EdgeIters,
    P2: EdgeIters,
{
    type Out<'a> = (P0::Out<'a>, P1::Out<'a>, P2::Out<'a>);

    fn edge_iters(edges: &Edges) -> Self::Out<'_> {
        (
            P0::edge_iters(edges),
            P1::edge_iters(edges),
            P2::edge_iters(edges),
        )
    }
}

pub trait Append {
    type Out<Item>: Append;
    fn append<Item>(self, item: Item) -> Self::Out<Item>;
}

impl<'a, T> Append for &'a T {
    type Out<Item> = (&'a T, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self, item)
    }
}

impl Append for () {
    type Out<Item> = (Item,);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (item,)
    }
}

impl<P0> Append for (P0,) {
    type Out<Item> = (P0, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self.0, item)
    }
}

impl<P0, P1> Append for (P0, P1) {
    type Out<Item> = (P0, P1, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self.0, self.1, item)
    }
}

impl<P0, P1, P2> Append for (P0, P1, P2) {
    type Out<Item> = (P0, P1, P2, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self.0, self.1, self.2, item)
    }
}

// Bound on GAT reduces bound noise elsewhere.
// However will have to end cyclically for max tuple size.
impl<P0, P1, P2, P3> Append for (P0, P1, P2, P3) {
    type Out<Item> = Self;
    fn append<Item>(self, _: Item) -> Self::Out<Item> {
        self
    }
}

trait NoFlatten {}

pub trait Flatten<Flattened: Append> {
    type Out: Append;
    fn flatten(self, flattened: Flattened) -> Self::Out;
}

impl<Flattened: Append> Flatten<Flattened> for () {
    type Out = Flattened;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        flattened
    }
}

impl<Flattened: Append, I0: NoFlatten> Flatten<Flattened> for I0 {
    type Out = <Flattened as Append>::Out<I0>;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        flattened.append(self)
    }
}

impl<Flattened: Append> Flatten<Flattened> for ((),) {
    type Out = Flattened;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        flattened
    }
}

impl<Flattened: Append, I0: NoFlatten> Flatten<Flattened> for (I0,) {
    type Out = <Flattened as Append>::Out<I0>;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        flattened.append(self.0)
    }
}

impl<Flattened: Append, I1> Flatten<Flattened> for ((), I1)
where
    I1: Flatten<Flattened>,
{
    type Out = <I1 as Flatten<Flattened>>::Out;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        self.1.flatten(flattened)
    }
}

impl<Flattened: Append, I0: NoFlatten, I1> Flatten<Flattened> for (I0, I1)
where
    I1: Flatten<<Flattened as Append>::Out<I0>>,
{
    type Out = <I1 as Flatten<<Flattened as Append>::Out<I0>>>::Out;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        self.1.flatten(flattened.append(self.0))
    }
}

impl<Flattened: Append, I1, I2> Flatten<Flattened> for ((), I1, I2)
where
    (I1, I2): Flatten<Flattened>,
{
    type Out = <(I1, I2) as Flatten<Flattened>>::Out;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        (self.1, self.2).flatten(flattened)
    }
}

impl<Flattened: Append, I0: NoFlatten, I1, I2> Flatten<Flattened> for (I0, I1, I2)
where
    (I1, I2): Flatten<<Flattened as Append>::Out<I0>>,
{
    type Out = <(I1, I2) as Flatten<<Flattened as Append>::Out<I0>>>::Out;
    fn flatten(self, flattened: Flattened) -> Self::Out {
        (self.1, self.2).flatten(flattened.append(self.0))
    }
}
