use std::{any::TypeId, collections::VecDeque, slice::Iter};

use bevy_utils::{HashMap, HashSet};

use crate::{entity::Entity, world::World};

use super::Edges;

// Precedence: Most data latering operation is preferred.
// Smaller number -> Higher precedence
#[derive(Copy, Clone)]
pub enum DespawnPolicy {
    RecursiveDespawn = 0,
    RecursiveDelink = 1,
    Reparent = 2,
    Orphan = 3,
}

pub enum Operation {
    Despawn(Entity),
    Delink(Entity, TypeId, Entity), // parent, relation, child
    Reparent(Entity, TypeId),       // child, relation
}

impl Operation {
    fn apply(&self, world: &mut World, ascended_parents: &AscendedParents) {
        match self {
            Operation::Despawn(entity) => {
                world.despawn(*entity);
            }
            Operation::Delink(parent, relation, child) => {
                if let Some(mut parent_mut) = world.get_entity_mut(*parent) {
                    let mut edges = parent_mut
                        .get_mut::<Edges>()
                        .expect("Edge component should exist");
                    let Some(policy) = DespawnPolicy::iterator().find(| &policy | edges.targets[*policy as usize].contains_key(relation)) else { return };

                    if let Some(targets) = edges.targets[*policy as usize].get_mut(relation) {
                        targets.remove(child);
                    }
                }

                if let Some(mut child_mut) = world.get_entity_mut(*child) {
                    let mut edges = child_mut
                        .get_mut::<Edges>()
                        .expect("Edge component should exist");

                    if let Some(fosters) = edges.fosters.get_mut(relation) {
                        fosters.remove(parent);
                    }
                }
            }
            Operation::Reparent(child, relation) => {
                let Some(mut child_mut) = world.get_entity_mut(*child) else { return };
                let mut edges = child_mut
                    .get_mut::<Edges>()
                    .expect("Edge component should exist");

                let fosters = edges.fosters.entry(*relation).or_default();

                let parent = fosters
                    .iter()
                    .next()
                    .and_then(|parent| ascended_parents.get_valid_parent(*parent, *relation));

                fosters.clear();

                if let Some((parent, storage_index)) = parent {
                    fosters.insert(parent);

                    if let Some(mut parent_mut) = world.get_entity_mut(parent) {
                        parent_mut
                            .get_mut::<Edges>()
                            .expect("Edge component should exist")
                            .targets[DespawnPolicy::Reparent as usize]
                            .entry(*relation)
                            .or_default()
                            .insert(*child, storage_index);
                    }
                }
            }
        }
    }
}

struct AscendedParents {
    parents: HashMap<(Entity, TypeId), Option<(Entity, usize)>>,
}

impl AscendedParents {
    fn ascend_parent(&mut self, world: &World, entity: Entity) {
        let Some(entity_ref) = world.get_entity(entity) else { return; };
        let edges = entity_ref
            .get::<Edges>()
            .expect("Edge component should exist");

        for (relation, _children) in edges.targets[DespawnPolicy::Reparent as usize].iter() {
            let key = (entity, *relation);
            let Some(fosters) = edges.fosters.get(relation) else { return };

            match fosters.iter().next() {
                Some(parent) => {
                    if let Some(grandparent) = self.parents.get(&(*parent, *relation)) {
                        self.parents.insert(key, *grandparent);
                    } else {
                        let storage_index = world
                            .get_entity(*parent)
                            .expect("Fosters should not contain dangling entry")
                            .get::<Edges>()
                            .expect("Edge component should exist")
                            .targets[DespawnPolicy::Reparent as usize]
                            .get(relation)
                            .expect("Target should have relation entry")
                            .get(&entity)
                            .expect("Target should contain entity");

                        self.parents.insert(key, Some((*parent, *storage_index)));
                    }
                }
                None => {
                    self.parents.insert(key, None);
                }
            };
        }
    }

    fn get_valid_parent(&self, entity: Entity, relation: TypeId) -> Option<(Entity, usize)> {
        let mut closest_living = None;

        let mut current_key = (entity, relation);
        while let Some(parent) = self.parents.get(&current_key) {
            match parent {
                Some((parent, storage_index)) => {
                    closest_living = Some((*parent, *storage_index));
                    current_key.0 = *parent;
                }
                None => {
                    return None;
                }
            }
        }
        closest_living
    }
}

