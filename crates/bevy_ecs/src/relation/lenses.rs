// TODO: All tuple
pub trait TypedGet<Types, Target, const POS: usize> {
    type Out;
    fn getter(&self) -> &Self::Out;
}

impl<K0, P0> TypedGet<K0, K0, 0> for P0 {
    type Out = Self;
    fn getter(&self) -> &Self::Out {
        self
    }
}

impl<K0, P0> TypedGet<(K0,), K0, 0> for (P0,) {
    type Out = P0;
    fn getter(&self) -> &Self::Out {
        &self.0
    }
}

impl<K0, K1, P0, P1> TypedGet<(K0, K1), K0, 0> for (P0, P1) {
    type Out = P0;
    fn getter(&self) -> &Self::Out {
        &self.0
    }
}

impl<K0, K1, P0, P1> TypedGet<(K0, K1), K1, 1> for (P0, P1) {
    type Out = P1;
    fn getter(&self) -> &Self::Out {
        &self.1
    }
}

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
