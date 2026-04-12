use std::collections::VecDeque;

use bevy::prelude::*;

use crate::AuthoredModel;

const DEFAULT_MAX_UNDO: usize = 200;

#[derive(Resource)]
pub struct History<M: AuthoredModel> {
    undo_stack: VecDeque<M>,
    redo_stack: Vec<M>,
    max_undo: usize,
    merging: bool,
}

impl<M: AuthoredModel> Default for History<M> {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
            max_undo: DEFAULT_MAX_UNDO,
            merging: false,
        }
    }
}

impl<M: AuthoredModel> History<M> {
    pub fn with_max_undo(mut self, max_undo: usize) -> Self {
        self.max_undo = max_undo;
        self
    }

    pub fn undo_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn save_snapshot(&mut self, snapshot: M) {
        if self.undo_stack.len() >= self.max_undo {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(snapshot);
        self.redo_stack.clear();
        self.merging = false;
    }

    /// Like `save_snapshot`, but consecutive merged commits collapse into one
    /// undo step. Use for drags: each frame calls `commit_merged`, and undo
    /// jumps back to the pre-drag state in one step.
    pub fn save_snapshot_merged(&mut self, snapshot: M) {
        if self.merging {
            // Already mid-merge — drop this snapshot, keep the original pre-drag state
        } else {
            // First frame of a merge sequence — save the pre-drag state
            if self.undo_stack.len() >= self.max_undo {
                self.undo_stack.pop_front();
            }
            self.undo_stack.push_back(snapshot);
            self.merging = true;
        }
        self.redo_stack.clear();
    }

    /// End a merge sequence. Call between separate drag gestures
    /// to ensure each is its own undo step.
    pub fn end_merge(&mut self) {
        self.merging = false;
    }

    /// Apply a mutation: cascade, propagate, validate. Returns the pre-mutation snapshot.
    pub(crate) fn apply_mutation(world: &mut World, f: impl FnOnce(&mut M)) -> M {
        let snapshot = world.resource::<M>().clone();
        {
            let mut model = world.resource_mut::<M>();
            f(&mut model);
            model.cascade_remove();
            model.propagate_dirty();
        }
        #[cfg(debug_assertions)]
        world.resource::<M>().validate();
        snapshot
    }

    /// Direct commit on `&mut World` (no deferred Commands).
    pub fn commit(world: &mut World, f: impl FnOnce(&mut M)) {
        let snapshot = Self::apply_mutation(world, f);
        world.resource_mut::<Self>().save_snapshot(snapshot);
    }

    /// Like `commit` but merges consecutive calls into one undo step.
    pub fn commit_merged(world: &mut World, f: impl FnOnce(&mut M)) {
        let snapshot = Self::apply_mutation(world, f);
        world.resource_mut::<Self>().save_snapshot_merged(snapshot);
    }

    pub fn undo(world: &mut World) -> bool {
        let previous = world.resource_mut::<Self>().undo_stack.pop_back();
        let Some(previous) = previous else {
            return false;
        };

        let current = world.resource::<M>().clone();
        *world.resource_mut::<M>() = previous;
        // Only dirty items that actually changed between the two versions
        world.resource_mut::<M>().mark_dirty_diff(&current);
        world.resource_mut::<M>().propagate_dirty();
        world.resource_mut::<Self>().redo_stack.push(current);
        world.resource_mut::<Self>().merging = false;
        true
    }

    pub fn redo(world: &mut World) -> bool {
        let next = world.resource_mut::<Self>().redo_stack.pop();
        let Some(next) = next else {
            return false;
        };

        let current = world.resource::<M>().clone();
        *world.resource_mut::<M>() = next;
        world.resource_mut::<M>().mark_dirty_diff(&current);
        world.resource_mut::<M>().propagate_dirty();
        world.resource_mut::<Self>().undo_stack.push_back(current);
        world.resource_mut::<Self>().merging = false;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Store;

    #[derive(Resource, Clone, Default)]
    struct TestModel {
        items: Store<ItemId, i32>,
    }

    impl AuthoredModel for TestModel {
        fn validate(&self) {}
    }

    #[derive(bevy::prelude::Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    struct ItemId(u64);

    impl crate::TypedId for ItemId {
        fn from_raw(raw: u64) -> Self { Self(raw) }
        fn to_raw(self) -> u64 { self.0 }
    }

    fn test_world() -> World {
        let mut world = World::new();
        world.insert_resource(TestModel::default());
        world.insert_resource(History::<TestModel>::default());
        world
    }

    fn commit(world: &mut World, f: impl FnOnce(&mut TestModel)) {
        let snapshot = world.resource::<TestModel>().clone();
        f(&mut *world.resource_mut::<TestModel>());
        world.resource_mut::<History<TestModel>>().save_snapshot(snapshot);
    }

    #[test]
    fn commit_undo_redo_round_trip() {
        let mut world = test_world();

        let mut id = ItemId(0);
        commit(&mut world, |model| {
            id = model.items.allocate_id();
            model.items.insert(id, 42);
        });

        assert_eq!(world.resource::<History<TestModel>>().undo_len(), 1);
        assert!(world.resource::<TestModel>().items.contains(id));

        assert!(History::<TestModel>::undo(&mut world));
        assert!(!world.resource::<TestModel>().items.contains(id));
        assert_eq!(world.resource::<History<TestModel>>().undo_len(), 0);
        assert_eq!(world.resource::<History<TestModel>>().redo_len(), 1);

        assert!(History::<TestModel>::redo(&mut world));
        assert!(world.resource::<TestModel>().items.contains(id));
        assert_eq!(world.resource::<History<TestModel>>().undo_len(), 1);
        assert_eq!(world.resource::<History<TestModel>>().redo_len(), 0);
    }

    #[test]
    fn commit_after_undo_clears_redo() {
        let mut world = test_world();

        commit(&mut world, |model| {
            let id = model.items.allocate_id();
            model.items.insert(id, 1);
        });
        assert!(History::<TestModel>::undo(&mut world));
        assert_eq!(world.resource::<History<TestModel>>().redo_len(), 1);

        commit(&mut world, |model| {
            let id = model.items.allocate_id();
            model.items.insert(id, 2);
        });

        assert_eq!(world.resource::<History<TestModel>>().undo_len(), 1);
        assert_eq!(world.resource::<History<TestModel>>().redo_len(), 0);
    }

    #[test]
    fn undo_redo_return_false_on_empty() {
        let mut world = test_world();
        assert!(!History::<TestModel>::undo(&mut world));
        assert!(!History::<TestModel>::redo(&mut world));
    }

    #[test]
    fn max_undo_limit() {
        let mut world = World::new();
        world.insert_resource(TestModel::default());
        world.insert_resource(History::<TestModel>::default().with_max_undo(3));

        for i in 0..5 {
            commit(&mut world, |model| {
                let id = model.items.allocate_id();
                model.items.insert(id, i);
            });
        }

        assert_eq!(world.resource::<History<TestModel>>().undo_len(), 3);
    }

    #[test]
    fn store_arc_shares_structure_on_clone() {
        let mut model = TestModel::default();
        let id = model.items.allocate_id();
        model.items.insert(id, 99);

        let snapshot = model.clone(); // O(1) — Arc refcount
        assert_eq!(snapshot.items.get(id), Some(&99));

        // Mutate original — snapshot is unaffected
        model.items.insert(id, 100);
        assert_eq!(model.items.get(id), Some(&100));
        assert_eq!(snapshot.items.get(id), Some(&99));
    }

    #[test]
    fn store_dirty_tracking() {
        let mut store = Store::<ItemId, i32>::default();
        assert!(!store.is_dirty());

        let id = store.allocate_id();
        store.insert(id, 1);
        assert!(store.is_dirty());
        assert!(store.is_id_dirty(id));

        let dirty = store.drain_dirty();
        assert_eq!(dirty, vec![id]);
        assert!(!store.is_dirty());
    }

    fn commit_merged(world: &mut World, f: impl FnOnce(&mut TestModel)) {
        let snapshot = world.resource::<TestModel>().clone();
        f(&mut *world.resource_mut::<TestModel>());
        world
            .resource_mut::<History<TestModel>>()
            .save_snapshot_merged(snapshot);
    }

    #[test]
    fn merged_commits_collapse_into_one_undo_step() {
        let mut world = test_world();

        let mut id = ItemId(0);
        commit(&mut world, |model| {
            id = model.items.allocate_id();
            model.items.insert(id, 0);
        });

        // Simulate a drag: 5 merged commits updating the same item
        for i in 1..=5 {
            commit_merged(&mut world, |model| {
                model.items.insert(id, i);
            });
        }

        // Should have 2 undo entries: pre-item + pre-drag
        assert_eq!(world.resource::<History<TestModel>>().undo_len(), 2);
        assert_eq!(world.resource::<TestModel>().items.get(id), Some(&5));

        // One undo reverts the entire drag back to value 0
        assert!(History::<TestModel>::undo(&mut world));
        assert_eq!(world.resource::<TestModel>().items.get(id), Some(&0));

        // Second undo removes the item entirely
        assert!(History::<TestModel>::undo(&mut world));
        assert!(!world.resource::<TestModel>().items.contains(id));
    }

    #[test]
    fn normal_commit_after_merged_resets_merge_state() {
        let mut world = test_world();

        let mut id = ItemId(0);
        commit(&mut world, |model| {
            id = model.items.allocate_id();
            model.items.insert(id, 0);
        });

        // Merged sequence
        commit_merged(&mut world, |model| { model.items.insert(id, 1); });
        commit_merged(&mut world, |model| { model.items.insert(id, 2); });

        // Normal commit breaks the merge
        commit(&mut world, |model| { model.items.insert(id, 100); });

        // 3 undo entries: pre-item, pre-drag, pre-100
        assert_eq!(world.resource::<History<TestModel>>().undo_len(), 3);

        // Undo the normal commit
        assert!(History::<TestModel>::undo(&mut world));
        assert_eq!(world.resource::<TestModel>().items.get(id), Some(&2));
    }

    #[test]
    fn clone_starts_with_clean_dirty() {
        let mut store = Store::<ItemId, i32>::default();
        let id = store.allocate_id();
        store.insert(id, 1);
        assert!(store.is_dirty());

        let cloned = store.clone();
        assert!(!cloned.is_dirty()); // snapshots start clean
    }
}
