use crate::change_detection::Mut;
use crate::query::{ReadOnlyWorldQuery, WorldQuery};
use crate::system::Query;
use std::any::TypeId;

use super::{tuple_traits::*, *};

// T _ Q: Join
// T S Q: Full Join
// O _ _: Left Join
// O S _: Full left Join

pub struct Drop;
pub struct Keep;
pub struct Wrap;

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

trait Join<'q, 'r, Target, Storage> {
    type Out;
    fn matches(&self, target: &Target) -> bool;
    fn joined(&'q mut self, target: Target, storage: &'r mut Storage) -> Self::Out;
}

impl<'q, 'r, 'w, 's, Q, F> Join<'q, 'r, (Entity, usize), ()> for &'q Query<'w, 's, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'q>;

    fn matches(&self, target: &(Entity, usize)) -> bool {
        (**self).get(target.0).is_ok()
    }

    fn joined(&'q mut self, target: (Entity, usize), _storage: &'r mut ()) -> Self::Out {
        (**self).get(target.0).unwrap()
    }
}

impl<'q, 'r, 'w, 's, Q, F, R> Join<'q, 'r, (Entity, usize), StorageWorldQuery<R>>
    for &'q Query<'w, 's, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (&'r R, <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'q>);

    fn matches(&self, target: &(Entity, usize)) -> bool {
        (**self).get(target.0).is_ok()
    }

    fn joined(
        &'q mut self,
        target: (Entity, usize),
        storage: &'r mut StorageWorldQuery<R>,
    ) -> Self::Out {
        (
            &storage.storage.values[target.1],
            (**self).get(target.0).unwrap(),
        )
    }
}

impl<'q, 'r, 'w, 's, Q, F, R> Join<'q, 'r, (Entity, usize), StorageWorldQueryMut<R>>
    for &'q Query<'w, 's, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (
        &'r mut R,
        <<Q as WorldQuery>::ReadOnly as WorldQuery>::Item<'q>,
    );

    fn matches(&self, target: &(Entity, usize)) -> bool {
        (**self).get(target.0).is_ok()
    }

    fn joined(
        &'q mut self,
        target: (Entity, usize),
        storage: &'r mut StorageWorldQueryMut<R>,
    ) -> Self::Out {
        (
            &mut storage.storage.values[target.1],
            (**self).get(target.0).unwrap(),
        )
    }
}

impl<'q, 'r, 'w, 's, Q, F> Join<'q, 'r, (Entity, usize), ()> for &'q mut Query<'w, 's, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
{
    type Out = <Q as WorldQuery>::Item<'q>;

    fn matches(&self, target: &(Entity, usize)) -> bool {
        (**self).get(target.0).is_ok()
    }

    fn joined(&'q mut self, target: (Entity, usize), _storage: &'r mut ()) -> Self::Out {
        (**self).get_mut(target.0).unwrap()
    }
}

impl<'q, 'r, 'w, 's, Q, F, R> Join<'q, 'r, (Entity, usize), StorageWorldQuery<R>>
    for &'q mut Query<'w, 's, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (&'r R, <Q as WorldQuery>::Item<'q>);

    fn matches(&self, target: &(Entity, usize)) -> bool {
        (**self).get(target.0).is_ok()
    }

    fn joined(
        &'q mut self,
        target: (Entity, usize),
        storage: &'r mut StorageWorldQuery<R>,
    ) -> Self::Out {
        (
            &storage.storage.values[target.1],
            (**self).get_mut(target.0).unwrap(),
        )
    }
}

impl<'q, 'r, 'w, 's, Q, F, R> Join<'q, 'r, (Entity, usize), StorageWorldQueryMut<R>>
    for &'q mut Query<'w, 's, Q, F>
where
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: Relation,
{
    type Out = (&'r R, <Q as WorldQuery>::Item<'q>);

    fn matches(&self, target: &(Entity, usize)) -> bool {
        (**self).get(target.0).is_ok()
    }

    fn joined(
        &'q mut self,
        target: (Entity, usize),
        storage: &'r mut StorageWorldQueryMut<R>,
    ) -> Self::Out {
        (
            &mut storage.storage.values[target.1],
            (**self).get_mut(target.0).unwrap(),
        )
    }
}

pub trait DeclarativeJoin<R, Item> {
    type JoinOut<T: Relation>;
    fn join<T: Relation>(self, item: Item) -> Self::JoinOut<T>;
}

impl<'q, 'w, 's, Q, R, F, Traversal, Extractions, StorageFiltereds, Joins, Item>
    DeclarativeJoin<R, Item>
    for Ops<
        &'q Query<'w, 's, (Q, Relations<R>), F>,
        Traversal,
        Extractions,
        StorageFiltereds,
        Joins,
    >
where
    Extractions: Append,
    StorageFiltereds: Append,
    Joins: Append,
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
{
    type JoinOut<T: Relation> = Ops<
        &'q Query<'w, 's, (Q, Relations<R>), F>,
        Traversal,
        <Extractions as Append>::Out<T>,
        <StorageFiltereds as Append>::Out<Drop>,
        <Joins as Append>::Out<Item>,
    >;

    fn join<T: Relation>(self, item: Item) -> Self::JoinOut<T> {
        Ops {
            query: self.query,
            traversal: PhantomData,
            extractions: PhantomData,
            storage_filters: self.storage_filters.append(Drop),
            joins: self.joins.append(item),
        }
    }
}

impl<'q, 'w, 's, Q, R, F, Traversal, Extractions, StorageFiltereds, Joins, Item>
    DeclarativeJoin<R, Item>
    for Ops<
        &'q mut Query<'w, 's, (Q, Relations<R>), F>,
        Traversal,
        Extractions,
        StorageFiltereds,
        Joins,
    >
where
    Extractions: Append,
    StorageFiltereds: Append,
    Joins: Append,
    Q: 'static + WorldQuery,
    F: 'static + ReadOnlyWorldQuery,
    R: RelationQuerySet,
{
    type JoinOut<T: Relation> = Ops<
        &'q mut Query<'w, 's, (Q, Relations<R>), F>,
        Traversal,
        <Extractions as Append>::Out<T>,
        <StorageFiltereds as Append>::Out<Drop>,
        <Joins as Append>::Out<Item>,
    >;

    fn join<T: Relation>(self, item: Item) -> Self::JoinOut<T> {
        Ops {
            query: self.query,
            traversal: PhantomData,
            extractions: PhantomData,
            storage_filters: self.storage_filters.append(Drop),
            joins: self.joins.append(item),
        }
    }
}

/*impl<'j, 'o, 'w, 's, Q, F, R, Joins, Item, Traversal, Path, const POS: usize>
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
