use super::*;
use crate::{
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::Query,
};

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

pub trait Append {
    type Out<Item>: Append;
    fn append<Item>(self, item: Item) -> Self::Out<Item>;
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

#[derive(Default)]
pub struct Drop;

#[derive(Default)]
pub struct Wipe;

#[derive(Default)]
pub struct Waive;

pub trait Comb<Items> {
    type Out;
    fn comb(items: Items) -> Self::Out;
}

impl<Item> Comb<Item> for Drop {
    type Out = ();
    fn comb(_: Item) -> Self::Out {}
}

#[derive(Default)]
pub struct Wiped;

impl<Item> Comb<Item> for Wipe {
    type Out = Wiped;
    fn comb(_: Item) -> Self::Out {
        Wiped
    }
}

impl<Item> Comb<Item> for Waive {
    type Out = Item;
    fn comb(item: Item) -> Self::Out {
        item
    }
}

impl<I0, P0> Comb<(I0,)> for (P0,)
where
    P0: Comb<I0>,
{
    type Out = (P0::Out,);
    fn comb((i0,): (I0,)) -> Self::Out {
        (P0::comb(i0),)
    }
}

impl<I0, I1, P0, P1> Comb<(I0, I1)> for (P0, P1)
where
    P0: Comb<I0>,
    P1: Comb<I1>,
{
    type Out = (P0::Out, P1::Out);
    fn comb((i0, i1): (I0, I1)) -> Self::Out {
        (P0::comb(i0), P1::comb(i1))
    }
}

impl<I0, I1, I2, P0, P1, P2> Comb<(I0, I1, I2)> for (P0, P1, P2)
where
    P0: Comb<I0>,
    P1: Comb<I1>,
    P2: Comb<I2>,
{
    type Out = (P0::Out, P1::Out, P2::Out);
    fn comb((i0, i1, i2): (I0, I1, I2)) -> Self::Out {
        (P0::comb(i0), P1::comb(i1), P2::comb(i2))
    }
}

trait NoFlatten {}

impl NoFlatten for Wiped {}
impl<T: NoFlatten> NoFlatten for Option<T> {}
impl<R: Relation> NoFlatten for R {}
impl<R: Relation> NoFlatten for StorageWorldQueryItem<'_, R> {}
impl<R: Relation> NoFlatten for StorageWorldQueryMutItem<'_, R> {}

impl<Q, F> NoFlatten for &'_ Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
}

impl<Q, F> NoFlatten for &'_ mut Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
}

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
