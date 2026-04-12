pub mod commands;
pub mod history;
mod macros;
pub mod projection;
pub mod storage;

pub mod prelude {
    pub use crate::commands::AuthorCommands;
    pub use crate::history::History;
    pub use crate::projection::{ProjectRow, RowProjection, sync_row};
    pub use crate::storage::{Store, TypedId};
    pub use crate::{AuthoredModel, AuthorPlugin};
    pub use bevy_author_macros::{AuthoredId, AuthoredStores};
}

#[doc(hidden)]
pub use bevy::prelude::World;

use std::marker::PhantomData;

use bevy::prelude::*;

/// The one trait a consumer's model must implement.
pub trait AuthoredModel: Resource + Clone + Default + Send + Sync + 'static {
    /// Mark all stores dirty (for initial load or full re-sync).
    fn mark_all_dirty(&mut self) {}
    /// Diff against another model version; only mark changed items dirty.
    /// Skips unchanged stores via Arc pointer comparison (O(1)).
    fn mark_dirty_diff(&mut self, _other: &Self) {}
    /// Remove orphaned items whose parent no longer exists.
    fn cascade_remove(&mut self) {}
    /// Cascade dirty flags through FK dependencies.
    fn propagate_dirty(&mut self) {}
    /// Validate model consistency. Called in debug builds after each commit.
    fn validate(&self) {}
}

pub struct AuthorPlugin<M: AuthoredModel> {
    max_undo: Option<usize>,
    _marker: PhantomData<M>,
}

impl<M: AuthoredModel> AuthorPlugin<M> {
    pub fn new() -> Self {
        Self {
            max_undo: None,
            _marker: PhantomData,
        }
    }

    pub fn with_max_undo(mut self, max_undo: usize) -> Self {
        self.max_undo = Some(max_undo);
        self
    }
}

impl<M: AuthoredModel> Default for AuthorPlugin<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: AuthoredModel> Plugin for AuthorPlugin<M> {
    fn build(&self, app: &mut App) {
        let mut history = history::History::<M>::default();
        if let Some(max) = self.max_undo {
            history = history.with_max_undo(max);
        }
        app.init_resource::<M>()
            .insert_resource(history);
    }
}

pub use bevy_author_macros::{AuthoredId, AuthoredStores};
pub use commands::AuthorCommands;
pub use history::History;
pub use projection::{ProjectRow, RowProjection, sync_row};
pub use storage::{Store, TypedId};
