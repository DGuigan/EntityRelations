use crate::change_detection::Mut;
use crate::query::{ReadOnlyWorldQuery, WorldQuery};
use crate::system::Query;
use std::any::TypeId;

use super::{tuple_traits::*, *};

// T _ Q: Join
// T S Q: Full Join
// O _ _: Left Join
// O S _: Full left Join

#[derive(Default)]
pub struct Drop;
pub struct Keep;

trait Filtered<Filtereds> {
    type Out;
    fn filtered(self) -> Self::Out;
}

impl<T> Filtered<Keep> for T {
    type Out = T;
    fn filtered(self) -> Self::Out {
        self
    }
}

impl<T> Filtered<Drop> for T {
    type Out = ();
    fn filtered(self) -> Self::Out {}
}

impl<P0, F0> Filtered<(F0,)> for (P0,)
where
    P0: Filtered<F0>,
{
    type Out = (P0::Out,);
    fn filtered(self) -> Self::Out {
        (self.0.filtered(),)
    }
}

pub trait Joinable {}

impl<Q, F> Joinable for &'_ Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
}

impl<Q, F> Joinable for &'_ mut Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
}

pub trait Join<'j, Storage> {
    type Out;
    fn matches(&self, target: Entity) -> bool;
    fn joined(&'j mut self, target_info: (Entity, usize), storage: &'j mut Storage) -> Self::Out;
}

impl<I0, S0, J0> ForEachPermutations for ((I0,), (S0,), (J0,))
where
    I0: Iterator<Item = (Entity, usize)>,
    J0: for<'j> Join<'j, S0>,
{
    type In<'a> = <J0 as Join<'a, S0>>::Out;
    fn for_each<F, R>(self, mut func: F)
    where
        R: Into<ControlFlow>,
        F: FnMut(Self::In<'_>) -> R,
    {
        let ((items0,), (mut storage0,), (mut joins0,)) = self;
        for i0 in items0 {
            if !joins0.matches(i0.0) {
                continue;
            }
            if let ControlFlow::Exit = func(joins0.joined(i0, &mut storage0)).into() {
                return;
            }
        }
    }
}

impl<I0, I1, S0, S1, J0, J1> ForEachPermutations for ((I0, I1), (S0, S1), (J0, J1))
where
    I0: Clone + Iterator<Item = (Entity, usize)>,
    I1: Clone + Iterator<Item = (Entity, usize)>,
    J0: for<'j> Join<'j, S0>,
    J1: for<'j> Join<'j, S1>,
{
    type In<'a> = (<J0 as Join<'a, S0>>::Out, <J1 as Join<'a, S1>>::Out);
    fn for_each<F, R>(self, mut func: F)
    where
        R: Into<ControlFlow>,
        F: FnMut(Self::In<'_>) -> R,
    {
        let ((items0, items1), (mut storage0, mut storage1), (mut joins0, mut joins1)) = self;
        for i0 in items0 {
            if !joins0.matches(i0.0) {
                continue;
            }
            for i1 in items1.clone() {
                if !joins1.matches(i1.0) {
                    continue;
                }
                if let ControlFlow::Exit = func((
                    joins0.joined(i0, &mut storage0),
                    joins1.joined(i1, &mut storage1),
                ))
                .into()
                {
                    return;
                }
            }
        }
    }
}

impl<I0, I1, I2, S0, S1, S2, J0, J1, J2> ForEachPermutations
    for ((I0, I1, I2), (S0, S1, S2), (J0, J1, J2))
where
    I0: Clone + Iterator<Item = (Entity, usize)>,
    I1: Clone + Iterator<Item = (Entity, usize)>,
    I2: Clone + Iterator<Item = (Entity, usize)>,
    J0: for<'j> Join<'j, S0>,
    J1: for<'j> Join<'j, S1>,
    J2: for<'j> Join<'j, S2>,
{
    type In<'a> = (
        <J0 as Join<'a, S0>>::Out,
        <J1 as Join<'a, S1>>::Out,
        <J2 as Join<'a, S2>>::Out,
    );
    fn for_each<F, R>(self, mut func: F)
    where
        R: Into<ControlFlow>,
        F: FnMut(Self::In<'_>) -> R,
    {
        let (
            (items0, items1, items2),
            (mut storage0, mut storage1, mut storage2),
            (mut joins0, mut joins1, mut joins2),
        ) = self;
        for i0 in items0 {
            if !joins0.matches(i0.0) {
                continue;
            }
            for i1 in items1.clone() {
                if !joins1.matches(i1.0) {
                    continue;
                }
                for i2 in items2.clone() {
                    if !joins2.matches(i2.0) {
                        continue;
                    }
                    if let ControlFlow::Exit = func((
                        joins0.joined(i0, &mut storage0),
                        joins1.joined(i1, &mut storage1),
                        joins2.joined(i2, &mut storage2),
                    ))
                    .into()
                    {
                        return;
                    }
                }
            }
        }
    }
}

impl<'j, Q, F> Join<'j, ()> for &'_ Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'j>;

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(&'j mut self, (target, _index): (Entity, usize), _storage: &'j mut ()) -> Self::Out {
        (**self).get(target).unwrap()
    }
}

impl<'j, Q, F, R> Join<'j, StorageWorldQuery<R>> for &'_ Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (&'j R, <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'j>);

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(
        &'j mut self,
        (target, index): (Entity, usize),
        storage: &'j mut StorageWorldQuery<R>,
    ) -> Self::Out {
        (
            &storage.storage.values[index],
            (**self).get(target).unwrap(),
        )
    }
}

