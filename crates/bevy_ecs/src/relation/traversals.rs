use std::collections::VecDeque;

use crate::{
    change_detection::Mut,
    entity::Entity,
    query::{ReadOnlyWorldQuery, WorldQuery},
    relation::{joins::*, lenses::*, *},
    system::Query,
};

impl<'o, 'w, 's, Q, F, R, Joins, const POS: usize> BreadthFrist<R, FetchItem<'o, R>, POS>
    for Ops<&'o Query<'w, 's, (Q, Relations<R>), F>, Joins>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
{
    type Out<T> = Ops<
        &'o Query<'w, 's, (Q, Relations<R>), F>,
        Joins,
        BreadthFirstTraversal<FetchItem<'o, R>, LensOut<FetchItem<'o, R>, R, T, POS>>,
        OptimalQueue<T>,
    >
    where
        T: Relation,
        FetchItem<'o, R>: TypedGet<R::Types, T, POS>;

    fn breadth_first<T>(self, start: Entity) -> Self::Out<T>
    where
        T: Relation,
        FetchItem<'o, R>: TypedGet<R::Types, T, POS>,
        OptimalQueue<T>: Queue<LensOut<FetchItem<'o, R>, R, T, POS>>,
    {
        let lens = <FetchItem<'o, R> as TypedGet<R::Types, T, POS>>::getter;
        let queue = OptimalQueue::<T>::new(start);

        Ops {
            query: self.query,
            joins: self.joins,
            traversal: BreadthFirstTraversal { lens },
            queue,
        }
    }
}

impl<'o, 'w, 's, Q, F, R, Joins, const POS: usize> BreadthFrist<R, FetchItemMut<'o, R>, POS>
    for Ops<&'o mut Query<'w, 's, (Q, Relations<R>), F>, Joins>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
{
    type Out<T> = Ops<
        &'o mut Query<'w, 's, (Q, Relations<R>), F>,
        Joins,
        BreadthFirstTraversal<FetchItemMut<'o, R>, LensOut<FetchItemMut<'o, R>, R, T, POS>>,
        OptimalQueue<T>,
    >
    where
        T: Relation,
        FetchItemMut<'o, R>: TypedGet<R::Types, T, POS>;

    fn breadth_first<T>(self, start: Entity) -> Self::Out<T>
    where
        T: Relation,
        FetchItemMut<'o, R>: TypedGet<R::Types, T, POS>,
        OptimalQueue<T>: Queue<LensOut<FetchItemMut<'o, R>, R, T, POS>>,
    {
        let lens = <FetchItemMut<'o, R> as TypedGet<R::Types, T, POS>>::getter;
        let queue = OptimalQueue::<T>::new(start);

        Ops {
            query: self.query,
            joins: self.joins,
            traversal: BreadthFirstTraversal { lens },
            queue,
        }
    }
}

impl<'o, 'w, 's, Q, F, R, Joins, Edges, TraversalQueue> LendingForEach
    for Ops<
        &'o Query<'w, 's, (Q, Relations<R>), F>,
        Joins,
        BreadthFirstTraversal<FetchItem<'o, R>, Edges>,
        TraversalQueue,
    >
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
    TraversalQueue: Queue<Edges>,
    for<'e, 'j> Joins: Joined<'j, FetchItem<'e, R>>,
{
    type In<'e, 'j> = (
        <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'o>,
        <Joins as Joined<'j, FetchItem<'o, R>>>::Out,
    );

    fn for_each(mut self, mut func: impl FnMut(Self::In<'_, '_>)) {
        while let Some((q, r)) = self
            .queue
            .take_next()
            .and_then(|entity| self.query.get(entity).ok())
        {
            let edges = (self.traversal.lens)(&r.world_query);
            self.queue.enqueue_back(edges);
            func((q, self.joins.joined(r.world_query)));
        }
    }
}

impl<'o, 'w, 's, Q, F, R, Joins, Edges, TraversalQueue> LendingForEach
    for Ops<
        &'o mut Query<'w, 's, (Q, Relations<R>), F>,
        Joins,
        BreadthFirstTraversal<FetchItemMut<'o, R>, Edges>,
        TraversalQueue,
    >
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
    TraversalQueue: Queue<Edges>,
    for<'e, 'j> Joins: Joined<'j, FetchItemMut<'e, R>>,
{
    type In<'e, 'j> = (
        <Q as WorldQuery>::Item<'e>,
        <Joins as Joined<'j, FetchItemMut<'e, R>>>::Out,
    );

    fn for_each(mut self, mut func: impl FnMut(Self::In<'_, '_>)) {
        while let Some((q, r)) = self
            .queue
            .take_next()
            // Safety: Is always safe becase input cannot escape closure
            .and_then(|entity| unsafe { self.query.get_unchecked(entity).ok() })
        {
            let edges = (self.traversal.lens)(&r.world_query);
            self.queue.enqueue_back(edges);
            func((q, self.joins.joined(r.world_query)));
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
        type Arity = Multi<Self>;
        type Storage = TableStorage;
    }

    impl Relation for C {
        type Arity = Exclusive<Self>;
        type Storage = TableStorage;
    }

    #[derive(Component)]
    struct E;

    fn breadth_first_immut(rq: Query<(&A, Relations<(&C, &B)>)>, d: Query<&D>, entity: Entity) {
        rq.ops()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|_| {});

        rq.ops()
            .join::<C>(&d)
            .breadth_first::<B>(entity)
            .for_each(|_| {});
    }

    fn breadth_first_mut(
        mut rq: Query<(&A, Relations<(&mut C, &mut B)>)>,
        d: Query<&D>,
        entity: Entity,
    ) {
        rq.ops_mut()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|_| {});

        rq.ops_mut()
            .join::<C>(&d)
            .breadth_first::<B>(entity)
            .for_each(|_| {});
    }

    fn breadth_first_immut_optional(
        rq: Query<(&A, Relations<(Option<&C>, Option<&B>)>)>,
        d: Query<&D>,
        entity: Entity,
    ) {
        rq.ops()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|_| {});

        rq.ops()
            .join::<C>(&d)
            .breadth_first::<B>(entity)
            .for_each(|_| {});
    }

    fn breadth_first_mut_optional(
        mut rq: Query<(&A, Relations<(Option<&mut C>, Option<&mut B>)>)>,
        d: Query<&D>,
        entity: Entity,
    ) {
        rq.ops_mut()
            .join::<B>(&d)
            .breadth_first::<C>(entity)
            .for_each(|_| {});

        rq.ops_mut()
            .join::<C>(&d)
            .breadth_first::<B>(entity)
            .for_each(|_| {});
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
    struct Root;

    #[derive(Default, Debug, Component, PartialEq, Eq, Clone, Copy)]
    struct Pos {
        x: i32,
        y: i32,
    }

    struct Child;

    impl Relation for Child {
        type Arity = Multi<Self>;
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
            .for_each(|(mut pos, _)| pos.x = 0);

        assert!(positions.iter().all(|(pos, _)| *pos == Pos { x: 0, y: 5 }));
    }

    #[test]
    fn propogation_test() {
        let mut world = World::new();
        run_system(&mut world, setup);
        run_system(&mut world, displace_all);
    }
}
