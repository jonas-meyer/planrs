use std::{fmt, marker::PhantomData};

#[derive(PartialEq, Eq, Hash)]
pub struct Key<T> {
    index: u32,
    generation: u32,
    _m: PhantomData<fn() -> T>,
}

impl<T> Copy for Key<T> {}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Key<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Key")
            .field(&self.index)
            .field(&self.generation)
            .finish()
    }
}

impl<T> Key<T> {
    fn from_parts(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _m: PhantomData,
        }
    }

    fn index(&self) -> usize {
        self.index as usize
    }
}

#[derive(Debug)]
struct Slot<T> {
    generation: u32,
    val: Option<T>,
}

#[derive(Debug)]
pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    len: usize,
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            len: 0,
        }
    }
}

impl<T> SlotMap<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            slots: Vec::with_capacity(cap),
            ..Default::default()
        }
    }
    pub fn insert(&mut self, value: T) -> Key<T> {
        if let Some(idx) = self.free.pop() {
            let s = &mut self.slots[idx as usize];
            debug_assert!(s.val.is_none());
            s.val = Some(value);
            self.len += 1;
            Key::from_parts(idx, s.generation)
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                val: Some(value),
            });
            self.len += 1;
            Key::from_parts(idx, 0)
        }
    }
    pub fn remove(&mut self, key: Key<T>) -> Option<T> {
        let slot = self.slots.get_mut(key.index())?;
        if slot.generation != key.generation {
            return None;
        }
        let val = slot.val.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(key.index);
        self.len -= 1;
        Some(val)
    }
    fn valid_index(&self, key: Key<T>) -> Option<usize> {
        let i = key.index();
        let slot = self.slots.get(i)?;
        (slot.generation == key.generation && slot.val.is_some()).then_some(i)
    }

    pub fn get(&self, key: Key<T>) -> Option<&T> {
        let i = self.valid_index(key)?;
        self.slots[i].val.as_ref()
    }

    pub fn get_mut(&mut self, key: Key<T>) -> Option<&mut T> {
        let i = self.valid_index(key)?;
        self.slots[i].val.as_mut()
    }
    pub fn iter(&self) -> impl Iterator<Item = (Key<T>, &T)> {
        self.slots.iter().enumerate().filter_map(|(idx, slot)| {
            slot.val
                .as_ref()
                .map(|value| (Key::from_parts(idx as u32, slot.generation), value))
        })
    }
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(_, value)| value)
    }
}