fn descend_despawn_relations(
    world: &World,
    operations: &mut Vec<Operation>,
    ascended_parents: &mut AscendedParents,
    root: Entity,
) {
    let mut to_visit: VecDeque<Entity> = VecDeque::from([root]);
    let mut staged_for_despawn: HashSet<Entity> = HashSet::new();

    while let Some(entity) = to_visit.pop_front() {
        if staged_for_despawn.contains(&entity) {
            continue;
        };

        if let Some(entity_ref) = world.get_entity(entity) {
            staged_for_despawn.insert(entity);
            let edges = entity_ref
                .get::<Edges>()
                .expect("Edge component should exist");

            ascended_parents.ascend_parent(world, root);

            for (relation, parents) in edges.fosters.iter() {
                for parent in parents.iter() {
                    if !staged_for_despawn.contains(parent) {
                        operations.push(Operation::Delink(*parent, *relation, entity));
                    }
                }
            }

            for (_relation, children) in
                edges.targets[DespawnPolicy::RecursiveDespawn as usize].iter()
            {
                for (child, _storage_id) in children.iter() {
                    operations.push(Operation::Despawn(*child));
                    to_visit.push_back(*child);
                }
            }
        }
    }

    for entity in staged_for_despawn.iter() {
        let edges = world
            .get_entity(*entity)
            .expect("Entity should exist")
            .get::<Edges>()
            .expect("Edge component should exist");

        for (relation, children) in edges.targets[DespawnPolicy::RecursiveDelink as usize].iter() {
            for (child, _storage_id) in children.iter() {
                if !staged_for_despawn.contains(child) {
                    descend_delink_relation(
                        world,
                        &staged_for_despawn,
                        operations,
                        *child,
                        *relation,
                    );
                }
            }
        }

        for (relation, children) in edges.targets[DespawnPolicy::Reparent as usize].iter() {
            for (child, _storage_id) in children.iter() {
                if !staged_for_despawn.contains(child) {
                    operations.push(Operation::Reparent(*child, *relation))
                }
            }
        }
    }
}

fn descend_delink_relation(
    world: &World,
    staged_for_despawn: &HashSet<Entity>,
    operations: &mut Vec<Operation>,
    root: Entity,
    relation: TypeId,
) {
    let mut to_visit: VecDeque<Entity> = VecDeque::from([root]);

    while let Some(entity) = to_visit.pop_front() {
        if staged_for_despawn.contains(&entity) {
            continue;
        }

        let edges = world
            .get_entity(entity)
            .expect("Entity should exist")
            .get::<Edges>()
            .expect("Edge component should exist");

        if let Some(children) =
            edges.targets[DespawnPolicy::RecursiveDelink as usize].get(&relation)
        {
            for (child, _storage_id) in children.iter() {
                operations.push(Operation::Delink(entity, relation, *child));
                to_visit.push_back(*child);
            }
        }
    }
}

fn apply_operations(
    world: &mut World,
    operations: &[Operation],
    ascended_parents: &AscendedParents,
) {
    for operation in operations.iter() {
        operation.apply(world, ascended_parents);
    }
}

impl DespawnPolicy {
    pub(crate) fn apply(&self, world: &mut World, initial_operation: Operation) {
        let mut operations = Vec::new(); // assume initial operation does not need to be applied
        let mut ascended_parents = AscendedParents {
            parents: HashMap::new(),
        };

        match initial_operation {
            Operation::Despawn(entity) => {
                descend_despawn_relations(world, &mut operations, &mut ascended_parents, entity);
            }
            Operation::Delink(_parent, relation, child) => {
                match self {
                    DespawnPolicy::RecursiveDespawn => {
                        operations.push(Operation::Despawn(child));
                        descend_despawn_relations(
                            world,
                            &mut operations,
                            &mut ascended_parents,
                            child,
                        );
                    }
                    DespawnPolicy::RecursiveDelink => {
                        descend_delink_relation(
                            world,
                            &HashSet::default(),
                            &mut operations,
                            child,
                            relation,
                        );
                    }
                    DespawnPolicy::Reparent => {
                        operations.push(Operation::Reparent(child, relation));
                    }
                    DespawnPolicy::Orphan => (),
                };
            }
            Operation::Reparent(child, relation) => {
                let Some(child_ref) = world.get_entity(child) else { return };
                let edges = child_ref
                    .get::<Edges>()
                    .expect("Edge component should exist");
                let Some(fosters) = edges.fosters.get(&relation) else { return };

                if let Some(parent) = fosters.iter().next() {
                    operations.push(Operation::Delink(*parent, relation, child));
                }
            }
        }

        apply_operations(world, &operations, &ascended_parents);
    }

    fn iterator() -> Iter<'static, DespawnPolicy> {
        [
            DespawnPolicy::RecursiveDespawn,
            DespawnPolicy::RecursiveDelink,
            DespawnPolicy::Reparent,
            DespawnPolicy::Orphan,
        ]
        .iter()
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::{self as bevy_ecs, component::TableStorage, prelude::*, relation::*};

    fn run_system<Param, S: IntoSystem<(), (), Param>>(world: &mut World, system: S) {
        let mut schedule = Schedule::default();
        schedule.add_systems(system);
        schedule.run(world);
    }

