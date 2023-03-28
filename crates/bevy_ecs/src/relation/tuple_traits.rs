use super::{Edges, Relation, Storage};
use crate::prelude::Entity;
use bevy_utils::HashMap;
use std::any::TypeId;

// TODO: All tuple
pub trait Lenses<Types, Target, const POS: usize> {
    type Get;
    type Set<Input>;
    fn get(&self) -> &Self::Get;
    fn set<Input>(self, value: Input) -> Self::Set<Input>;
}

impl<K0, P0> Lenses<K0, K0, 0> for P0 {
    type Get = Self;
    type Set<Input> = Input;

    fn get(&self) -> &Self::Get {
        self
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        value
    }
}

impl<K0, P0> Lenses<(K0,), K0, 0> for (P0,) {
    type Get = P0;
    type Set<Input> = (Input,);

    fn get(&self) -> &Self::Get {
        &self.0
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        (value,)
    }
}

impl<K0, K1, P0, P1> Lenses<(K0, K1), K0, 0> for (P0, P1) {
    type Get = P0;
    type Set<Input> = (Input, P1);

    fn get(&self) -> &Self::Get {
        &self.0
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        (value, self.1)
    }
}

impl<K0, K1, P0, P1> Lenses<(K0, K1), K1, 1> for (P0, P1) {
    type Get = P1;
    type Set<Input> = (P0, Input);

    fn get(&self) -> &Self::Get {
        &self.1
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        (self.0, value)
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

trait Flatten<Flattened: Append> {
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
