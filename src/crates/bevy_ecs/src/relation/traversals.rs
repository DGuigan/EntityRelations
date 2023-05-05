use super::{joins::*, *};
use crate::{
    entity::Entity,
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::Query,
};
use std::collections::VecDeque;

pub struct BreadthFirst<R: Relation> {
    start: Entity,
    _phantom: PhantomData<R>,
}

impl<Query, Joins, EdgeComb, StorageComb> Ops<Query, Joins, EdgeComb, StorageComb> {
    pub fn breadth_first<R: Relation>(
        self,
        start: Entity,
    ) -> Ops<Query, Joins, EdgeComb, StorageComb, BreadthFirst<R>> {
        Ops {
            query: self.query,
            joins: self.joins,
            edge_comb: self.edge_comb,
            storage_comb: self.storage_comb,
            traversal: BreadthFirst {
                start,
                _phantom: PhantomData,
            },
        }
    }
}

impl<T, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<&'_ Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb, BreadthFirst<T>>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    T: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = ()>,
{
    type Components<'c> = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> = ();

    fn for_each<Func, Ret>(self, mut func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'r, 'c, 'i, 'a, 'j> FnMut(
            &'r mut Self::Components<'c>,
            Self::Joins<'i, 'a, 'j>,
        ) -> Ret,
    {
        let mut queue = VecDeque::from([self.traversal.start]);

        while let Some((mut components, relations)) =
            queue.pop_front().and_then(|e| self.query.get(e).ok())
        {
            for (e, _) in relations.edges.iter::<T>() {
                queue.push_back(e)
            }

            if let ControlFlow::Exit = func(&mut components, ()).into() {
                return;
            }
        }
    }
}

impl<T, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<
        &'_ mut Query<'_, '_, (Q, Relations<R>), F>,
        Joins,
        EdgeComb,
        StorageComb,
        BreadthFirst<T>,
    >
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    T: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = ()>,
{
    type Components<'c> = <Q as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> = ();

    fn for_each<Func, Ret>(self, mut func: Func)
    where
        Ret: Into<ControlFlow>,
        Func: for<'r, 'c, 'i, 'a, 'j> FnMut(
            &'r mut Self::Components<'c>,
            Self::Joins<'i, 'a, 'j>,
        ) -> Ret,
    {
        let mut queue = VecDeque::from([self.traversal.start]);

        while let Some((mut components, relations)) =
            queue.pop_front().and_then(|e| self.query.get_mut(e).ok())
        {
            for (e, _) in relations.edges.iter::<T>() {
                queue.push_back(e)
            }

            if let ControlFlow::Exit = func(&mut components, ()).into() {
                return;
            }
        }
    }
}

