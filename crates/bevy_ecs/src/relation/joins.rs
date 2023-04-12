use crate::change_detection::Mut;
use crate::query::{ReadOnlyWorldQuery, WorldQuery};
use crate::system::Query;
use std::any::TypeId;

use super::{tuple_traits::*, *};

// T _ Q: Join
// T S Q: Full Join
// O _ _: Left Join
// O S _: Full left Join

pub trait Joinable<'a, Keys> {
    type Out;

    fn contains(&self, keys: Keys) -> bool;
    fn get(&'a mut self, keys: Keys) -> Self::Out;
}

impl<'a, Q, F> Joinable<'a, Entity> for &'_ Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'a>;

    fn contains(&self, entity: Entity) -> bool {
        (**self).contains(entity)
    }

    fn get(&'a mut self, entity: Entity) -> Self::Out {
        (**self).get(entity).unwrap()
    }
}

impl<'a, Q, F> Joinable<'a, Entity> for &'_ mut Query<'_, '_, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <Q as WorldQuery>::Item<'a>;

    fn contains(&self, entity: Entity) -> bool {
        (**self).contains(entity)
    }

    fn get(&'a mut self, entity: Entity) -> Self::Out {
        (**self).get_mut(entity).unwrap()
    }
}

impl<'a, K0, P0> Joinable<'a, (K0,)> for (P0,)
where
    P0: Joinable<'a, K0>,
{
    type Out = (P0::Out,);

    fn contains(&self, (k0,): (K0,)) -> bool {
        self.0.contains(k0)
    }

    fn get(&'a mut self, (k0,): (K0,)) -> Self::Out {
        (self.0.get(k0),)
    }
}

impl<'a, K0, K1, P0, P1> Joinable<'a, (K0, K1)> for (P0, P1)
where
    P0: Joinable<'a, K0>,
    P1: Joinable<'a, K1>,
{
    type Out = (P0::Out, P1::Out);

    fn contains(&self, (k0, k1): (K0, K1)) -> bool {
        self.0.contains(k0) && self.1.contains(k1)
    }

    fn get(&'a mut self, (k0, k1): (K0, K1)) -> Self::Out {
        (self.0.get(k0), self.1.get(k1))
    }
}

pub trait Attach<'a, Keys, Items> {
    type Out;
    fn attach(&'a mut self, keys: Keys, items: Items) -> Self::Out;
}

impl<'a, Items> Attach<'a, usize, Items> for Wiped {
    type Out = Items;

    fn attach(&'a mut self, _index: usize, items: Items) -> Self::Out {
        items
    }
}

