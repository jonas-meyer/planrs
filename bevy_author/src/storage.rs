use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

pub trait TypedId:
    bevy::prelude::Component + Copy + Eq + Hash + std::fmt::Debug + Send + Sync + 'static
{
    fn from_raw(raw: u64) -> Self;
    fn to_raw(self) -> u64;
}

#[derive(Debug)]
pub struct Store<Id, Row> {
    next_raw: u64,
    items: Arc<HashMap<Id, Row>>,
    dirty: HashSet<Id>,
}

impl<Id, Row> Clone for Store<Id, Row> {
    fn clone(&self) -> Self {
        Self {
            next_raw: self.next_raw,
            items: Arc::clone(&self.items),
            dirty: HashSet::new(), // snapshots start clean
        }
    }
}

impl<Id, Row> Default for Store<Id, Row> {
    fn default() -> Self {
        Self {
            next_raw: 0,
            items: Arc::new(HashMap::default()),
            dirty: HashSet::new(),
        }
    }
}

impl<Id: TypedId, Row: Clone> Store<Id, Row> {
    pub fn allocate_id(&mut self) -> Id {
        let id = Id::from_raw(self.next_raw);
        self.next_raw += 1;
        id
    }

    pub fn get(&self, id: Id) -> Option<&Row> {
        self.items.get(&id)
    }

    pub fn insert(&mut self, id: Id, row: Row) -> Option<Row> {
        self.next_raw = self.next_raw.max(id.to_raw() + 1);
        self.dirty.insert(id);
        Arc::make_mut(&mut self.items).insert(id, row)
    }

    pub fn remove(&mut self, id: Id) -> Option<Row> {
        self.dirty.insert(id);
        Arc::make_mut(&mut self.items).remove(&id)
    }

    pub fn contains(&self, id: Id) -> bool {
        self.items.contains_key(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &Row)> {
        self.items.iter().map(|(id, row)| (*id, row))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn drain_dirty(&mut self) -> Vec<Id> {
        self.dirty.drain().collect()
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    pub fn is_id_dirty(&self, id: Id) -> bool {
        self.dirty.contains(&id)
    }

    pub fn mark_dirty(&mut self, id: Id) {
        self.dirty.insert(id);
    }

    /// Mark all items matching a predicate as dirty.
    pub fn mark_dirty_where(&mut self, predicate: impl Fn(&Row) -> bool) {
        for (id, row) in self.items.iter() {
            if predicate(row) {
                self.dirty.insert(*id);
            }
        }
    }

    /// Mark only the items that differ from `other`. Skips entirely if
    /// both stores share the same Arc (O(1) check).
    pub fn mark_dirty_diff(&mut self, other: &Self)
    where
        Row: PartialEq,
    {
        if std::sync::Arc::ptr_eq(&self.items, &other.items) {
            return;
        }
        // Items in self that are new or modified
        for (id, row) in self.items.iter() {
            match other.items.get(id) {
                None => { self.dirty.insert(*id); }
                Some(old) if row != old => { self.dirty.insert(*id); }
                _ => {}
            }
        }
        // Items in other that were deleted (need despawn)
        for (id, _) in other.items.iter() {
            if !self.items.contains_key(id) {
                self.dirty.insert(*id);
            }
        }
    }

    pub fn mark_all_dirty(&mut self) {
        for (id, _) in self.items.iter() {
            self.dirty.insert(*id);
        }
    }

    /// Remove all items matching a predicate. Returns the removed IDs.
    pub fn remove_where(&mut self, predicate: impl Fn(&Row) -> bool) -> Vec<Id> {
        let to_remove: Vec<Id> = self
            .items
            .iter()
            .filter(|(_, row)| predicate(row))
            .map(|(id, _)| *id)
            .collect();
        for &id in &to_remove {
            self.remove(id);
        }
        to_remove
    }

    /// Update all items matching a predicate by applying a mutation function.
    pub fn update_where(
        &mut self,
        predicate: impl Fn(&Row) -> bool,
        update: impl Fn(&mut Row),
    ) {
        let to_update: Vec<Id> = self
            .items
            .iter()
            .filter(|(_, row)| predicate(row))
            .map(|(id, _)| *id)
            .collect();
        for id in to_update {
            if let Some(row) = self.get(id).cloned() {
                let mut updated = row;
                update(&mut updated);
                self.insert(id, updated);
            }
        }
    }
}