impl<T, E0, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<&'_ Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb, BreadthFirst<T>>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    T: Relation,
    E0: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0,)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity,), (bool,)>,
    for<'i> StorageComb: Comb<RelationItem<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItem<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out: Attach<
        'a,
        (usize,),
        <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,), (bool,)>>::Out,
    >,
{
    type Components<'c> = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize,),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,), (bool,)>>::Out,
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
        let mut queue = VecDeque::from([self.traversal.start]);

        while let Some((mut components, relations)) =
            queue.pop_front().and_then(|e| self.query.get(e).ok())
        {
            for (e, _) in relations.edges.iter::<T>() {
                queue.push_back(e)
            }

            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            'l0: for (e0, i0) in relations.edges.iter::<E0>() {
                let (m0,) = joins.contains((e0,));
                if !m0 {
                    continue 'l0;
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

impl<T, E0, E1, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<&'_ Query<'_, '_, (Q, Relations<R>), F>, Joins, EdgeComb, StorageComb, BreadthFirst<T>>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    T: Relation,
    E0: Relation,
    E1: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0, E1)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity, Entity), (bool, bool)>,
    for<'i> StorageComb: Comb<RelationItem<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItem<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out: Attach<
        'a,
        (usize, usize),
        <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity), (bool, bool)>>::Out,
    >,
{
    type Components<'c> = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItem<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize, usize),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity), (bool, bool)>>::Out,
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
        let mut queue = VecDeque::from([self.traversal.start]);

        while let Some((mut components, relations)) =
            queue.pop_front().and_then(|e| self.query.get(e).ok())
        {
            for (e, _) in relations.edges.iter::<T>() {
                queue.push_back(e)
            }

            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            'l0: for (e0, i0) in relations.edges.iter::<E0>() {
                'l1: for (e1, i1) in relations.edges.iter::<E1>() {
                    let (m0, m1) = joins.contains((e0, e1));
                    if !m0 {
                        continue 'l0;
                    }
                    if !m1 {
                        continue 'l1;
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

impl<T, E0, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<
        &'_ mut Query<'_, '_, (Q, Relations<R>), F>,
        Joins,
        EdgeComb,
        StorageComb,
        BreadthFirst<T>,
    >
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    T: Relation,
    E0: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0,)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity,), (bool,)>,
    for<'i> StorageComb: Comb<RelationItemMut<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItemMut<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out:
        Attach<
            'a,
            (usize,),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,), (bool,)>>::Out,
        >,
{
    type Components<'c> = <Q as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize,),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity,), (bool,)>>::Out,
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
        let mut queue = VecDeque::from([self.traversal.start]);

        while let Some((mut components, relations)) =
            queue.pop_front().and_then(|e| self.query.get_mut(e).ok())
        {
            for (e, _) in relations.edges.iter::<T>() {
                queue.push_back(e)
            }

            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            'l0: for (e0, i0) in relations.edges.iter::<E0>() {
                let (m0,) = joins.contains((e0,));
                if !m0 {
                    continue 'l0;
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

impl<T, E0, E1, Q, R, F, Joins, EdgeComb, StorageComb> ForEachPermutations
    for Ops<
        &'_ mut Query<'_, '_, (Q, Relations<R>), F>,
        Joins,
        EdgeComb,
        StorageComb,
        BreadthFirst<T>,
    >
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
    T: Relation,
    E0: Relation,
    E1: Relation,
    EdgeComb: Comb<R::Types>,
    <EdgeComb as Comb<R::Types>>::Out: Flatten<(), Out = (E0, E1)>,
    Joins: Flatten<()>,
    for<'j> <Joins as Flatten<()>>::Out: Joinable<'j, (Entity, Entity), (bool, bool)>,
    for<'i> StorageComb: Comb<RelationItemMut<'i, R>>,
    for<'i> <StorageComb as Comb<RelationItemMut<'i, R>>>::Out: Flatten<()>,
    for<'i, 'a, 'j> <<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out:
        Attach<
            'a,
            (usize, usize),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity), (bool, bool)>>::Out,
        >,
{
    type Components<'c> = <Q as WorldQuery>::Item<'c>;
    type Joins<'i, 'a, 'j> =
        <<<StorageComb as Comb<RelationItemMut<'i, R>>>::Out as Flatten<()>>::Out as Attach<
            'a,
            (usize, usize),
            <<Joins as Flatten<()>>::Out as Joinable<'j, (Entity, Entity), (bool, bool)>>::Out,
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
        let mut queue = VecDeque::from([self.traversal.start]);

        while let Some((mut components, relations)) =
            queue.pop_front().and_then(|e| self.query.get_mut(e).ok())
        {
            for (e, _) in relations.edges.iter::<T>() {
                queue.push_back(e)
            }

            let mut storage = StorageComb::comb(relations.world_query).flatten(());
            'l0: for (e0, i0) in relations.edges.iter::<E0>() {
                'l1: for (e1, i1) in relations.edges.iter::<E1>() {
                    let (m0, m1) = joins.contains((e0, e1));
                    if !m0 {
                        continue 'l0;
                    }
                    if !m1 {
                        continue 'l1;
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
    use crate::{component::TableStorage, prelude::*};

    #[derive(Component)]
    struct A;

    #[derive(Component)]
    struct B;

    #[derive(Component)]
    struct C;

    #[derive(Component)]
    struct D;

    impl Relation for B {
        type Storage = TableStorage;
    }

    impl Relation for C {
        type Storage = TableStorage;
    }

    fn no_join(left: Query<(&A, Relations<&B>)>, start: Entity) {
        left.ops().breadth_first::<B>(start).for_each(|a, _| {})
    }

    fn no_join_mut(mut left: Query<(&A, Relations<&B>)>, start: Entity) {
        left.ops_mut().breadth_first::<B>(start).for_each(|a, _| {})
    }

    fn breadth_first_immut(left: Query<(&A, Relations<(&B, &C)>)>, d: Query<&D>, entity: Entity) {
        left.ops()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, (d,)| {});

        left.ops()
            .total_join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, ((b, d),)| {});
    }

    fn breadth_first_mut(
        mut left: Query<(&A, Relations<(&mut B, &mut C)>)>,
        d: Query<&D>,
        entity: Entity,
    ) {
        left.ops_mut()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, (d,)| {});

        left.ops_mut()
            .total_join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, ((b, d),)| {});
    }

    fn breadth_first_immut_optional(
        left: Query<(&A, Relations<(Option<&B>, Option<&C>)>)>,
        d: Query<&D>,
        entity: Entity,
    ) {
        left.ops()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, (d,)| {});

        left.ops()
            .total_join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, ((b, d),)| {});
    }

    fn breadth_first_mut_optional(
        mut left: Query<(&A, Relations<(Option<&mut B>, Option<&mut C>)>)>,
        d: Query<&D>,
        entity: Entity,
    ) {
        left.ops_mut()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, (d,)| {});

        left.ops_mut()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|a, (d,)| {});
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::{self as bevy_ecs, component::TableStorage, prelude::*};

    fn run_system<Param, S: IntoSystem<(), (), Param>>(world: &mut World, system: S) {
        let mut schedule = Schedule::default();
        schedule.add_systems(system);
        schedule.run(world);
    }

    #[derive(Component)]
    struct Root;

    #[derive(Default, Debug, Component, PartialEq, Eq, Clone, Copy)]
    struct Pos {
        x: i32,
        y: i32,
    }

    struct Child;

    impl Relation for Child {
        type Storage = TableStorage;
    }

    fn setup(mut commands: Commands) {
        let ctrl = commands.spawn((Pos { x: 0, y: 5 }, Root)).id();
        let a = commands.spawn(Pos { x: 1, y: 5 }).id();
        let b = commands.spawn(Pos { x: 2, y: 5 }).id();
        let c = commands.spawn(Pos { x: 3, y: 5 }).id();
        let d = commands.spawn(Pos { x: 4, y: 5 }).id();

        commands.add(Set {
            foster: ctrl,
            target: a,
            relation: Child,
        });

        commands.add(Set {
            foster: ctrl,
            target: b,
            relation: Child,
        });

        commands.add(Set {
            foster: b,
            target: c,
            relation: Child,
        });

        commands.add(Set {
            foster: b,
            target: d,
            relation: Child,
        });
    }

    fn displace_all(
        root: Query<Entity, With<Root>>,
        mut positions: Query<(&mut Pos, Relations<Option<&Child>>)>,
    ) {
        positions
            .ops_mut()
            .breadth_first::<Child>(root.get_single().unwrap())
            .for_each(|pos, _| pos.x = 0);

        assert!(positions.iter().all(|(pos, _)| *pos == Pos { x: 0, y: 5 }));
    }

    #[test]
    fn propogation_test() {
        let mut world = World::new();
        run_system(&mut world, setup);
        run_system(&mut world, displace_all);
    }
}