impl<'a, Items, R: Relation> Attach<'a, usize, Items> for StorageWorldQuery<R> {
    type Out = (&'a R, Items);
    fn attach(&'a mut self, index: usize, items: Items) -> Self::Out {
        (self.storage.values.get(index).unwrap(), items)
    }
}

impl<'a, Items, R: Relation> Attach<'a, usize, Items> for StorageWorldQueryMut<R> {
    type Out = (&'a mut R, Items);

    fn attach(&'a mut self, index: usize, items: Items) -> Self::Out {
        (self.storage.values.get_mut(index).unwrap(), items)
    }
}

impl<'a, K0, I0, P0> Attach<'a, (K0,), (I0,)> for (P0,)
where
    P0: Attach<'a, K0, I0>,
{
    type Out = (P0::Out,);

    fn attach(&'a mut self, (k0,): (K0,), (i0,): (I0,)) -> Self::Out {
        (self.0.attach(k0, i0),)
    }
}

impl<'a, K0, K1, I0, I1, P0, P1> Attach<'a, (K0, K1), (I0, I1)> for (P0, P1)
where
    P0: Attach<'a, K0, I0>,
    P1: Attach<'a, K1, I1>,
{
    type Out = (P0::Out, P1::Out);

    fn attach(&'a mut self, (k0, k1): (K0, K1), (i0, i1): (I0, I1)) -> Self::Out {
        (self.0.attach(k0, i0), self.1.attach(k1, i1))
    }
}

pub trait DeclarativeJoin<R, Joins, EdgeComb, StorageComb, Item, const POS: usize>
where
    R: RelationQuerySet,
    Item: for<'a> Joinable<'a, Entity>,
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
    Item: for<'a> Joinable<'a, Entity>,
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
    Item: for<'a> Joinable<'a, Entity>,
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

#[rustfmt::skip]
type RelationItem<'a, R> = <<<R as RelationQuerySet>
    ::WorldQuery as WorldQuery>
    ::ReadOnly as WorldQuery>
    ::Item<'a>;

#[rustfmt::skip]
type RelationItemMut<'a, R> = <<R as RelationQuerySet>
    ::WorldQuery as WorldQuery>
    ::Item<'a>;

impl<E0, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<&'_ Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    E0: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0,)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity,)>,
    for<'i> StorageComb: Comb<RelationItem<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItem<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out:
        Attach<'a, (usize,), <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,)>>::Out>,
{
    type Components<'c> = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize,),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,)>>::Out,
        >>::Out;

    fn for_each<Func, Ret>(self, mut func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'r, 'c, 'i, 'a, 'j> FnMut(
            &'r mut Self::Components<'c>,
            Self::Joins<'i, 'a, 'j>,
        ) -> Ret,
    {
        let mut joins = self.joins.flatten(());
        for (mut components, relations) in self.query.iter() {
            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            for (e0, i0) in relations.edges.iter::<E0>() {
                if !joins.contains((e0,)) {
                    continue;
                }
                if let ControlFlow::Exit =
                    func(&mut components, storage.attach((i0,), joins.get((e0,)))).into()
                {
                    return;
                }
            }
        }
    }
}

impl<E0, E1, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<&'_ Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    E0: Relation,
    E1: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0, E1)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity, Entity)>,
    for<'i> StorageComb: Comb<RelationItem<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItem<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out: Attach<
        'a,
        (usize, usize),
        <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity)>>::Out,
    >,
{
    type Components<'c> = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize, usize),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity)>>::Out,
        >>::Out;

    fn for_each<Func, Ret>(self, mut func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'r, 'c, 'i, 'a, 'j> FnMut(
            &'r mut Self::Components<'c>,
            Self::Joins<'i, 'a, 'j>,
        ) -> Ret,
    {
        let mut joins = self.joins.flatten(());
        for (mut components, relations) in self.query.iter() {
            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            for (e0, i0) in relations.edges.iter::<E0>() {
                for (e1, i1) in relations.edges.iter::<E1>() {
                    if !joins.contains((e0, e1)) {
                        continue;
                    }
                    if let ControlFlow::Exit = func(
                        &mut components,
                        storage.attach((i0, i1), joins.get((e0, e1))),
                    )
                    .into()
                    {
                        return;
                    }
                }
            }
        }
    }
}

impl<E0, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<&'_ mut Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    E0: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0,)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity,)>,
    for<'i> StorageComb: Comb<RelationItemMut<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItemMut<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out:
        Attach<'a, (usize,), <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,)>>::Out>,
{
    type Components<'c> = <Q as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize,),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,)>>::Out,
        >>::Out;

    fn for_each<Func, Ret>(self, mut func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'r, 'c, 'i, 'a, 'j> FnMut(
            &'r mut Self::Components<'c>,
            Self::Joins<'i, 'a, 'j>,
        ) -> Ret,
    {
        let mut joins = self.joins.flatten(());
        for (mut components, relations) in self.query.iter_mut() {
            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            for (e0, i0) in relations.edges.iter::<E0>() {
                if !joins.contains((e0,)) {
                    continue;
                }
                if let ControlFlow::Exit =
                    func(&mut components, storage.attach((i0,), joins.get((e0,)))).into()
                {
                    return;
                }
            }
        }
    }
}

impl<E0, E1, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<&'_ mut Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    E0: Relation,
    E1: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0, E1)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity, Entity)>,
    for<'i> StorageComb: Comb<RelationItemMut<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItemMut<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out:
        Attach<
            'a,
            (usize, usize),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity)>>::Out,
        >,
{
    type Components<'c> = <Q as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize, usize),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity)>>::Out,
        >>::Out;

    fn for_each<Func, Ret>(self, mut func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'r, 'c, 'i, 'a, 'j> FnMut(
            &'r mut Self::Components<'c>,
            Self::Joins<'i, 'a, 'j>,
        ) -> Ret,
    {
        let mut joins = self.joins.flatten(());
        for (mut components, relations) in self.query.iter_mut() {
            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            for (e0, i0) in relations.edges.iter::<E0>() {
                for (e1, i1) in relations.edges.iter::<E1>() {
                    if !joins.contains((e0, e1)) {
                        continue;
                    }
                    if let ControlFlow::Exit = func(
                        &mut components,
                        storage.attach((i0, i1), joins.get((e0, e1))),
                    )
                    .into()
                    {
                        return;
                    }
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

    fn join_immut(left: Query<(&A, Relations<(&B, &C)>)>, d: Query<&D>, e: Query<&E>) {
        left.ops()
            .join::<B>(&d)
            .join::<C>(&e)
            .for_each(|a, (d, e)| {});
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
