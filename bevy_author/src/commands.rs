use std::marker::PhantomData;

use bevy::prelude::*;

use crate::history::History;
use crate::AuthoredModel;

struct Commit<M: AuthoredModel, F: FnOnce(&mut M) + Send + 'static> {
    f: F,
    _marker: PhantomData<M>,
}

impl<M: AuthoredModel, F: FnOnce(&mut M) + Send + 'static> Command for Commit<M, F> {
    fn apply(self, world: &mut World) {
        let snapshot = History::<M>::apply_mutation(world, self.f);
        world.resource_mut::<History<M>>().save_snapshot(snapshot);
    }
}

struct CommitMerged<M: AuthoredModel, F: FnOnce(&mut M) + Send + 'static> {
    f: F,
    _marker: PhantomData<M>,
}

impl<M: AuthoredModel, F: FnOnce(&mut M) + Send + 'static> Command for CommitMerged<M, F> {
    fn apply(self, world: &mut World) {
        let snapshot = History::<M>::apply_mutation(world, self.f);
        world.resource_mut::<History<M>>().save_snapshot_merged(snapshot);
    }
}

struct EndMergeCmd<M: AuthoredModel>(PhantomData<M>);

impl<M: AuthoredModel> Command for EndMergeCmd<M> {
    fn apply(self, world: &mut World) {
        world.resource_mut::<History<M>>().end_merge();
    }
}

struct UndoCmd<M: AuthoredModel>(PhantomData<M>);

impl<M: AuthoredModel> Command for UndoCmd<M> {
    fn apply(self, world: &mut World) {
        History::<M>::undo(world);
    }
}

struct RedoCmd<M: AuthoredModel>(PhantomData<M>);

impl<M: AuthoredModel> Command for RedoCmd<M> {
    fn apply(self, world: &mut World) {
        History::<M>::redo(world);
    }
}

pub trait AuthorCommands {
    fn commit<M: AuthoredModel>(&mut self, f: impl FnOnce(&mut M) + Send + 'static);
    /// Commit that merges with consecutive merged commits into one undo step.
    /// Use for drags: call every frame, undo reverts the entire drag at once.
    fn commit_merged<M: AuthoredModel>(&mut self, f: impl FnOnce(&mut M) + Send + 'static);
    /// End a merge sequence so the next commit_merged starts a new undo step.
    fn end_merge<M: AuthoredModel>(&mut self);
    fn undo<M: AuthoredModel>(&mut self);
    fn redo<M: AuthoredModel>(&mut self);
}

impl AuthorCommands for Commands<'_, '_> {
    fn commit<M: AuthoredModel>(&mut self, f: impl FnOnce(&mut M) + Send + 'static) {
        self.queue(Commit::<M, _> {
            f,
            _marker: PhantomData,
        });
    }

    fn commit_merged<M: AuthoredModel>(&mut self, f: impl FnOnce(&mut M) + Send + 'static) {
        self.queue(CommitMerged::<M, _> {
            f,
            _marker: PhantomData,
        });
    }

    fn end_merge<M: AuthoredModel>(&mut self) {
        self.queue(EndMergeCmd::<M>(PhantomData));
    }

    fn undo<M: AuthoredModel>(&mut self) {
        self.queue(UndoCmd::<M>(PhantomData));
    }

    fn redo<M: AuthoredModel>(&mut self) {
        self.queue(RedoCmd::<M>(PhantomData));
    }
}
