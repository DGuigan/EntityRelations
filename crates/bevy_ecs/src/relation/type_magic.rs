use itertools::Itertools;

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

/*pub trait FlatIterProduct {
    type Out;
    fn flat_iter_product(self) -> Self::Out;
}

impl<I0> FlatIterProduct for (I0,)
where
    I0: Iterator,
{
    type Out = Self;
    fn flat_iter_product(self) -> Self::Out {
        self
    }
}

impl<I0, I1> FlatIterProduct for (I0, I1)
where
    I0: Iterator + Clone,
    I1: Iterator + Clone,
{
    type Out = itertools::Product<I0, I1>;
    fn flat_iter_product(self) -> Self::Out {
        self.0.cartesian_product(self.1)
    }
}*/
