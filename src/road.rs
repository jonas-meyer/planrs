use crate::{
    segment::Segment,
    slotmap::Key,
    world::{World, WorldStore},
};

#[derive(Debug, Clone)]
pub struct Road {
    pub name: String,
    pub segments: Vec<Key<Segment>>,
}

pub trait RoadExt {
    fn add_road(&mut self, name: impl Into<String>) -> Key<Road>;
}

impl RoadExt for World
where
    World: WorldStore<Road>,
{
    fn add_road(&mut self, name: impl Into<String>) -> Key<Road> {
        self.add(Road {
            name: name.into(),
            segments: Vec::new(),
        })
    }
}
