use crate::change_detection::Mut;
use crate::query::{ReadOnlyWorldQuery, WorldQuery};
use crate::system::Query;

use super::{type_magic::*, *};

// TODO:
// ---- All tuple Joined impl
// ---- Isolate unsafe (lfetime transmute) to LendingForEach to make it easier to check correctness
//      + Joined<'a>: WithLifetime<'a>
//      + unsafe trait WithLifetime<'a> {
//          type Out: 'a;
//          unsafe transmute(self) -> Self::Out
//      }
//

// FUTURE WORK (Declarative joins 2.0):
// ---- Lending iterators (whenever those are possible)
//
// ---- Support different types of joins (join is currently implicitly "left join").
//      There are use cases for "inner joins" and possibly "outer joins" too.
//
// ---- Add typed builder for joins that produces a variable sized tuple.
//          + Reduces tuple nesting.
//          + Removes the need for a `_` for ZST + unjoined traversals.
//          + Allows for more joins than there are relations more ergonomically.
//          + More powerful in conjunction with more join types.
//          + Order of joins specifies order of arguments.
//          + Illustrated:
//              fn sys(
//                  mut rq: Query<(&A, &B, Relations<(&C, &mut D, &E, &X)>)>,
//                  f: Query<&F>,
//                  g: Query<&G>,
//                  h: Query<&H>,
//                  mut i: Query<&mut I>,
//                  start: Entity
//              ) {
//                  rq.ops_mut()
//                      .get::<D>()
//                      .inner_join::<C>(&f)
//                      .left_join::<C>((&g, &h))
//                      .outer_join::<E>(&mut i)
//                      .breadth_first::<X>(start)
//                      .for_each(|(As, Bs), (Ds, CFs, (CGs, CHs), EIs)| {
//
//                      });
//              }
//
// ---- Add kinded entites and kinded entities as targets.
//      This is a separate feature that is not entierly orthogonal to relations.
//      Both features complement for_eachother in surprising ways but specifically for relations:
//          + Can be used to create indices.
//          + Allows for more parallelism because kinded entities change the type signature hence
//          making more queries disjoint.
//

pub trait Filtered<Items> {}

pub trait Joined<'j, Items> {
    type Out: Iterator;
    fn joined(&'j mut self, items: Items) -> Self::Out;
}

/*pub trait DeclarativeJoin<'j, R, Joins, Item, const POS: usize>
where
    R: RelationSet,
{
    type Out<T>
    where
        Joins: TypedSet<R::Types, T, POS>,
        T: 'j;

    fn join<T>(self, item: Item) -> Self::Out<T>
    where
        T: Relation + 'j,
        Joins: TypedSet<R::Types, T, POS>,
        Joins::Out<Item>: Joined<'j, R::WorldQuery>;
}

impl<'j, 'o, 'w, 's, Q, F, R, Joins, Item, Traversal, Path, const POS: usize>
    DeclarativeJoin<'j, R, Joins, Item, POS>
    for Ops<&'o Query<'w, 's, (Q, Relations<R>), F>, Joins, Traversal, Path>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
{
    type Out<T> = Ops<&'o Query<'w, 's, (Q, Relations<R>), F>, Joins::Out<Item>, Traversal, Path>
    where
        Joins: TypedSet<R::Types, T, POS>,
        T: 'j;

    fn join<T>(self, item: Item) -> Self::Out<T>
    where
        T: Relation + 'j,
        Joins: TypedSet<R::Types, T, POS>,
        Joins::Out<Item>: Joined<'j, R::WorldQuery>,
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            traversal: self.traversal,
            queue: self.queue,
        }
    }
}

impl<'j, 'o, 'w, 's, Q, F, R, Joins, Item, Traversal, Path, const POS: usize>
    DeclarativeJoin<'j, R, Joins, Item, POS>
    for Ops<&'o mut Query<'w, 's, (Q, Relations<R>), F>, Joins, Traversal, Path>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
{
    type Out<T> = Ops<&'o mut Query<'w, 's, (Q, Relations<R>), F>, Joins::Out<Item>, Traversal, Path>
    where
        Joins: TypedSet<R::Types, T, POS>,
        T: 'j;

    fn join<T>(self, item: Item) -> Self::Out<T>
    where
        T: Relation + 'j,
        Joins: TypedSet<R::Types, T, POS>,
        Joins::Out<Item>: Joined<'j, R::WorldQuery>,
    {
        Ops {
            query: self.query,
            joins: self.joins.set(item),
            traversal: self.traversal,
            queue: self.queue,
        }
    }
}

impl<'o, 'w, 's, Q, F, R, Joins> LendingForEach
    for Ops<&'o Query<'w, 's, (Q, Relations<R>), F>, Joins>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
    for<'e, 'j> Joins: Joined<'j, FetchItem<'e, R>>,
{
    type In<'e, 'j> = (
        <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'e>,
        <Joins as Joined<'j, FetchItem<'e, R>>>::Out,
    );

    fn for_each(mut self, mut func: impl FnMut(Self::In<'_, '_>)) {
        for (q, r) in self.query.iter() {
            func((q, self.joins.joined(r.world_query)))
        }
    }
}
impl<'o, 'w, 's, Q, F, R, Joins> LendingForEach
    for Ops<&'o mut Query<'w, 's, (Q, Relations<R>), F>, Joins>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationSet + Send + Sync,
    for<'e, 'j> Joins: Joined<'j, FetchItemMut<'e, R>>,
{
    type In<'e, 'j> = (
        <Q as WorldQuery>::Item<'e>,
        <Joins as Joined<'j, FetchItemMut<'e, R>>>::Out,
    );

    fn for_each(mut self, mut func: impl FnMut(Self::In<'_, '_>)) {
        for (q, r) in self.query.iter_mut() {
            func((q, self.joins.joined(r.world_query)))
        }
    }
}

impl<T> LendingForEach for Option<T>
where
    T: LendingForEach,
{
    type In<'e, 'j> = T::In<'e, 'j>;

    fn for_each(self, func: impl FnMut(Self::In<'_, '_>)) {
        if let Some(t) = self {
            t.for_each(func)
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
