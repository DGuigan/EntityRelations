use super::{Register, Relation, Storage};
use crate::prelude::Entity;
use bevy_utils::HashMap;
use std::any::TypeId;

// TODO: All tuple
pub trait Extract<Types, Target, const POS: usize> {
    type ExtractedVal;
    type RemainingKeys;
    type RemainingVals;

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals);
}

impl<K0, P0> Extract<K0, K0, 0> for P0 {
    type ExtractedVal = P0;
    type RemainingKeys = ();
    type RemainingVals = ();

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self, ())
    }
}

impl<K0, P0> Extract<(K0,), K0, 0> for (P0,) {
    type ExtractedVal = P0;
    type RemainingKeys = ();
    type RemainingVals = ();

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self.0, ())
    }
}

impl<K0, K1, P0, P1> Extract<(K0, K1), K0, 0> for (P0, P1) {
    type ExtractedVal = P0;
    type RemainingKeys = (K1,);
    type RemainingVals = (P1,);

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self.0, (self.1,))
    }
}

impl<K0, K1, P0, P1> Extract<(K0, K1), K1, 1> for (P0, P1) {
    type ExtractedVal = P1;
    type RemainingKeys = (K0,);
    type RemainingVals = (P0,);

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self.1, (self.0,))
    }
}

trait Append {
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