impl<'j, Q, F, R> Join<'j, StorageWorldQueryMut<R>> for &'_ Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (
        &'j mut R,
        <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'j>,
    );

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(
        &'j mut self,
        (target, index): (Entity, usize),
        storage: &'j mut StorageWorldQueryMut<R>,
    ) -> Self::Out {
        (
            &mut storage.storage.values[index],
            (**self).get(target).unwrap(),
        )
    }
}

impl<'j, Q, F> Join<'j, ()> for &'_ mut Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <Q as WorldQuery>::Item<'j>;

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(&'j mut self, (target, _index): (Entity, usize), _storage: &'j mut ()) -> Self::Out {
        (**self).get_mut(target).unwrap()
    }
}

impl<'j, Q, F, R> Join<'j, StorageWorldQuery<R>> for &'_ mut Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (&'j R, <Q as WorldQuery>::Item<'j>);

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(
        &'j mut self,
        (target, index): (Entity, usize),
        storage: &'j mut StorageWorldQuery<R>,
    ) -> Self::Out {
        (
            &storage.storage.values[index],
            (**self).get_mut(target).unwrap(),
        )
    }
}

impl<'j, Q, F, R> Join<'j, StorageWorldQueryMut<R>> for &'_ mut Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (&'j R, <Q as WorldQuery>::Item<'j>);

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(
        &'j mut self,
        (target, index): (Entity, usize),
        storage: &'j mut StorageWorldQueryMut<R>,
    ) -> Self::Out {
        (
            &mut storage.storage.values[index],
            (**self).get_mut(target).unwrap(),
        )
    }
}

pub trait DeclarativeJoin<R, Joins, Filters, Item, const POS: usize>
where
    R: RelationQuerySet,
    Item: Joinable,
{
    type Joined<T: Relation>
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>;

    type TotalJoined<T: Relation>
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>;

    fn join<T: Relation>(self, item: Item) -> Self::Joined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>;

    fn total_join<T: Relation>(self, item: Item) -> Self::TotalJoined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>;
}

impl<'o, 'w, 's, Q, R, F, Joins, Filters, Traversal, Item, const POS: usize>
    DeclarativeJoin<R, Joins, Filters, Item, POS>
    for Ops<&'o Query<'w, 's, (Q, Relations<R>), F>, Joins, Filters, Traversal>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    Item: Joinable,
{
    type Joined<T: Relation> = Ops<
        &'o Query<'w, 's, (Q, Relations<R>), F>,
        <Joins as TypedSet<R::Types, T, POS>>::Out<Item>,
        <Filters as TypedSet<R::Types, T, POS>>::Out<Drop>,
        Traversal
    >
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>;

    type TotalJoined<T: Relation> = Ops<
        &'o Query<'w, 's, (Q, Relations<R>), F>,
        <Joins as TypedSet<R::Types, T, POS>>::Out<Item>,
        <Filters as TypedSet<R::Types, T, POS>>::Out<Keep>,
        Traversal
    >
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>;

    fn join<T: Relation>(self, item: Item) -> Self::Joined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>,
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            filters: self.filters.set(Drop),
            traversal: PhantomData,
        }
    }

    fn total_join<T: Relation>(self, item: Item) -> Self::TotalJoined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        Filters: TypedSet<R::Types, T, POS>,
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            filters: self.filters.set(Keep),
            traversal: PhantomData,
        }
    }
}

