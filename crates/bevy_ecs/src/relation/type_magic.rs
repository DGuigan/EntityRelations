//struct TypePadding<const N: u8> {}

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

// (T, U) + (A, B) -> ((T, A), (U, B))
// (T, U) + (A, NoStitch) -> ((T, A), U)
pub trait Stitch {}