    #[derive(Component)]
    struct Root;
    #[derive(Component)]
    struct ShouldDespawn;
    #[derive(Component)]
    struct ShouldDelink;
    #[derive(Component)]
    struct ShouldReparent;
    #[derive(Component)]
    struct TriggerPoint;

    struct DespawnRelation;
    struct DelinkRelation;
    struct ReparentRelation;

    impl Relation for DespawnRelation {
        type Storage = TableStorage;
        const EXCLUSIVE: bool = true;
        const DESPAWN_POLICY: DespawnPolicy = DespawnPolicy::RecursiveDespawn;
    }

    impl Relation for DelinkRelation {
        type Storage = TableStorage;
        const DESPAWN_POLICY: DespawnPolicy = DespawnPolicy::RecursiveDelink;
    }

    impl Relation for ReparentRelation {
        type Storage = TableStorage;
        const DESPAWN_POLICY: DespawnPolicy = DespawnPolicy::Reparent;
    }

    fn setup(mut commands: Commands) {
        let root = commands.spawn(Root).id();

        let a = commands.spawn(TriggerPoint).id();

        let b = commands.spawn(ShouldDespawn).id();
        let c = commands.spawn(ShouldDespawn).id();

        let d = commands.spawn(ShouldDelink).id();
        let e = commands.spawn(ShouldDelink).id();
        let f = commands.spawn(()).id();

        let g = commands.spawn(ShouldReparent).id();

        commands.add(Set {
            foster: root,
            target: a,
            relation: ReparentRelation,
        });

        commands.add(Set {
            foster: root,
            target: a,
            relation: DespawnRelation,
        });

        commands.add(Set {
            foster: a,
            target: b,
            relation: DespawnRelation,
        });

        commands.add(Set {
            foster: b,
            target: c,
            relation: DespawnRelation,
        });

        commands.add(Set {
            foster: a,
            target: d,
            relation: DelinkRelation,
        });

        commands.add(Set {
            foster: d,
            target: e,
            relation: DelinkRelation,
        });

        commands.add(Set {
            foster: e,
            target: f,
            relation: DelinkRelation,
        });

        commands.add(Set {
            foster: a,
            target: g,
            relation: ReparentRelation,
        });
    }

    fn trigger_policies_via_despawn(mut commands: Commands, q: Query<Entity, With<TriggerPoint>>) {
        let entity = q.single();

        commands.add(CheckedDespawn { entity });
    }

    fn trigger_policies_via_delink(
        mut commands: Commands,
        rq: Query<Entity, With<Root>>,
        tq: Query<Entity, With<TriggerPoint>>,
    ) {
        let root = rq.single();

        let child = tq.single();

        commands.add(UnSet {
            foster: root,
            target: child,
            _phantom: PhantomData::<DespawnRelation>,
        });
    }

    fn trigger_policies_via_exclusive(mut commands: Commands, rq: Query<Entity, With<Root>>) {
        let root = rq.single();
        let new_child = commands.spawn(()).id();

        commands.add(Set {
            foster: root,
            target: new_child,
            relation: DespawnRelation,
        });
    }

    fn verify_policy(
        root: Query<Entity, With<Root>>,
        should_despawn: Query<Entity, With<ShouldDespawn>>,
        should_delink: Query<&Edges, With<ShouldDelink>>,
        should_reparent: Query<&Edges, With<ShouldReparent>>,
    ) {
        assert!(should_despawn.iter().len() == 0);

        for edges in should_delink.iter() {
            let targets = edges.targets[DespawnPolicy::RecursiveDelink as usize]
                .get(&TypeId::of::<Storage<DelinkRelation>>())
                .unwrap();
            assert!(targets.is_empty());
        }

        let root = root.single();
        let parent = should_reparent
            .single()
            .fosters
            .get(&TypeId::of::<Storage<ReparentRelation>>())
            .expect("Entity should have relation to foster")
            .iter()
            .next()
            .expect("Entity should not be orphaned");
        assert!(*parent == root);
    }

    #[test]
    fn policy_via_despawn() {
        let mut world = World::new();
        run_system(&mut world, setup);
        run_system(&mut world, trigger_policies_via_despawn);
        run_system(&mut world, verify_policy);
    }

    #[test]
    fn policy_via_delink() {
        let mut world = World::new();
        run_system(&mut world, setup);
        run_system(&mut world, trigger_policies_via_delink);
        run_system(&mut world, verify_policy);
    }

    #[test]
    fn policy_via_exclusive() {
        let mut world = World::new();
        run_system(&mut world, setup);
        run_system(&mut world, trigger_policies_via_exclusive);
        run_system(&mut world, verify_policy);
    }
}
