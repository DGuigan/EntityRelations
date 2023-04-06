use crate::change_detection::Mut;
use crate::query::{ReadOnlyWorldQuery, WorldQuery};
use crate::system::Query;
use std::any::TypeId;

use super::{tuple_traits::*, *};

// T _ Q: Join
// T S Q: Full Join
// O _ _: Left Join
// O S _: Full left Join

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

//trait Check<Key> {}
//trait Join<Key> {}

pub trait Join<'j, Storage> {
    type Out;
    fn matches(&self, target: Entity) -> bool;
    fn joined(&'j mut self, target_info: (Entity, usize), storage: &'j mut Storage) -> Self::Out;
}

impl<'j, Q, F> Join<'j, Wiped> for &'_ Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'j>;

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(
        &'j mut self,
        (target, _index): (Entity, usize),
        _storage: &'j mut Wiped,
    ) -> Self::Out {
        (**self).get(target).unwrap()
    }
}

impl<'j, 'a, Q, F, R> Join<'j, &'a Storage<R>> for &'_ Query<'_, '_, Q, F>
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
        storage: &'j mut &'a Storage<R>,
    ) -> Self::Out {
        (&storage.values[index], (**self).get(target).unwrap())
    }
}

/*impl<'j, Q, F, R> Join<'j, StorageWorldQueryMut<R>> for &'_ Query<'_, '_, Q, F>
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
}*/

impl<'j, Q, F> Join<'j, Wiped> for &'_ mut Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <Q as WorldQuery>::Item<'j>;

    fn matches(&self, target: Entity) -> bool {
        (**self).get(target).is_ok()
    }

    fn joined(
        &'j mut self,
        (target, _index): (Entity, usize),
        _storage: &'j mut Wiped,
    ) -> Self::Out {
        (**self).get_mut(target).unwrap()
    }
}

impl<'j, 'a, Q, F, R> Join<'j, &'a Storage<R>> for &'_ mut Query<'_, '_, Q, F>
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
        storage: &'j mut &'a Storage<R>,
    ) -> Self::Out {
        (&storage.values[index], (**self).get_mut(target).unwrap())
    }
}

/*impl<'j, Q, F, R> Join<'j, StorageWorldQueryMut<R>> for &'_ mut Query<'_, '_, Q, F>
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
}*/

pub trait DeclarativeJoin<R, Joins, EdgeComb, StorageComb, Item, const POS: usize>
where
    R: RelationQuerySet,
    Item: Joinable,
{
    type Joined<T: Relation>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;

    type TotalJoined<T: Relation>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;

    fn join<T: Relation>(self, item: Item) -> Self::Joined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;

    fn total_join<T: Relation>(self, item: Item) -> Self::TotalJoined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;
}

