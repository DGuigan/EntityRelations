// TODO: All tuple
pub trait TypedLens<Types, Target, const POS: usize> {
    type Get;
    type Set<Input>;
    type ExtractedVal;
    type RemainingKeys;
    type RemainingVals;

    fn get(&self) -> &Self::Get;
    fn set<Input>(self, value: Input) -> Self::Set<Input>;
    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals);
}

impl<K0, P0> TypedLens<K0, K0, 0> for P0 {
    type Get = Self;
    type Set<Input> = Input;
    type ExtractedVal = P0;
    type RemainingKeys = ();
    type RemainingVals = ();

    fn get(&self) -> &Self::Get {
        self
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        value
    }

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self, ())
    }
}

impl<K0, P0> TypedLens<(K0,), K0, 0> for (P0,) {
    type Get = P0;
    type Set<Input> = (Input,);
    type ExtractedVal = P0;
    type RemainingKeys = ();
    type RemainingVals = ();

    fn get(&self) -> &Self::Get {
        &self.0
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        (value,)
    }

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self.0, ())
    }
}

impl<K0, K1, P0, P1> TypedLens<(K0, K1), K0, 0> for (P0, P1) {
    type Get = P0;
    type Set<Input> = (Input, P1);
    type ExtractedVal = P0;
    type RemainingKeys = (K1,);
    type RemainingVals = (P1,);

    fn get(&self) -> &Self::Get {
        &self.0
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        (value, self.1)
    }

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self.0, (self.1,))
    }
}

impl<K0, K1, P0, P1> TypedLens<(K0, K1), K1, 1> for (P0, P1) {
    type Get = P1;
    type Set<Input> = (P0, Input);
    type ExtractedVal = P1;
    type RemainingKeys = (K0,);
    type RemainingVals = (P0,);

    fn get(&self) -> &Self::Get {
        &self.1
    }

    fn set<Input>(self, value: Input) -> Self::Set<Input> {
        (self.0, value)
    }

    fn extract(self) -> (Self::ExtractedVal, Self::RemainingVals) {
        (self.1, (self.0,))
    }
}

trait TupleAppend {
    type Out<Item>: TupleAppend;
    fn append<Item>(self, item: Item) -> Self::Out<Item>;
}

impl<'a, T> TupleAppend for &'a T {
    type Out<Item> = (&'a T, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self, item)
    }
}

impl<P0> TupleAppend for (P0,) {
    type Out<Item> = (P0, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self.0, item)
    }
}

impl<P0, P1> TupleAppend for (P0, P1) {
    type Out<Item> = (P0, P1, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self.0, self.1, item)
    }
}

impl<P0, P1, P2> TupleAppend for (P0, P1, P2) {
    type Out<Item> = (P0, P1, P2, Item);
    fn append<Item>(self, item: Item) -> Self::Out<Item> {
        (self.0, self.1, self.2, item)
    }
}

// Bound on GAT reduces bound noise elsewhere.
// However will have to end cyclically for max tuple size.
impl<P0, P1, P2, P3> TupleAppend for (P0, P1, P2, P3) {
    type Out<Item> = Self;
    fn append<Item>(self, _: Item) -> Self::Out<Item> {
        self
    }
}