impl<Q, R, F, Joins, Filters> ForEachPermutations
    for Ops<&'_ Query<'_, '_, (Q, Relations<R>), F>, Joins, Filters>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    R::WorldQuery: Filtered<Filters>,
    <R::WorldQuery as Filtered<Filters>>::Out: Flatten<()>,
{
    type In<'a> = u8;
    fn for_each<Func, Ret>(self, func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: FnMut(Self::In<'_>) -> Ret,
    {
        todo!()
    }
}

/*#[cfg(test)]
#[allow(dead_code)]
#[allow(unused_variables)]
mod compile_tests {
    use super::*;
    use crate::{component::TableStorage, prelude::*};

    #[derive(Component)]
    struct A;

    #[derive(Component)]
    struct B;

    impl Relation for B {
        type Arity = Exclusive<Self>;
        type Storage = TableStorage;
    }

    #[derive(Component)]
    struct C;

    impl Relation for C {
        type Arity = Multi<Self>;
        type Storage = TableStorage;
    }

    #[derive(Component)]
    struct D;

    #[derive(Component)]
    struct E;

    fn join_immut(rq: Query<(&A, Relations<(&C, &B)>)>, d: Query<&D>, e: Query<&E>) {
        rq.ops()
            .join::<B>(&e)
            .join::<C>(&d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn join_left_mut(mut rq: Query<(&A, Relations<(&mut C, &mut B)>)>, d: Query<&D>, e: Query<&E>) {
        rq.ops_mut()
            .join::<B>(&e)
            .join::<C>(&d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn join_right_mut(rq: Query<(&A, Relations<(&C, &B)>)>, mut d: Query<&D>, mut e: Query<&E>) {
        rq.ops()
            .join::<B>(&mut e)
            .join::<C>(&mut d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn join_full_mut(
        mut rq: Query<(&A, Relations<(&mut C, &mut B)>)>,
        mut d: Query<&D>,
        mut e: Query<&E>,
    ) {
        rq.ops_mut()
            .join::<B>(&mut e)
            .join::<C>(&mut d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn join_immut_optional(
        rq: Query<(&A, Relations<(Option<&C>, Option<&B>)>)>,
        d: Query<&D>,
        e: Query<&E>,
    ) {
        rq.ops()
            .join::<B>(&e)
            .join::<C>(&d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn join_left_mut_optional(
        mut rq: Query<(&A, Relations<(Option<&mut C>, Option<&mut B>)>)>,
        d: Query<&D>,
        e: Query<&E>,
    ) {
        rq.ops_mut()
            .join::<B>(&e)
            .join::<C>(&d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn join_right_mut_optional(
        rq: Query<(&A, Relations<(Option<&C>, Option<&B>)>)>,
        mut d: Query<&D>,
        mut e: Query<&E>,
    ) {
        rq.ops()
            .join::<B>(&mut e)
            .join::<C>(&mut d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn join_full_mut_optional(
        mut rq: Query<(&A, Relations<(Option<&mut C>, Option<&mut B>)>)>,
        mut d: Query<&D>,
        mut e: Query<&E>,
    ) {
        rq.ops_mut()
            .join::<B>(&mut e)
            .join::<C>(&mut d)
            .for_each(|(_, (cd, be))| cd.for_each(|_| {}));
    }

    fn generic<R: Relation>(rq: Query<(&A, Relations<&R>)>, b: Query<&B>) {
        rq.ops().join::<R>(&b).for_each(|_| {})
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::{self as bevy_ecs, component::TableStorage, prelude::*};

    #[derive(StageLabel)]
    struct UpdateStage;

    fn run_system<Param, S: IntoSystem<(), (), Param>>(world: &mut World, system: S) {
        let mut schedule = Schedule::default();
        let mut update = SystemStage::parallel();
        update.add_system(system);
        schedule.add_stage(UpdateStage, update);
        schedule.run(world);
    }

    #[derive(Component)]
    struct Alice;

    #[derive(Component)]
    struct Bob;

    #[derive(Component)]
    struct Fruit(&'static str);

    #[derive(Component)]
    struct Vegetable(&'static str);

    struct Owns(usize);

    impl Relation for Owns {
        type Arity = Multi<Self>;
        type Storage = TableStorage;
    }

    fn setup(mut commands: Commands) {
        let fruit_ids = ["Mango", "Lychee", "Guava", "Pomelo", "Kiwi", "Nashi pear"]
            .into_iter()
            .map(Fruit)
            .map(|fruit| commands.spawn(fruit).id())
            .collect::<Vec<_>>();

        let veg_ids = ["Onions", "Ube", "Okra", "Bak choy", "Fennel"]
            .into_iter()
            .map(Vegetable)
            .map(|veg| commands.spawn(veg).id())
            .collect::<Vec<_>>();

        let alice = commands.spawn(Alice).id();
        let bob = commands.spawn(Bob).id();

        fruit_ids.iter().enumerate().take(3).for_each(|(n, fruit)| {
            commands.add(Set {
                foster: alice,
                target: *fruit,
                relation: Owns(n),
            })
        });

        veg_ids.iter().enumerate().skip(2).for_each(|(n, fruit)| {
            commands.add(Set {
                foster: alice,
                target: *fruit,
                relation: Owns(n),
            })
        });

        fruit_ids.iter().enumerate().skip(2).for_each(|(n, fruit)| {
            commands.add(Set {
                foster: bob,
                target: *fruit,
                relation: Owns(n),
            })
        });

        veg_ids.iter().enumerate().take(3).for_each(|(n, fruit)| {
            commands.add(Set {
                foster: bob,
                target: *fruit,
                relation: Owns(n),
            })
        });
    }

    fn nutrients(
        alice: Query<(&Alice, Relations<&Owns>)>,
        bob: Query<(&Bob, Relations<&Owns>)>,
        fruits: Query<&Fruit>,
        veggies: Query<&Vegetable>,
    ) {
        let mut owned = vec![];

        alice
            .ops()
            .join::<Owns>(&fruits)
            .for_each(|(_alice, fruits)| {
                fruits.for_each(|(quantity, fruit)| owned.push((quantity.0, fruit.0)))
            });

        owned.sort_by_key(|(quantity, _)| *quantity);
        assert_eq!(owned, vec![(0, "Mango"), (1, "Lychee"), (2, "Guava")]);

        let mut owned = vec![];

        alice
            .ops()
            .join::<Owns>(&veggies)
            .for_each(|(_alice, veg)| {
                veg.for_each(|(quantity, veg)| owned.push((quantity.0, veg.0)))
            });

        owned.sort_by_key(|(quantity, _)| *quantity);
        assert_eq!(owned, vec![(2, "Okra"), (3, "Bak choy"), (4, "Fennel")]);

        let mut owned = vec![];

        bob.ops().join::<Owns>(&fruits).for_each(|(_bob, fruits)| {
            fruits.for_each(|(quantity, fruit)| owned.push((quantity.0, fruit.0)))
        });

        owned.sort_by_key(|(quantity, _)| *quantity);
        assert_eq!(
            owned,
            vec![(2, "Guava"), (3, "Pomelo"), (4, "Kiwi"), (5, "Nashi pear")]
        );

        let mut owned = vec![];

        bob.ops().join::<Owns>(&veggies).for_each(|(_bob, veg)| {
            veg.for_each(|(quantity, veg)| owned.push((quantity.0, veg.0)))
        });

        owned.sort_by_key(|(quantity, _)| *quantity);
        assert_eq!(owned, vec![(0, "Onions"), (1, "Ube"), (2, "Okra")])
    }

    #[test]
    fn multi_join_test() {
        let mut world = World::new();
        run_system(&mut world, setup);
        run_system(&mut world, nutrients);
    }
}*/