#[rustfmt::skip]
impl<'o, 'w, 's, Q, R, F, Joins, EdgeComb, StorageComb, Traversal, Item, const POS: usize>
    DeclarativeJoin<R, Joins, EdgeComb, StorageComb, Item, POS>
    for Ops<&'o Query<'w, 's, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb, Traversal>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    Item: Joinable,
{
    type Joined<T: Relation> = Ops<
        &'o Query<'w, 's, (Q, Relations<R>), F>,
        <Joins as TypedSet<R::Types, T, POS>>::Out<Item>,
        <EdgeComb as TypedSet<R::Types, T, POS>>::Out<Waive>,
        <StorageComb as TypedSet<R::Types, T, POS>>::Out<Wipe>,
        Traversal
    >
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;

    type TotalJoined<T: Relation> = Ops<
        &'o Query<'w, 's, (Q, Relations<R>), F>,
        <Joins as TypedSet<R::Types, T, POS>>::Out<Item>,
        <EdgeComb as TypedSet<R::Types, T, POS>>::Out<Waive>,
        <StorageComb as TypedSet<R::Types, T, POS>>::Out<Waive>,
        Traversal
    >
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;

    fn join<T: Relation>(self, item: Item) -> Self::Joined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            edge_comb: PhantomData,
            storage_comb: PhantomData,
            traversal: PhantomData,
        }
    }

    fn total_join<T: Relation>(self, item: Item) -> Self::TotalJoined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            edge_comb: PhantomData,
            storage_comb: PhantomData,
            traversal: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<'o, 'w, 's, Q, R, F, Joins, EdgeComb, StorageComb, Traversal, Item, const POS: usize>
    DeclarativeJoin<R, Joins, EdgeComb, StorageComb, Item, POS>
    for Ops<&'o mut Query<'w, 's, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb, Traversal>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    Item: Joinable,
{
    type Joined<T: Relation> = Ops<
        &'o mut Query<'w, 's, (Q, Relations<R>), F>,
        <Joins as TypedSet<R::Types, T, POS>>::Out<Item>,
        <EdgeComb as TypedSet<R::Types, T, POS>>::Out<Waive>,
        <StorageComb as TypedSet<R::Types, T, POS>>::Out<Wipe>,
        Traversal
    >
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;

    type TotalJoined<T: Relation> = Ops<
        &'o mut Query<'w, 's, (Q, Relations<R>), F>,
        <Joins as TypedSet<R::Types, T, POS>>::Out<Item>,
        <EdgeComb as TypedSet<R::Types, T, POS>>::Out<Waive>,
        <StorageComb as TypedSet<R::Types, T, POS>>::Out<Waive>,
        Traversal
    >
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>;

    fn join<T: Relation>(self, item: Item) -> Self::Joined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            edge_comb: PhantomData,
            storage_comb: PhantomData,
            traversal: PhantomData,
        }
    }

    fn total_join<T: Relation>(self, item: Item) -> Self::TotalJoined<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        EdgeComb: TypedSet<R::Types, T, POS>,
        StorageComb: TypedSet<R::Types, T, POS>
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            edge_comb: PhantomData,
            storage_comb: PhantomData,
            traversal: PhantomData,
        }
    }
}

type RelationItem<'a, R> =
    <<<R as RelationQuerySet>::WorldQuery as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>;

impl<E0, S0, J0, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations<(E0,), (S0,), (J0,)>
    for Ops<&'_ Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0,)>,
    for<'a> StorageComb: Comb<RelationItem<'a, R>>,
    for<'a> <StorageComb as Comb<RelationItem<'a, R>>>::Out: Flatten<(), Out = (S0,)>,
    Joins: Flatten<(), Out = (J0,)>,
    E0: Relation,
    J0: for<'j> Join<'j, S0>,
{
    type Components<'a> = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>;
    //type Joins<'a> = <J0 as Join<'a, S0>>::Out;

    fn for_each<Func, Ret>(self, mut func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'a> FnMut(&mut Self::Components<'a> /*, Self::Joins<'a>*/) -> Ret,
    {
        let (mut j0,) = self.joins.flatten(());
        for (mut componantes, relations) in self.query.iter() {
            let (mut s0,) = StorageComb::comb(relations.world_query).flatten(());
            for e0 in relations.edges.iter::<E0>() {
                if !j0.matches(e0.0) {
                    continue;
                }
                if let ControlFlow::Exit =
                    func(&mut componantes /*, j0.joined(e0, &mut s0)*/).into()
                {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
#[allow(unused_variables)]
mod compile_tests {
    use super::*;
    use crate::relation::ForEachPermutations;
    use crate::{component::TableStorage, prelude::*};

    #[derive(Component)]
    struct A;

    #[derive(Component)]
    struct B;

    impl Relation for B {
        type Storage = TableStorage;
    }

    #[derive(Component)]
    struct C;

    impl Relation for C {
        type Storage = TableStorage;
    }

    #[derive(Component)]
    struct D;

    #[derive(Component)]
    struct E;

    fn join_immut(rq: Query<(&A, Relations<&B>)>, d: Query<&D>, e: Query<&E>) {
        rq.ops().join::<B>(&e).for_each(|_| {});
    }

    /*fn join_left_mut(mut rq: Query<(&A, Relations<(&mut C, &mut B)>)>, d: Query<&D>, e: Query<&E>) {
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
    }*/
}

/*#[cfg(test)]
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
