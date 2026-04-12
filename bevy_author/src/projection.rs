use std::collections::HashMap;

use bevy::prelude::*;

use crate::storage::{Store, TypedId};
use crate::AuthoredModel;

#[derive(Resource, Debug)]
pub struct RowProjection<Id: TypedId> {
    entities: HashMap<Id, Entity>,
}

impl<Id: TypedId> RowProjection<Id> {
    pub fn get(&self, id: &Id) -> Option<Entity> {
        self.entities.get(id).copied()
    }

    pub fn contains(&self, id: &Id) -> bool {
        self.entities.contains_key(id)
    }

    pub fn insert(&mut self, id: Id, entity: Entity) {
        self.entities.insert(id, entity);
    }
}

impl<Id: TypedId> Default for RowProjection<Id> {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }
}

/// Declares how a row type projects into ECS entities.
pub trait ProjectRow<M: AuthoredModel>: Clone + Send + Sync + 'static {
    type Id: TypedId;
    type Extra: Bundle;

    fn store(model: &M) -> &Store<Self::Id, Self>;
    fn store_mut(model: &mut M) -> &mut Store<Self::Id, Self>;

    /// Project a row into ECS components. The model is passed separately
    /// (it's removed from world during sync). Other `RowProjection` resources
    /// remain in world for cross-projection lookups.
    fn project(id: Self::Id, row: &Self, model: &M, world: &World) -> Option<Self::Extra>;
}

/// Sync one row type from Store to ECS. Call from an exclusive system.
///
/// Order calls so dependencies sync first (e.g. nodes before edges,
/// edges before attachments).
pub fn sync_row<M: AuthoredModel, R: ProjectRow<M>>(world: &mut World) {
    if !world.contains_resource::<RowProjection<R::Id>>() {
        world.insert_resource(RowProjection::<R::Id>::default());
    }

    let mut model = world.remove_resource::<M>().unwrap();
    let mut proj = world.remove_resource::<RowProjection<R::Id>>().unwrap();

    let dirty_ids = R::store_mut(&mut model).drain_dirty();
    let mut re_mark = Vec::new();

    for id in dirty_ids {
        let Some(row) = R::store(&model).get(id).cloned() else {
            if let Some(entity) = proj.entities.remove(&id) {
                // Entity may already be despawned (e.g. ChildOf cascade)
                if let Ok(e) = world.get_entity_mut(entity) {
                    e.despawn();
                }
            }
            continue;
        };

        match R::project(id, &row, &model, world) {
            Some(extra) => {
                let entity = *proj
                    .entities
                    .entry(id)
                    .or_insert_with(|| world.spawn_empty().id());
                world.entity_mut(entity).insert((id, extra));
            }
            None => re_mark.push(id),
        }
    }

    for id in re_mark {
        R::store_mut(&mut model).mark_dirty(id);
    }

    world.insert_resource(proj);
    world.insert_resource(model);
}
